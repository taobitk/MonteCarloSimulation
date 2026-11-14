//! Phân rã Cholesky N x N & Hiệu chỉnh Ma trận Tương quan Xác định Dương (PSD)

/// Phân rã Cholesky L * L^T = A cho ma trận đối xứng N x N.
/// Trả về Ma trận tam giác dưới L nếu A là Positive Definite, hoặc None nếu không khả thi.
pub fn try_cholesky(matrix: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = matrix.len();
    if n == 0 { return Some(Vec::new()); }
    let mut l = vec![vec![0.0; n]; n];

    for j in 0..n {
        let mut sum_diag = 0.0;
        for k in 0..j {
            sum_diag += l[j][k] * l[j][k];
        }
        let val = matrix[j][j] - sum_diag;
        if val <= 1e-12 {
            return None;
        }
        let l_jj = val.sqrt();
        l[j][j] = l_jj;

        for i in (j + 1)..n {
            let mut sum_non_diag = 0.0;
            for k in 0..j {
                sum_non_diag += l[i][k] * l[j][k];
            }
            l[i][j] = (matrix[i][j] - sum_non_diag) / l_jj;
        }
    }
    Some(l)
}

/// Tự động kiểm tra và hiệu chỉnh Shrinkage Σ' = (1 - α)Σ + α*I nếu ma trận vi phạm PSD.
/// Trả về (Ma trận Cholesky L, cờ psd_repaired).
pub fn cholesky_with_psd_repair(matrix: &[Vec<f64>]) -> (Vec<Vec<f64>>, bool) {
    let n = matrix.len();
    if n <= 1 {
        let val = if n == 1 { matrix[0][0].max(1e-6).sqrt() } else { 1.0 };
        return (vec![vec![val]], false);
    }

    if let Some(l) = try_cholesky(matrix) {
        return (l, false);
    }

    // Shrinkage binary search tìm alpha nhỏ nhất trong [0.01, 0.99] để ma trận khả thi
    let mut low = 0.01;
    let mut high = 0.99;
    let mut best_l = vec![vec![0.0; n]; n];
    for i in 0..n { best_l[i][i] = 1.0; } // Fallback ma trận đơn vị

    for _ in 0..25 {
        let mid = (low + high) * 0.5;
        let mut shrunk = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                let diag = if i == j { 1.0 } else { 0.0 };
                shrunk[i][j] = (1.0 - mid) * matrix[i][j] + mid * diag;
            }
        }
        if let Some(l) = try_cholesky(&shrunk) {
            best_l = l;
            high = mid; // Thử co nhỏ hơn
        } else {
            low = mid;  // Cần co mạnh hơn
        }
    }

    (best_l, true)
}

/// Tạo ma trận tương quan hoảng loạn khi sập sâu (kéo mọi cặp hội tụ về max(ρ_ij, crisis_corr))
pub fn build_crisis_matrix(normal: &[Vec<f64>], crisis_corr: f64) -> Vec<Vec<f64>> {
    let n = normal.len();
    let mut crisis = vec![vec![1.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            if i != j {
                crisis[i][j] = normal[i][j].max(crisis_corr).clamp(-0.99, 0.99);
            }
        }
    }
    crisis
}

/// Nhân ma trận tam giác dưới L (N x N) với vector z (N x 1) ghi thẳng vào bộ đệm `out`
#[inline(always)]
pub fn multiply_lower_triangular(l: &[Vec<f64>], z: &[f64], out: &mut [f64]) {
    for (i, row) in l.iter().enumerate() {
        let mut s = 0.0;
        for k in 0..=i {
            s += row[k] * z[k];
        }
        out[i] = s;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cholesky_negative_corr() {
        let mat = vec![
            vec![1.0, -0.6],
            vec![-0.6, 1.0],
        ];
        let l = try_cholesky(&mat).expect("Phải phân rã thành công tương quan âm");
        assert!((l[0][0] - 1.0).abs() < 1e-9);
        assert!((l[1][0] - (-0.6)).abs() < 1e-9);
        assert!((l[1][1] - (1.0 - 0.36_f64).sqrt()).abs() < 1e-9);
    }

    #[test]
    fn test_psd_repair() {
        // Ma trận vi phạm bất đẳng thức tam giác
        let mat = vec![
            vec![1.0, 0.9, 0.9],
            vec![0.9, 1.0, -0.9],
            vec![0.9, -0.9, 1.0],
        ];
        let (_l, repaired) = cholesky_with_psd_repair(&mat);
        assert!(repaired, "Phải phát hiện và sửa PSD thành công");
    }
}

