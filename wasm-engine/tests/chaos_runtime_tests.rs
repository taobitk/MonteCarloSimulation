use wasm_engine::models::chaos_runtime::ChaosSimState;
use wasm_engine::models::chaos_spec::{
    CascadeLink, ChaosEvent, CompiledChaosPlan, DrawdownTrigger, EventScenario, ModOp,
    ModifierMode, ModifierSpec, ParamTarget,
};
use wasm_engine::models::{AssetSpec, DcaConfig, EngineParams, ValueDist};
use wasm_engine::rng::FastRng;

fn mk_cfg(chaos_events: Vec<ChaosEvent>) -> DcaConfig {
    DcaConfig {
        initial_nav: 100.0, periodic_cash: 10.0, dca_interval_days: 21,
        is_dynamic_dca: true, tolerance_band_pct: 0.10, sell_fee_pct: 0.02,
        ruin_drawdown_pct: 0.20, is_stress_test: false, is_fat_tail: false,
        assets: vec![AssetSpec { name: "Stock".into(), weight: 1.0, mu: 0.10, sigma: 0.20 }],
        corr_matrix: vec![vec![1.0]], normal_corr: 0.0, crisis_corr: 0.8,
        crisis_threshold_pct: 0.15, chaos_events, engine: EngineParams::default(),
    }
}

fn step_dd_event(id: &str, mode: ModifierMode, target: ParamTarget, op: ModOp, val: f64, dd: f64, cd: usize) -> ChaosEvent {
    ChaosEvent {
        id: id.into(), name: id.into(), prob_schedule: vec![],
        drawdown_trigger: Some(DrawdownTrigger { asset: "Stock".into(), drop_from_peak_pct: dd }),
        cooldown_days: cd, max_triggers_per_sim: 5,
        scenarios: vec![EventScenario {
            name: "s".into(), weight: 1.0,
            modifiers: vec![ModifierSpec { mode, target, asset: Some("Stock".into()), op, value: ValueDist::Fixed(val) }],
            cascades: vec![],
        }],
    }
}

#[test]
fn test_runtime_two_pass_sell_fee_and_temporary_expiry() {
    let dt = 1.0 / 252.0;
    let events = vec![
        step_dd_event("add", ModifierMode::Temporary { duration_days: 5 }, ParamTarget::SellFee, ModOp::Add, 0.03, 0.0, 999),
        step_dd_event("set", ModifierMode::Temporary { duration_days: 10 }, ParamTarget::SellFee, ModOp::Set, 0.10, 0.0, 999),
    ];
    let cfg = mk_cfg(events);
    let plan = CompiledChaosPlan::compile(&cfg.chaos_events, &cfg.assets, dt, &cfg.engine);
    let (mut state, mut rng) = (ChaosSimState::new(1, 2), FastRng::new(1));
    let (mut vals, mut up, mut pk) = ([100.0], [1.0], [1.0]);
    state.reset(&cfg, dt);

    state.step(&plan, 1, dt, &cfg.engine, &mut vals, &mut up, &mut pk, &mut rng);
    assert!((state.eff_fee - 0.13).abs() < 1e-12, "Set=0.10 then Add=0.03 expected 0.13, got {}", state.eff_fee);

    state.step(&plan, 6, dt, &cfg.engine, &mut vals, &mut up, &mut pk, &mut rng);
    assert!((state.eff_fee - 0.10).abs() < 1e-12, "After Add expires, Set=0.10 expected, got {}", state.eff_fee);

    state.step(&plan, 11, dt, &cfg.engine, &mut vals, &mut up, &mut pk, &mut rng);
    assert!((state.eff_fee - 0.02).abs() < 1e-12, "All expired expected base 0.02, got {}", state.eff_fee);
}

