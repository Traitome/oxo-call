//! Workflow parsing and expansion benchmarks.
//!
//! These functions can be used from Criterion benchmark targets or called directly
//! from the `oxo-bench` CLI for reproducibility evaluation.

use std::time::{Duration, Instant};

/// Result of a single workflow benchmark run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchWorkflowResult {
    /// Name of the workflow being benchmarked.
    pub workflow_name: String,
    /// Number of concrete tasks after wildcard expansion.
    pub expanded_tasks: usize,
    /// Time to parse the TOML content (nanoseconds).
    pub parse_ns: u64,
    /// Time to expand wildcards and build the task graph (nanoseconds).
    pub expand_ns: u64,
    /// Whether a cycle was detected (should always be false for valid workflows).
    pub has_cycle: bool,
}

/// Built-in workflow TOML content used for benchmarking (small, fast to parse).
///
/// These are representative templates that cover different dependency patterns.
pub const BENCH_WORKFLOW_RNASEQ: &str = include_str!("../../../../workflows/native/rnaseq.toml");

pub const BENCH_WORKFLOW_WGS: &str = include_str!("../../../../workflows/native/wgs.toml");

pub const BENCH_WORKFLOW_ATACSEQ: &str = include_str!("../../../../workflows/native/atacseq.toml");

pub const BENCH_WORKFLOW_METAGENOMICS: &str =
    include_str!("../../../../workflows/native/metagenomics.toml");

pub const BENCH_WORKFLOW_CHIPSEQ: &str = include_str!("../../../../workflows/native/chipseq.toml");

pub const BENCH_WORKFLOW_SCRNASEQ: &str =
    include_str!("../../../../workflows/native/scrnaseq.toml");

pub const BENCH_WORKFLOW_LONGREADS: &str =
    include_str!("../../../../workflows/native/longreads.toml");

/// All benchmark TOML contents paired with their names.
pub const ALL_BENCH_WORKFLOWS: &[(&str, &str)] = &[
    ("rnaseq", BENCH_WORKFLOW_RNASEQ),
    ("wgs", BENCH_WORKFLOW_WGS),
    ("atacseq", BENCH_WORKFLOW_ATACSEQ),
    ("metagenomics", BENCH_WORKFLOW_METAGENOMICS),
    ("chipseq", BENCH_WORKFLOW_CHIPSEQ),
    ("scrnaseq", BENCH_WORKFLOW_SCRNASEQ),
    ("longreads", BENCH_WORKFLOW_LONGREADS),
];

/// Internal: parsed representation suitable for expansion without depending on
/// the main `oxo-call` binary at bench time.  We only need to count tasks to
/// verify correctness.
#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct WorkflowDef {
    workflow: WorkflowMeta,
    wildcards: Option<toml::Table>,
    #[serde(rename = "step")]
    steps: Vec<StepDef>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct WorkflowMeta {
    name: String,
}

#[derive(Debug, serde::Deserialize)]
struct StepDef {
    name: String,
    #[serde(default)]
    gather: bool,
    #[serde(default)]
    depends_on: Vec<String>,
}

/// Parse the workflow TOML and measure parse time.
pub fn bench_workflow_parse(name: &str, toml_content: &str) -> BenchWorkflowResult {
    let t0 = Instant::now();
    let def: Result<WorkflowDef, _> = toml::from_str(toml_content);
    let parse_dur = t0.elapsed();

    match def {
        Err(_e) => BenchWorkflowResult {
            workflow_name: name.to_string(),
            expanded_tasks: 0,
            parse_ns: parse_dur.as_nanos() as u64,
            expand_ns: 0,
            has_cycle: false,
        },
        Ok(workflow) => {
            let t1 = Instant::now();
            let (n_tasks, has_cycle) = count_expanded_tasks(&workflow);
            let expand_dur = t1.elapsed();

            BenchWorkflowResult {
                workflow_name: name.to_string(),
                expanded_tasks: n_tasks,
                parse_ns: parse_dur.as_nanos() as u64,
                expand_ns: expand_dur.as_nanos() as u64,
                has_cycle,
            }
        }
    }
}

