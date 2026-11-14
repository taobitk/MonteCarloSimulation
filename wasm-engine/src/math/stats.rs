//! Hạt nguyên tử Stats: Thống kê dạng luồng (Online Streaming Statistics).
//! Dùng thuật toán Welford (1962) và Chan-Golub-LeVeque (1979) để gộp đa luồng.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelfordStats {
    pub count: u64,
    pub mean: f64,
    pub m2: f64,
    pub min: f64,
    pub max: f64,
}

impl Default for WelfordStats {
    fn default() -> Self {
        Self::new()
    }
}

impl WelfordStats {
    pub fn new() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            m2: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    /// Cập nhật luồng với 1 mẫu số mới (O(1) time, O(1) space).
    #[inline(always)]
    pub fn update(&mut self, x: f64) {
        self.count += 1;
        let delta = x - self.mean;
        self.mean += delta / (self.count as f64);
        let delta2 = x - self.mean;
        self.m2 += delta * delta2;

        if x < self.min { self.min = x; }
        if x > self.max { self.max = x; }
    }

    /// Gộp thống kê từ luồng/worker khác theo thuật toán Chan-Golub-LeVeque (1979).
    pub fn merge(&mut self, other: &WelfordStats) {
        if other.count == 0 { return; }
        if self.count == 0 {
            *self = other.clone();
            return;
        }

        let new_count = self.count + other.count;
        let delta = other.mean - self.mean;
        let n_a = self.count as f64;
        let n_b = other.count as f64;
        let n_ab = new_count as f64;

        self.mean += delta * (n_b / n_ab);
        self.m2 += other.m2 + delta * delta * (n_a * n_b / n_ab);
        self.count = new_count;

        if other.min < self.min { self.min = other.min; }
        if other.max > self.max { self.max = other.max; }
    }

    /// Phương sai mẫu (Sample Variance).
    #[inline(always)]
    pub fn sample_variance(&self) -> f64 {
        if self.count < 2 { 0.0 } else { self.m2 / ((self.count - 1) as f64) }
    }

    /// Độ lệch chuẩn mẫu (Sample Standard Deviation).
    #[inline(always)]
    pub fn sample_std_dev(&self) -> f64 {
        self.sample_variance().sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welford_accuracy() {
        let mut stats = WelfordStats::new();
        for &val in &[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0] {
            stats.update(val);
        }
        assert_eq!(stats.count, 8);
        assert!((stats.mean - 5.0).abs() < 1e-10);
        assert!((stats.sample_variance() - 4.5714285714).abs() < 1e-6);
        assert!((stats.sample_std_dev() - 2.1380899352).abs() < 1e-6);
        assert_eq!(stats.min, 2.0);
        assert_eq!(stats.max, 9.0);
    }

    #[test]
    fn test_parallel_merge_equivalence() {
        let mut s1 = WelfordStats::new();
        for &x in &[10.0, 20.0, 30.0, 40.0, 50.0] { s1.update(x); }

        let mut s2 = WelfordStats::new();
        for &x in &[60.0, 70.0, 80.0, 90.0, 100.0] { s2.update(x); }

        s1.merge(&s2);

        let mut direct = WelfordStats::new();
        for &x in &[10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0] {
            direct.update(x);
        }

        assert_eq!(s1.count, direct.count);
        assert!((s1.mean - direct.mean).abs() < 1e-12);
        assert!((s1.sample_variance() - direct.sample_variance()).abs() < 1e-12);
        assert_eq!(s1.min, direct.min);
        assert_eq!(s1.max, direct.max);
    }
}
