/// oxo-call native workflow engine.
///
/// Provides a lightweight, dependency-aware DAG executor for bioinformatics
/// pipelines defined in `.oxo.toml` files.  No external workflow manager
/// (Snakemake, Nextflow, Conda) is required — only the bioinformatics tools
/// themselves need to be installed.
///
/// # Workflow file format (`.oxo.toml`)
///
/// ```toml
/// [workflow]
/// name = "rnaseq"
/// description = "Bulk RNA-seq pipeline"
///
/// [wildcards]
/// sample = ["s1", "s2", "s3"]   # {sample} expands for each value
///
/// [params]
/// threads = "8"
/// star_index = "/data/star_hg38"
/// gtf       = "/data/gencode.v44.gtf"
///
/// [[step]]
/// name    = "fastp"
/// cmd     = "fastp --in1 data/{sample}_R1.fq.gz --in2 data/{sample}_R2.fq.gz ..."
/// inputs  = ["data/{sample}_R1.fq.gz", "data/{sample}_R2.fq.gz"]
/// outputs = ["trimmed/{sample}_R1.fq.gz", "trimmed/{sample}_R2.fq.gz"]
///
/// [[step]]
/// name       = "star"
/// depends_on = ["fastp"]
/// cmd        = "STAR --genomeDir {params.star_index} ..."
/// inputs     = ["trimmed/{sample}_R1.fq.gz", "trimmed/{sample}_R2.fq.gz"]
/// outputs    = ["aligned/{sample}/Aligned.sortedByCoord.out.bam"]
///
/// [[step]]
/// name       = "multiqc"
/// gather     = true          # runs ONCE after all {sample} instances of deps finish
/// depends_on = ["fastp", "star"]
/// cmd        = "multiqc qc/ aligned/ -o results/multiqc/"
/// outputs    = ["results/multiqc/multiqc_report.html"]
/// ```
///
/// # Export
///
/// Any `.oxo.toml` workflow can be exported to Snakemake or Nextflow DSL2
/// for integration with HPC environments that require those formats.
use crate::error::{OxoError, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{Instant, SystemTime};

// ─── Workflow definition (parsed from TOML) ───────────────────────────────────

/// `[workflow]` block — human-readable metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowMeta {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: String,
}

/// A single `[[step]]` entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDef {
    /// Unique step identifier (used in `depends_on` lists).
    pub name: String,
    /// Shell command.  Supports `{wildcard}` and `{params.key}` substitution.
    pub cmd: String,
    /// Names of steps that must complete before this step starts.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Input file patterns for freshness checking.
    #[serde(default)]
    pub inputs: Vec<String>,
    /// Output file patterns for freshness checking and skip-if-fresh logic.
    #[serde(default)]
    pub outputs: Vec<String>,
    /// When true, runs once after ALL wildcard expansions of dependency steps
    /// complete (like a gather/aggregate step, e.g. MultiQC).
    #[serde(default)]
    pub gather: bool,
}

/// Top-level workflow definition (the parsed `.oxo.toml`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowDef {
    pub workflow: WorkflowMeta,
    /// Named wildcard lists.  Each key can appear as `{key}` in step fields.
    #[serde(default)]
    pub wildcards: HashMap<String, Vec<String>>,
    /// String parameters accessible as `{params.key}` in step commands.
    #[serde(default)]
    pub params: HashMap<String, String>,
    /// Pipeline steps (ordered; later steps may depend on earlier ones).
    #[serde(rename = "step", default)]
    pub steps: Vec<StepDef>,
}

impl WorkflowDef {
    /// Load and parse a workflow from a TOML file on disk.
    pub fn from_file(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Self::from_str_content(&text)
    }

    /// Parse a workflow from a TOML string.
    pub fn from_str_content(s: &str) -> Result<Self> {
        Ok(toml::from_str(s)?)
    }
}

// ─── Concrete task (wildcard-expanded) ───────────────────────────────────────

