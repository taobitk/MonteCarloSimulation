use wasm_engine::math::apply_dca_inflow_and_rebalance;
use wasm_engine::models::chaos_runtime::ChaosSimState;
use wasm_engine::models::chaos_spec::{
    ChaosEvent, CompiledChaosPlan, DrawdownTrigger, EventScenario, ModOp, ModifierMode,
    ModifierSpec, ParamTarget,
};
use wasm_engine::models::{AssetSpec, DcaConfig, EngineParams, ValueDist};
use wasm_engine::rng::FastRng;

fn single_asset_cfg(events: Vec<ChaosEvent>) -> DcaConfig {
    DcaConfig {
        initial_nav: 100.0, periodic_cash: 10.0, dca_interval_days: 21,
        is_dynamic_dca: true, tolerance_band_pct: 0.10, sell_fee_pct: 0.02,
        ruin_drawdown_pct: 0.20, is_stress_test: false, is_fat_tail: false,
        assets: vec![AssetSpec { name: "Stock".into(), weight: 1.0, mu: 0.10, sigma: 0.20 }],
        corr_matrix: vec![vec![1.0]], normal_corr: 0.0, crisis_corr: 0.8,
        crisis_threshold_pct: 0.15, chaos_events: events, engine: EngineParams::default(),
    }
}

#[test]
fn test_chaos_max_triggers_per_sim_enforced_after_cooldown() {
    let dt = 1.0 / 252.0;
    let events = vec![ChaosEvent {
        id: "cap".into(), name: "Cap".into(), prob_schedule: vec![],
        drawdown_trigger: Some(DrawdownTrigger { asset: "Stock".into(), drop_from_peak_pct: 0.0 }),
        cooldown_days: 1, max_triggers_per_sim: 2,
        scenarios: vec![EventScenario { name: "noop".into(), weight: 1.0, modifiers: vec![], cascades: vec![] }],
    }];
    let cfg = single_asset_cfg(events);
    let plan = CompiledChaosPlan::compile(&cfg.chaos_events, &cfg.assets, dt, &cfg.engine);
    let (mut state, mut rng) = (ChaosSimState::new(1, 1), FastRng::new(10));
    let (mut vals, mut up, mut pk) = ([100.0], [1.0], [1.0]);
    state.reset(&cfg, dt);

    for step in 1..=6 {
        state.step(&plan, step, dt, &cfg.engine, &mut vals, &mut up, &mut pk, &mut rng);
    }
    assert_eq!(state.total_events_fired, 2);
}

#[test]
fn test_chaos_positive_asset_shock_raises_peak_unit_price() {
    let dt = 1.0 / 252.0;
    let events = vec![
        ChaosEvent {
            id: "boom".into(), name: "Boom".into(), prob_schedule: vec![],
            drawdown_trigger: Some(DrawdownTrigger { asset: "Stock".into(), drop_from_peak_pct: 0.0 }),
            cooldown_days: 99, max_triggers_per_sim: 1,
            scenarios: vec![EventScenario {
                name: "up".into(), weight: 1.0,
                modifiers: vec![ModifierSpec { mode: ModifierMode::Instant, target: ParamTarget::AssetValue, asset: Some("Stock".into()), op: ModOp::AddPct, value: ValueDist::Fixed(1.0) }],
                cascades: vec![],
            }],
        },
        ChaosEvent {
            id: "bust".into(), name: "Bust".into(), prob_schedule: vec![],
            drawdown_trigger: Some(DrawdownTrigger { asset: "Stock".into(), drop_from_peak_pct: 0.25 }),
            cooldown_days: 99, max_triggers_per_sim: 1,
            scenarios: vec![EventScenario { name: "hit".into(), weight: 1.0, modifiers: vec![], cascades: vec![] }],
        },
    ];
    let cfg = single_asset_cfg(events);
    let plan = CompiledChaosPlan::compile(&cfg.chaos_events, &cfg.assets, dt, &cfg.engine);
    let (mut state, mut rng) = (ChaosSimState::new(1, 2), FastRng::new(11));
    let (mut vals, mut up, mut pk) = ([100.0], [1.0], [1.0]);
    state.reset(&cfg, dt);

    state.step(&plan, 1, dt, &cfg.engine, &mut vals, &mut up, &mut pk, &mut rng);
    assert!((up[0] - 2.0).abs() < 1e-12 && (pk[0] - 2.0).abs() < 1e-12);

    up[0] = 1.5; // -25% from peak 2.0 (even though still +50% above initial 1.0!)
    state.step(&plan, 2, dt, &cfg.engine, &mut vals, &mut up, &mut pk, &mut rng);
    assert_eq!(state.total_events_fired, 2);
}

#[test]
fn test_chaos_temporary_rebalance_and_crisis_corr_expiry_revert() {
    let dt = 1.0 / 252.0;
    let events = vec![ChaosEvent {
        id: "flags".into(), name: "Flags".into(), prob_schedule: vec![],
        drawdown_trigger: Some(DrawdownTrigger { asset: "Stock".into(), drop_from_peak_pct: 0.0 }),
        cooldown_days: 99, max_triggers_per_sim: 1,
        scenarios: vec![EventScenario {
            name: "freeze".into(), weight: 1.0,
            modifiers: vec![
                ModifierSpec { mode: ModifierMode::Temporary { duration_days: 4 }, target: ParamTarget::RebalanceEnabled, asset: None, op: ModOp::Set, value: ValueDist::Fixed(0.0) },
                ModifierSpec { mode: ModifierMode::Temporary { duration_days: 4 }, target: ParamTarget::CrisisCorrActive, asset: None, op: ModOp::Set, value: ValueDist::Fixed(1.0) },
            ],
            cascades: vec![],
        }],
    }];
    let cfg = single_asset_cfg(events);
    let plan = CompiledChaosPlan::compile(&cfg.chaos_events, &cfg.assets, dt, &cfg.engine);
    let (mut state, mut rng) = (ChaosSimState::new(1, 1), FastRng::new(12));
    let (mut vals, mut up, mut pk) = ([100.0], [1.0], [1.0]);
    state.reset(&cfg, dt);

    state.step(&plan, 1, dt, &cfg.engine, &mut vals, &mut up, &mut pk, &mut rng);
    assert!(!state.eff_rebal && state.eff_crisis_corr);

    state.step(&plan, 5, dt, &cfg.engine, &mut vals, &mut up, &mut pk, &mut rng);
    assert!(state.eff_rebal && !state.eff_crisis_corr);
}

#[test]
fn test_dca_math_exact_band_threshold_does_not_sell_and_above_sells() {
    let weights = [0.5, 0.5];
    let mut vals_eq = [75.0, 25.0]; // exact 0.25 drift == 0.25 band
    let (_t1, sold1, fee1) = apply_dca_inflow_and_rebalance(&mut vals_eq, &weights, 0.0, true, true, 0.25, 0.02);
    assert!(!sold1 && fee1 == 0.0);

    let mut vals_over = [76.0, 24.0]; // 0.26 drift > 0.25 band
    let (t2, sold2, fee2) = apply_dca_inflow_and_rebalance(&mut vals_over, &weights, 0.0, true, true, 0.25, 0.02);
    assert!(sold2 && (fee2 - 0.52).abs() < 1e-9 && (t2 - 99.48).abs() < 1e-9);
}

