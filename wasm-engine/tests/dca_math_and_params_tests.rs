use wasm_engine::math::{apply_dca_inflow_and_rebalance, cholesky_with_psd_repair};
use wasm_engine::models::{EngineParams, RareDist, ValueDist};
use wasm_engine::rng::FastRng;

#[test]
fn test_dca_math_zero_cash_still_rebalances_and_clamps_negative_fee() {
    let mut vals = vec![90.0, 10.0];
    let weights = vec![0.5, 0.5];
    let (tot, sold, fee) =
        apply_dca_inflow_and_rebalance(&mut vals, &weights, 0.0, true, true, 0.05, -0.10);

    assert!(sold);
    assert_eq!(fee, 0.0);
    assert!((tot - 100.0).abs() < 1e-9);
    assert!((vals[0] - 50.0).abs() < 1e-9 && (vals[1] - 50.0).abs() < 1e-9);
}

#[test]
fn test_dca_math_balanced_portfolio_falls_back_to_target_weights_on_inflow() {
    let mut vals = vec![60.0, 40.0];
    let weights = vec![0.6, 0.4];
    let (tot, sold, fee) =
        apply_dca_inflow_and_rebalance(&mut vals, &weights, 25.0, true, true, 0.05, 0.02);

    assert!(!sold && fee == 0.0);
    assert!((tot - 125.0).abs() < 1e-9);
    assert!((vals[0] - 75.0).abs() < 1e-9 && (vals[1] - 50.0).abs() < 1e-9);
}

#[test]
fn test_dca_math_zero_nav_returns_cleanly() {
    let mut vals = vec![0.0, 0.0];
    let weights = vec![0.5, 0.5];
    let (tot, sold, fee) =
        apply_dca_inflow_and_rebalance(&mut vals, &weights, 0.0, true, true, 0.05, 0.02);
    assert_eq!(tot, 0.0);
    assert!(!sold);
    assert_eq!(fee, 0.0);
}

#[test]
fn test_value_dist_sampling_respects_bounds_and_signs() {
    let mut rng = FastRng::new(77);
    assert_eq!(ValueDist::Fixed(-0.15).sample(&mut rng), -0.15);

    let uniform = ValueDist::Dist(RareDist::Uniform { min: -0.25, max: -0.05 });
    let normal_rev = ValueDist::Dist(RareDist::Normal {
        mean: 0.0,
        std: 1.0,
        min: 0.10,
        max: -0.10, // reversed bounds should still clamp into [-0.10, 0.10]
    });
    let pareto_neg = ValueDist::Dist(RareDist::Pareto {
        scale: 0.08,
        alpha: 2.2,
        sign: -1.0,
        max_abs: 0.55,
    });
    let pareto_pos = ValueDist::Dist(RareDist::Pareto {
        scale: 0.05,
        alpha: 2.0,
        sign: 1.0,
        max_abs: 0.40,
    });

    for _ in 0..2_000 {
        let u = uniform.sample(&mut rng);
        assert!((-0.25..=-0.05).contains(&u), "uniform out of bounds: {u}");

        let n = normal_rev.sample(&mut rng);
        assert!((-0.10..=0.10).contains(&n), "normal out of bounds: {n}");

        let pn = pareto_neg.sample(&mut rng);
        assert!((-0.55..=-0.08).contains(&pn), "pareto neg out of bounds: {pn}");

        let pp = pareto_pos.sample(&mut rng);
        assert!((0.05..=0.40).contains(&pp), "pareto pos out of bounds: {pp}");
    }
}

#[test]
fn test_engine_params_defaults_and_1x1_cholesky() {
    let def = EngineParams::default();
    let pdef = EngineParams::portfolio_default();
    assert_eq!(def.risk_free_rate, 0.0);
    assert_eq!(pdef.risk_free_rate, 0.03);
    assert!((def.fat_tail_norm() - (2.0_f64 / 4.0).sqrt()).abs() < 1e-12);

    let tiny = EngineParams { fat_tail_df: 1.0, ..def };
    assert!((tiny.fat_tail_norm() - (0.01_f64 / 2.01).sqrt()).abs() < 1e-12);

    let (l, repaired) = cholesky_with_psd_repair(&[vec![0.0]]);
    assert!(!repaired);
    assert!((l[0][0] - (1e-6_f64).sqrt()).abs() < 1e-12);
}