/// Count expanded tasks and check for dependency cycles (Kahn's algorithm).
fn count_expanded_tasks(def: &WorkflowDef) -> (usize, bool) {
    // Determine the number of wildcard combinations.
    let n_combos = if let Some(wildcards) = &def.wildcards {
        wildcards
            .values()
            .filter_map(|v| v.as_array().map(|a| a.len()))
            .product::<usize>()
            .max(1)
    } else {
        1
    };

    // Gather steps expand to a single task regardless of wildcard count.
    let n_gather = def.steps.iter().filter(|s| s.gather).count();
    let n_regular = def.steps.iter().filter(|s| !s.gather).count();
    let n_tasks = n_regular * n_combos + n_gather;

    // Simple cycle check on the step-level dependency graph.
    let step_names: Vec<&str> = def.steps.iter().map(|s| s.name.as_str()).collect();
    let mut in_degree: std::collections::HashMap<&str, usize> =
        step_names.iter().map(|&n| (n, 0)).collect();

    for step in &def.steps {
        for dep in &step.depends_on {
            if in_degree.contains_key(dep.as_str()) {
                *in_degree.entry(step.name.as_str()).or_insert(0) += 1;
            }
        }
    }

    let mut queue: std::collections::VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(&n, _)| n)
        .collect();

    let mut visited = 0;
    let mut adj: std::collections::HashMap<&str, Vec<&str>> =
        step_names.iter().map(|&n| (n, vec![])).collect();
    for step in &def.steps {
        for dep in &step.depends_on {
            if let Some(v) = adj.get_mut(dep.as_str()) {
                v.push(step.name.as_str());
            }
        }
    }

    while let Some(node) = queue.pop_front() {
        visited += 1;
        if let Some(neighbors) = adj.get(node) {
            for &next in neighbors {
                let d = in_degree.entry(next).or_insert(0);
                *d = d.saturating_sub(1);
                if *d == 0 {
                    queue.push_back(next);
                }
            }
        }
    }

    let has_cycle = visited < step_names.len();
    (n_tasks, has_cycle)
}

/// Expand all built-in workflow templates and return benchmark results.
pub fn bench_workflow_expand(n_runs: usize) -> Vec<BenchWorkflowResult> {
    let mut results = Vec::new();
    for &(name, content) in ALL_BENCH_WORKFLOWS {
        // Average over `n_runs` to reduce timing noise.
        let mut total_parse = Duration::ZERO;
        let mut total_expand = Duration::ZERO;
        let mut final_result = BenchWorkflowResult {
            workflow_name: name.to_string(),
            expanded_tasks: 0,
            parse_ns: 0,
            expand_ns: 0,
            has_cycle: false,
        };

        for _ in 0..n_runs {
            let r = bench_workflow_parse(name, content);
            total_parse += Duration::from_nanos(r.parse_ns);
            total_expand += Duration::from_nanos(r.expand_ns);
            final_result = r;
        }

        final_result.parse_ns = (total_parse / n_runs as u32).as_nanos() as u64;
        final_result.expand_ns = (total_expand / n_runs as u32).as_nanos() as u64;
        results.push(final_result);
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_all_workflows_no_cycle() {
        for &(name, content) in ALL_BENCH_WORKFLOWS {
            let result = bench_workflow_parse(name, content);
            assert!(
                !result.has_cycle,
                "Workflow '{}' should not have dependency cycles",
                name
            );
            assert!(
                result.expanded_tasks > 0,
                "Workflow '{}' should expand to at least one task",
                name
            );
        }
    }

    #[test]
    fn test_bench_rnaseq_task_count() {
        // rnaseq has 3 samples × 4 non-gather steps + 1 gather step = 13 tasks.
        let result = bench_workflow_parse("rnaseq", BENCH_WORKFLOW_RNASEQ);
        assert_eq!(result.expanded_tasks, 13, "rnaseq should have 13 tasks");
    }

    #[test]
    fn test_bench_parse_is_fast() {
        // Each parse should complete in < 10 ms on any reasonable hardware.
        for &(name, content) in ALL_BENCH_WORKFLOWS {
            let result = bench_workflow_parse(name, content);
            assert!(
                result.parse_ns < 10_000_000,
                "Parsing '{}' took {}ms which is too slow",
                name,
                result.parse_ns / 1_000_000
            );
        }
    }
}
