//! Criterion micro-benchmark: workflow wildcard expansion.
//!
//! Run with:
//!   cargo bench --package oxo-bench --bench workflow_expand

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use oxo_bench::bench::workflow::{ALL_BENCH_WORKFLOWS, bench_workflow_parse};

fn bench_expand(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_expansion");

    for &(name, content) in ALL_BENCH_WORKFLOWS {
        group.bench_with_input(
            BenchmarkId::new("expand", name),
            &(name, content),
            |b, &(name, content)| {
                b.iter(|| black_box(bench_workflow_parse(name, content)));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_expand);
criterion_main!(benches);
