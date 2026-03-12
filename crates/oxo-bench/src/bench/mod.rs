//! Benchmark harnesses for core oxo-call operations.
//!
//! These benchmarks are designed to be run both:
//! 1. As Criterion micro-benchmarks (via `cargo bench`), for per-operation timing.
//! 2. As integration evaluation tasks (via the `oxo-bench` CLI), for higher-level
//!    accuracy and reproducibility measurement.

pub mod workflow;
pub mod llm;

pub use workflow::{BenchWorkflowResult, bench_workflow_parse, bench_workflow_expand};
pub use llm::{ModelBenchConfig, ModelBenchResult, run_model_bench};