/// A fully expanded, runnable unit of work.
#[derive(Debug, Clone)]
pub struct ConcreteTask {
    /// Unique task identifier: `"<step>"` or `"<step>[key=val,…]"`.
    pub id: String,
    pub step_name: String,
    /// Fully substituted shell command (ready to pass to `sh -c`).
    pub cmd: String,
    /// Fully substituted input file paths.
    pub inputs: Vec<String>,
    /// Fully substituted output file paths.
    pub outputs: Vec<String>,
    /// IDs of tasks that must complete before this task can start.
    pub deps: Vec<String>,
    pub gather: bool,
}

// ─── Wildcard expansion helpers ────────────────────────────────────────────────

/// Enumerate every combination of wildcard bindings.
fn wildcard_combinations(wildcards: &HashMap<String, Vec<String>>) -> Vec<HashMap<String, String>> {
    if wildcards.is_empty() {
        return vec![HashMap::new()];
    }
    let mut result: Vec<HashMap<String, String>> = vec![HashMap::new()];
    // Iterate in a deterministic order (sorted keys).
    let mut keys: Vec<&String> = wildcards.keys().collect();
    keys.sort();
    for key in keys {
        let values = &wildcards[key];
        let mut next = Vec::new();
        for val in values {
            for existing in &result {
                let mut m = existing.clone();
                m.insert(key.clone(), val.clone());
                next.push(m);
            }
        }
        result = next;
    }
    result
}

/// Substitute `{key}` (wildcard) and `{params.key}` placeholders.
fn substitute(
    template: &str,
    bindings: &HashMap<String, String>,
    params: &HashMap<String, String>,
) -> String {
    let mut s = template.to_string();
    // Wildcard substitution first (higher precedence).
    for (k, v) in bindings {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    // Param substitution ({params.key} and bare {key} if not shadowed).
    for (k, v) in params {
        s = s.replace(&format!("{{params.{k}}}", k = k), v);
        // Bare {key} only if not already a wildcard key.
        if !bindings.contains_key(k.as_str()) {
            s = s.replace(&format!("{{{k}}}"), v);
        }
    }
    s
}

/// Build a canonical task ID from step name + wildcard bindings.
fn task_id(step_name: &str, bindings: &HashMap<String, String>) -> String {
    if bindings.is_empty() {
        return step_name.to_string();
    }
    let mut parts: Vec<String> = bindings.iter().map(|(k, v)| format!("{k}={v}")).collect();
    parts.sort();
    format!("{step_name}[{}]", parts.join(","))
}

/// Returns true if the step uses any wildcard key in any of its fields.
fn uses_wildcards(step: &StepDef, wildcards: &HashMap<String, Vec<String>>) -> bool {
    wildcards.keys().any(|k| {
        let pat = format!("{{{k}}}");
        step.cmd.contains(&pat)
            || step.inputs.iter().any(|i| i.contains(&pat))
            || step.outputs.iter().any(|o| o.contains(&pat))
    })
}

// ─── DAG construction ─────────────────────────────────────────────────────────

/// Expand a `WorkflowDef` into a flat list of fully resolved `ConcreteTask`s
/// with explicit inter-task dependency edges.
pub fn expand(def: &WorkflowDef) -> Result<Vec<ConcreteTask>> {
    let combos = wildcard_combinations(&def.wildcards);
    let mut tasks: Vec<ConcreteTask> = Vec::new();

    // Map: step_name → list of task IDs produced by that step.
    let mut step_tasks: HashMap<String, Vec<String>> = HashMap::new();

    for step in &def.steps {
        let wc = uses_wildcards(step, &def.wildcards);

        if step.gather || !wc || combos.len() <= 1 {
            // ── Single task (gather, no-wildcard, or only one combo) ────────
            let bindings: HashMap<String, String> = if !step.gather && wc && combos.len() == 1 {
                combos[0].clone()
            } else {
                HashMap::new()
            };

            let id = if step.gather {
                step.name.clone() // gather tasks always use the bare step name
            } else {
                task_id(&step.name, &bindings)
            };

            let deps = if step.gather {
                // Gather: depends on ALL tasks of each listed dep step.
                step.depends_on
                    .iter()
                    .flat_map(|dep| step_tasks.get(dep).cloned().unwrap_or_default())
                    .collect()
            } else {
                // Single combo: depends on same-combo task of each dep step.
                step.depends_on
                    .iter()
                    .flat_map(|dep| {
                        step_tasks
                            .get(dep)
                            .cloned()
                            .unwrap_or_default()
                            .into_iter()
                            .filter(|t| bindings.is_empty() || *t == task_id(dep, &bindings))
                    })
                    .collect()
            };

            let t = ConcreteTask {
                id: id.clone(),
                step_name: step.name.clone(),
                cmd: substitute(&step.cmd, &bindings, &def.params),
                inputs: step
                    .inputs
                    .iter()
                    .map(|i| substitute(i, &bindings, &def.params))
                    .collect(),
                outputs: step
                    .outputs
                    .iter()
                    .map(|o| substitute(o, &bindings, &def.params))
                    .collect(),
                deps,
                gather: step.gather,
            };
            step_tasks.entry(step.name.clone()).or_default().push(id);
            tasks.push(t);
        } else {
            // ── Per-combo expansion ─────────────────────────────────────────
            for bindings in &combos {
                let id = task_id(&step.name, bindings);

                let deps: Vec<String> = step
                    .depends_on
                    .iter()
                    .flat_map(|dep| {
                        step_tasks
                            .get(dep)
                            .cloned()
                            .unwrap_or_default()
                            .into_iter()
                            .filter(|t| *t == task_id(dep, bindings))
                    })
                    .collect();

                let t = ConcreteTask {
                    id: id.clone(),
                    step_name: step.name.clone(),
                    cmd: substitute(&step.cmd, bindings, &def.params),
                    inputs: step
                        .inputs
                        .iter()
                        .map(|i| substitute(i, bindings, &def.params))
                        .collect(),
                    outputs: step
                        .outputs
                        .iter()
                        .map(|o| substitute(o, bindings, &def.params))
                        .collect(),
                    deps,
                    gather: false,
                };
                step_tasks.entry(step.name.clone()).or_default().push(id);
                tasks.push(t);
            }
        }
    }

    // Detect dependency cycles via a simple forward-reachability check.
    let id_to_idx: HashMap<&str, usize> = tasks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id.as_str(), i))
        .collect();
    for task in &tasks {
        for dep in &task.deps {
            if !id_to_idx.contains_key(dep.as_str()) {
                return Err(OxoError::ExecutionError(format!(
                    "Step '{}' depends on unknown step/task '{dep}'",
                    task.step_name
                )));
            }
        }
    }

    Ok(tasks)
}

