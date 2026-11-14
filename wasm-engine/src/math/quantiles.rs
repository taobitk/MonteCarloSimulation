//! In-place Quantile extraction & Streaming Top-2 Worst Path filter
use crate::models::engine_params::QuantileBand;

#[inline]
pub fn extract_quantile_bands_inplace(nav_by_cp: &mut [Vec<f32>]) -> QuantileBand {
    let num_cps = nav_by_cp.len();
    let mut bands = QuantileBand {
        p10: Vec::with_capacity(num_cps), p25: Vec::with_capacity(num_cps),
        p50: Vec::with_capacity(num_cps), p75: Vec::with_capacity(num_cps),
        p90: Vec::with_capacity(num_cps),
    };
    for cp_vec in nav_by_cp.iter_mut() {
        let n = cp_vec.len();
        if n == 0 { continue; }
        let idx90 = ((n as f64 * 0.90) as usize).min(n - 1);
        let idx75 = (n as f64 * 0.75) as usize;
        let idx50 = (n as f64 * 0.50) as usize;
        let idx25 = (n as f64 * 0.25) as usize;
        let idx10 = (n as f64 * 0.10) as usize;

        if n < 10 {
            cp_vec.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        } else {
            cp_vec.select_nth_unstable_by(idx90, |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            cp_vec[..idx90].select_nth_unstable_by(idx75, |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            cp_vec[..idx75].select_nth_unstable_by(idx50, |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            cp_vec[..idx50].select_nth_unstable_by(idx25, |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            cp_vec[..idx25].select_nth_unstable_by(idx10, |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        }

        bands.p10.push(cp_vec[idx10]); bands.p25.push(cp_vec[idx25]);
        bands.p50.push(cp_vec[idx50]); bands.p75.push(cp_vec[idx75]);
        bands.p90.push(cp_vec[idx90]);
    }
    bands
}

#[inline]
pub fn update_top2_worst(worst: &mut Vec<(f64, Vec<f32>)>, min_nav: f64, path_buf: &[f32]) {
    if worst.len() < 2 {
        worst.push((min_nav, path_buf.to_vec()));
        worst.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    } else if min_nav < worst[1].0 {
        worst[1] = (min_nav, path_buf.to_vec());
        worst.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }
}

