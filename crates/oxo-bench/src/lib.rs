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
//! 3. **Report generation** (`report`) — aggregate results into human-readable
//!    and machine-readable formats (JSON, Markdown).
//!
//! # Quick start
//!
//! ```bash
//! # Run all criterion micro-benchmarks
//! cargo bench --package oxo-bench
//!
//! # Run the LLM model evaluation CLI
//! oxo-bench eval-models --models gpt-4o-mini,gpt-4o-nano --tasks 50
//!
//! # Simulate 100 paired-end RNA-seq samples and measure command generation
//! oxo-bench sim-rnaseq --samples 100 --output /tmp/bench_rnaseq
//! ```

pub mod bench;
pub mod report;
pub mod sim;
