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
/// depends_on = ["fastp"]
/// cmd        = "multiqc qc/ -o results/multiqc/ --force"
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
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
use std::time::SystemTime;

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
    /// Optional shell preamble executed before the main command.
    ///
    /// Use this to activate conda environments, set interpreter paths, or
    /// configure environment variables for tools that need a specific runtime
    /// (e.g., a step requiring Python 2 vs. Python 3).
    ///
    /// Example values:
    /// - `"conda activate py2_env &&"`
    /// - `"source /opt/py2/bin/activate &&"`
    /// - `"export PATH=/opt/star-2.7.11b/bin:$PATH &&"`
    #[serde(default)]
    pub env: Option<String>,
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
    /// Optional shell preamble (e.g., conda activate, PATH override).
    pub env: Option<String>,
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
                env: step.env.clone(),
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
                    env: step.env.clone(),
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
        format!(
            "{}h {:02}m {:02}s",
            secs / 3600,
            (secs % 3600) / 60,
            secs % 60
        )
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

        println!("  {:>9}  {}", phase_num.cyan(), group_strs.join("  │  "));

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

    // Build the full shell command, prepending env preamble if set.
    let full_cmd = match &task.env {
        Some(env) => format!("{env} {}", task.cmd),
        None => task.cmd.clone(),
    };

    let status = Command::new("sh")
        .arg("-c")
        .arg(&full_cmd)
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
    if let Some(env) = &task.env {
        println!("    env:     {}", env.dimmed());
    }
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

// ─── Verify ───────────────────────────────────────────────────────────────────

/// A single diagnostic produced by [`verify`].
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagLevel,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagLevel {
    Error,
    Warning,
}

impl std::fmt::Display for DiagLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagLevel::Error => write!(f, "error"),
            DiagLevel::Warning => write!(f, "warning"),
        }
    }
}

/// Verify a parsed [`WorkflowDef`] for semantic correctness and return
/// a list of diagnostics (errors + warnings).
///
/// Returns `Ok(diags)` even if there are errors — the caller decides whether
/// to abort.  The vector is empty when the workflow is fully valid.
pub fn verify(def: &WorkflowDef) -> Vec<Diagnostic> {
    let mut diags: Vec<Diagnostic> = Vec::new();

    // ── Metadata checks ──────────────────────────────────────────────────────
    if def.workflow.name.is_empty() {
        diags.push(Diagnostic {
            level: DiagLevel::Warning,
            message: "[workflow] name is empty".to_string(),
        });
    }

    // ── Step name uniqueness ─────────────────────────────────────────────────
    let mut seen_names: HashSet<&str> = HashSet::new();
    for step in &def.steps {
        if step.name.is_empty() {
            diags.push(Diagnostic {
                level: DiagLevel::Error,
                message: "A step has an empty name".to_string(),
            });
        }
        if !seen_names.insert(step.name.as_str()) {
            diags.push(Diagnostic {
                level: DiagLevel::Error,
                message: format!("Duplicate step name: '{}'", step.name),
            });
        }
        if step.cmd.trim().is_empty() {
            diags.push(Diagnostic {
                level: DiagLevel::Error,
                message: format!("Step '{}' has an empty cmd", step.name),
            });
        }
    }

    // ── Unknown depends_on references ────────────────────────────────────────
    let all_names: HashSet<&str> = def.steps.iter().map(|s| s.name.as_str()).collect();
    for step in &def.steps {
        for dep in &step.depends_on {
            if !all_names.contains(dep.as_str()) {
                diags.push(Diagnostic {
                    level: DiagLevel::Error,
                    message: format!("Step '{}' depends on unknown step '{dep}'", step.name),
                });
            }
        }
    }

    // ── Forward-dependency ordering (informational) ──────────────────────────
    let name_to_pos: HashMap<&str, usize> = def
        .steps
        .iter()
        .enumerate()
        .map(|(i, s)| (s.name.as_str(), i))
        .collect();
    for step in &def.steps {
        let step_pos = name_to_pos[step.name.as_str()];
        for dep in &step.depends_on {
            if let Some(&dep_pos) = name_to_pos.get(dep.as_str())
                && dep_pos > step_pos
            {
                diags.push(Diagnostic {
                    level: DiagLevel::Warning,
                    message: format!(
                        "Step '{}' references '{}' which appears later in the file — \
                         dependency will be silently ignored at runtime",
                        step.name, dep
                    ),
                });
            }
        }
    }

    // ── Wildcard references in cmds ──────────────────────────────────────────
    let wc_keys: HashSet<String> = def.wildcards.keys().cloned().collect();
    let param_keys: HashSet<String> = def.params.keys().cloned().collect();
    for step in &def.steps {
        // Find {placeholder} references in cmd, inputs, outputs.
        let all_text = std::iter::once(step.cmd.as_str())
            .chain(step.inputs.iter().map(|s| s.as_str()))
            .chain(step.outputs.iter().map(|s| s.as_str()));
        for text in all_text {
            // Scan for {placeholder} patterns, skipping shell-style ${var} expansions.
            let bytes = text.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i] == b'{'
                    // Skip ${...} — shell variable expansion, not a workflow placeholder.
                    && (i == 0 || bytes[i - 1] != b'$')
                {
                    let rest = &text[i + 1..];
                    if let Some(end) = rest.find('}') {
                        let placeholder = &rest[..end];
                        if let Some(key) = placeholder.strip_prefix("params.") {
                            if !param_keys.contains(key) {
                                diags.push(Diagnostic {
                                    level: DiagLevel::Warning,
                                    message: format!(
                                        "Step '{}' references {{params.{key}}} but it is not defined in [params]",
                                        step.name
                                    ),
                                });
                            }
                        } else if !placeholder.is_empty()
                            && !wc_keys.contains(placeholder)
                            && !param_keys.contains(placeholder)
                        {
                            diags.push(Diagnostic {
                                level: DiagLevel::Warning,
                                message: format!(
                                    "Step '{}' references {{{placeholder}}} which is not a defined wildcard or param",
                                    step.name
                                ),
                            });
                        }
                        i += 1 + end + 1; // skip past the closing '}'
                        continue;
                    }
                }
                i += 1;
            }
        }
    }

    // ── Cycle detection via DAG expand ───────────────────────────────────────
    if diags.iter().all(|d| d.level != DiagLevel::Error)
        && let Err(e) = expand(def)
    {
        diags.push(Diagnostic {
            level: DiagLevel::Error,
            message: format!("DAG expansion failed: {e}"),
        });
    }

    diags
}

