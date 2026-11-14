use wasm_engine::models::chaos_spec::{
    CascadeLink, ChaosEvent, CompiledChaosPlan, DrawdownTrigger, EventScenario, ModOp,
    ModifierMode, ModifierSpec, ParamTarget, ProbPoint,
};
use wasm_engine::models::{AssetSpec, EngineParams, ValueDist};

fn sample_assets() -> Vec<AssetSpec> {
    vec![
        AssetSpec { name: "Cash".into(), weight: 0.5, mu: 0.05, sigma: 0.01 },
        AssetSpec { name: "Stock".into(), weight: 0.5, mu: 0.10, sigma: 0.20 },
    ]
}

#[test]
fn test_compile_sorts_prob_schedule_clamps_and_converts_daily_prob() {
    let dt = 1.0 / 252.0;
    let ep = EngineParams::default();
    let events = vec![ChaosEvent {
        id: "sched_ev".into(),
        name: "Unsorted".into(),
        prob_schedule: vec![
            ProbPoint { from_year: 2.0, annual_prob: 1.5 }, // clamped to 0.9999
            ProbPoint { from_year: 0.5, annual_prob: 0.20 },
            ProbPoint { from_year: 0.0, annual_prob: -0.10 }, // clamped to 0.0
        ],
        drawdown_trigger: None,
        cooldown_days: 30,
        max_triggers_per_sim: 2,
        scenarios: vec![],
    }];

    let plan = CompiledChaosPlan::compile(&events, &sample_assets(), dt, &ep);
    assert!(!plan.is_empty());
    let ev = &plan.events[0];

    assert_eq!(ev.daily_prob_at(0), 0.0);
    assert_eq!(ev.daily_prob_at(125), 0.0);
    let exp_05 = 1.0 - (1.0 - 0.20_f64).powf(dt);
    assert!((ev.daily_prob_at(126) - exp_05).abs() < 1e-12);
    let exp_20 = 1.0 - (1.0 - ep.annual_prob_max).powf(dt);
    assert!((ev.daily_prob_at(504) - exp_20).abs() < 1e-12);
}

#[test]
fn test_compile_normalizes_weights_and_skips_unknown_refs() {
    let dt = 1.0 / 252.0;
    let ep = EngineParams::default();
    let events = vec![ChaosEvent {
        id: "root".into(),
        name: "Root".into(),
        prob_schedule: vec![],
        drawdown_trigger: Some(DrawdownTrigger {
            asset: "NonExistent".into(),
            drop_from_peak_pct: 0.25,
        }),
        cooldown_days: 10,
        max_triggers_per_sim: 1,
        scenarios: vec![
            EventScenario {
                name: "S1".into(),
                weight: 2.0,
                modifiers: vec![
                    ModifierSpec {
                        mode: ModifierMode::Instant,
                        target: ParamTarget::AssetValue,
                        asset: Some("MissingAsset".into()),
                        op: ModOp::AddPct,
                        value: ValueDist::Fixed(-0.10),
                    },
                    ModifierSpec {
                        mode: ModifierMode::Temporary { duration_days: 30 },
                        target: ParamTarget::SellFee,
                        asset: None,
                        op: ModOp::Add,
                        value: ValueDist::Fixed(0.02),
                    },
                ],
                cascades: vec![
                    CascadeLink {
                        target_event_id: "MissingEvent".into(),
                        prob: 0.9,
                        delay_min_days: 5,
                        delay_max_days: 10,
                    },
                    CascadeLink {
                        target_event_id: "root".into(),
                        prob: 1.5,
                        delay_min_days: 0, // clamped to min 1
                        delay_max_days: 0, // clamped to >= delay_min
                    },
                ],
            },
            EventScenario {
                name: "S2".into(),
                weight: 6.0,
                modifiers: vec![],
                cascades: vec![],
            },
        ],
    }];

    let plan = CompiledChaosPlan::compile(&events, &sample_assets(), dt, &ep);
    let ev = &plan.events[0];
    assert!(ev.dd_asset.is_none());
    assert!((ev.scenarios[0].cum_weight - 0.25).abs() < 1e-12);
    assert!((ev.scenarios[1].cum_weight - 1.00).abs() < 1e-12);
    assert_eq!(ev.scenarios[0].modifiers.len(), 1);
    assert_eq!(ev.scenarios[0].cascades.len(), 1);
    assert_eq!(ev.scenarios[0].cascades[0].prob, 1.0);
    assert_eq!(ev.scenarios[0].cascades[0].delay_min, 1);
    assert_eq!(ev.scenarios[0].cascades[0].delay_max, 1);
}