// ─── Output-freshness caching ─────────────────────────────────────────────────

fn mtime(path: &str) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

/// Returns `true` if all outputs exist and are at least as new as all inputs.
fn is_up_to_date(task: &ConcreteTask) -> bool {
    if task.outputs.is_empty() {
        return false; // No declared outputs — always run.
    }
    let Some(oldest_output) = task
        .outputs
        .iter()
        .map(|o| mtime(o))
        .collect::<Option<Vec<_>>>()
        .and_then(|v| v.into_iter().min())
    else {
        return false; // At least one output is missing.
    };
    // Every input must be older than (or equal to) the oldest output.
    task.inputs
        .iter()
        .all(|i| mtime(i).is_none_or(|t| t <= oldest_output))
}

// ─── DAG phase computation ────────────────────────────────────────────────────

/// Compute execution phases: groups of tasks that can run in parallel.
///
/// Each phase contains tasks whose dependencies are all satisfied by
/// earlier phases.  Within a phase, all tasks are independent and can
/// execute concurrently.
pub fn compute_phases(tasks: &[ConcreteTask]) -> Vec<Vec<&ConcreteTask>> {
    let mut phases: Vec<Vec<&ConcreteTask>> = Vec::new();
    let mut assigned: HashSet<&str> = HashSet::new();
    let mut remaining: Vec<&ConcreteTask> = tasks.iter().collect();

    while !remaining.is_empty() {
        let (ready, rest): (Vec<&ConcreteTask>, Vec<&ConcreteTask>) = remaining
            .into_iter()
            .partition(|t| t.deps.iter().all(|d| assigned.contains(d.as_str())));

        if ready.is_empty() {
            // All remaining tasks have unsatisfied deps — cycle or error.
            break;
        }

        for t in &ready {
            assigned.insert(&t.id);
        }
        phases.push(ready);
        remaining = rest;
    }

    phases
}

