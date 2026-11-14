//! Native Multi-Threaded Data-Driven Rust CLI cho mô phỏng DCA (AVX2/FMA, Zero-Alloc)
use serde::Deserialize;
use std::{env, fs, path::Path, thread, time::Instant};
use wasm_engine::models::{
    chaos_spec::ChaosEvent, cli_report::{merge_dca_results, CliRunOutput, CliSuiteOutput},
    dca_types::{DcaConfig, DcaSimResult}, run_dca_simulation,
};

#[derive(Deserialize)]
struct SweepSpec { name: String, risk_mult: f64, is_stress_test: bool }
#[derive(Deserialize)]
struct PortfolioEntry { name: String, config: DcaConfig }
#[derive(Deserialize)]
struct CliSuiteConfig {
    years: Vec<usize>, sims: usize, seed: u64, thread_seed_stride: u64,
    sweeps: Vec<SweepSpec>, portfolios: Vec<PortfolioEntry>,
}

fn run_parallel_dca(cfg: &DcaConfig, years: f64, steps: usize, sims: usize, seed: u64, stride: u64, threads: usize) -> DcaSimResult {
    let (base, rem) = (sims / threads, sims % threads);
    let mut results: Vec<DcaSimResult> = Vec::with_capacity(threads);
    thread::scope(|s| {
        let mut handles = Vec::with_capacity(threads);
        for i in 0..threads {
            let chunk = base + usize::from(i < rem);
            if chunk == 0 { continue; }
            let c_clone = cfg.clone();
            handles.push(s.spawn(move || run_dca_simulation(c_clone, years, steps, chunk, seed + (i as u64) * stride)));
        }
        for h in handles { results.push(h.join().expect("Worker panic")); }
    });
    merge_dca_results(results)
}

fn parse_arg(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}

fn default_out_path() -> String {
    if Path::new("static").exists() { "static/results/latest_suite.json".into() } else { "../static/results/latest_suite.json".into() }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cfg_path = parse_arg(&args, "--config").unwrap_or_else(|| "data/configs/default_dca_suite.json".to_string());
    let out_path = parse_arg(&args, "--out").unwrap_or_else(default_out_path);
    let raw_suite = fs::read_to_string(&cfg_path).unwrap_or_else(|e| panic!("Cannot read {cfg_path}: {e}"));
    let suite: CliSuiteConfig = serde_json::from_str(&raw_suite).expect("Invalid CLI suite JSON");

    let years_list: Vec<usize> = parse_arg(&args, "--years")
        .map(|s| s.split(',').filter_map(|v| v.trim().parse().ok()).collect())
        .unwrap_or_else(|| suite.years.clone());
    let sims = parse_arg(&args, "--sims").and_then(|s| s.parse().ok()).unwrap_or(suite.sims);
    let preset_filter = parse_arg(&args, "--preset").unwrap_or_else(|| "all".to_string());
    let fee_override: Option<f64> = parse_arg(&args, "--fee").and_then(|s| s.parse().ok());
    let chaos_events: Vec<ChaosEvent> = parse_arg(&args, "--chaos").map(|p| {
        serde_json::from_str(&fs::read_to_string(&p).expect("Failed to read chaos JSON")).expect("Invalid chaos JSON")
    }).unwrap_or_default();
    let threads = thread::available_parallelism().map(|n| n.get()).unwrap_or(8);

    println!("🔥 DATA-DRIVEN DCA CLI | Suite: {cfg_path} | Threads: {threads} | Sims: {sims} | ChaosEvents: {}", chaos_events.len());
    let t_total = Instant::now();
    let mut runs = Vec::new();

    for &y in &years_list {
        for p in suite.portfolios.iter().filter(|p| preset_filter == "all" || p.name.eq_ignore_ascii_case(&preset_filter)) {
            let steps = y * p.config.engine.trading_days_per_year.max(1);
            for sw in &suite.sweeps {
                let mut cfg = p.config.clone();
                cfg.is_stress_test = sw.is_stress_test;
                if let Some(f) = fee_override { if cfg.is_dynamic_dca { cfg.sell_fee_pct = f; } }
                if !chaos_events.is_empty() { cfg.chaos_events = chaos_events.clone(); }
                for a in &mut cfg.assets { a.sigma *= sw.risk_mult; }

                let t0 = Instant::now();
                let res = run_parallel_dca(&cfg, y as f64, steps, sims, suite.seed, suite.thread_seed_stride, threads);
                let elapsed_ms = t0.elapsed().as_millis();
                let last = res.bands.p50.len().saturating_sub(1);
                println!(
                    "[{y:>2}y | {:<18} | {:<9}] {:>6} ms | Contrib:{:>5.0} | P10:{:>8.1} P50:{:>8.1} P90:{:>9.1} ({:>5.2}x) | Win:{:>6.2}% | Sharpe:{:>5.2} | Breach:{:>5.2}% ({:>5}) | Sell:{:>5.2} Fee:{:>6.1} | Chaos:{:>4.2}",
                    p.name, sw.name, elapsed_ms, res.total_contributed, res.bands.p10[last], res.median_terminal_nav, res.bands.p90[last], res.moic_p50,
                    res.win_rate * 100.0, res.sharpe_ratio, res.breach_rate * 100.0, res.breach_count, res.avg_sell_count, res.avg_total_fees, res.avg_chaos_events
                );
                runs.push(CliRunOutput {
                    id: format!("{y}y_{}_{}", p.name, sw.name),
                    portfolio_name: p.name.clone(),
                    sweep_name: sw.name.clone(),
                    years: y,
                    horizon_days: steps,
                    is_dynamic_dca: cfg.is_dynamic_dca,
                    tolerance_band_pct: cfg.tolerance_band_pct,
                    sell_fee_pct: cfg.sell_fee_pct,
                    ruin_drawdown_pct: cfg.ruin_drawdown_pct,
                    initial_nav: cfg.initial_nav,
                    periodic_cash: cfg.periodic_cash,
                    assets: cfg.assets,
                    elapsed_ms,
                    result: res,
                });
            }
        }
    }

    let total_elapsed_ms = t_total.elapsed().as_millis();
    let suite_out = CliSuiteOutput { config_path: cfg_path, chaos_enabled: !chaos_events.is_empty(), chaos_events_count: chaos_events.len(), total_elapsed_ms, runs };
    if let Some(parent) = Path::new(&out_path).parent() { if !parent.as_os_str().is_empty() { let _ = fs::create_dir_all(parent); } }
    fs::write(&out_path, serde_json::to_string_pretty(&suite_out).expect("Failed to serialize JSON")).expect("Failed to write JSON output");
    println!("✅ TỔNG THỜI GIAN: {:.2}s | Đã xuất JSON -> {out_path}", (total_elapsed_ms as f64) / 1000.0);
}