#[test]
fn test_runtime_permanent_clamps_and_flags() {
    let dt = 1.0 / 252.0;
    let events = vec![ChaosEvent {
        id: "perm".into(), name: "Perm".into(), prob_schedule: vec![],
        drawdown_trigger: Some(DrawdownTrigger { asset: "Stock".into(), drop_from_peak_pct: 0.0 }),
        cooldown_days: 999, max_triggers_per_sim: 1,
        scenarios: vec![EventScenario {
            name: "p".into(), weight: 1.0,
            modifiers: vec![
                ModifierSpec { mode: ModifierMode::Permanent, target: ParamTarget::AssetMu, asset: Some("Stock".into()), op: ModOp::Set, value: ValueDist::Fixed(-9.0) },
                ModifierSpec { mode: ModifierMode::Permanent, target: ParamTarget::AssetSigma, asset: Some("Stock".into()), op: ModOp::Set, value: ValueDist::Fixed(9.0) },
                ModifierSpec { mode: ModifierMode::Permanent, target: ParamTarget::PeriodicCash, asset: None, op: ModOp::Add, value: ValueDist::Fixed(-50.0) },
                ModifierSpec { mode: ModifierMode::Permanent, target: ParamTarget::SellFee, asset: None, op: ModOp::Set, value: ValueDist::Fixed(2.0) },
                ModifierSpec { mode: ModifierMode::Permanent, target: ParamTarget::RebalanceEnabled, asset: None, op: ModOp::Set, value: ValueDist::Fixed(0.0) },
                ModifierSpec { mode: ModifierMode::Permanent, target: ParamTarget::CrisisCorrActive, asset: None, op: ModOp::Set, value: ValueDist::Fixed(1.0) },
            ],
            cascades: vec![],
        }],
    }];
    let cfg = mk_cfg(events);
    let plan = CompiledChaosPlan::compile(&cfg.chaos_events, &cfg.assets, dt, &cfg.engine);
    let (mut state, mut rng) = (ChaosSimState::new(1, 1), FastRng::new(2));
    let (mut vals, mut up, mut pk) = ([100.0], [1.0], [1.0]);
    state.reset(&cfg, dt);
    state.step(&plan, 1, dt, &cfg.engine, &mut vals, &mut up, &mut pk, &mut rng);

    let ep = &cfg.engine;
    let exp_drift = (ep.mu_min - 0.5 * ep.sigma_max * ep.sigma_max) * dt;
    assert!((state.eff_sigmas[0] - ep.sigma_max).abs() < 1e-12);
    assert!((state.eff_drifts[0] - exp_drift).abs() < 1e-12);
    assert_eq!(state.eff_cash, 0.0);
    assert!((state.eff_fee - ep.fee_max).abs() < 1e-12);
    assert!(!state.eff_rebal && state.eff_crisis_corr);
}

#[test]
fn test_runtime_drawdown_peak_reset_portfolio_nav_and_cascade() {
    let dt = 1.0 / 252.0;
    let events = vec![
        ChaosEvent {
            id: "dd".into(), name: "DD".into(), prob_schedule: vec![],
            drawdown_trigger: Some(DrawdownTrigger { asset: "Stock".into(), drop_from_peak_pct: 0.20 }),
            cooldown_days: 5, max_triggers_per_sim: 5,
            scenarios: vec![EventScenario {
                name: "hit".into(), weight: 1.0,
                modifiers: vec![ModifierSpec { mode: ModifierMode::Instant, target: ParamTarget::PortfolioNav, asset: None, op: ModOp::AddPct, value: ValueDist::Fixed(-0.15) }],
                cascades: vec![CascadeLink { target_event_id: "after".into(), prob: 1.0, delay_min_days: 2, delay_max_days: 2 }],
            }],
        },
        ChaosEvent {
            id: "after".into(), name: "After".into(), prob_schedule: vec![], drawdown_trigger: None,
            cooldown_days: 99, max_triggers_per_sim: 1,
            scenarios: vec![EventScenario {
                name: "fx".into(), weight: 1.0,
                modifiers: vec![ModifierSpec { mode: ModifierMode::Instant, target: ParamTarget::AssetValue, asset: Some("Stock".into()), op: ModOp::AddPct, value: ValueDist::Fixed(-0.10) }],
                cascades: vec![],
            }],
        },
    ];
    let cfg = mk_cfg(events);
    let plan = CompiledChaosPlan::compile(&cfg.chaos_events, &cfg.assets, dt, &cfg.engine);
    let (mut state, mut rng) = (ChaosSimState::new(1, 2), FastRng::new(3));
    let (mut vals, mut up, mut pk) = ([100.0], [0.8001], [1.0]);
    state.reset(&cfg, dt);

    state.step(&plan, 0, dt, &cfg.engine, &mut vals, &mut up, &mut pk, &mut rng);
    assert_eq!(state.total_events_fired, 0, "0.8001 (-19.99%) must not trigger -20% dd");

    up[0] = 0.80; // IEEE-754 exact decimal -20%: 1.0 - 0.80 == 0.19999999999999996
    state.step(&plan, 1, dt, &cfg.engine, &mut vals, &mut up, &mut pk, &mut rng);
    assert_eq!(state.total_events_fired, 1);
    assert!((vals[0] - 85.0).abs() < 1e-9);
    assert!((up[0] - 0.80).abs() < 1e-12, "PortfolioNav must not mutate unit_prices");

    state.step(&plan, 3, dt, &cfg.engine, &mut vals, &mut up, &mut pk, &mut rng);
    assert_eq!(state.total_events_fired, 2, "cascade scheduled for step 1+2=3 must fire");
    assert!((vals[0] - 76.5).abs() < 1e-9);
    assert!((up[0] - 0.72).abs() < 1e-9);

    state.step(&plan, 6, dt, &cfg.engine, &mut vals, &mut up, &mut pk, &mut rng);
    assert_eq!(state.total_events_fired, 2, "peak reset to 0.80 so 0.72 (-10%) must not re-fire dd(-20%)");
}
