use wasm_engine::math::{
    build_crisis_matrix, chi_squared, extract_quantile_bands_inplace, multiply_lower_triangular,
    standard_gamma, update_top2_worst, WelfordStats,
};
use wasm_engine::rng::FastRng;

#[test]
fn test_quantile_bands_exact_values_and_empty_checkpoint() {
    let vals: Vec<f32> = (0..=100).rev().map(|x| x as f32).collect();
    let mut cps = vec![vec![], vals];
    let bands = extract_quantile_bands_inplace(&mut cps);

    assert_eq!(bands.p50.len(), 1);
    assert!((bands.p10[0] - 10.0).abs() < 1e-6);
    assert!((bands.p25[0] - 25.0).abs() < 1e-6);
    assert!((bands.p50[0] - 50.0).abs() < 1e-6);
    assert!((bands.p75[0] - 75.0).abs() < 1e-6);
    assert!((bands.p90[0] - 90.0).abs() < 1e-6);
    assert!(bands.p10[0] <= bands.p25[0] && bands.p25[0] <= bands.p50[0]);
    assert!(bands.p50[0] <= bands.p75[0] && bands.p75[0] <= bands.p90[0]);
}

#[test]
fn test_update_top2_worst_keeps_two_lowest_paths_sorted() {
    let mut worst = Vec::new();
    update_top2_worst(&mut worst, 0.80, &[100.0, 80.0]);
    update_top2_worst(&mut worst, 0.90, &[100.0, 90.0]);
    update_top2_worst(&mut worst, 0.95, &[100.0, 95.0]);
    update_top2_worst(&mut worst, 0.45, &[100.0, 45.0]);
    update_top2_worst(&mut worst, 0.60, &[100.0, 60.0]);

    assert_eq!(worst.len(), 2);
    assert!((worst[0].0 - 0.45).abs() < 1e-12);
    assert!((worst[1].0 - 0.60).abs() < 1e-12);
    assert_eq!(worst[0].1, vec![100.0, 45.0]);
    assert_eq!(worst[1].1, vec![100.0, 60.0]);
}

#[test]
fn test_crisis_matrix_bounds_and_lower_triangular_multiply() {
    let normal = vec![
        vec![1.0, -0.5, 0.995],
        vec![-0.5, 1.0, 0.1],
        vec![0.995, 0.1, 1.0],
    ];
    let crisis = build_crisis_matrix(&normal, 0.75);
    assert_eq!(crisis[0][0], 1.0);
    assert_eq!(crisis[1][1], 1.0);
    assert!((crisis[0][1] - 0.75).abs() < 1e-12);
    assert!((crisis[0][2] - 0.99).abs() < 1e-12);

    let l = vec![vec![2.0, 0.0], vec![-0.5, 1.5]];
    let z = [3.0, -2.0];
    let mut out = [0.0; 2];
    multiply_lower_triangular(&l, &z, &mut out);
    assert!((out[0] - 6.0).abs() < 1e-12);
    assert!((out[1] - (-4.5)).abs() < 1e-12);
}

#[test]
fn test_gamma_and_chi_squared_moments() {
    let mut rng = FastRng::new(2026);
    let n = 40_000;
    let mut gamma_sum = 0.0;
    let mut chi2_sum = 0.0;
    let mut chi2_sq_sum = 0.0;

    for _ in 0..n {
        let g = standard_gamma(3.5, &mut rng);
        let c = chi_squared(6.0, &mut rng);
        assert!(g > 0.0 && c > 0.0);
        gamma_sum += g;
        chi2_sum += c;
        chi2_sq_sum += c * c;
    }

    let gamma_mean = gamma_sum / (n as f64);
    let chi2_mean = chi2_sum / (n as f64);
    let chi2_var = (chi2_sq_sum / (n as f64)) - chi2_mean * chi2_mean;
    assert!((gamma_mean - 3.5).abs() < 0.06, "gamma_mean={gamma_mean}");
    assert!((chi2_mean - 6.0).abs() < 0.08, "chi2_mean={chi2_mean}");
    assert!((chi2_var - 12.0).abs() < 0.5, "chi2_var={chi2_var}");
}

#[test]
fn test_welford_empty_merge_and_single_sample_variance() {
    let mut empty = WelfordStats::new();
    let mut one = WelfordStats::new();
    one.update(42.0);
    assert_eq!(one.sample_variance(), 0.0);

    one.merge(&empty);
    assert_eq!(one.count, 1);
    assert_eq!(one.mean, 42.0);

    empty.merge(&one);
    assert_eq!(empty.count, 1);
    assert_eq!(empty.min, 42.0);
    assert_eq!(empty.max, 42.0);
}

