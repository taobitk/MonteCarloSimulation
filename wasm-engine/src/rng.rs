//! Hạt nguyên tử RNG: Xoshiro256** Pseudo-Random Number Generator.
//! Thuần túy bitwise, không phụ thuộc thư viện ngoài, tối ưu tuyệt đối cho WASM.

#[derive(Debug, Clone)]
pub struct FastRng {
    s: [u64; 4],
}

impl FastRng {
    /// Khởi tạo RNG từ một seed u64 (dùng SplitMix64 để trải đều entropy).
    pub fn new(seed: u64) -> Self {
        let mut sm_state = seed;
        let mut s = [0u64; 4];
        for item in &mut s {
            sm_state = sm_state.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = sm_state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            *item = z ^ (z >> 31);
        }
        Self { s }
    }

    /// Sinh số nguyên u64 ngẫu nhiên kế tiếp theo thuật toán Xoshiro256**.
    #[inline(always)]
    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;

        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];

        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);

        result
    }

    /// Sinh số thực f64 phân phối đều trong nửa khoảng [0.0, 1.0).
    #[inline(always)]
    pub fn next_f64(&mut self) -> f64 {
        // Lấy 53 bits có nghĩa cho chuẩn IEEE 754 float
        let v = self.next_u64() >> 11;
        (v as f64) * (1.0 / ((1u64 << 53) as f64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_range() {
        let mut rng = FastRng::new(12345);
        for _ in 0..10_000 {
            let val = rng.next_f64();
            assert!((0.0..1.0).contains(&val));
        }
    }

    #[test]
    fn test_rng_determinism() {
        let mut rng1 = FastRng::new(999);
        let mut rng2 = FastRng::new(999);
        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }
}

