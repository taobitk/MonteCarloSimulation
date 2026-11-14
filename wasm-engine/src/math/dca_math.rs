//! Toán học xử lý dòng tiền DCA Tĩnh & DCA Động kèm Phí bán tái cân bằng

#[inline]
pub fn apply_dca_inflow_and_rebalance(
    vals: &mut [f64],
    weights: &[f64],
    periodic_cash: f64,
    is_dynamic_inflow: bool,
    allow_sell_rebalance: bool,
    tolerance_band_pct: f64,
    sell_fee_pct: f64,
) -> (f64, bool, f64) {
    let n = vals.len();
    if !is_dynamic_inflow {
        if periodic_cash > 0.0 {
            for i in 0..n { vals[i] += periodic_cash * weights[i]; }
        }
        return (vals.iter().sum::<f64>(), false, 0.0);
    }

    let cur_total: f64 = vals.iter().sum();
    let after_inflow_total = cur_total + periodic_cash;
    if periodic_cash > 0.0 && after_inflow_total > 0.0 {
        let mut gaps = vec![0.0; n];
        let mut sum_gaps = 0.0;
        for i in 0..n {
            let target_val = after_inflow_total * weights[i];
            let gap = (target_val - vals[i]).max(0.0);
            gaps[i] = gap;
            sum_gaps += gap;
        }
        if sum_gaps > 1e-9 {
            for i in 0..n { vals[i] += periodic_cash * (gaps[i] / sum_gaps); }
        } else {
            for i in 0..n { vals[i] += periodic_cash * weights[i]; }
        }
    }

    let nav_now: f64 = vals.iter().sum();
    if nav_now <= 1e-9 || !allow_sell_rebalance { return (nav_now.max(0.0), false, 0.0); }
    let mut max_drift = 0.0_f64;
    for i in 0..n {
        let w_curr = vals[i] / nav_now;
        let drift = (w_curr - weights[i]).abs();
        if drift > max_drift { max_drift = drift; }
    }

    if max_drift > tolerance_band_pct {
        let mut excess_sold = 0.0;
        for i in 0..n {
            let ideal = nav_now * weights[i];
            if vals[i] > ideal { excess_sold += vals[i] - ideal; }
        }
        let fee_paid = excess_sold * sell_fee_pct.max(0.0);
        let final_nav = (nav_now - fee_paid).max(0.0);
        for i in 0..n { vals[i] = final_nav * weights[i]; }
        (final_nav, true, fee_paid)
    } else {
        (nav_now, false, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_dca_exact_allocation_and_zero_fee() {
        let mut vals = vec![60.0, 40.0];
        let weights = vec![0.6, 0.4];
        let (tot, sold, fee) = apply_dca_inflow_and_rebalance(&mut vals, &weights, 50.0, false, false, 0.05, 0.01);
        assert!((tot - 150.0).abs() < 1e-9 && !sold && fee == 0.0);
        assert!((vals[0] - 90.0).abs() < 1e-9 && (vals[1] - 60.0).abs() < 1e-9);
    }

    #[test]
    fn test_dynamic_dca_inflow_fixes_drift_without_selling() {
        let mut vals = vec![50.0, 40.0];
        let weights = vec![0.6, 0.4];
        let (tot, sold, fee) = apply_dca_inflow_and_rebalance(&mut vals, &weights, 10.0, true, true, 0.05, 0.01);
        assert!((tot - 100.0).abs() < 1e-9 && !sold && fee == 0.0);
        assert!((vals[0] - 60.0).abs() < 1e-9 && (vals[1] - 40.0).abs() < 1e-9);
    }

    #[test]
    fn test_dynamic_dca_band_breach_triggers_sell_and_deducts_exact_fee() {
        let mut vals = vec![80.0, 10.0];
        let weights = vec![0.5, 0.5];
        let (tot, sold, fee) = apply_dca_inflow_and_rebalance(&mut vals, &weights, 10.0, true, true, 0.05, 0.02);
        assert!(sold && (fee - 0.60).abs() < 1e-9 && (tot - 99.40).abs() < 1e-9);
        assert!((vals[0] - 49.70).abs() < 1e-9 && (vals[1] - 49.70).abs() < 1e-9);
    }

    #[test]
    fn test_sell_blocked_keeps_smart_inflow_without_selling() {
        let mut vals = vec![80.0, 10.0];
        let weights = vec![0.5, 0.5];
        let (tot, sold, fee) = apply_dca_inflow_and_rebalance(&mut vals, &weights, 10.0, true, false, 0.05, 0.02);
        assert!(!sold && fee == 0.0 && (tot - 100.0).abs() < 1e-9);
        assert!((vals[0] - 80.0).abs() < 1e-9 && (vals[1] - 20.0).abs() < 1e-9);
    }
}
