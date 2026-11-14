//! Data-Driven DCA & Chaos Monkey Simulation Engine

pub mod chaos_runtime;
pub mod chaos_spec;
pub mod cli_report;
pub mod dca;
pub mod dca_types;
pub mod engine_params;

pub use chaos_spec::ChaosEvent;
pub use cli_report::{merge_dca_results, CliRunOutput, CliSuiteOutput};
pub use dca::run_dca_simulation;
pub use dca_types::{DcaConfig, DcaSimResult};
pub use engine_params::{AssetSpec, EngineParams, QuantileBand, RareDist, ValueDist};
