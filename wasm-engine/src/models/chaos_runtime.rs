//! Zero-Alloc Runtime State for Data-Driven Chaos Monkey Simulation
use super::chaos_spec::{CompiledChaosPlan, ModOp, ModifierMode, ParamTarget};
use super::dca_types::DcaConfig;
use super::engine_params::EngineParams;
use crate::rng::FastRng;

#[derive(Debug, Clone)]
struct ActiveTempMod { expire_step: usize, target: ParamTarget, asset_idx: usize, op: ModOp, value: f64 }

pub struct ChaosSimState {
    perm_mu: Vec<f64>, perm_sigma: Vec<f64>, tmp_mu: Vec<f64>, tmp_sigma: Vec<f64>,
    perm_cash: f64, perm_fee: f64, perm_rebal: bool, perm_crisis_corr: bool,
    pub eff_drifts: Vec<f64>, pub eff_sigmas: Vec<f64>,
    pub eff_cash: f64, pub eff_fee: f64, pub eff_rebal: bool, pub eff_crisis_corr: bool,
    trigger_counts: Vec<usize>, cooldown_until: Vec<usize>, event_peak_prices: Vec<f64>,
    active_temps: Vec<ActiveTempMod>, pending_cascades: Vec<(usize, usize)>,
    pub total_events_fired: usize,
}

fn apply_op(curr: f64, op: ModOp, val: f64) -> f64 {
    match op { ModOp::Set => val, ModOp::Add => curr + val, ModOp::AddPct => curr * (1.0 + val), ModOp::Mul => curr * val }
}

impl ChaosSimState {
    pub fn new(n_assets: usize, n_events: usize) -> Self {
        Self {
            perm_mu: vec![0.0; n_assets], perm_sigma: vec![0.0; n_assets],
            tmp_mu: vec![0.0; n_assets], tmp_sigma: vec![0.0; n_assets],
            perm_cash: 0.0, perm_fee: 0.0, perm_rebal: true, perm_crisis_corr: false,
            eff_drifts: vec![0.0; n_assets], eff_sigmas: vec![0.0; n_assets],
            eff_cash: 0.0, eff_fee: 0.0, eff_rebal: true, eff_crisis_corr: false,
            trigger_counts: vec![0; n_events], cooldown_until: vec![0; n_events], event_peak_prices: vec![1.0; n_events],
            active_temps: Vec::with_capacity(16), pending_cascades: Vec::with_capacity(16),
            total_events_fired: 0,
        }
    }

    pub fn reset(&mut self, cfg: &DcaConfig, dt: f64) {
        for (i, a) in cfg.assets.iter().enumerate() { self.perm_mu[i] = a.mu; self.perm_sigma[i] = a.sigma; }
        self.perm_cash = cfg.periodic_cash; self.perm_fee = cfg.sell_fee_pct;
        self.perm_rebal = cfg.is_dynamic_dca; self.perm_crisis_corr = false;
        self.trigger_counts.fill(0); self.cooldown_until.fill(0); self.event_peak_prices.fill(1.0);
        self.active_temps.clear(); self.pending_cascades.clear();
        self.recompute(dt, &cfg.engine);
    }

    fn apply_temp_mod(&mut self, m: &ActiveTempMod, ep: &EngineParams) {
        match m.target {
            ParamTarget::AssetMu => self.tmp_mu[m.asset_idx] = apply_op(self.tmp_mu[m.asset_idx], m.op, m.value),
            ParamTarget::AssetSigma => self.tmp_sigma[m.asset_idx] = apply_op(self.tmp_sigma[m.asset_idx], m.op, m.value),
            ParamTarget::PeriodicCash => self.eff_cash = apply_op(self.eff_cash, m.op, m.value).max(0.0),
            ParamTarget::SellFee => self.eff_fee = apply_op(self.eff_fee, m.op, m.value).clamp(0.0, ep.fee_max),
            ParamTarget::RebalanceEnabled => self.eff_rebal = apply_op(if self.eff_rebal { 1.0 } else { 0.0 }, m.op, m.value) > 0.5,
            ParamTarget::CrisisCorrActive => self.eff_crisis_corr = apply_op(if self.eff_crisis_corr { 1.0 } else { 0.0 }, m.op, m.value) > 0.5,
            ParamTarget::AssetValue | ParamTarget::PortfolioNav => {}
        }
    }

    fn recompute(&mut self, dt: f64, ep: &EngineParams) {
        self.tmp_mu.copy_from_slice(&self.perm_mu);
        self.tmp_sigma.copy_from_slice(&self.perm_sigma);
        self.eff_cash = self.perm_cash; self.eff_fee = self.perm_fee;
        self.eff_rebal = self.perm_rebal; self.eff_crisis_corr = self.perm_crisis_corr;

        for pass in 0..2 {
            let n = self.active_temps.len();
            for i in 0..n {
                let m = self.active_temps[i].clone();
                let is_base = matches!(m.op, ModOp::Set | ModOp::Mul);
                if (pass == 0 && is_base) || (pass == 1 && !is_base) { self.apply_temp_mod(&m, ep); }
            }
        }
        for i in 0..self.tmp_mu.len() {
            let m_c = self.tmp_mu[i].clamp(ep.mu_min, ep.mu_max);
            let s_c = self.tmp_sigma[i].clamp(ep.sigma_min, ep.sigma_max);
            self.eff_sigmas[i] = s_c;
            self.eff_drifts[i] = (m_c - 0.5 * s_c * s_c) * dt;
        }
    }

