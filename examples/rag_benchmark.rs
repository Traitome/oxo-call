//! RAG Benchmark Example
//!
//! Run this example to compare baseline doc mode with RAG-enhanced mode.
//!
//! Usage:
//!   cargo run --example rag_benchmark --release

use oxo_call::bench::{BenchmarkConfig, RagBenchmark, create_quick_suite};

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║       RAG Benchmark - Doc Mode Accuracy Comparison         ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Configure benchmark
    let config = BenchmarkConfig {
        model: "github-copilot/claude-3.5-sonnet".to_string(),
        enable_rag: true,
        max_retries: 1,
        timeout_secs: 60,
    };

    let benchmark = RagBenchmark::new(config);

    // Run with quick suite for demonstration
    println!("Running with Quick Test Suite (3 cases)...\n");
    let quick_suite = create_quick_suite();

    let report = benchmark.run_comparison(&quick_suite);

    // Print detailed comparison report
    report.print_report();

    // Summary
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║                      Summary                               ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!(
        "║ Baseline Accuracy:      {:>6.1}%                           ║",
        report.baseline.accuracy() * 100.0
    );
    println!(
        "║ RAG-Enhanced Accuracy:  {:>6.1}%                           ║",
        report.rag_enhanced.accuracy() * 100.0
    );
    println!("║                                                            ║");
    println!(
        "║ Improvement:             {:>+6.1}%                          ║",
        report.improvement.accuracy_delta * 100.0
    );
    println!(
        "║ Additional Correct:      {:>6}                            ║",
        report.improvement.additional_correct
    );
    println!("╚════════════════════════════════════════════════════════════╝");
}
