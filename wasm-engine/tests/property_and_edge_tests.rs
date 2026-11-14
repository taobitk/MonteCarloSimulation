use wasm_engine::math::{extract_quantile_bands_inplace, WelfordStats};
use wasm_engine::models::chaos_spec::{ChaosEvent, EventScenario, ModOp, ModifierMode, ModifierSpec, ParamTarget, ProbPoint};
use wasm_engine::models::dca_types::build_dca_summary;
use wasm_engine::models::{run_dca_simulation, AssetSpec, DcaConfig, EngineParams, QuantileBand, RareDist, ValueDist};

fn portfolio_cfg(is_dynamic: bool, is_fat_tail: bool, crisis_corr: f64, chaos: Vec<ChaosEvent>) -> DcaConfig {
    DcaConfig {
        initial_nav: 100.0, periodic_cash: 10.0, dca_interval_days: 21,
        is_dynamic_dca: is_dynamic, tolerance_band_pct: 0.10, sell_fee_pct: 0.02,
        ruin_drawdown_pct: 0.20, is_stress_test: false, is_fat_tail,
        assets: vec![
            AssetSpec { name: "Cash".into(), weight: 0.20, mu: 0.05, sigma: 0.01 },
            AssetSpec { name: "Gold".into(), weight: 0.40, mu: 0.09, sigma: 0.18 },
            AssetSpec { name: "Stock".into(), weight: 0.40, mu: 0.11, sigma: 0.22 },
        ],
        corr_matrix: vec![vec![1.0, 0.0, -0.1], vec![0.0, 1.0, 0.1], vec![-0.1, 0.1, 1.0]],
        normal_corr: 0.0, crisis_corr, crisis_threshold_pct: 0.15,
        chaos_events: chaos, engine: EngineParams::default(),
    }
}

#[test]
fn test_property_quantile_bands_safe_and_monotonic_for_all_sizes_0_to_25() {
    for n in 0..=25 {
        let vals: Vec<f32> = (0..n).rev().map(|x| (x as f32) * 3.0 + 1.0).collect();
        let mut cps = vec![vals];
        let b = extract_quantile_bands_inplace(&mut cps);
        if n == 0 {
            assert!(b.p50.is_empty());
        } else {
            assert_eq!(b.p50.len(), 1);
            assert!(b.p10[0] <= b.p25[0] && b.p25[0] <= b.p50[0], "n={n} order failed");
            assert!(b.p50[0] <= b.p75[0] && b.p75[0] <= b.p90[0], "n={n} order failed");
        }
    }
}

#[test]
fn test_tiny_num_sims_0_to_9_summary_and_simulation_no_panic() {
    let ep = EngineParams::default();
    let empty_band = QuantileBand { p10: vec![], p25: vec![], p50: vec![], p75: vec![], p90: vec![] };
    let z = build_dca_summary(
        0, empty_band.clone(), vec![100.0], vec![80.0], vec![],
        100.0, 0.0, 0, 0, 0.0, false, 0, vec![], vec![], vec![],
        WelfordStats::new(), &ep, 0, 0.0, 0, vec![0.0], false,
    );
    assert_eq!(z.total_sims, 0);
    assert_eq!(z.median_terminal_nav, 0.0);

    let one = build_dca_summary(
        1, QuantileBand { p10: vec![90.0], p25: vec![90.0], p50: vec![90.0], p75: vec![90.0], p90: vec![90.0] },
        vec![100.0], vec![80.0], vec![], 100.0, 100.0, 0, 0, 0.0, false, 0,
        vec![90.0], vec![-10.0], vec![0.9], WelfordStats::new(), &ep, 0, 0.0, 0, vec![0.0], false,
    );
    assert_eq!(one.var_95_net_loss, 10.0);
    assert_eq!(one.cvar_95_net_loss, 10.0);

    for sims in [1, 2, 4, 9] {
        let res = run_dca_simulation(portfolio_cfg(true, true, 0.8, vec![]), 1.0, 252, sims, sims as u64);
        assert_eq!(res.total_sims, sims);
        assert!(res.median_terminal_nav.is_finite() && res.var_95_net_loss.is_finite());
    }
}

#[test]
fn test_property_static_dca_invariants_across_seeds_and_fat_tails() {
    for &(seed, fat, crisis) in &[(1_u64, false, 0.5), (17, true, 0.85), (99, true, 0.99)] {
        let res = run_dca_simulation(portfolio_cfg(false, fat, crisis, vec![]), 2.0, 504, 40, seed);
        assert_eq!(res.avg_sell_count, 0.0);
        assert_eq!(res.avg_total_fees, 0.0);
        assert!((0.0..=1.0).contains(&res.win_rate));
        assert!((0.0..=1.0).contains(&res.breach_rate));
        for i in 0..res.bands.p50.len() {
            assert!(res.bands.p10[i] <= res.bands.p25[i] && res.bands.p25[i] <= res.bands.p50[i]);
            assert!(res.bands.p50[i] <= res.bands.p75[i] && res.bands.p75[i] <= res.bands.p90[i]);
        }
    }
}

#[test]
fn test_property_extreme_shocks_preserve_finite_non_negative_nav() {
    let extreme = vec![ChaosEvent {
        id: "doomsday".into(), name: "Doom".into(),
        prob_schedule: vec![ProbPoint { from_year: 0.0, annual_prob: 0.99 }],
        drawdown_trigger: None, cooldown_days: 5, max_triggers_per_sim: 20,
        scenarios: vec![EventScenario {
            name: "crash".into(), weight: 1.0,
            modifiers: vec![
                ModifierSpec { mode: ModifierMode::Instant, target: ParamTarget::AssetValue, asset: Some("Stock".into()), op: ModOp::AddPct, value: ValueDist::Dist(RareDist::Pareto { scale: 0.25, alpha: 1.8, sign: -1.0, max_abs: 0.80 }) },
                ModifierSpec { mode: ModifierMode::Instant, target: ParamTarget::PortfolioNav, asset: None, op: ModOp::AddPct, value: ValueDist::Fixed(-0.50) },
                ModifierSpec { mode: ModifierMode::Temporary { duration_days: 63 }, target: ParamTarget::AssetSigma, asset: Some("Stock".into()), op: ModOp::Mul, value: ValueDist::Fixed(4.0) },
                ModifierSpec { mode: ModifierMode::Temporary { duration_days: 63 }, target: ParamTarget::AssetMu, asset: Some("Stock".into()), op: ModOp::Set, value: ValueDist::Fixed(-0.90) },
            ],
            cascades: vec![],
        }],
    }];
    let res = run_dca_simulation(portfolio_cfg(true, true, 0.90, extreme), 3.0, 756, 30, 555);
    assert!(res.median_terminal_nav.is_finite() && res.median_terminal_nav >= 0.0);
    assert!(res.cvar_95_net_loss.is_finite() && res.sharpe_ratio.is_finite());
    assert!(res.avg_chaos_events > 1.0);
}