/// Print verify diagnostics to stdout in a human-friendly format.
/// Returns `true` if there are any errors.
pub fn print_verify_report(def: &WorkflowDef, diags: &[Diagnostic]) -> bool {
    let errors = diags.iter().filter(|d| d.level == DiagLevel::Error).count();
    let warnings = diags
        .iter()
        .filter(|d| d.level == DiagLevel::Warning)
        .count();

    println!(
        "{} workflow '{}' — {} step(s), {} wildcard(s)",
        "◆".cyan().bold(),
        def.workflow.name.bold(),
        def.steps.len(),
        def.wildcards.len()
    );

    if diags.is_empty() {
        println!("{} No issues found — workflow is valid", "✓".green().bold());
        return false;
    }

    println!("{}", "─".repeat(60).dimmed());
    for d in diags {
        let (icon, colored_level) = match d.level {
            DiagLevel::Error => (
                "✗".red().bold().to_string(),
                "error".red().bold().to_string(),
            ),
            DiagLevel::Warning => (
                "⚠".yellow().bold().to_string(),
                "warning".yellow().bold().to_string(),
            ),
        };
        println!("  {} [{}] {}", icon, colored_level, d.message);
    }
    println!("{}", "─".repeat(60).dimmed());
    println!(
        "  {} error(s), {} warning(s)",
        if errors > 0 {
            errors.to_string().red().bold().to_string()
        } else {
            errors.to_string()
        },
        if warnings > 0 {
            warnings.to_string().yellow().bold().to_string()
        } else {
            warnings.to_string()
        },
    );

    errors > 0
}

// ─── Format ───────────────────────────────────────────────────────────────────