/// Format elapsed time as human-readable string.
fn format_elapsed(elapsed: std::time::Duration) -> String {
    let secs = elapsed.as_secs();
    if secs < 60 {
        format!("{:.1}s", elapsed.as_secs_f64())
    } else if secs < 3600 {
        format!("{}m {:02}s", secs / 60, secs % 60)
    } else {
        format!("{}h {:02}m {:02}s", secs / 3600, (secs % 3600) / 60, secs % 60)
    }
}

/// Print the DAG phase diagram for a set of tasks.
fn print_dag_phases(tasks: &[ConcreteTask]) {
    let phases = compute_phases(tasks);
    if phases.is_empty() {
        return;
    }

    println!();
    println!(
        "  {} {}",
        "Pipeline DAG".bold(),
        format!("({} phases, {} tasks)", phases.len(), tasks.len()).dimmed()
    );
    println!();

    for (i, phase) in phases.iter().enumerate() {
        let phase_num = format!("Phase {}", i + 1);

        // Group by step_name to show compact per-sample expansion.
        let mut groups: Vec<(String, Vec<String>)> = Vec::new();
        for t in phase.iter() {
            let label = if t.gather {
                format!("{} [gather]", t.id)
            } else {
                t.id.clone()
            };
            if let Some(g) = groups.last_mut().filter(|(name, _)| name == &t.step_name) {
                g.1.push(label);
            } else {
                groups.push((t.step_name.clone(), vec![label]));
            }
        }

        let mut group_strs: Vec<String> = Vec::new();
        for (_, items) in &groups {
            if items.len() <= 3 {
                group_strs.extend(items.iter().cloned());
            } else {
                // Show first two and a count.
                group_strs.push(items[0].clone());
                group_strs.push(format!("… +{} more", items.len() - 1));
            }
        }

        println!(
            "  {:>9}  {}",
            phase_num.cyan(),
            group_strs.join("  │  ")
        );

        if i + 1 < phases.len() {
            println!("  {:>9}  {}", "", "↓".dimmed());
        }
    }

    println!();
}

// ─── Async executor ────────────────────────────────────────────────────────────

