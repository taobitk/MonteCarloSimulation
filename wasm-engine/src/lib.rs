//! Core DCA & Chaos Monkey Monte Carlo Simulation Library

pub mod math;
pub mod models;
pub mod rng;

#[cfg(test)]
mod tests {
    use super::models::{run_dca_simulation, AssetSpec, DcaConfig};

    #[test]
    fn test_dca_stress_10yr_importance_sampling_normalized() {
        let base_cfg = |is_stress_test: bool| DcaConfig {
            initial_nav: 100.0, periodic_cash: 10.0, dca_interval_days: 21,
            is_dynamic_dca: true, tolerance_band_pct: 0.10, sell_fee_pct: 0.005,
            ruin_drawdown_pct: 0.20, is_stress_test, is_fat_tail: true,
            assets: vec![
                AssetSpec { name: "Cash".into(), weight: 0.10, mu: 0.055, sigma: 0.01 },
                AssetSpec { name: "Crypto".into(), weight: 0.10, mu: 0.25, sigma: 0.60 },
                AssetSpec { name: "Gold".into(), weight: 0.40, mu: 0.095, sigma: 0.17 },
                AssetSpec { name: "Stock".into(), weight: 0.40, mu: 0.105, sigma: 0.18 },
            ],
            corr_matrix: vec![
                vec![1.0, 0.0, 0.10, -0.15],
                vec![0.0, 1.0, 0.20, 0.30],
                vec![0.10, 0.20, 1.0, 0.05],
                vec![-0.15, 0.30, 0.05, 1.0],
            ],
            normal_corr: 0.10, crisis_corr: 0.85, crisis_threshold_pct: 0.15, chaos_events: vec![], engine: Default::default(),
        };
        let normal = run_dca_simulation(base_cfg(false), 10.0, 2520, 2000, 42);
        let stress = run_dca_simulation(base_cfg(true), 10.0, 2520, 2000, 42);
        assert!(stress.breach_count > normal.breach_count, "stress={} <= normal={}", stress.breach_count, normal.breach_count);
        assert!(stress.breach_rate > 0.0 && stress.breach_rate < 0.30, "unexpected breach_rate={}", stress.breach_rate);
        assert!(stress.win_rate > 0.50, "unexpected win_rate={}", stress.win_rate);
    }