/// Serialize a [`WorkflowDef`] back to canonical `.oxo.toml` TOML text.
///
/// The output is deterministically ordered:
/// `[workflow]` → `[wildcards]` → `[params]` → `[[step]]` blocks.
pub fn format_toml(def: &WorkflowDef) -> String {
    let mut out = String::new();

    // ── [workflow] ────────────────────────────────────────────────────────────
    out.push_str("[workflow]\n");
    out.push_str(&format!("name        = {:?}\n", def.workflow.name));
    if !def.workflow.description.is_empty() {
        out.push_str(&format!("description = {:?}\n", def.workflow.description));
    }
    if !def.workflow.version.is_empty() {
        out.push_str(&format!("version     = {:?}\n", def.workflow.version));
    }
    out.push('\n');

    // ── [wildcards] ────────────────────────────────────────────────────────────
    if !def.wildcards.is_empty() {
        out.push_str("[wildcards]\n");
        let mut keys: Vec<&String> = def.wildcards.keys().collect();
        keys.sort();
        for k in &keys {
            let vals = &def.wildcards[*k];
            let quoted: Vec<String> = vals.iter().map(|v| format!("{v:?}")).collect();
            out.push_str(&format!("{k} = [{}]\n", quoted.join(", ")));
        }
        out.push('\n');
    }

    // ── [params] ──────────────────────────────────────────────────────────────
    if !def.params.is_empty() {
        out.push_str("[params]\n");
        let mut pkeys: Vec<&String> = def.params.keys().collect();
        pkeys.sort();
        for k in &pkeys {
            out.push_str(&format!("{k:<12} = {:?}\n", def.params[*k]));
        }
        out.push('\n');
    }

    // ── [[step]] blocks ────────────────────────────────────────────────────────
    for step in &def.steps {
        out.push_str("[[step]]\n");
        out.push_str(&format!("name       = {:?}\n", step.name));
        if step.gather {
            out.push_str("gather     = true\n");
        }
        if let Some(env) = &step.env {
            out.push_str(&format!("env        = {:?}\n", env));
        }
        if !step.depends_on.is_empty() {
            let deps: Vec<String> = step.depends_on.iter().map(|d| format!("{d:?}")).collect();
            out.push_str(&format!("depends_on = [{}]\n", deps.join(", ")));
        }
        // cmd: use multi-line literal for long commands
        let cmd = &step.cmd;
        if cmd.contains('\n') || cmd.len() > MAX_INLINE_CMD_LENGTH {
            // Render as triple-quoted string with backslash-continuation lines.
            out.push_str("cmd        = \"\"\"\\\n");
            for line in cmd.lines() {
                out.push_str(&format!("{line}\n"));
            }
            out.push_str("\"\"\"\n");
        } else {
            out.push_str(&format!("cmd        = {:?}\n", cmd));
        }
        if !step.inputs.is_empty() {
            let ins: Vec<String> = step.inputs.iter().map(|i| format!("{i:?}")).collect();
            out.push_str(&format!("inputs     = [{}]\n", ins.join(", ")));
        }
        if !step.outputs.is_empty() {
            let outs: Vec<String> = step.outputs.iter().map(|o| format!("{o:?}")).collect();
            out.push_str(&format!("outputs    = [{}]\n", outs.join(", ")));
        }
        out.push('\n');
    }

    // Remove trailing newline.
    out.pop();
    out
}

// Maximum command length for single-line formatting; longer commands use multi-line syntax.
const MAX_INLINE_CMD_LENGTH: usize = 80;

/// Print a rich DAG visualisation for a workflow to stdout.
///
/// Shows the phase diagram, step dependency table, and a summary.
pub fn visualize_workflow(def: &WorkflowDef) -> Result<()> {
    let tasks = expand(def)?;
    let phases = compute_phases(&tasks);

    println!();
    println!(
        "{} {} {}",
        "◆".cyan().bold(),
        format!("Workflow: {}", def.workflow.name).bold(),
        format!(
            "({} steps, {} tasks, {} phases)",
            def.steps.len(),
            tasks.len(),
            phases.len()
        )
        .dimmed()
    );
    if !def.workflow.description.is_empty() {
        println!("  {}", def.workflow.description.dimmed());
    }

    // ── Wildcard summary ──────────────────────────────────────────────────────
    if !def.wildcards.is_empty() {
        println!();
        println!("  {}", "Wildcards:".bold());
        let mut keys: Vec<&String> = def.wildcards.keys().collect();
        keys.sort();
        for k in &keys {
            let vals = &def.wildcards[k.as_str()];
            let preview = if vals.len() <= 4 {
                vals.join(", ")
            } else {
                format!("{}, … ({} total)", vals[..3].join(", "), vals.len())
            };
            println!("    {:<12} = [{}]", k.cyan(), preview);
        }
    }

    // ── Phase diagram (reuse existing helper) ─────────────────────────────────
    println!();
    println!("{}", "─".repeat(60).dimmed());
    print_dag_phases(&tasks);
    println!("{}", "─".repeat(60).dimmed());

    // ── Per-step dependency table ─────────────────────────────────────────────
    println!();
    println!("  {}", "Step details:".bold());
    println!(
        "  {:<18} {:<8} {:<8} {}",
        "Step".bold(),
        "Gather".bold(),
        "Tasks".bold(),
        "Depends on".bold()
    );
    println!("  {}", "─".repeat(56).dimmed());
    for step in &def.steps {
        let task_count = tasks.iter().filter(|t| t.step_name == step.name).count();
        let gather_str = if step.gather { "yes" } else { "" };
        let deps_str = if step.depends_on.is_empty() {
            "(none)".dimmed().to_string()
        } else {
            step.depends_on.join(", ").cyan().to_string()
        };
        println!(
            "  {:<18} {:<8} {:<8} {}",
            step.name.cyan(),
            gather_str,
            task_count,
            deps_str
        );
    }

    println!();
    Ok(())
}

