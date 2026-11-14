//! CLI Report Structures & Multi-Threaded Result Merger
use super::dca_types::DcaSimResult;
use super::engine_params::{AssetSpec, QuantileBand};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CliRunOutput {
    pub id: String,
    pub portfolio_name: String,
    pub sweep_name: String,
    pub years: usize,
    pub horizon_days: usize,
    pub is_dynamic_dca: bool,
    pub tolerance_band_pct: f64,
    pub sell_fee_pct: f64,
    pub ruin_drawdown_pct: f64,
    pub initial_nav: f64,
    pub periodic_cash: f64,
    pub assets: Vec<AssetSpec>,
    pub elapsed_ms: u128,
    pub result: DcaSimResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct CliSuiteOutput {
    pub config_path: String,
    pub chaos_enabled: bool,
    pub chaos_events_count: usize,
    pub total_elapsed_ms: u128,
    pub runs: Vec<CliRunOutput>,
}

/// Merge partial multi-threaded `DcaSimResult` chunks into one unified result.
pub fn merge_dca_results(partials: Vec<DcaSimResult>) -> DcaSimResult {
    let total_sims: usize = partials.iter().map(|r| r.total_sims).sum();
    let total_f = total_sims.max(1) as f64;
    let first = &partials[0];
    let n_cp = first.bands.p50.len();
    let n_assets = first.asset_drawdowns_at_ruin.len();

    let (mut p10, mut p25, mut p50, mut p75, mut p90) =
        (vec![0.0_f32; n_cp], vec![0.0_f32; n_cp], vec![0.0_f32; n_cp], vec![0.0_f32; n_cp], vec![0.0_f32; n_cp]);
    let (mut breach_count, mut breach_rate, mut weighted_breach_day) = (0_usize, 0.0, 0.0);
    let (mut median_nav, mut net_p50, mut moic_p50, mut var_95, mut cvar_95, mut avg_actual_contrib) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let (mut win_rate, mut sharpe, mut avg_sells, mut avg_fees, mut avg_chaos) = (0.0, 0.0, 0.0, 0.0, 0.0);
    let mut asset_dd = vec![0.0_f64; n_assets];
    let mut worst_paths: Vec<Vec<f32>> = Vec::new();

    for r in &partials {
        let w = (r.total_sims as f64) / total_f;
        let wf = w as f32;
        for i in 0..n_cp {
            p10[i] += r.bands.p10[i] * wf;
            p25[i] += r.bands.p25[i] * wf;
            p50[i] += r.bands.p50[i] * wf;
            p75[i] += r.bands.p75[i] * wf;
            p90[i] += r.bands.p90[i] * wf;
        }
        breach_count += r.breach_count;
        breach_rate += r.breach_rate * w;
        weighted_breach_day += r.avg_breach_day * (r.breach_count as f64);
        median_nav += r.median_terminal_nav * w;
        net_p50 += r.net_profit_p50 * w;
        moic_p50 += r.moic_p50 * w;
        var_95 += r.var_95_net_loss * w;
        cvar_95 += r.cvar_95_net_loss * w;
        avg_actual_contrib += r.avg_actual_contributed * w;
        win_rate += r.win_rate * w;
        sharpe += r.sharpe_ratio * w;
        avg_sells += r.avg_sell_count * w;
        avg_fees += r.avg_total_fees * w;
        avg_chaos += r.avg_chaos_events * w;
        for (a_idx, &dd) in r.asset_drawdowns_at_ruin.iter().enumerate() {
            if a_idx < n_assets { asset_dd[a_idx] += dd * (r.breach_count as f64); }
        }
        worst_paths.extend(r.worst_breach_paths.iter().cloned());
    }

    if breach_count > 0 {
        for dd in &mut asset_dd { *dd /= breach_count as f64; }
    }
    worst_paths.sort_by(|a, b| {
        let min_a = a.iter().copied().fold(f32::INFINITY, f32::min);
        let min_b = b.iter().copied().fold(f32::INFINITY, f32::min);
        min_a.partial_cmp(&min_b).unwrap_or(std::cmp::Ordering::Equal)
    });
    worst_paths.truncate(5);

    DcaSimResult {
        total_sims,
        bands: QuantileBand { p10, p25, p50, p75, p90 },
        contributed_band: first.contributed_band.clone(),
        floor_band: first.floor_band.clone(),
        worst_breach_paths: worst_paths,
        total_contributed: first.total_contributed,
        avg_actual_contributed: avg_actual_contrib,
        breach_count,
        breach_rate,
        avg_breach_day: if breach_count > 0 { weighted_breach_day / (breach_count as f64) } else { 0.0 },
        median_terminal_nav: median_nav,
        net_profit_p50: net_p50,
        moic_p50,
        var_95_net_loss: var_95,
        cvar_95_net_loss: cvar_95,
        win_rate,
        sharpe_ratio: sharpe,
        avg_sell_count: avg_sells,
        avg_total_fees: avg_fees,
        avg_chaos_events: avg_chaos,
        asset_drawdowns_at_ruin: asset_dd,
        psd_repaired: partials.iter().any(|r| r.psd_repaired),
    }
}

