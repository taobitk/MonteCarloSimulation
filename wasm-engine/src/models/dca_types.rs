//! Cấu trúc cấu hình, kết quả và tổng hợp thống kê cho mô phỏng DCA
use super::chaos_spec::ChaosEvent;
use super::engine_params::{AssetSpec, EngineParams, QuantileBand};
use crate::math::WelfordStats;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcaConfig {
    pub initial_nav: f64, pub periodic_cash: f64, pub dca_interval_days: usize,
    pub is_dynamic_dca: bool, pub tolerance_band_pct: f64, pub sell_fee_pct: f64,
    pub ruin_drawdown_pct: f64, pub is_stress_test: bool, pub is_fat_tail: bool,
    pub assets: Vec<AssetSpec>,
    #[serde(default)] pub corr_matrix: Vec<Vec<f64>>,
    pub normal_corr: f64, pub crisis_corr: f64, pub crisis_threshold_pct: f64,
    #[serde(default)] pub chaos_events: Vec<ChaosEvent>,
    #[serde(default)] pub engine: EngineParams,
}

#[derive(Debug, Clone, Serialize)]
pub struct DcaSimResult {
    pub total_sims: usize, pub bands: QuantileBand,
    pub contributed_band: Vec<f32>, pub floor_band: Vec<f32>,
    pub worst_breach_paths: Vec<Vec<f32>>, pub total_contributed: f64,
    pub avg_actual_contributed: f64,
    pub breach_count: usize, pub breach_rate: f64, pub avg_breach_day: f64,
    pub median_terminal_nav: f64, pub net_profit_p50: f64, pub moic_p50: f64,
    pub var_95_net_loss: f64, pub cvar_95_net_loss: f64, pub win_rate: f64,
    pub sharpe_ratio: f64, pub avg_sell_count: f64, pub avg_total_fees: f64,
    pub avg_chaos_events: f64, pub asset_drawdowns_at_ruin: Vec<f64>, pub psd_repaired: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn build_dca_summary(
    num_sims: usize, bands: QuantileBand, contributed_band: Vec<f32>, floor_band: Vec<f32>,
    worst_breach_paths: Vec<Vec<f32>>, total_contributed: f64, actual_contrib_sum: f64,
    breach_count: usize, total_breach_days: usize, sum_is_weights: f64, is_stress_test: bool,
    win_count: usize, mut terminal_navs: Vec<f64>, mut net_profits: Vec<f64>, mut moics: Vec<f64>,
    ret_stats: WelfordStats, ep: &EngineParams, total_sells: usize, total_fees: f64,
    total_chaos: usize, asset_ruin_drops: Vec<f64>, psd_repaired: bool,
) -> DcaSimResult {
    let sort_f64 = |v: &mut Vec<f64>| v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sort_f64(&mut terminal_navs); sort_f64(&mut net_profits); sort_f64(&mut moics);
    let tail_5 = (((num_sims as f64) * ep.var_alpha).max(1.0) as usize).min(net_profits.len());
    let b_cnt = breach_count.max(1) as f64;
    let sims_f = num_sims.max(1) as f64;
    let mid = num_sims / 2;
    let cvar_profit = if tail_5 > 0 { net_profits[0..tail_5].iter().sum::<f64>() / (tail_5 as f64) } else { 0.0 };
    let var_profit = net_profits.get(tail_5).or_else(|| net_profits.last()).copied().unwrap_or(0.0);

    DcaSimResult {
        total_sims: num_sims, bands, contributed_band, floor_band, worst_breach_paths, total_contributed,
        avg_actual_contributed: actual_contrib_sum / sims_f,
        breach_count,
        breach_rate: if is_stress_test { (sum_is_weights / sims_f).clamp(0.0, 1.0) } else { (breach_count as f64) / sims_f },
        avg_breach_day: if breach_count > 0 { (total_breach_days as f64) / b_cnt } else { 0.0 },
        median_terminal_nav: terminal_navs.get(mid).copied().unwrap_or(0.0),
        net_profit_p50: net_profits.get(mid).copied().unwrap_or(0.0),
        moic_p50: moics.get(mid).copied().unwrap_or(0.0),
        var_95_net_loss: (-var_profit).max(0.0),
        cvar_95_net_loss: (-cvar_profit).max(0.0),
        win_rate: (win_count as f64) / sims_f,
        sharpe_ratio: if ret_stats.sample_std_dev() > 1e-6 { (ret_stats.mean - ep.risk_free_rate) / ret_stats.sample_std_dev() } else { 0.0 },
        avg_sell_count: (total_sells as f64) / sims_f, avg_total_fees: total_fees / sims_f,
        avg_chaos_events: (total_chaos as f64) / sims_f,
        asset_drawdowns_at_ruin: asset_ruin_drops.into_iter().map(|d| d / b_cnt).collect(), psd_repaired,
    }
}