/// Post-execution LLM verification for a completed workflow.
///
/// Inspects each step's expected output files, then asks the LLM to summarise
/// the overall state of the workflow outputs and surface any issues.
///
/// This is a best-effort check: failures from the LLM are printed as warnings
/// rather than propagated as errors, so the exit code of `workflow run` is not
/// affected.
#[cfg(not(target_arch = "wasm32"))]
pub async fn verify_workflow_results(def: &WorkflowDef, config: &crate::config::Config) {
    use crate::llm::LlmClient;
    use crate::runner::make_spinner;
    use colored::Colorize;

    // Collect output file metadata for every step.
    let mut step_lines: Vec<String> = Vec::new();
    for step in &def.steps {
        step_lines.push(format!(
            "### Step: `{}`\nCommand: `{}`",
            step.name, step.cmd
        ));
        if step.outputs.is_empty() {
            step_lines.push("Outputs: (none declared)".to_string());
        } else {
            for path in &step.outputs {
                let size_info = match std::fs::metadata(path).ok().map(|m| m.len()) {
                    Some(bytes) => format!("{bytes} bytes"),
                    None => "**MISSING**".to_string(),
                };
                step_lines.push(format!("- `{path}` — {size_info}"));
            }
        }
    }

    // Collect all declared output files with sizes for the LLM call.
    let all_outputs: Vec<(String, Option<u64>)> = def
        .steps
        .iter()
        .flat_map(|s| s.outputs.iter())
        .map(|p| {
            let size = std::fs::metadata(p).ok().map(|m| m.len());
            (p.clone(), size)
        })
        .collect();

    let step_summary = step_lines.join("\n");
    let workflow_task = format!(
        "Workflow '{}': {}\n\n## Step Output Status\n{}",
        def.workflow.name, def.workflow.description, step_summary
    );
    let workflow_cmd = format!("oxo-call workflow run ({})", def.workflow.name);

    let spinner = make_spinner("Verifying workflow results with LLM...");
    let llm = LlmClient::new(config.clone());
    let verification = match llm
        .verify_run_result(
            &def.workflow.name,
            &workflow_task,
            &workflow_cmd,
            0,
            "",
            &all_outputs,
        )
        .await
    {
        Ok(v) => {
            spinner.finish_and_clear();
            v
        }
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!(
                "{} LLM workflow verification failed: {}",
                "warning:".yellow().bold(),
                e
            );
            return;
        }
    };

    println!();
    println!("{}", "─".repeat(60).dimmed());
    let label = if verification.success {
        "LLM Workflow Verification: OK".bold().green().to_string()
    } else {
        "LLM Workflow Verification: Issues detected"
            .bold()
            .red()
            .to_string()
    };
    println!("  {}", label);
    if !verification.summary.is_empty() {
        println!("  {}", verification.summary);
    }
    if !verification.issues.is_empty() {
        println!();
        println!("  {}", "Issues:".bold().yellow());
        for issue in &verification.issues {
            println!("    {} {}", "•".yellow(), issue);
        }
    }
    if !verification.suggestions.is_empty() {
        println!();
        println!("  {}", "Suggestions:".bold().cyan());
        for sug in &verification.suggestions {
            println!("    {} {}", "→".cyan(), sug);
        }
    }
    println!("{}", "─".repeat(60).dimmed());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to build a minimal ConcreteTask for testing.
    fn task(id: &str, deps: &[&str]) -> ConcreteTask {
        ConcreteTask {
            id: id.to_string(),
            step_name: id.split('[').next().unwrap_or(id).to_string(),
            cmd: format!("echo {id}"),
            inputs: vec![],
            outputs: vec![],
            deps: deps.iter().map(|d| d.to_string()).collect(),
            gather: false,
            env: None,
        }
    }

    fn gather_task(id: &str, deps: &[&str]) -> ConcreteTask {
        let mut t = task(id, deps);
        t.gather = true;
        t
    }

    #[test]
    fn test_compute_phases_linear_chain() {
        let tasks = vec![task("a", &[]), task("b", &["a"]), task("c", &["b"])];
        let phases = compute_phases(&tasks);
        assert_eq!(phases.len(), 3);
        assert_eq!(phases[0].len(), 1);
        assert_eq!(phases[0][0].id, "a");
        assert_eq!(phases[1][0].id, "b");
        assert_eq!(phases[2][0].id, "c");
    }

    #[test]
    fn test_compute_phases_parallel_tasks() {
        let tasks = vec![task("a", &[]), task("b", &[]), task("c", &["a", "b"])];
        let phases = compute_phases(&tasks);
        assert_eq!(phases.len(), 2);
        // Phase 1: a and b in parallel.
        assert_eq!(phases[0].len(), 2);
        // Phase 2: c after both.
        assert_eq!(phases[1].len(), 1);
        assert_eq!(phases[1][0].id, "c");
    }

    #[test]
    fn test_compute_phases_diamond_dag() {
        // a → b, a → c, b+c → d
        let tasks = vec![
            task("a", &[]),
            task("b", &["a"]),
            task("c", &["a"]),
            task("d", &["b", "c"]),
        ];
        let phases = compute_phases(&tasks);
        assert_eq!(phases.len(), 3);
        assert_eq!(phases[0].len(), 1); // a
        assert_eq!(phases[1].len(), 2); // b, c
        assert_eq!(phases[2].len(), 1); // d
    }

    #[test]
    fn test_compute_phases_with_gather() {
        // Mimics rnaseq: fastp[s1], fastp[s2] → multiqc (gather)
        let tasks = vec![
            task("fastp[sample=s1]", &[]),
            task("fastp[sample=s2]", &[]),
            gather_task("multiqc", &["fastp[sample=s1]", "fastp[sample=s2]"]),
        ];
        let phases = compute_phases(&tasks);
        assert_eq!(phases.len(), 2);
        assert_eq!(phases[0].len(), 2); // both fastp tasks
        assert_eq!(phases[1].len(), 1); // multiqc
        assert!(phases[1][0].gather);
    }

    #[test]
    fn test_compute_phases_empty() {
        let tasks: Vec<ConcreteTask> = vec![];
        let phases = compute_phases(&tasks);
        assert_eq!(phases.len(), 0);
    }

    #[test]
    fn test_compute_phases_single_task() {
        let tasks = vec![task("only", &[])];
        let phases = compute_phases(&tasks);
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].len(), 1);
    }

    #[test]
    fn test_compute_phases_complex_pipeline() {
        // Mimics the real RNA-seq pipeline with upstream QC aggregation:
        // fastp[s1,s2] → { multiqc (gather), star[s1,s2] } → samtools_index[s1,s2] → featurecounts[s1,s2]
        // MultiQC is an upstream QC aggregation step that depends on fastp only
        // and runs in parallel with the STAR alignment branch.
        let tasks = vec![
            task("fastp[sample=s1]", &[]),
            task("fastp[sample=s2]", &[]),
            gather_task("multiqc", &["fastp[sample=s1]", "fastp[sample=s2]"]),
            task("star[sample=s1]", &["fastp[sample=s1]"]),
            task("star[sample=s2]", &["fastp[sample=s2]"]),
            task("samtools_index[sample=s1]", &["star[sample=s1]"]),
            task("samtools_index[sample=s2]", &["star[sample=s2]"]),
            task("featurecounts[sample=s1]", &["samtools_index[sample=s1]"]),
            task("featurecounts[sample=s2]", &["samtools_index[sample=s2]"]),
        ];
        let phases = compute_phases(&tasks);
        assert_eq!(phases.len(), 4);
        assert_eq!(phases[0].len(), 2); // fastp[s1], fastp[s2]
        // Phase 2: multiqc + star[s1,s2] run in parallel (all depend only on fastp)
        assert_eq!(phases[1].len(), 3); // multiqc, star[s1], star[s2]
        assert!(
            phases[1].iter().any(|t| t.id == "multiqc"),
            "multiqc should be in the same phase as star"
        );
        assert_eq!(phases[2].len(), 2); // samtools_index[s1], samtools_index[s2]
        assert_eq!(phases[3].len(), 2); // featurecounts[s1], featurecounts[s2]
    }

    #[test]
    fn test_format_elapsed_seconds() {
        let d = std::time::Duration::from_secs_f64(5.3);
        assert_eq!(format_elapsed(d), "5.3s");
    }

    #[test]
    fn test_format_elapsed_minutes() {
        let d = std::time::Duration::from_secs(125);
        assert_eq!(format_elapsed(d), "2m 05s");
    }

    #[test]
    fn test_format_elapsed_hours() {
        let d = std::time::Duration::from_secs(3723);
        assert_eq!(format_elapsed(d), "1h 02m 03s");
    }

    #[test]
    fn test_expand_rnaseq_template() {
        // Verify that the built-in RNA-seq template parses and expands correctly.
        let toml = include_str!("../workflows/native/rnaseq.toml");
        let def = WorkflowDef::from_str_content(toml).expect("parse rnaseq template");
        let tasks = expand(&def).expect("expand rnaseq DAG");

        // rnaseq has 3 samples and 5 steps:
        // Per-sample: fastp, star, samtools_index, featurecounts (4 × 3 = 12)
        // Gather: multiqc (1)
        assert_eq!(tasks.len(), 13);

        // multiqc is an upstream QC aggregation step that depends on fastp
        let multiqc = tasks.iter().find(|t| t.step_name == "multiqc").unwrap();
        assert!(multiqc.gather);
        // multiqc depends on all fastp instances
        assert_eq!(multiqc.deps.len(), 3);
        for dep in &multiqc.deps {
            assert!(
                dep.starts_with("fastp["),
                "multiqc dep should be fastp: {dep}"
            );
        }

        // Verify phases: multiqc runs in parallel with star (both depend on fastp)
        let phases = compute_phases(&tasks);
        assert_eq!(phases[0].len(), 3); // 3 fastp tasks
        // multiqc and star should be in the same phase (phase 1)
        let multiqc_phase = phases
            .iter()
            .position(|p| p.iter().any(|t| t.step_name == "multiqc"))
            .unwrap();
        let star_phase = phases
            .iter()
            .position(|p| p.iter().any(|t| t.step_name == "star"))
            .unwrap();
        assert_eq!(
            multiqc_phase, star_phase,
            "multiqc and star should execute in the same phase"
        );
    }

    #[test]
    fn test_expand_chipseq_template() {
        // ChIP-seq has parallel macs3 + bigwig branches.
        let toml = include_str!("../workflows/native/chipseq.toml");
        let def = WorkflowDef::from_str_content(toml).expect("parse chipseq template");
        let tasks = expand(&def).expect("expand chipseq DAG");

        // multiqc is upstream QC and depends only on fastp
        let multiqc = tasks.iter().find(|t| t.step_name == "multiqc").unwrap();
        assert!(multiqc.gather);
        assert_eq!(multiqc.deps.len(), 3); // 3 samples × fastp
        for dep in &multiqc.deps {
            assert!(
                dep.starts_with("fastp["),
                "multiqc dep should be fastp: {dep}"
            );
        }

        // multiqc and bowtie2 should be in the same phase (both depend on fastp)
        let phases = compute_phases(&tasks);
        let multiqc_phase = phases
            .iter()
            .position(|p| p.iter().any(|t| t.step_name == "multiqc"))
            .unwrap();
        let bowtie2_phase = phases
            .iter()
            .position(|p| p.iter().any(|t| t.step_name == "bowtie2"))
            .unwrap();
        assert_eq!(
            multiqc_phase, bowtie2_phase,
            "multiqc and bowtie2 should execute in the same phase"
        );

        // Verify parallel phases: macs3 and bigwig should be in the same phase
        let macs3_phase = phases
            .iter()
            .position(|p| p.iter().any(|t| t.step_name == "macs3"))
            .unwrap();
        let bigwig_phase = phases
            .iter()
            .position(|p| p.iter().any(|t| t.step_name == "bigwig"))
            .unwrap();
        assert_eq!(
            macs3_phase, bigwig_phase,
            "macs3 and bigwig should execute in the same phase"
        );
    }

    #[test]
    fn test_expand_longreads_template() {
        // Long-reads has parallel nanostat + flye branches.
        let toml = include_str!("../workflows/native/longreads.toml");
        let def = WorkflowDef::from_str_content(toml).expect("parse longreads template");
        let tasks = expand(&def).expect("expand longreads DAG");

        // Verify nanostat and flye are in the same phase (both depend on nanoq only)
        let phases = compute_phases(&tasks);
        let nanostat_phase = phases
            .iter()
            .position(|p| p.iter().any(|t| t.step_name == "nanostat"))
            .unwrap();
        let flye_phase = phases
            .iter()
            .position(|p| p.iter().any(|t| t.step_name == "flye"))
            .unwrap();
        assert_eq!(
            nanostat_phase, flye_phase,
            "nanostat and flye should execute in parallel"
        );
    }

    #[test]
    fn test_all_builtin_templates_parse_and_expand() {
        // Ensure every built-in template can be parsed and expanded without errors.
        let templates = [
            ("rnaseq", include_str!("../workflows/native/rnaseq.toml")),
            ("wgs", include_str!("../workflows/native/wgs.toml")),
            ("atacseq", include_str!("../workflows/native/atacseq.toml")),
            ("chipseq", include_str!("../workflows/native/chipseq.toml")),
            (
                "metagenomics",
                include_str!("../workflows/native/metagenomics.toml"),
            ),
            (
                "amplicon16s",
                include_str!("../workflows/native/amplicon16s.toml"),
            ),
            (
                "scrnaseq",
                include_str!("../workflows/native/scrnaseq.toml"),
            ),
            (
                "longreads",
                include_str!("../workflows/native/longreads.toml"),
            ),
            (
                "methylseq",
                include_str!("../workflows/native/methylseq.toml"),
            ),
        ];
        for (name, toml) in &templates {
            let def = WorkflowDef::from_str_content(toml)
                .unwrap_or_else(|e| panic!("Failed to parse {name}: {e}"));
            let tasks = expand(&def).unwrap_or_else(|e| panic!("Failed to expand {name}: {e}"));
            assert!(!tasks.is_empty(), "{name} should have at least one task");

            // Verify multiqc is an upstream QC aggregation step (not the final phase)
            let multiqc = tasks.iter().find(|t| t.step_name == "multiqc");
            assert!(multiqc.is_some(), "{name}: should have a multiqc task");
            assert!(
                multiqc.unwrap().gather,
                "{name}: multiqc should be a gather step"
            );
        }
    }

    #[test]
    fn test_env_field_parsing_and_expansion() {
        // Verify that the optional `env` field is parsed and propagated to tasks.
        let toml = r#"
[workflow]
name = "env-test"

[wildcards]
sample = ["s1"]

[[step]]
name = "py2_step"
env  = "conda activate py2_env &&"
cmd  = "python2 script.py {sample}"

[[step]]
name       = "py3_step"
depends_on = ["py2_step"]
cmd        = "python3 analysis.py {sample}"
"#;
        let def = WorkflowDef::from_str_content(toml).expect("parse env-test");
        assert_eq!(
            def.steps[0].env.as_deref(),
            Some("conda activate py2_env &&")
        );
        assert!(def.steps[1].env.is_none());

        let tasks = expand(&def).expect("expand env-test");
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].env.as_deref(), Some("conda activate py2_env &&"));
        assert!(tasks[1].env.is_none());
    }

    #[test]
    fn test_format_toml_includes_env() {
        let toml = r#"
[workflow]
name = "env-fmt"

[[step]]
name = "step1"
env  = "source /opt/venv/bin/activate &&"
cmd  = "run_tool"
"#;
        let def = WorkflowDef::from_str_content(toml).expect("parse");
        let formatted = format_toml(&def);
        assert!(
            formatted.contains("env"),
            "formatted TOML should include env field"
        );
        assert!(
            formatted.contains("source /opt/venv/bin/activate"),
            "env value should be preserved"
        );
    }
}
