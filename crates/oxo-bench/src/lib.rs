//! oxo-bench — systematic benchmarking for oxo-call
//!
//! This crate provides:
//!
//! 1. **Simulation utilities** (`sim`) — generate realistic omics datasets for
//!    benchmarking without requiring real experimental data.
//!
//! 2. **Benchmark harnesses** (`bench`) — measure accuracy, reproducibility, and
//!    performance of core oxo-call operations (workflow parsing, LLM command
//!    generation, etc.).
//!
//! 3. **Scenario & description generation** (`bench::scenario`) — parse built-in
//!    skill files and produce reference commands (10 per tool) plus diverse
//!    English usage descriptions (10 per scenario) for LLM evaluation.
//!
//! 4. **Command comparison** (`bench::compare`) — flag-order-aware comparison of
//!    generated vs. reference commands with multiple accuracy metrics.
//!
//! 5. **Multi-model benchmark runner** (`bench::runner`) — evaluate one or more
//!    LLM models by calling `oxo-call dry-run` and measuring accuracy,
//!    consistency, latency, and token counts.
//!
//! 6. **Configuration** (`config`) — TOML-based multi-model benchmark config
//!    supporting serial/parallel execution and configurable repeat counts.
//!
//! 7. **Report generation** (`report`) — aggregate results into human-readable
//!    and machine-readable formats (JSON, Markdown, CSV).
//!
//! # Quick start
//!
//! ```bash
//! # Generate reference commands and usage descriptions from skill files
//! oxo-bench generate --skills-dir skills/ --output bench_data/
//!
//! # Run LLM model evaluation with a config file
//! oxo-bench eval --config bench_config.toml
//!
//! # View cross-model comparison summary
//! oxo-bench summary --results-dir bench_results/
//!
//! # Run all criterion micro-benchmarks
//! cargo bench --package oxo-bench
//! ```

pub mod bench;
pub mod config;
pub mod report;
pub mod sim;
