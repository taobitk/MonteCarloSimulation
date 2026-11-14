//! Configurable Engine Parameters & Continuous Rare-Event Distributions
use crate::math::box_muller;
use crate::rng::FastRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSpec {
    pub name: String,
    pub weight: f64,
    pub mu: f64,
    pub sigma: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantileBand {
    pub p10: Vec<f32>,
    pub p25: Vec<f32>,
    pub p50: Vec<f32>,
    pub p75: Vec<f32>,
    pub p90: Vec<f32>,
}

fn d_df() -> f64 { 4.0 }
fn d_clamp() -> f64 { 5.0 }
fn d_tdays() -> usize { 252 }
fn d_tilt() -> f64 { -2.0 }
fn d_cap() -> f64 { 10.0 }
fn d_alpha() -> f64 { 0.05 }
fn d_mu_min() -> f64 { -0.50 }
fn d_mu_max() -> f64 { 0.60 }
fn d_sig_min() -> f64 { 0.005 }
fn d_sig_max() -> f64 { 2.50 }
fn d_fee_max() -> f64 { 0.50 }
fn d_prob_max() -> f64 { 0.9999 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EngineParams {
    #[serde(default = "d_df")] pub fat_tail_df: f64,
    #[serde(default = "d_clamp")] pub shock_clamp: f64,
    #[serde(default = "d_tdays")] pub trading_days_per_year: usize,
    #[serde(default = "d_tilt")] pub is_tilt_strength: f64,
    #[serde(default = "d_cap")] pub is_weight_cap: f64,
    #[serde(default = "d_alpha")] pub var_alpha: f64,
    #[serde(default)] pub risk_free_rate: f64,
    #[serde(default = "d_mu_min")] pub mu_min: f64,
    #[serde(default = "d_mu_max")] pub mu_max: f64,
    #[serde(default = "d_sig_min")] pub sigma_min: f64,
    #[serde(default = "d_sig_max")] pub sigma_max: f64,
    #[serde(default = "d_fee_max")] pub fee_max: f64,
    #[serde(default = "d_prob_max")] pub annual_prob_max: f64,
}

impl Default for EngineParams {
    fn default() -> Self {
        Self {
            fat_tail_df: 4.0, shock_clamp: 5.0, trading_days_per_year: 252,
            is_tilt_strength: -2.0, is_weight_cap: 10.0, var_alpha: 0.05, risk_free_rate: 0.0,
            mu_min: -0.50, mu_max: 0.60, sigma_min: 0.005, sigma_max: 2.50,
            fee_max: 0.50, annual_prob_max: 0.9999,
        }
    }
}

impl EngineParams {
    pub fn portfolio_default() -> Self { Self { risk_free_rate: 0.03, ..Self::default() } }
    pub fn fat_tail_norm(&self) -> f64 {
        let df = self.fat_tail_df.max(2.01);
        ((df - 2.0) / df).sqrt()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "dist")]
pub enum RareDist {
    Uniform { min: f64, max: f64 },
    Normal { mean: f64, std: f64, min: f64, max: f64 },
    Pareto { scale: f64, alpha: f64, sign: f64, max_abs: f64 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueDist {
    Fixed(f64),
    Dist(RareDist),
}

impl ValueDist {
    pub fn sample(&self, rng: &mut FastRng) -> f64 {
        match *self {
            Self::Fixed(v) => v,
            Self::Dist(RareDist::Uniform { min, max }) => min + (max - min) * rng.next_f64(),
            Self::Dist(RareDist::Normal { mean, std, min, max }) => {
                let z = box_muller(rng.next_f64(), rng.next_f64()).0;
                (mean + std * z).clamp(min.min(max), max.max(min))
            }
            Self::Dist(RareDist::Pareto { scale, alpha, sign, max_abs }) => {
                let u = rng.next_f64().clamp(1e-12, 1.0 - 1e-12);
                let raw = scale.abs() / u.powf(1.0 / alpha.max(0.1));
                raw.min(max_abs.abs()) * if sign >= 0.0 { 1.0 } else { -1.0 }
            }
        }
    }
}

