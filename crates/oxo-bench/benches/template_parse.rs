//! Criterion micro-benchmark: workflow TOML template parsing.
//!
//! Run with:
//!   cargo bench --package oxo-bench --bench template_parse

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use oxo_bench::bench::workflow::ALL_BENCH_WORKFLOWS;

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("template_parse");

    for &(name, content) in ALL_BENCH_WORKFLOWS {
        group.bench_with_input(
            BenchmarkId::new("parse_toml", name),
            content,
            |b, content| {
                b.iter(|| {
                    // Parse raw TOML — the core deserialization step.
                    let _: Result<toml::Value, _> = toml::from_str(black_box(content));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
