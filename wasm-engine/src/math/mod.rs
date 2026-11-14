//! Module Toán học & Thống kê thuần túy

pub mod cholesky;
pub mod dca_math;
pub mod distributions;
pub mod quantiles;
pub mod stats;

pub use cholesky::{build_crisis_matrix, cholesky_with_psd_repair, multiply_lower_triangular, try_cholesky};
pub use dca_math::apply_dca_inflow_and_rebalance;
pub use distributions::{box_muller, chi_squared, standard_gamma, student_t};
pub use quantiles::{extract_quantile_bands_inplace, update_top2_worst};
pub use stats::WelfordStats;
