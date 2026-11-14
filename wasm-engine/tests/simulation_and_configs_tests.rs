use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use wasm_engine::math::cholesky_with_psd_repair;
use wasm_engine::models::chaos_spec::{CompiledChaosPlan, ParamTarget};
use wasm_engine::models::{run_dca_simulation, AssetSpec, ChaosEvent, DcaConfig};

#[derive(Deserialize)]
struct SuitePortfolio { name: String, config: DcaConfig }
#[derive(Deserialize)]
struct SuiteConfig { years: Vec<usize>, portfolios: Vec<SuitePortfolio> }

fn load_data_file(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("Failed reading {}: {e}", p.display()))
}

fn two_asset_cfg(is_dynamic_dca: bool, band: f64, interval: usize) -> DcaConfig {
    DcaConfig {
        initial_nav: 100.0, periodic_cash: 10.0, dca_interval_days: interval,
        is_dynamic_dca, tolerance_band_pct: band, sell_fee_pct: 0.02,
        ruin_drawdown_pct: 0.25, is_stress_test: false, is_fat_tail: true,
        assets: vec![
            AssetSpec { name: "Tiền gửi".into(), weight: 0.30, mu: 0.055, sigma: 0.01 },
            AssetSpec { name: "Cổ phiếu".into(), weight: 0.70, mu: 0.11, sigma: 0.22 },
        ],
        corr_matrix: vec![vec![1.0, -0.1], vec![-0.1, 1.0]],
        normal_corr: -0.1, crisis_corr: 0.85, crisis_threshold_pct: 0.15,
        chaos_events: vec![], engine: Default::default(),
    }
}

#[test]
fn test_simulation_seed_determinism_and_zero_interval_contrib() {
    let cfg = two_asset_cfg(true, 0.05, 0);
    let r1 = run_dca_simulation(cfg.clone(), 2.0, 504, 60, 1234);
    let r2 = run_dca_simulation(cfg, 2.0, 504, 60, 1234);

    assert_eq!(r1.median_terminal_nav.to_bits(), r2.median_terminal_nav.to_bits());
    assert_eq!(r1.bands.p50, r2.bands.p50);
    assert_eq!(r1.total_contributed, 100.0);
    assert_eq!(r1.avg_actual_contributed, 100.0);
}

#[test]
fn test_static_zero_fees_and_wider_band_reduces_sells() {
    let stat = run_dca_simulation(two_asset_cfg(false, 0.05, 21), 5.0, 1260, 100, 7);
    assert_eq!(stat.avg_sell_count, 0.0);
    assert_eq!(stat.avg_total_fees, 0.0);

    let tight = run_dca_simulation(two_asset_cfg(true, 0.02, 21), 5.0, 1260, 100, 7);
    let wide = run_dca_simulation(two_asset_cfg(true, 0.25, 21), 5.0, 1260, 100, 7);
    assert!(tight.avg_sell_count >= wide.avg_sell_count);
}

#[test]
fn test_default_dca_suite_json_valid_weights_and_psd() {
    let suite: SuiteConfig = serde_json::from_str(&load_data_file("data/configs/default_dca_suite.json")).unwrap();
    assert_eq!(suite.years, vec![10, 20, 40]);
    assert_eq!(suite.portfolios.len(), 6);

    for p in &suite.portfolios {
        let w_sum: f64 = p.config.assets.iter().map(|a| a.weight).sum();
        assert!((w_sum - 1.0).abs() < 1e-6, "{} weights sum to {w_sum}", p.name);
        let (_l, repaired) = cholesky_with_psd_repair(&p.config.corr_matrix);
        assert!(!repaired, "{} corr_matrix required PSD repair", p.name);
    }

    let smoke = run_dca_simulation(suite.portfolios[0].config.clone(), 1.0, 252, 16, 42);
    assert!(smoke.median_terminal_nav > 0.0);
}

#[test]
fn test_chaos_vn_32_events_compiles_cleanly_on_real_portfolios() {
    let events: Vec<ChaosEvent> = serde_json::from_str(&load_data_file("data/configs/chaos_vn_32_events.json")).unwrap();
    assert_eq!(events.len(), 32);

    let mut ids = HashSet::new();
    for ev in &events {
        assert!(ids.insert(ev.id.as_str()), "Duplicate chaos event id: {}", ev.id);
        let w_sum: f64 = ev.scenarios.iter().map(|s| s.weight).sum();
        assert!(w_sum > 0.0, "{} scenarios have zero total weight", ev.id);
    }

    let suite: SuiteConfig = serde_json::from_str(&load_data_file("data/configs/default_dca_suite.json")).unwrap();
    let cfg4 = &suite.portfolios[0].config;
    let plan4 = CompiledChaosPlan::compile(&events, &cfg4.assets, 1.0 / 252.0, &cfg4.engine);

    for (ev, cev) in events.iter().zip(plan4.events.iter()) {
        if let Some(dd) = &ev.drawdown_trigger {
            assert!(cev.dd_asset.is_some(), "{} unknown dd asset {}", ev.id, dd.asset);
        }
        for (sc, csc) in ev.scenarios.iter().zip(cev.scenarios.iter()) {
            assert_eq!(csc.modifiers.len(), sc.modifiers.len(), "{} dropped modifier in {}", ev.id, sc.name);
            assert_eq!(csc.cascades.len(), sc.cascades.len(), "{} dropped cascade in {}", ev.id, sc.name);
            for m in &csc.modifiers {
                assert!(matches!(m.target, ParamTarget::AssetValue | ParamTarget::AssetMu | ParamTarget::AssetSigma | ParamTarget::PeriodicCash | ParamTarget::SellFee | ParamTarget::RebalanceEnabled | ParamTarget::CrisisCorrActive | ParamTarget::PortfolioNav));
            }
        }
    }

    let mut cfg_with_chaos = cfg4.clone();
    cfg_with_chaos.chaos_events = events;
    let res = run_dca_simulation(cfg_with_chaos, 5.0, 1260, 24, 99);
    assert!(res.avg_chaos_events > 0.0 && res.median_terminal_nav.is_finite());
}

