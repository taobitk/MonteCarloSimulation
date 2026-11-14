//! Hạt nguyên tử Distributions: Biến đổi phân phối thống kê.
//! Hàm thuần túy (pure functions), không trạng thái, không IO.

use std::f64::consts::PI;
use crate::rng::FastRng;

/// Biến đổi 2 số ngẫu nhiên đều u1, u2 trong (0, 1] sang 2 số phân phối chuẩn Gauss N(0, 1).
#[inline(always)]
pub fn box_muller(u1: f64, u2: f64) -> (f64, f64) {
    let safe_u1 = if u1 <= 1e-15 { 1e-15 } else { u1 };
    let r = (-2.0 * safe_u1.ln()).sqrt();
    let theta = 2.0 * PI * u2;
    (r * theta.cos(), r * theta.sin())
}

/// Sinh biến ngẫu nhiên Gamma(alpha, 1) dùng thuật toán Marsaglia and Tsang (2000).
/// Yêu cầu alpha >= 1.0.
pub fn standard_gamma(alpha: f64, rng: &mut FastRng) -> f64 {
    let d = alpha - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();
    loop {
        let (z, _) = box_muller(rng.next_f64(), rng.next_f64());
        let v = 1.0 + c * z;
        if v <= 0.0 {
            continue;
        }
        let v3 = v * v * v;
        let u = rng.next_f64();
        if u < 1.0 - 0.0331 * z * z * z * z {
            return d * v3;
        }
        if u.ln() < 0.5 * z * z + d * (1.0 - v3 + v3.ln()) {
            return d * v3;
        }
    }
}

/// Sinh biến ngẫu nhiên Chi-bình phương với bậc tự do df (Chi-Square distribution).
/// Khi df = k nguyên, nếu k chẵn thì Chi^2(k) = Gamma(k/2, 2).
#[inline(always)]
pub fn chi_squared(df: f64, rng: &mut FastRng) -> f64 {
    2.0 * standard_gamma(df * 0.5, rng)
}

/// Sinh biến ngẫu nhiên Student's t với bậc tự do df (Student's t-distribution - Fat-tailed).
/// T = Z / sqrt(V / df) với Z ~ N(0, 1), V ~ ChiSquare(df).
#[inline(always)]
pub fn student_t(df: f64, rng: &mut FastRng) -> f64 {
    let (z, _) = box_muller(rng.next_f64(), rng.next_f64());
    let v = chi_squared(df, rng);
    let scale = (v / df).sqrt();
    if scale <= 1e-15 {
        z
    } else {
        z / scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_muller_properties() {
        let mut rng = FastRng::new(54321);
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        let n = 50_000;

        for _ in 0..n {
            let u1 = rng.next_f64();
            let u2 = rng.next_f64();
            let (z1, z2) = box_muller(u1, u2);
            sum += z1 + z2;
            sum_sq += z1 * z1 + z2 * z2;
        }

        let total = (n * 2) as f64;
        let mean = sum / total;
        let variance = (sum_sq / total) - (mean * mean);

        assert!(mean.abs() < 0.02, "Mean should be close to 0: {}", mean);
        assert!((variance - 1.0).abs() < 0.05, "Variance should be close to 1: {}", variance);
    }

    #[test]
    fn test_student_t_fat_tails() {
        let mut rng = FastRng::new(99999);
        let n = 50_000;
        let mut outliers_3sigma = 0;

        for _ in 0..n {
            let val = student_t(3.0, &mut rng);
            if val.abs() > 3.0 {
                outliers_3sigma += 1;
            }
        }

        let rate = (outliers_3sigma as f64) / (n as f64);
        // Với Normal distribution, P(|Z| > 3) ~ 0.0027 (0.27%).
        // Với Student-t (df=3), P(|T| > 3) ~ 0.057 (5.7%) - Đuôi béo hơn rất nhiều!
        assert!(rate > 0.03, "Student-t should exhibit fat tails, rate: {}", rate);
    }
}