/// Execute a task graph with maximum parallelism.
///
/// Independent tasks (no mutual dependencies) run concurrently via
/// `tokio::task::JoinSet`.  Dependent tasks wait until all their prerequisites
/// have succeeded or been skipped.  If any task fails the whole run is aborted.
#[cfg(not(target_arch = "wasm32"))]
pub async fn execute(tasks: Vec<ConcreteTask>, dry_run: bool) -> Result<()> {
    use tokio::task::JoinSet;

    if tasks.is_empty() {
        println!("{}", "Workflow has no steps to execute.".yellow());
        return Ok(());
    }

    let total = tasks.len();
    let start_time = Instant::now();

    println!();
    println!(
        "{} {} — {} task(s){}",
        "◆".cyan().bold(),
        "oxo workflow".bold(),
        total,
        if dry_run { " (dry-run)" } else { "" }
    );
    println!("{}", "─".repeat(60).dimmed());

    // Print DAG phase diagram.
    print_dag_phases(&tasks);

    println!("{}", "─".repeat(60).dimmed());

    // Set of completed task IDs (includes both run and skipped).
    let mut done: HashSet<String> = HashSet::new();
    // Tasks that have been dispatched (to avoid double-dispatch).
    let mut started: HashSet<String> = HashSet::new();
    // Separately track which tasks were skipped (up-to-date).
    let mut skipped_count: usize = 0;
    let mut completed_count: usize = 0;
    let mut join_set: JoinSet<Result<(String, bool /*skipped*/)>> = JoinSet::new();

    let mut iterations_without_progress = 0usize;

    loop {
        // Find all tasks whose dependencies are fully satisfied.
        let newly_ready: Vec<&ConcreteTask> = tasks
            .iter()
            .filter(|t| !started.contains(&t.id) && t.deps.iter().all(|d| done.contains(d)))
            .collect();

        for task in newly_ready {
            started.insert(task.id.clone());
            if dry_run {
                print_task_dry_run(task, completed_count + 1, total);
                done.insert(task.id.clone());
                completed_count += 1;
            } else {
                let t = task.clone();
                join_set.spawn(async move { run_single_task(t).await });
            }
        }

        if dry_run {
            // In dry-run we process synchronously; no JoinSet to wait on.
            if done.len() == total {
                break;
            }
            iterations_without_progress += 1;
            if iterations_without_progress > total {
                return Err(OxoError::ExecutionError(
                    "Workflow dry-run stalled: possible dependency cycle".to_string(),
                ));
            }
            continue;
        }

        if done.len() == total {
            break;
        }

        match join_set.join_next().await {
            None => {
                // JoinSet is empty but we haven't finished — cycle or missing dep.
                if done.len() < total {
                    return Err(OxoError::ExecutionError(
                        "Workflow stalled: possible dependency cycle or missing step".to_string(),
                    ));
                }
                break;
            }
            Some(result) => {
                let (id, skipped) = result
                    .map_err(|e| OxoError::ExecutionError(format!("Task join error: {e}")))??;
                completed_count += 1;
                if skipped {
                    skipped_count += 1;
                    println!(
                        "  {} [{}/{}] {} {}",
                        "↷".dimmed(),
                        completed_count,
                        total,
                        id.dimmed(),
                        "(up to date)".dimmed()
                    );
                } else {
                    println!(
                        "  {} [{}/{}] {}",
                        "✓".green().bold(),
                        completed_count,
                        total,
                        id.green()
                    );
                }
                done.insert(id);
            }
        }
    }

    let elapsed = start_time.elapsed();
    println!("{}", "─".repeat(60).dimmed());
    let run_count = started.len();
    if dry_run {
        println!(
            "\n{} {} task(s) would execute across {} phase(s)",
            "◆".cyan().bold(),
            run_count,
            compute_phases(&tasks).len()
        );
    } else {
        println!(
            "\n{} Workflow complete — {} task(s) run, {} up to date  ({})",
            "✓".green().bold(),
            run_count.saturating_sub(skipped_count),
            skipped_count,
            format_elapsed(elapsed)
        );
    }

    Ok(())
}