    #[test]
    fn test_chaos_multi_milestone_scenarios_and_cascades() {
        let json = r#"[
          {
            "id": "ev_a",
            "prob_schedule": [{ "from_year": 0.0, "annual_prob": 0.0 }, { "from_year": 1.0, "annual_prob": 0.60 }],
            "cooldown_days": 252, "max_triggers_per_sim": 2,
            "scenarios": [
              { "name": "Light", "weight": 0.5, "modifiers": [{ "mode": "Instant", "target": "AssetValue", "asset": "Stock", "op": "AddPct", "value": -0.20 }], "cascades": [{ "target_event_id": "ev_b", "prob": 1.0, "delay_min_days": 5, "delay_max_days": 5 }] },
              { "name": "TempFee", "weight": 0.5, "modifiers": [{ "mode": "Temporary", "duration_days": 63, "target": "SellFee", "op": "Set", "value": 0.08 }], "cascades": [] }
            ]
          },
          {
            "id": "ev_b", "prob_schedule": [], "cooldown_days": 252, "max_triggers_per_sim": 1,
            "scenarios": [{ "name": "PermBreak", "weight": 1.0, "modifiers": [{ "mode": "Permanent", "target": "AssetMu", "asset": "Stock", "op": "Set", "value": 0.02 }], "cascades": [] }]
          }
        ]"#;
        let chaos_events: Vec<super::models::ChaosEvent> = serde_json::from_str(json).unwrap();
        let cfg = DcaConfig {
            initial_nav: 100.0, periodic_cash: 10.0, dca_interval_days: 21,
            is_dynamic_dca: true, tolerance_band_pct: 0.10, sell_fee_pct: 0.02,
            ruin_drawdown_pct: 0.20, is_stress_test: false, is_fat_tail: false,
            assets: vec![AssetSpec { name: "Stock".into(), weight: 1.0, mu: 0.12, sigma: 0.18 }],
            corr_matrix: vec![vec![1.0]], normal_corr: 0.0, crisis_corr: 0.85, crisis_threshold_pct: 0.15, chaos_events, engine: Default::default(),
        };
        let res = run_dca_simulation(cfg, 5.0, 1260, 500, 7);
        assert!(res.avg_chaos_events > 1.0, "expected milestone + cascade fires, got {}", res.avg_chaos_events);
    }

    #[test]
    fn test_engine_params_and_pareto_value_dist() {
        let json = r#"{
          "engine": { "fat_tail_df": 6.0, "shock_clamp": 6.0, "sigma_max": 4.0, "risk_free_rate": 0.02 },
          "chaos_events": [{
            "id": "pareto_crash", "prob_schedule": [{ "from_year": 0.0, "annual_prob": 0.50 }], "cooldown_days": 63,
            "scenarios": [{
              "weight": 1.0,
              "modifiers": [{ "mode": "Instant", "target": "AssetValue", "asset": "Stock", "op": "AddPct", "value": { "dist": "Pareto", "scale": 0.10, "alpha": 2.5, "sign": -1.0, "max_abs": 0.70 } }]
            }]
          }]
        }"#;
        #[derive(serde::Deserialize)]
        struct Wrapper { engine: super::models::EngineParams, chaos_events: Vec<super::models::ChaosEvent> }
        let w: Wrapper = serde_json::from_str(json).unwrap();
        let cfg = DcaConfig {
            initial_nav: 100.0, periodic_cash: 10.0, dca_interval_days: 21, is_dynamic_dca: false, tolerance_band_pct: 0.10, sell_fee_pct: 0.0,
            ruin_drawdown_pct: 0.20, is_stress_test: false, is_fat_tail: true,
            assets: vec![AssetSpec { name: "Stock".into(), weight: 1.0, mu: 0.10, sigma: 0.20 }],
            corr_matrix: vec![vec![1.0]], normal_corr: 0.0, crisis_corr: 0.85, crisis_threshold_pct: 0.15,
            chaos_events: w.chaos_events, engine: w.engine,
        };
        let res = run_dca_simulation(cfg, 3.0, 756, 300, 99);
        assert!(res.avg_chaos_events > 0.5, "expected pareto chaos fires, got {}", res.avg_chaos_events);
    }

    #[test]
    fn test_unit_price_dd_trigger_ignores_rebalance_sells() {
        let json = r#"[{"id":"dd","drawdown_trigger":{"asset":"Stock","drop_from_peak_pct":0.30},"cooldown_days":21,"max_triggers_per_sim":3,"scenarios":[{"weight":1.0,"modifiers":[]}]}]"#;
        let chaos_events: Vec<super::models::ChaosEvent> = serde_json::from_str(json).unwrap();
        let cfg = DcaConfig {
            initial_nav: 100.0, periodic_cash: 10.0, dca_interval_days: 21, is_dynamic_dca: true, tolerance_band_pct: 0.05, sell_fee_pct: 0.0,
            ruin_drawdown_pct: 0.20, is_stress_test: false, is_fat_tail: false,
            assets: vec![
                AssetSpec { name: "Cash".into(), weight: 0.5, mu: 0.0, sigma: 0.005 },
                AssetSpec { name: "Stock".into(), weight: 0.5, mu: 0.50, sigma: 0.005 },
            ],
            corr_matrix: vec![vec![1.0, 0.0], vec![0.0, 1.0]], normal_corr: 0.0, crisis_corr: 0.5, crisis_threshold_pct: 0.2, chaos_events, engine: Default::default(),
        };
        let res = run_dca_simulation(cfg, 2.0, 504, 50, 11);
        assert_eq!(res.avg_chaos_events, 0.0, "rebalance sells must not trigger unit-price drawdown");
    }

    #[test]
    fn test_actual_contrib_and_portfolio_nav_accounting() {
        let json = r#"[{"id":"cut","prob_schedule":[{"from_year":0.0,"annual_prob":0.999}],"cooldown_days":999,"max_triggers_per_sim":1,"scenarios":[{"weight":1.0,"modifiers":[{"mode":"Temporary","duration_days":999,"target":"PeriodicCash","op":"Set","value":0.0},{"mode":"Instant","target":"PortfolioNav","op":"AddPct","value":-0.10}]}]}]"#;
        let chaos_events: Vec<super::models::ChaosEvent> = serde_json::from_str(json).unwrap();
        let cfg = DcaConfig {
            initial_nav: 100.0, periodic_cash: 10.0, dca_interval_days: 21, is_dynamic_dca: false, tolerance_band_pct: 0.1, sell_fee_pct: 0.0,
            ruin_drawdown_pct: 0.50, is_stress_test: false, is_fat_tail: false,
            assets: vec![AssetSpec { name: "Stock".into(), weight: 1.0, mu: 0.15, sigma: 0.005 }],
            corr_matrix: vec![vec![1.0]], normal_corr: 0.0, crisis_corr: 0.5, crisis_threshold_pct: 0.2, chaos_events, engine: Default::default(),
        };
        let res = run_dca_simulation(cfg, 2.0, 504, 50, 22);
        assert!(res.avg_actual_contributed < res.total_contributed * 0.6, "actual={} vs planned={}", res.avg_actual_contributed, res.total_contributed);
        assert!(res.win_rate > 0.90, "win_rate should reflect actual sim_contrib, got {}", res.win_rate);
    }
}
