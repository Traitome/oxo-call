//! Benchmark harnesses for core oxo-call operations.
//!
//! These benchmarks are designed to be run both:
//! 1. As Criterion micro-benchmarks (via `cargo bench`), for per-operation timing.
//! 2. As integration evaluation tasks (via the `oxo-bench` CLI), for higher-level
//!    accuracy and reproducibility measurement.

pub mod compare;
pub mod llm;
pub mod runner;
pub mod scenario;
pub mod workflow;

pub use llm::{ModelBenchConfig, ModelBenchResult, ablation_eval_tasks, run_model_bench};
pub use workflow::{BenchWorkflowResult, bench_workflow_expand, bench_workflow_parse};