/// Run a single concrete task via `sh -c`.
#[cfg(not(target_arch = "wasm32"))]
async fn run_single_task(task: ConcreteTask) -> Result<(String, bool)> {
    use tokio::process::Command;

    // Skip if all outputs are newer than all inputs.
    if is_up_to_date(&task) {
        return Ok((task.id, true));
    }

    println!("  {} {}", "▶".cyan().bold(), task.id.cyan().bold());
    println!("    {}", task.cmd.dimmed());

    // Ensure output parent directories exist.
    for out in &task.outputs {
        if let Some(parent) = Path::new(out).parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let status = Command::new("sh")
        .arg("-c")
        .arg(&task.cmd)
        .status()
        .await
        .map_err(|e| {
            OxoError::ExecutionError(format!("Failed to start task '{}': {e}", task.id))
        })?;

    if !status.success() {
        return Err(OxoError::ExecutionError(format!(
            "Task '{}' failed with exit code {}",
            task.id,
            status.code().unwrap_or(-1)
        )));
    }

    Ok((task.id, false))
}

/// Print a dry-run preview for a single task.
fn print_task_dry_run(task: &ConcreteTask, current: usize, total: usize) {
    println!(
        "  {} [{}/{}] {}",
        "▷".cyan(),
        current,
        total,
        if task.gather {
            format!("{} [gather]", task.id).cyan().bold().to_string()
        } else {
            task.id.cyan().bold().to_string()
        }
    );
    println!("    $ {}", task.cmd.dimmed());
    if !task.deps.is_empty() {
        println!("    after:   {}", task.deps.join(", ").dimmed());
    }
    if !task.inputs.is_empty() {
        println!("    inputs:  {}", task.inputs.join("  ").dimmed());
    }
    if !task.outputs.is_empty() {
        println!("    outputs: {}", task.outputs.join("  ").dimmed());
    }
    println!();
}

// ─── Snakemake export ─────────────────────────────────────────────────────────

/// Convert a native `WorkflowDef` into a Snakemake `Snakefile`.
pub fn to_snakemake(def: &WorkflowDef) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "# Generated by oxo-call — workflow '{}'\n",
        def.workflow.name
    ));
    if !def.workflow.description.is_empty() {
        out.push_str(&format!("# {}\n", def.workflow.description));
    }
    out.push_str("# Edit this file or re-run 'oxo-call workflow export' to regenerate.\n\n");

    out.push_str("configfile: \"config.yaml\"\n\n");

    // Wildcard lists as Python variables.
    if !def.wildcards.is_empty() {
        out.push_str("# ── Wildcard sample lists ─────────────────────────────────────────────\n");
        let mut keys: Vec<&String> = def.wildcards.keys().collect();
        keys.sort();
        for k in &keys {
            let vals = &def.wildcards[*k];
            let quoted: Vec<String> = vals.iter().map(|v| format!("\"{v}\"")).collect();
            out.push_str(&format!("{} = [{}]\n", k.to_uppercase(), quoted.join(", ")));
        }
        out.push('\n');
    }

    // rule all — collect the "leaf" outputs (steps no other step depends on).
    let depended_on: HashSet<&str> = def
        .steps
        .iter()
        .flat_map(|s| s.depends_on.iter().map(|d| d.as_str()))
        .collect();
    let leaf_steps: Vec<&StepDef> = def
        .steps
        .iter()
        .filter(|s| !depended_on.contains(s.name.as_str()))
        .collect();

    out.push_str("rule all:\n    input:\n");
    for step in &leaf_steps {
        for out_pat in &step.outputs {
            if step.gather || !uses_wildcards(step, &def.wildcards) || def.wildcards.is_empty() {
                out.push_str(&format!("        \"{out_pat}\",\n"));
            } else {
                // Use Snakemake's expand() for wildcard outputs.
                let mut keys: Vec<&String> = def.wildcards.keys().collect();
                keys.sort();
                let expand_args: Vec<String> = keys
                    .iter()
                    .map(|k| format!("{k}={ku}", ku = k.to_uppercase()))
                    .collect();
                out.push_str(&format!(
                    "        expand(\"{out_pat}\", {}),\n",
                    expand_args.join(", ")
                ));
            }
        }
    }
    out.push('\n');

    // Individual rules.
    for step in &def.steps {
        out.push_str(&format!("\nrule {}:\n", step.name));
        if !step.inputs.is_empty() {
            out.push_str("    input:\n");
            for inp in &step.inputs {
                out.push_str(&format!("        \"{inp}\",\n"));
            }
        }
        if !step.outputs.is_empty() {
            out.push_str("    output:\n");
            for outp in &step.outputs {
                out.push_str(&format!("        \"{outp}\",\n"));
            }
        }
        out.push_str(&format!("    log: \"logs/{}.log\"\n", step.name));
        out.push_str("    shell:\n");
        // Rewrite wildcards and params for Snakemake.
        let mut cmd = step.cmd.clone();
        let mut keys: Vec<&String> = def.wildcards.keys().collect();
        keys.sort();
        for k in &keys {
            cmd = cmd.replace(&format!("{{{k}}}"), &format!("{{wildcards.{k}}}"));
        }
        let mut pkeys: Vec<&String> = def.params.keys().collect();
        pkeys.sort();
        for k in &pkeys {
            cmd = cmd.replace(&format!("{{params.{k}}}"), &format!("{{config[\"{k}\"]}}"));
        }
        // Escape for Snakemake shell string (double-quote safe).
        let cmd_escaped = cmd.replace('"', "\\\"");
        out.push_str(&format!("        \"{cmd_escaped}\"\n"));
    }

    // config.yaml comment block.
    if !def.params.is_empty() {
        out.push_str(
            "\n# ─── config.yaml (create this file next to your Snakefile) ───────────────\n",
        );
        out.push_str("# ");
        let mut pkeys: Vec<&String> = def.params.keys().collect();
        pkeys.sort();
        for k in pkeys {
            out.push_str(&format!("{k}: \"{}\"\n# ", def.params[k]));
        }
        out.push('\n');
    }

    out
}

