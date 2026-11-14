use wasm_engine::math::WelfordStats;
use wasm_engine::models::dca_types::build_dca_summary;
use wasm_engine::models::{merge_dca_results, DcaSimResult, EngineParams, QuantileBand};

fn sample_band(v: f32) -> QuantileBand {
    QuantileBand { p10: vec![v], p25: vec![v], p50: vec![v], p75: vec![v], p90: vec![v] }
}

fn mock_res(sims: usize, p50: f64, breaches: usize, bday: f64, dd: f64, worst: Vec<Vec<f32>>, psd: bool) -> DcaSimResult {
    DcaSimResult {
        total_sims: sims, bands: sample_band(p50 as f32), contributed_band: vec![100.0], floor_band: vec![80.0],
        worst_breach_paths: worst, total_contributed: 100.0, avg_actual_contributed: 95.0,
        breach_count: breaches, breach_rate: (breaches as f64) / (sims.max(1) as f64), avg_breach_day: bday,
        median_terminal_nav: p50, net_profit_p50: p50 - 95.0, moic_p50: p50 / 95.0,
        var_95_net_loss: 10.0, cvar_95_net_loss: 20.0, win_rate: 0.80, sharpe_ratio: 1.0,
        avg_sell_count: 2.0, avg_total_fees: 4.0, avg_chaos_events: 1.5,
        asset_drawdowns_at_ruin: vec![dd], psd_repaired: psd,
    }
}

#[test]
fn test_build_dca_summary_profitable_zero_breach_and_zero_std() {
    let ep = EngineParams::default();
    let mut stats = WelfordStats::new();
    for _ in 0..20 { stats.update(0.5); }

    let res = build_dca_summary(
        20, sample_band(150.0), vec![100.0], vec![80.0], vec![],
        100.0, 2000.0, 0, 0, 0.0, false, 20,
        vec![150.0; 20], vec![50.0; 20], vec![1.5; 20],
        stats, &ep, 0, 0.0, 10, vec![0.0, 0.0], false,
    );

    assert_eq!(res.avg_breach_day, 0.0);
    assert_eq!(res.var_95_net_loss, 0.0);
    assert_eq!(res.cvar_95_net_loss, 0.0);
    assert_eq!(res.sharpe_ratio, 0.0);
    assert_eq!(res.win_rate, 1.0);
    assert_eq!(res.avg_actual_contributed, 100.0);
    assert_eq!(res.asset_drawdowns_at_ruin, vec![0.0, 0.0]);
}

#[test]
fn test_build_dca_summary_tail_losses_and_stress_breach_rate() {
    let ep = EngineParams { var_alpha: 0.05, risk_free_rate: 0.05, ..EngineParams::default() };
    let mut stats = WelfordStats::new();
    let mut profits = Vec::with_capacity(40);
    let mut navs = Vec::with_capacity(40);
    let mut moics = Vec::with_capacity(40);
    for i in 0..40 {
        let profit = if i == 0 { -60.0 } else if i == 1 { -40.0 } else if i == 2 { -20.0 } else { 20.0 };
        profits.push(profit);
        navs.push(100.0 + profit);
        moics.push((100.0 + profit) / 100.0);
        stats.update(profit / 100.0);
    }

    let res = build_dca_summary(
        40, sample_band(120.0), vec![100.0], vec![80.0], vec![],
        100.0, 4000.0, 4, 200, 3.2, true, 37, navs, profits, moics,
        stats, &ep, 8, 12.0, 16, vec![1.0], true,
    );

    // tail_5 = 40 * 0.05 = 2 -> VaR uses profits[2] = -20.0, CVaR uses avg(-60, -40) = -50.0
    assert!((res.var_95_net_loss - 20.0).abs() < 1e-12);
    assert!((res.cvar_95_net_loss - 50.0).abs() < 1e-12);
    assert!((res.breach_rate - 3.2 / 40.0).abs() < 1e-12);
    assert!((res.avg_breach_day - 50.0).abs() < 1e-12);
    assert!((res.asset_drawdowns_at_ruin[0] - 0.25).abs() < 1e-12);
    assert!(res.sharpe_ratio > 0.0 && res.psd_repaired);
}

#[test]
fn test_merge_dca_results_weighted_and_top5_worst() {
    let p1 = mock_res(
        250, 200.0, 2, 100.0, 0.30,
        vec![vec![90.0, 30.0], vec![90.0, 70.0], vec![90.0, 60.0]], false,
    );
    let p2 = mock_res(
        750, 400.0, 6, 300.0, 0.50,
        vec![vec![90.0, 10.0], vec![90.0, 50.0], vec![90.0, 20.0], vec![90.0, 40.0]], true,
    );

    let merged = merge_dca_results(vec![p1, p2]);
    assert_eq!(merged.total_sims, 1000);
    assert!((merged.median_terminal_nav - 350.0).abs() < 1e-9);
    assert!((merged.bands.p50[0] - 350.0).abs() < 1e-4);
    assert_eq!(merged.breach_count, 8);
    assert!((merged.avg_breach_day - 250.0).abs() < 1e-9); // (2*100 + 6*300)/8 = 250
    assert!((merged.asset_drawdowns_at_ruin[0] - 0.45).abs() < 1e-9); // (2*0.3 + 6*0.5)/8
    assert!(merged.psd_repaired);
    assert_eq!(merged.worst_breach_paths.len(), 5);
    assert_eq!(merged.worst_breach_paths[0], vec![90.0, 10.0]);
    assert_eq!(merged.worst_breach_paths[4], vec![90.0, 50.0]);
}

#[test]
fn test_merge_dca_results_zero_breaches_safe() {
    let m = merge_dca_results(vec![mock_res(100, 150.0, 0, 0.0, 0.0, vec![], false), mock_res(100, 250.0, 0, 0.0, 0.0, vec![], false)]);
    assert_eq!(m.breach_count, 0);
    assert_eq!(m.avg_breach_day, 0.0);
    assert_eq!(m.asset_drawdowns_at_ruin, vec![0.0]);
    assert!(!m.psd_repaired);
}

