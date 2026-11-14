//! Mô hình Mô phỏng DCA Tĩnh & Động Zero-Alloc kèm Điểm gãy, Đơn giá & Phí bán
use super::chaos_runtime::ChaosSimState;
use super::chaos_spec::CompiledChaosPlan;
use super::dca_types::{build_dca_summary, DcaConfig, DcaSimResult};
use crate::math::{apply_dca_inflow_and_rebalance, box_muller, build_crisis_matrix, chi_squared,
    cholesky_with_psd_repair, extract_quantile_bands_inplace, multiply_lower_triangular, update_top2_worst, WelfordStats};
use crate::rng::FastRng;

pub fn run_dca_simulation(cfg: DcaConfig, time_years: f64, steps: usize, num_sims: usize, seed: u64) -> DcaSimResult {
    let mut rng = FastRng::new(seed);
    let dt = time_years / (steps as f64);
    let ep = cfg.engine;
    let (sqrt_dt, n_assets) = (dt.sqrt(), cfg.assets.len().max(1));
    let cp_stride = (steps / ep.trading_days_per_year.max(1)).max(1);
    let mut num_cps = 1;
    for s in 1..=steps { if s % cp_stride == 0 || s == steps { num_cps += 1; } }

    let mut nav_by_cp = vec![Vec::with_capacity(num_sims); num_cps];
    let (mut contrib_band, mut floor_band) = (Vec::with_capacity(num_cps), Vec::with_capacity(num_cps));
    let mut running_contrib = cfg.initial_nav;
    contrib_band.push(running_contrib as f32);
    floor_band.push((running_contrib * (1.0 - cfg.ruin_drawdown_pct)) as f32);
    for s in 1..=steps {
        if cfg.dca_interval_days > 0 && s % cfg.dca_interval_days == 0 { running_contrib += cfg.periodic_cash; }
        if s % cp_stride == 0 || s == steps {
            contrib_band.push(running_contrib as f32);
            floor_band.push((running_contrib * (1.0 - cfg.ruin_drawdown_pct)) as f32);
        }
    }
    let total_contributed = running_contrib;
    let mut path_buf = Vec::with_capacity(num_cps);
    let (mut worst_candidates, mut breach_count, mut total_breach_days) = (Vec::with_capacity(2), 0, 0);
    let (mut total_sells, mut total_fees, mut win_count, mut actual_contrib_sum) = (0, 0.0, 0, 0.0);
    let (mut terminal_navs, mut net_profits, mut moics) = (Vec::with_capacity(num_sims), Vec::with_capacity(num_sims), Vec::with_capacity(num_sims));
    let mut asset_ruin_drops = vec![0.0; n_assets];
    let drifts: Vec<f64> = cfg.assets.iter().map(|a| (a.mu - 0.5 * a.sigma * a.sigma) * dt).collect();
    let (sigmas, weights): (Vec<f64>, Vec<f64>) = (cfg.assets.iter().map(|a| a.sigma).collect(), cfg.assets.iter().map(|a| a.weight).collect());
    let chaos_plan = CompiledChaosPlan::compile(&cfg.chaos_events, &cfg.assets, dt, &ep);
    let (has_chaos, norm_factor) = (!chaos_plan.is_empty(), ep.fat_tail_norm());
    let mut chaos = ChaosSimState::new(n_assets, chaos_plan.events.len());
    let mut matrix = if cfg.corr_matrix.len() == n_assets { cfg.corr_matrix.clone() } else {
        let mut m = vec![vec![cfg.normal_corr; n_assets]; n_assets];
        for i in 0..n_assets { m[i][i] = 1.0; }
        m
    };
    for i in 0..n_assets { matrix[i][i] = 1.0; }
    let ((l_normal, psd_repaired), (l_crisis, _)) = (cholesky_with_psd_repair(&matrix), cholesky_with_psd_repair(&build_crisis_matrix(&matrix, cfg.crisis_corr)));
    let theta = if cfg.is_stress_test { ep.is_tilt_strength / ((steps * n_assets).max(1) as f64).sqrt() } else { 0.0 };
    let (half_theta_sq, mut sum_is_weights, mut ret_stats) = (0.5 * theta * theta, 0.0, WelfordStats::new());
    let (mut vals, mut unit_prices, mut peak_unit_prices) = (vec![0.0; n_assets], vec![1.0; n_assets], vec![1.0; n_assets]);
    let (mut z_raw, mut shocks) = (vec![0.0; n_assets], vec![0.0; n_assets]);

    for _ in 0..num_sims {
        if has_chaos { chaos.reset(&cfg, dt); }
        for (i, a) in cfg.assets.iter().enumerate() { vals[i] = cfg.initial_nav * a.weight; unit_prices[i] = 1.0; peak_unit_prices[i] = 1.0; }
        path_buf.clear(); path_buf.push(cfg.initial_nav as f32);
        nav_by_cp[0].push(cfg.initial_nav as f32);
        let (mut peak_nav, mut breached, mut breach_day) = (cfg.initial_nav, false, 0);
        let (mut min_path_ratio, mut sim_contrib, mut cp_idx, mut log_lik) = (1.0, cfg.initial_nav, 0, 0.0);

        for step in 1..=steps {
            if has_chaos { chaos.step(&chaos_plan, step, dt, &ep, &mut vals, &mut unit_prices, &mut peak_unit_prices, &mut rng); }
            let is_cp = step % cp_stride == 0 || step == steps;
            let cur_nav: f64 = vals.iter().sum();
            if cur_nav > peak_nav { peak_nav = cur_nav; }
            let dd = (peak_nav - cur_nav) / peak_nav.max(1e-6);
            let l_curr = if dd >= cfg.crisis_threshold_pct || (has_chaos && chaos.eff_crisis_corr) { &l_crisis } else { &l_normal };
            for z in z_raw.iter_mut() { *z = box_muller(rng.next_f64(), rng.next_f64()).0 + theta; if !breached { log_lik += -theta * (*z) + half_theta_sq; } }
            multiply_lower_triangular(l_curr, &z_raw, &mut shocks);
            if cfg.is_fat_tail { let scale = (chi_squared(ep.fat_tail_df, &mut rng) / ep.fat_tail_df).sqrt().max(1e-12); for s in shocks.iter_mut() { *s = (*s / scale) * norm_factor; } }
            let (d_slice, s_slice) = if has_chaos { (&chaos.eff_drifts[..], &chaos.eff_sigmas[..]) } else { (&drifts[..], &sigmas[..]) };
            for i in 0..n_assets {
                let ret_fac = (d_slice[i] + s_slice[i] * sqrt_dt * shocks[i].clamp(-ep.shock_clamp, ep.shock_clamp)).exp();
                vals[i] *= ret_fac;
                unit_prices[i] *= ret_fac;
                if unit_prices[i] > peak_unit_prices[i] { peak_unit_prices[i] = unit_prices[i]; }
            }
            if cfg.dca_interval_days > 0 && step % cfg.dca_interval_days == 0 {
                let (cash, allow_sell, fee_pct) = if has_chaos { (chaos.eff_cash, chaos.eff_rebal, chaos.eff_fee) } else { (cfg.periodic_cash, cfg.is_dynamic_dca, cfg.sell_fee_pct) };
                sim_contrib += cash;
                let (_, sold, fee) = apply_dca_inflow_and_rebalance(&mut vals, &weights, cash, cfg.is_dynamic_dca, allow_sell, cfg.tolerance_band_pct, fee_pct);
                if sold { total_sells += 1; total_fees += fee; }
            }
            let next_nav: f64 = vals.iter().sum();
            let dyn_floor = sim_contrib * (1.0 - cfg.ruin_drawdown_pct);
            let nav_ratio = next_nav / sim_contrib.max(1e-6);
            if nav_ratio < min_path_ratio { min_path_ratio = nav_ratio; }
            if is_cp { cp_idx += 1; path_buf.push(next_nav as f32); nav_by_cp[cp_idx].push(next_nav as f32); }
            if next_nav <= dyn_floor && !breached {
                breached = true; breach_day = step;
                for i in 0..n_assets {
                    let asset_dd = (peak_unit_prices[i] - unit_prices[i]).max(0.0) / peak_unit_prices[i].max(1e-6);
                    asset_ruin_drops[i] += cfg.assets[i].weight * asset_dd;
                }
            }
        }
        if breached {
            breach_count += 1; total_breach_days += breach_day;
            sum_is_weights += if cfg.is_stress_test { log_lik.exp().min(ep.is_weight_cap) } else { 1.0 };
            update_top2_worst(&mut worst_candidates, min_path_ratio, &path_buf);
        }
        let term_nav: f64 = vals.iter().sum();
        actual_contrib_sum += sim_contrib;
        if term_nav > sim_contrib { win_count += 1; }
        terminal_navs.push(term_nav);
        net_profits.push(term_nav - sim_contrib);
        moics.push(term_nav / sim_contrib.max(1.0));
        ret_stats.update((term_nav - sim_contrib) / sim_contrib.max(1.0));
    }

    let worst_paths = worst_candidates.into_iter().map(|(_, p)| p).collect();
    let bands = extract_quantile_bands_inplace(&mut nav_by_cp);
    build_dca_summary(
        num_sims, bands, contrib_band, floor_band, worst_paths, total_contributed, actual_contrib_sum,
        breach_count, total_breach_days, sum_is_weights, cfg.is_stress_test, win_count, terminal_navs,
        net_profits, moics, ret_stats, &ep, total_sells, total_fees, chaos.total_events_fired, asset_ruin_drops, psd_repaired,
    )
}