// ─── Nextflow export ──────────────────────────────────────────────────────────

/// Convert a native `WorkflowDef` into a Nextflow DSL2 `.nf` file.
pub fn to_nextflow(def: &WorkflowDef) -> String {
    let mut out = String::new();

    out.push_str("#!/usr/bin/env nextflow\n");
    out.push_str(&format!(
        "// Generated by oxo-call — workflow '{}'\n",
        def.workflow.name
    ));
    if !def.workflow.description.is_empty() {
        out.push_str(&format!("// {}\n", def.workflow.description));
    }
    out.push_str("// Edit this file or re-run 'oxo-call workflow export' to regenerate.\n\n");
    out.push_str("nextflow.enable.dsl = 2\n\n");

    // params block.
    let mut pkeys: Vec<&String> = def.params.keys().collect();
    pkeys.sort();
    for k in &pkeys {
        let v = def.params.get(k.as_str()).map(String::as_str).unwrap_or("");
        out.push_str(&format!("params.{k} = \"{v}\"\n"));
    }
    if !def.wildcards.is_empty() {
        out.push_str("params.samplesheet = \"samplesheet.csv\"\n");
    }
    out.push('\n');

    // Samplesheet channel.
    if !def.wildcards.is_empty() {
        let mut keys: Vec<&String> = def.wildcards.keys().collect();
        keys.sort();
        out.push_str(
            "// ── Sample channel ──────────────────────────────────────────────────────\n",
        );
        out.push_str("Channel\n");
        out.push_str("    .fromPath(params.samplesheet)\n");
        out.push_str("    .splitCsv(header: true)\n");
        let fields: Vec<String> = keys.iter().map(|k| format!("row.{k}")).collect();
        out.push_str(&format!(
            "    .map {{ row -> tuple({}) }}\n",
            fields.join(", ")
        ));
        out.push_str("    .set { samples_ch }\n\n");
    }

    // process blocks.
    for step in &def.steps {
        out.push_str(&format!("process {} {{\n", step.name.to_uppercase()));
        if !step.inputs.is_empty() {
            out.push_str("    input:\n");
            for inp in &step.inputs {
                out.push_str(&format!("    path '{}'\n", inp));
            }
        }
        if !step.outputs.is_empty() {
            out.push_str("    output:\n");
            for outp in &step.outputs {
                out.push_str(&format!("    path '{}'\n", outp));
            }
        }
        out.push_str("\n    script:\n    \"\"\"\n");

        // Rewrite {wildcard} → ${wildcard_val} and {params.key} → ${params.key}.
        let mut cmd = step.cmd.clone();
        let mut wc_keys: Vec<&String> = def.wildcards.keys().collect();
        wc_keys.sort();
        for k in &wc_keys {
            cmd = cmd.replace(&format!("{{{k}}}"), &format!("${{{k}}}"));
        }
        let mut p_keys: Vec<&String> = def.params.keys().collect();
        p_keys.sort();
        for k in &p_keys {
            cmd = cmd.replace(&format!("{{params.{k}}}"), &format!("${{params.{k}}}"));
        }

        out.push_str(&format!("    {cmd}\n"));
        out.push_str("    \"\"\"\n}\n\n");
    }

    // workflow block (linear chain, simplified).
    out.push_str("workflow {\n");
    if !def.wildcards.is_empty() {
        out.push_str("    ch = samples_ch\n");
    }
    for step in &def.steps {
        let process = step.name.to_uppercase();
        if def.wildcards.is_empty() {
            out.push_str(&format!("    {process}()\n"));
        } else if step.gather {
            out.push_str(&format!("    {process}(ch.collect())\n"));
        } else {
            out.push_str(&format!("    ch = {process}(ch)\n"));
        }
    }
    out.push_str("}\n");

    out
}
