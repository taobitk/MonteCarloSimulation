//! Data-driven Chaos Monkey & Multi-Scenario Event Specification
use super::engine_params::{EngineParams, ValueDist};
use super::engine_params::AssetSpec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_max_triggers() -> usize { usize::MAX }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbPoint { pub from_year: f64, pub annual_prob: f64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownTrigger { pub asset: String, pub drop_from_peak_pct: f64 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "mode")]
pub enum ModifierMode { Instant, Temporary { duration_days: usize }, Permanent }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParamTarget { AssetValue, AssetMu, AssetSigma, PeriodicCash, SellFee, RebalanceEnabled, CrisisCorrActive, PortfolioNav }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModOp { Set, Add, AddPct, Mul }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierSpec {
    #[serde(flatten)] pub mode: ModifierMode,
    pub target: ParamTarget, #[serde(default)] pub asset: Option<String>,
    pub op: ModOp, pub value: ValueDist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeLink {
    pub target_event_id: String, pub prob: f64,
    #[serde(default)] pub delay_min_days: usize, #[serde(default)] pub delay_max_days: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventScenario {
    #[serde(default)] pub name: String, pub weight: f64,
    #[serde(default)] pub modifiers: Vec<ModifierSpec>, #[serde(default)] pub cascades: Vec<CascadeLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosEvent {
    pub id: String, #[serde(default)] pub name: String,
    #[serde(default)] pub prob_schedule: Vec<ProbPoint>,
    #[serde(default)] pub drawdown_trigger: Option<DrawdownTrigger>,
    #[serde(default)] pub cooldown_days: usize,
    #[serde(default = "default_max_triggers")] pub max_triggers_per_sim: usize,
    pub scenarios: Vec<EventScenario>,
}

#[derive(Debug, Clone)]
pub struct CompiledProbPoint { pub from_step: usize, pub daily_prob: f64 }
#[derive(Debug, Clone)]
pub struct CompiledModifier { pub mode: ModifierMode, pub target: ParamTarget, pub asset_idx: usize, pub op: ModOp, pub value: ValueDist }
#[derive(Debug, Clone)]
pub struct CompiledCascade { pub target_idx: usize, pub prob: f64, pub delay_min: usize, pub delay_max: usize }
#[derive(Debug, Clone)]
pub struct CompiledScenario { pub cum_weight: f64, pub modifiers: Vec<CompiledModifier>, pub cascades: Vec<CompiledCascade> }

#[derive(Debug, Clone)]
pub struct CompiledEvent {
    pub prob_schedule: Vec<CompiledProbPoint>,
    pub dd_asset: Option<usize>,
    pub dd_pct: f64,
    pub cooldown: usize,
    pub max_triggers: usize,
    pub scenarios: Vec<CompiledScenario>,
}

impl CompiledEvent {
    pub fn daily_prob_at(&self, step: usize) -> f64 {
        let mut p = 0.0;
        for pt in &self.prob_schedule {
            if pt.from_step <= step { p = pt.daily_prob; } else { break; }
        }
        p
    }
}

#[derive(Debug, Clone)]
pub struct CompiledChaosPlan { pub events: Vec<CompiledEvent> }

impl CompiledChaosPlan {
    pub fn is_empty(&self) -> bool { self.events.is_empty() }

    pub fn compile(events: &[ChaosEvent], assets: &[AssetSpec], dt: f64, ep: &EngineParams) -> Self {
        let asset_map: HashMap<&str, usize> = assets.iter().enumerate().map(|(i, a)| (a.name.as_str(), i)).collect();
        let event_map: HashMap<&str, usize> = events.iter().enumerate().map(|(i, e)| (e.id.as_str(), i)).collect();
        let steps_per_year = (1.0 / dt.max(1e-6)).round();

        let compiled = events.iter().map(|ev| {
            let mut sched: Vec<CompiledProbPoint> = ev.prob_schedule.iter().map(|p| {
                let annual = p.annual_prob.clamp(0.0, ep.annual_prob_max);
                let daily = if annual > 0.0 { 1.0 - (1.0 - annual).powf(dt) } else { 0.0 };
                CompiledProbPoint { from_step: (p.from_year.max(0.0) * steps_per_year).round() as usize, daily_prob: daily }
            }).collect();
            sched.sort_by_key(|p| p.from_step);

            let (dd_asset, dd_pct) = ev.drawdown_trigger.as_ref()
                .and_then(|d| asset_map.get(d.asset.as_str()).map(|&i| (Some(i), d.drop_from_peak_pct)))
                .unwrap_or((None, 1.0));

            let total_w: f64 = ev.scenarios.iter().map(|s| s.weight.max(0.0)).sum::<f64>().max(1e-12);
            let mut cum = 0.0;
            let scenarios = ev.scenarios.iter().map(|sc| {
                cum += sc.weight.max(0.0) / total_w;
                let modifiers = sc.modifiers.iter().filter_map(|m| {
                    let asset_idx = match m.target {
                        ParamTarget::AssetValue | ParamTarget::AssetMu | ParamTarget::AssetSigma => {
                            let name = m.asset.as_deref()?;
                            *asset_map.get(name)?
                        }
                        _ => 0,
                    };
                    Some(CompiledModifier { mode: m.mode, target: m.target, asset_idx, op: m.op, value: m.value })
                }).collect();
                let cascades = sc.cascades.iter().filter_map(|c| {
                    let &target_idx = event_map.get(c.target_event_id.as_str())?;
                    let dmin = c.delay_min_days.max(1);
                    Some(CompiledCascade { target_idx, prob: c.prob.clamp(0.0, 1.0), delay_min: dmin, delay_max: c.delay_max_days.max(dmin) })
                }).collect();
                CompiledScenario { cum_weight: cum, modifiers, cascades }
            }).collect();

            CompiledEvent {
                prob_schedule: sched, dd_asset, dd_pct,
                cooldown: ev.cooldown_days, max_triggers: ev.max_triggers_per_sim, scenarios,
            }
        }).collect();

        Self { events: compiled }
    }
}
