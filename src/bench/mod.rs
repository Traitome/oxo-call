//! RAG Benchmark Module
//!
//! Provides benchmarking capabilities to measure the accuracy improvement
//! of RAG (Retrieval-Augmented Generation) over standard doc mode.

pub mod metrics;
pub mod rag_bench;
pub mod test_cases;

// Re-export commonly used types for examples
#[allow(unused_imports)]
pub use rag_bench::{BenchmarkConfig, RagBenchmark};
#[allow(unused_imports)]
pub use test_cases::{create_bioinformatics_suite, create_quick_suite};