    pub fn step(&mut self, plan: &CompiledChaosPlan, step: usize, dt: f64, ep: &EngineParams, vals: &mut [f64], unit_prices: &mut [f64], peak_unit_prices: &mut [f64], rng: &mut FastRng) {
        let before = self.active_temps.len();
        self.active_temps.retain(|m| m.expire_step > step);
        let mut dirty = self.active_temps.len() != before;

        for (idx, ev) in plan.events.iter().enumerate() {
            if let Some(a) = ev.dd_asset {
                if unit_prices[a] > self.event_peak_prices[idx] { self.event_peak_prices[idx] = unit_prices[a]; }
            }
            let mut due_cascade = false;
            self.pending_cascades.retain(|&(t_idx, fire_s)| { if t_idx == idx && fire_s <= step { due_cascade = true; false } else { true } });
            if self.trigger_counts[idx] >= ev.max_triggers || step < self.cooldown_until[idx] { continue; }
            let dd_hit = ev.dd_asset.map_or(false, |a| (self.event_peak_prices[idx] - unit_prices[a]) / self.event_peak_prices[idx].max(1e-12) >= ev.dd_pct - 1e-12);
            if !(due_cascade || dd_hit || rng.next_f64() < ev.daily_prob_at(step)) || ev.scenarios.is_empty() { continue; }

            self.trigger_counts[idx] += 1;
            self.cooldown_until[idx] = step + ev.cooldown;
            self.total_events_fired += 1;
            if let Some(a) = ev.dd_asset { self.event_peak_prices[idx] = unit_prices[a]; }

            let roll = rng.next_f64();
            let sc = ev.scenarios.iter().find(|s| roll <= s.cum_weight).unwrap_or_else(|| &ev.scenarios[ev.scenarios.len() - 1]);
            for m in &sc.modifiers {
                let sampled = m.value.sample(rng);
                match m.mode {
                    ModifierMode::Instant => match m.target {
                        ParamTarget::AssetValue => {
                            let old_v = vals[m.asset_idx];
                            let new_v = apply_op(old_v, m.op, sampled).max(1e-6);
                            vals[m.asset_idx] = new_v;
                            unit_prices[m.asset_idx] = (unit_prices[m.asset_idx] * (new_v / old_v.max(1e-12))).max(1e-9);
                            if unit_prices[m.asset_idx] > peak_unit_prices[m.asset_idx] { peak_unit_prices[m.asset_idx] = unit_prices[m.asset_idx]; }
                        }
                        ParamTarget::PortfolioNav => for v in vals.iter_mut() { *v = apply_op(*v, m.op, sampled).max(1e-6); },
                        _ => {}
                    },
                    ModifierMode::Temporary { duration_days } => {
                        self.active_temps.push(ActiveTempMod { expire_step: step + duration_days.max(1), target: m.target, asset_idx: m.asset_idx, op: m.op, value: sampled });
                        dirty = true;
                    }
                    ModifierMode::Permanent => {
                        match m.target {
                            ParamTarget::AssetMu => self.perm_mu[m.asset_idx] = apply_op(self.perm_mu[m.asset_idx], m.op, sampled).clamp(ep.mu_min, ep.mu_max),
                            ParamTarget::AssetSigma => self.perm_sigma[m.asset_idx] = apply_op(self.perm_sigma[m.asset_idx], m.op, sampled).clamp(ep.sigma_min, ep.sigma_max),
                            ParamTarget::PeriodicCash => self.perm_cash = apply_op(self.perm_cash, m.op, sampled).max(0.0),
                            ParamTarget::SellFee => self.perm_fee = apply_op(self.perm_fee, m.op, sampled).clamp(0.0, ep.fee_max),
                            ParamTarget::RebalanceEnabled => self.perm_rebal = apply_op(if self.perm_rebal { 1.0 } else { 0.0 }, m.op, sampled) > 0.5,
                            ParamTarget::CrisisCorrActive => self.perm_crisis_corr = apply_op(if self.perm_crisis_corr { 1.0 } else { 0.0 }, m.op, sampled) > 0.5,
                            ParamTarget::AssetValue | ParamTarget::PortfolioNav => {}
                        }
                        dirty = true;
                    }
                }
            }
            for c in &sc.cascades {
                if rng.next_f64() < c.prob {
                    let span = (c.delay_max - c.delay_min + 1) as f64;
                    let delay = c.delay_min + ((rng.next_f64() * span) as usize).min(c.delay_max - c.delay_min);
                    self.pending_cascades.push((c.target_idx, step + delay));
                }
            }
        }
        if dirty { self.recompute(dt, ep); }
    }
}
