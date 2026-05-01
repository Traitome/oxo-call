//! User-defined named jobs — a command-shortcut library with full lifecycle
//! management.
//!
//! `job` lets users store named shell command shortcuts and manage their full
//! lifecycle: description, tags, scheduling (cron), execution history, status.
//!
//! Jobs are stored in `~/.local/share/oxo-call/jobs.toml`.
//! Execution run history is stored in `~/.local/share/oxo-call/job_runs.jsonl`.
//!
//! On first use the old `cmds.toml` is automatically migrated to `jobs.toml`.

use crate::config::Config;
use crate::error::{OxoError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ─── Data structures ──────────────────────────────────────────────────────────

/// A single user-defined job entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEntry {
    /// Short name used to invoke this job (must be unique).
    pub name: String,
    /// The shell command string to execute.
    pub command: String,
    /// Optional human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional tags for filtering and organization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Optional cron expression for scheduled execution (e.g. "0 * * * *").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
    /// Total number of times this job has been executed.
    #[serde(default)]
    pub run_count: u64,
    /// Timestamp of the most recent execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run: Option<DateTime<Utc>>,
    /// Exit code from the most recent execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_exit_code: Option<i32>,
    /// When the entry was first created.
    pub created_at: DateTime<Utc>,
    /// When the entry was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Top-level document stored in `jobs.toml`.
#[derive(Debug, Default, Serialize, Deserialize)]
struct JobFile {
    #[serde(default)]
    jobs: Vec<JobEntry>,
}

// ─── Execution run record ────────────────────────────────────────────────────

/// One execution record appended to `job_runs.jsonl`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRun {
    /// Name of the job that was run.
    pub job_name: String,
    /// The actual command string that was executed.
    pub command: String,
    /// Optional remote server name used for SSH execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    /// Exit code (0 = success).
    pub exit_code: i32,
    /// When the execution started.
    pub started_at: DateTime<Utc>,
    /// Duration in seconds.
    pub duration_secs: f64,
}

/// Manages the per-job run history stored in `job_runs.jsonl`.
pub struct JobRunStore;

impl JobRunStore {
    fn runs_path() -> Result<PathBuf> {
        Ok(Config::data_dir()?.join("job_runs.jsonl"))
    }

    /// Append one run record.
    pub fn append(run: &JobRun) -> Result<()> {
        let path = Self::runs_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let line = serde_json::to_string(run)?;
        use std::io::Write as _;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(f, "{line}")?;
        Ok(())
    }

    /// Load all run records (oldest first), optionally filtered by job name.
    pub fn load(job_name_filter: Option<&str>) -> Result<Vec<JobRun>> {
        let path = Self::runs_path()?;
        if !path.exists() {
            return Ok(vec![]);
        }
        let content = std::fs::read_to_string(&path)?;
        let mut runs: Vec<JobRun> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        if let Some(name) = job_name_filter {
            runs.retain(|r| r.job_name == name);
        }
        Ok(runs)
    }
}

// ─── Built-in job templates ──────────────────────────────────────────────────

/// A built-in read-only job template shipped with oxo-call.
#[derive(Debug, Clone)]
pub struct BuiltinJob {
    pub name: &'static str,
    pub command: &'static str,
    pub description: &'static str,
    pub tags: &'static [&'static str],
}

/// Pre-defined easy-to-remember jobs for common operations.
pub const BUILTIN_JOBS: &[BuiltinJob] = &[
    // ── System info ─────────────────────────────────────────────────────────
    BuiltinJob {
        name: "disk",
        command: "df -h",
        description: "Show disk usage in human-readable form",
        tags: &["system", "ops"],
    },
    BuiltinJob {
        name: "mem",
        command: "free -h",
        description: "Show memory usage",
        tags: &["system", "ops"],
    },
    BuiltinJob {
        name: "cpu",
        command: "top -bn1 | head -20",
        description: "Show CPU and process info (one snapshot)",
        tags: &["system", "ops"],
    },
    BuiltinJob {
        name: "gpu",
        command: "nvidia-smi",
        description: "Show NVIDIA GPU status and utilisation",
        tags: &["gpu", "ops"],
    },
    BuiltinJob {
        name: "ps-me",
        command: "ps aux | grep $USER",
        description: "List all processes belonging to the current user",
        tags: &["system", "ops"],
    },
    // ── Network ─────────────────────────────────────────────────────────────
    BuiltinJob {
        name: "ports",
        command: "ss -tulnp",
        description: "List listening TCP/UDP ports and the owning processes",
        tags: &["network", "ops"],
    },
    BuiltinJob {
        name: "iface",
        command: "ip addr show",
        description: "Show all network interfaces and their addresses",
        tags: &["network", "ops"],
    },
    // ── Filesystem / files ───────────────────────────────────────────────────
    BuiltinJob {
        name: "big-files",
        command: "du -sh * 2>/dev/null | sort -rh | head -20",
        description: "Top 20 largest items in the current directory",
        tags: &["fs", "ops"],
    },
    BuiltinJob {
        name: "find-big",
        command: "find . -type f -size +100M -exec ls -lh {} \\; 2>/dev/null",
        description: "Find files larger than 100 MB under the current directory",
        tags: &["fs", "ops"],
    },
    // ── SLURM cluster ───────────────────────────────────────────────────────
    BuiltinJob {
        name: "squeue-me",
        command: "squeue -u $USER",
        description: "Show my SLURM jobs",
        tags: &["slurm", "hpc", "cluster"],
    },
    BuiltinJob {
        name: "squeue-all",
        command: "squeue -a",
        description: "Show all SLURM jobs in the queue",
        tags: &["slurm", "hpc", "cluster"],
    },
    BuiltinJob {
        name: "sacct-me",
        command: "sacct -u $USER --format=JobID,JobName,State,ExitCode,Elapsed,Start,End",
        description: "Show recent SLURM accounting records for the current user",
        tags: &["slurm", "hpc", "cluster"],
    },
    BuiltinJob {
        name: "sinfo",
        command: "sinfo -o '%20P %5a %.10l %6D %6t %N'",
        description: "Show SLURM partition/node status",
        tags: &["slurm", "hpc", "cluster"],
    },
    BuiltinJob {
        name: "scancel-me",
        command: "scancel -u $USER",
        description: "Cancel ALL of my running/pending SLURM jobs",
        tags: &["slurm", "hpc", "cluster"],
    },
    // ── PBS/Torque cluster ───────────────────────────────────────────────────
    BuiltinJob {
        name: "qstat-me",
        command: "qstat -u $USER",
        description: "Show my PBS/Torque jobs",
        tags: &["pbs", "hpc", "cluster"],
    },
    BuiltinJob {
        name: "pbsnodes",
        command: "pbsnodes -a | grep -E 'state|jobs|np'",
        description: "Show PBS node states and running jobs",
        tags: &["pbs", "hpc", "cluster"],
    },
    // ── LSF cluster ─────────────────────────────────────────────────────────
    BuiltinJob {
        name: "bjobs-me",
        command: "bjobs",
        description: "Show my LSF jobs",
        tags: &["lsf", "hpc", "cluster"],
    },
    BuiltinJob {
        name: "bhosts",
        command: "bhosts",
        description: "Show LSF host status",
        tags: &["lsf", "hpc", "cluster"],
    },
    // ── Kubernetes ──────────────────────────────────────────────────────────
    BuiltinJob {
        name: "k8s-pods",
        command: "kubectl get pods --all-namespaces",
        description: "List all Kubernetes pods across all namespaces",
        tags: &["k8s", "cluster"],
    },
    BuiltinJob {
        name: "k8s-nodes",
        command: "kubectl get nodes -o wide",
        description: "Show Kubernetes node status with IPs",
        tags: &["k8s", "cluster"],
    },
    BuiltinJob {
        name: "k8s-events",
        command: "kubectl get events --sort-by=.lastTimestamp",
        description: "Show recent Kubernetes events sorted by time",
        tags: &["k8s", "cluster"],
    },
    // ── Docker ──────────────────────────────────────────────────────────────
    BuiltinJob {
        name: "docker-ps",
        command: "docker ps --format 'table {{.ID}}\\t{{.Image}}\\t{{.Status}}\\t{{.Names}}'",
        description: "List running Docker containers in a table",
        tags: &["docker", "ops"],
    },
    BuiltinJob {
        name: "docker-clean",
        command: "docker system prune -f",
        description: "Remove stopped containers, dangling images, and unused networks",
        tags: &["docker", "ops"],
    },
    // ── Git ─────────────────────────────────────────────────────────────────
    BuiltinJob {
        name: "git-log",
        command: "git log --oneline --graph --decorate -20",
        description: "Show the last 20 commits as a compact graph",
        tags: &["git", "dev"],
    },
    BuiltinJob {
        name: "git-stash-list",
        command: "git stash list",
        description: "List all Git stash entries",
        tags: &["git", "dev"],
    },
    // ── Bioinformatics / data ────────────────────────────────────────────────
    BuiltinJob {
        name: "count-reads",
        command: "for f in *.fastq *.fastq.gz; do [ -f \"$f\" ] || continue; lines=$(zcat -f \"$f\" | wc -l); echo \"$f: $((lines / 4)) reads\"; done",
        description: "Count reads in all FASTQ / FASTQ.gz files in the current directory (uses zcat -f for both plain and gzipped)",
        tags: &["bioinformatics", "data"],
    },
    BuiltinJob {
        name: "bam-stats",
        command: "for f in *.bam; do echo \"=== $f ===\"; samtools flagstat $f; done",
        description: "Run samtools flagstat on every BAM in the current directory",
        tags: &["bioinformatics", "samtools"],
    },
    BuiltinJob {
        name: "find-bam",
        command: "find . -name '*.bam' -type f | sort",
        description: "Find all BAM files under the current directory",
        tags: &["bioinformatics", "fs"],
    },
    BuiltinJob {
        name: "find-fastq",
        command: "find . \\( -name '*.fastq' -o -name '*.fastq.gz' -o -name '*.fq' -o -name '*.fq.gz' \\) -type f | sort",
        description: "Find all FASTQ/FASTQ.gz files under the current directory",
        tags: &["bioinformatics", "fs"],
    },
    BuiltinJob {
        name: "bam-index",
        command: "for f in *.bam; do [ -f \"$f\" ] || continue; echo \"Indexing $f\"; samtools index \"$f\"; done",
        description: "Index all BAM files in the current directory with samtools",
        tags: &["bioinformatics", "samtools"],
    },
    // ── System / monitoring ──────────────────────────────────────────────────
    BuiltinJob {
        name: "uptime",
        command: "uptime",
        description: "Show system uptime and load averages",
        tags: &["system", "ops"],
    },
    BuiltinJob {
        name: "load",
        command: "cat /proc/loadavg",
        description: "Show current system load averages (1/5/15 min)",
        tags: &["system", "ops"],
    },
    BuiltinJob {
        name: "who",
        command: "who",
        description: "List users currently logged in",
        tags: &["system", "ops"],
    },
    BuiltinJob {
        name: "inode",
        command: "df -i",
        description: "Show inode usage per filesystem",
        tags: &["fs", "ops"],
    },
    BuiltinJob {
        name: "tmp",
        command: "du -sh /tmp/* 2>/dev/null | sort -rh | head -20",
        description: "Show the largest items in /tmp",
        tags: &["fs", "ops"],
    },
    BuiltinJob {
        name: "cpu-info",
        command: "lscpu",
        description: "Show detailed CPU architecture information",
        tags: &["system", "ops"],
    },
    BuiltinJob {
        name: "find-recent",
        command: "find . -type f -newer /tmp -mtime -1 2>/dev/null | sort",
        description: "Find files modified in the last 24 hours under the current directory",
        tags: &["fs", "ops"],
    },
    // ── SGE / Grid Engine cluster ────────────────────────────────────────────
    BuiltinJob {
        name: "qstat-sge",
        command: "qstat -u $USER",
        description: "Show my SGE/Grid Engine jobs",
        tags: &["sge", "hpc", "cluster"],
    },
    BuiltinJob {
        name: "qhost",
        command: "qhost",
        description: "Show SGE cluster host status",
        tags: &["sge", "hpc", "cluster"],
    },
    BuiltinJob {
        name: "qdel-me",
        command: "qdel -u $USER",
        description: "Delete all my SGE jobs",
        tags: &["sge", "hpc", "cluster"],
    },
    // ── Docker (extended) ────────────────────────────────────────────────────
    BuiltinJob {
        name: "docker-images",
        command: "docker images --format 'table {{.Repository}}\\t{{.Tag}}\\t{{.Size}}\\t{{.CreatedSince}}'",
        description: "List all local Docker images in a table",
        tags: &["docker", "ops"],
    },
    BuiltinJob {
        name: "docker-all",
        command: "docker ps -a --format 'table {{.ID}}\\t{{.Image}}\\t{{.Status}}\\t{{.Names}}'",
        description: "List all Docker containers including stopped ones",
        tags: &["docker", "ops"],
    },
    // ── Kubernetes (extended) ────────────────────────────────────────────────
    BuiltinJob {
        name: "k8s-svc",
        command: "kubectl get svc --all-namespaces",
        description: "List all Kubernetes services across all namespaces",
        tags: &["k8s", "cluster"],
    },
    BuiltinJob {
        name: "k8s-top",
        command: "kubectl top nodes",
        description: "Show Kubernetes node resource usage",
        tags: &["k8s", "cluster"],
    },
    // ── Git (extended) ───────────────────────────────────────────────────────
    BuiltinJob {
        name: "git-status",
        command: "git status --short --branch",
        description: "Show concise Git working-tree status",
        tags: &["git", "dev"],
    },
    BuiltinJob {
        name: "git-branch",
        command: "git branch -a --sort=-committerdate",
        description: "List all Git branches sorted by most recent commit",
        tags: &["git", "dev"],
    },
    // ── Development ──────────────────────────────────────────────────────────
    BuiltinJob {
        name: "conda-envs",
        command: "conda env list",
        description: "List all conda environments",
        tags: &["conda", "dev"],
    },
    BuiltinJob {
        name: "screen-ls",
        command: "screen -ls",
        description: "List all screen sessions",
        tags: &["dev", "ops"],
    },
    BuiltinJob {
        name: "tmux-ls",
        command: "tmux list-sessions 2>/dev/null || echo 'No tmux sessions'",
        description: "List all tmux sessions",
        tags: &["dev", "ops"],
    },
];

/// Return all built-in jobs, optionally filtered by tag.
pub fn builtin_jobs(tag_filter: Option<&str>) -> Vec<&'static BuiltinJob> {
    BUILTIN_JOBS
        .iter()
        .filter(|j| {
            if let Some(tag) = tag_filter {
                j.tags.contains(&tag)
            } else {
                true
            }
        })
        .collect()
}

// ─── Manager ─────────────────────────────────────────────────────────────────

/// Manages the user's personal job library.
pub struct JobManager;

impl JobManager {
    fn store_path() -> Result<PathBuf> {
        Ok(Config::data_dir()?.join("jobs.toml"))
    }

    /// Path of the legacy `cmds.toml` file (for migration).
    fn legacy_path() -> Result<PathBuf> {
        Ok(Config::data_dir()?.join("cmds.toml"))
    }

    fn load_file() -> Result<JobFile> {
        let path = Self::store_path()?;
        if !path.exists() {
            // Migrate from legacy cmds.toml if present
            return Self::migrate_from_legacy();
        }
        let content = std::fs::read_to_string(&path)?;
        toml::from_str(&content)
            .map_err(|e| OxoError::ConfigError(format!("failed to parse jobs.toml: {e}")))
    }

    /// Migrate old `cmds.toml` (CmdEntry-compatible) to the new `JobFile` format.
    fn migrate_from_legacy() -> Result<JobFile> {
        let legacy = Self::legacy_path()?;
        if !legacy.exists() {
            return Ok(JobFile::default());
        }
        // The old file used key `cmds`, parse it as raw TOML
        #[derive(Deserialize)]
        struct LegacyCmdEntry {
            name: String,
            command: String,
            #[serde(default)]
            description: Option<String>,
            #[serde(default)]
            tags: Vec<String>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }
        #[derive(Deserialize, Default)]
        struct LegacyFile {
            #[serde(default)]
            cmds: Vec<LegacyCmdEntry>,
        }
        let content = std::fs::read_to_string(&legacy)?;
        let lf: LegacyFile = toml::from_str(&content).unwrap_or_default();
        let jobs: Vec<JobEntry> = lf
            .cmds
            .into_iter()
            .map(|c| JobEntry {
                name: c.name,
                command: c.command,
                description: c.description,
                tags: c.tags,
                schedule: None,
                run_count: 0,
                last_run: None,
                last_exit_code: None,
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect();
        let file = JobFile { jobs };
        // Write the migrated data to jobs.toml and leave cmds.toml in place
        // so the user still has a backup.
        Self::save_file(&file)?;
        Ok(file)
    }

    fn save_file(file: &JobFile) -> Result<()> {
        let path = Self::store_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(file)
            .map_err(|e| OxoError::ConfigError(format!("failed to serialize jobs: {e}")))?;
        // Atomic write via temp file
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, &content)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// List all jobs, optionally filtered by tag.
    pub fn list(tag_filter: Option<&str>) -> Result<Vec<JobEntry>> {
        let file = Self::load_file()?;
        let entries = if let Some(tag) = tag_filter {
            file.jobs
                .into_iter()
                .filter(|e| e.tags.iter().any(|t| t == tag))
                .collect()
        } else {
            file.jobs
        };
        Ok(entries)
    }

    /// Find a job by name.
    pub fn find(name: &str) -> Result<Option<JobEntry>> {
        let file = Self::load_file()?;
        Ok(file.jobs.into_iter().find(|e| e.name == name))
    }

    /// Add a new job entry.  Fails if the name is already taken.
    pub fn add(entry: JobEntry) -> Result<()> {
        let mut file = Self::load_file()?;
        if file.jobs.iter().any(|e| e.name == entry.name) {
            return Err(OxoError::ConfigError(format!(
                "Job '{}' already exists. Use 'job edit' to update it.",
                entry.name
            )));
        }
        file.jobs.push(entry);
        Self::save_file(&file)
    }

    /// Remove a job by name.
    pub fn remove(name: &str) -> Result<()> {
        let mut file = Self::load_file()?;
        let before = file.jobs.len();
        file.jobs.retain(|e| e.name != name);
        if file.jobs.len() == before {
            return Err(OxoError::ConfigError(format!(
                "No job found with name '{name}'"
            )));
        }
        Self::save_file(&file)
    }

    /// Edit an existing job entry in place.
    ///
    /// Only fields that are `Some` are updated; `None` leaves the field
    /// unchanged.  Pass `clear_description = true` to explicitly erase the
    /// description, or `clear_schedule = true` to remove the cron schedule.
    #[allow(clippy::too_many_arguments)]
    pub fn edit(
        name: &str,
        new_command: Option<&str>,
        new_description: Option<&str>,
        clear_description: bool,
        new_tags: Option<Vec<String>>,
        new_schedule: Option<&str>,
        clear_schedule: bool,
    ) -> Result<()> {
        let mut file = Self::load_file()?;
        let entry = file
            .jobs
            .iter_mut()
            .find(|e| e.name == name)
            .ok_or_else(|| OxoError::ConfigError(format!("No job found with name '{name}'")))?;
        if let Some(cmd) = new_command {
            entry.command = cmd.to_string();
        }
        if clear_description {
            entry.description = None;
        } else if let Some(desc) = new_description {
            entry.description = Some(desc.to_string());
        }
        if let Some(tags) = new_tags {
            entry.tags = tags;
        }
        if clear_schedule {
            entry.schedule = None;
        } else if let Some(sched) = new_schedule {
            entry.schedule = Some(sched.to_string());
        }
        entry.updated_at = Utc::now();
        Self::save_file(&file)
    }

    /// Rename a job entry.
    pub fn rename(old_name: &str, new_name: &str) -> Result<()> {
        let mut file = Self::load_file()?;
        if file.jobs.iter().any(|e| e.name == new_name) {
            return Err(OxoError::ConfigError(format!(
                "Job '{new_name}' already exists."
            )));
        }
        let entry = file
            .jobs
            .iter_mut()
            .find(|e| e.name == old_name)
            .ok_or_else(|| OxoError::ConfigError(format!("No job found with name '{old_name}'")))?;
        entry.name = new_name.to_string();
        entry.updated_at = Utc::now();
        Self::save_file(&file)
    }

    /// Record a completed execution: update stats in the entry and append a
    /// run record to `job_runs.jsonl`.
    pub fn record_run(
        name: &str,
        command: &str,
        server: Option<String>,
        exit_code: i32,
        started_at: DateTime<Utc>,
        duration_secs: f64,
    ) -> Result<()> {
        // Update entry stats
        let mut file = Self::load_file()?;
        if let Some(entry) = file.jobs.iter_mut().find(|e| e.name == name) {
            entry.run_count += 1;
            entry.last_run = Some(started_at);
            entry.last_exit_code = Some(exit_code);
            entry.updated_at = Utc::now();
        }
        Self::save_file(&file)?;
        // Append to run log
        JobRunStore::append(&JobRun {
            job_name: name.to_string(),
            command: command.to_string(),
            server,
            exit_code,
            started_at,
            duration_secs,
        })
    }

    /// Set or clear a cron schedule on an existing job.
    pub fn set_schedule(name: &str, schedule: Option<&str>) -> Result<()> {
        let mut file = Self::load_file()?;
        let entry = file
            .jobs
            .iter_mut()
            .find(|e| e.name == name)
            .ok_or_else(|| OxoError::ConfigError(format!("No job found with name '{name}'")))?;
        entry.schedule = schedule.map(str::to_string);
        entry.updated_at = Utc::now();
        Self::save_file(&file)
    }

    /// Return all jobs that have a schedule set.
    ///
    /// Intended for use by a scheduled runner (e.g. a wrapper cron script that
    /// calls `oxo-call job run <name>` for each scheduled job).
    #[allow(dead_code)]
    pub fn scheduled_jobs() -> Result<Vec<JobEntry>> {
        let file = Self::load_file()?;
        Ok(file
            .jobs
            .into_iter()
            .filter(|e| e.schedule.is_some())
            .collect())
    }
}

// ─── Variable interpolation ──────────────────────────────────────────────────

/// Interpolate placeholders in a command string for one input item.
///
/// Built-in placeholders (all case-sensitive):
///
/// | Placeholder | Expands to |
/// |-------------|-----------|
/// | `{item}` / `{line}` / `{}` | the current input item (e.g. a file path) |
/// | `{nr}` | 1-based line / item number |
/// | `{basename}` | `Path::file_name()` of `{item}`, or `{item}` when not a path |
/// | `{dir}` | parent directory of `{item}`, or `.` when there is no parent |
/// | `{stem}` | filename stem without the last extension (`sample` for `sample.bam`) |
/// | `{ext}` | file extension without leading dot (`bam` for `sample.bam`) |
///
/// **Empty `item`**: When `item` is an empty string (vars-only single run),
/// the path-derived placeholders resolve as: `{item}` / `{line}` / `{}` → `""`,
/// `{basename}` → `""`, `{stem}` → `""`, `{ext}` → `""`, `{dir}` → `"."`.
///
/// User-defined vars (`--var KEY=VALUE`) are substituted first, so they take
/// precedence over the built-in names if a user deliberately reuses them.
pub fn interpolate_command(
    cmd: &str,
    item: &str,
    nr: usize,
    vars: &HashMap<String, String>,
) -> String {
    use std::path::Path;

    let path = Path::new(item);
    let basename = path.file_name().and_then(|s| s.to_str()).unwrap_or(item);
    let dir = path
        .parent()
        .and_then(|p| p.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(".");
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(item);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let mut result = cmd.to_string();

    // User-defined vars first (they can shadow the built-in names).
    for (k, v) in vars {
        result = result.replace(&format!("{{{k}}}"), v);
    }

    // Built-in placeholders.
    // `{}` is the rush-compatible shorthand for `{item}` and must be expanded
    // before `{item}` / `{line}` so that the order is deterministic regardless
    // of which alias the user chose.
    result = result.replace("{}", item);
    result = result.replace("{item}", item);
    result = result.replace("{line}", item);
    result = result.replace("{nr}", &nr.to_string());
    result = result.replace("{basename}", basename);
    result = result.replace("{dir}", dir);
    result = result.replace("{stem}", stem);
    result = result.replace("{ext}", ext);

    result
}

/// Parse a `KEY=VALUE` string into `(key, value)`.
///
/// Returns an error when no `=` separator is present or when the key is empty.
pub fn parse_var(s: &str) -> Result<(String, String)> {
    let pos = s
        .find('=')
        .ok_or_else(|| OxoError::ConfigError(format!("invalid --var '{s}': must be KEY=VALUE")))?;
    let key = &s[..pos];
    if key.is_empty() {
        return Err(OxoError::ConfigError(format!(
            "invalid --var '{s}': key must not be empty (expected KEY=VALUE)"
        )));
    }
    Ok((key.to_string(), s[pos + 1..].to_string()))
}

/// Read non-blank, non-comment lines from a file into a `Vec<String>`.
///
/// Lines that are empty (after trimming whitespace) or that start with `#`
/// are silently skipped — this lets users annotate their input lists.
///
/// Returns an error if the file cannot be opened **or** if an IO error occurs
/// while reading any line (i.e. errors are propagated, not silently truncated).
pub fn read_input_list(path: &str) -> Result<Vec<String>> {
    use std::io::{BufRead, BufReader};
    let f = std::fs::File::open(path)
        .map_err(|e| OxoError::ConfigError(format!("cannot open --input-list '{path}': {e}")))?;
    let mut lines: Vec<String> = Vec::new();
    for line_result in BufReader::new(f).lines() {
        let line = line_result.map_err(|e| {
            OxoError::ConfigError(format!("IO error reading --input-list '{path}': {e}"))
        })?;
        if !line.trim().is_empty() && !line.trim_start().starts_with('#') {
            lines.push(line);
        }
    }
    Ok(lines)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(name: &str, command: &str) -> JobEntry {
        let now = Utc::now();
        JobEntry {
            name: name.to_string(),
            command: command.to_string(),
            description: None,
            tags: vec![],
            schedule: None,
            run_count: 0,
            last_run: None,
            last_exit_code: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_job_entry_serialization_round_trip() {
        let entry = make_entry("gpu-check", "nvidia-smi");
        let serialized = toml::to_string_pretty(&JobFile {
            jobs: vec![entry.clone()],
        })
        .unwrap();
        let deserialized: JobFile = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.jobs.len(), 1);
        assert_eq!(deserialized.jobs[0].name, "gpu-check");
        assert_eq!(deserialized.jobs[0].command, "nvidia-smi");
    }

    #[test]
    fn test_job_entry_with_tags_round_trip() {
        let mut entry = make_entry("job1", "squeue -u $USER");
        entry.tags = vec!["slurm".to_string(), "hpc".to_string()];
        entry.description = Some("Show my SLURM jobs".to_string());

        let file = JobFile { jobs: vec![entry] };
        let s = toml::to_string_pretty(&file).unwrap();
        let back: JobFile = toml::from_str(&s).unwrap();
        assert_eq!(back.jobs[0].tags, vec!["slurm", "hpc"]);
        assert_eq!(
            back.jobs[0].description.as_deref(),
            Some("Show my SLURM jobs")
        );
    }

    #[test]
    fn test_job_file_default_is_empty() {
        let file = JobFile::default();
        assert!(file.jobs.is_empty());
    }

    #[test]
    fn test_job_entry_empty_tags_skip_serialized() {
        let entry = make_entry("empty-label", "echo hi");
        let s = toml::to_string_pretty(&JobFile { jobs: vec![entry] }).unwrap();
        assert!(
            !s.contains("tags = "),
            "empty tags should not appear in TOML, got: {s}"
        );
    }

    #[test]
    fn test_job_entry_no_description_skip_serialized() {
        let entry = make_entry("no-desc", "echo hi");
        let s = toml::to_string_pretty(&JobFile { jobs: vec![entry] }).unwrap();
        assert!(
            !s.contains("description ="),
            "absent description should not appear in TOML, got: {s}"
        );
    }

    #[test]
    fn test_job_entry_with_schedule_round_trip() {
        let mut entry = make_entry("hourly-check", "df -h");
        entry.schedule = Some("0 * * * *".to_string());
        let s = toml::to_string_pretty(&JobFile { jobs: vec![entry] }).unwrap();
        let back: JobFile = toml::from_str(&s).unwrap();
        assert_eq!(back.jobs[0].schedule.as_deref(), Some("0 * * * *"));
    }

    #[test]
    fn test_builtin_jobs_non_empty() {
        assert!(!BUILTIN_JOBS.is_empty());
    }

    #[test]
    fn test_builtin_jobs_tag_filter() {
        let slurm = builtin_jobs(Some("slurm"));
        assert!(!slurm.is_empty());
        for j in &slurm {
            assert!(j.tags.contains(&"slurm"));
        }
    }

    #[test]
    fn test_job_run_store_round_trip() {
        let run = JobRun {
            job_name: "test-job".to_string(),
            command: "echo hi".to_string(),
            server: None,
            exit_code: 0,
            started_at: Utc::now(),
            duration_secs: 0.1,
        };
        let line = serde_json::to_string(&run).unwrap();
        let back: JobRun = serde_json::from_str(&line).unwrap();
        assert_eq!(back.job_name, "test-job");
        assert_eq!(back.exit_code, 0);
    }

    // ─── JobEntry additional ──────────────────────────────────────────────────

    #[test]
    fn test_job_entry_default_fields() {
        let entry = JobEntry {
            name: "test".to_string(),
            command: "echo hi".to_string(),
            description: None,
            tags: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            schedule: None,
            run_count: 0,
            last_run: None,
            last_exit_code: None,
        };
        let toml = toml::to_string(&entry).unwrap();
        assert!(toml.contains("name = \"test\""));
        assert!(toml.contains("command = \"echo hi\""));
        // Optional None fields should be absent
        assert!(!toml.contains("description ="));
    }

    #[test]
    fn test_job_entry_with_all_fields() {
        let entry = JobEntry {
            name: "align".to_string(),
            command: "bwa mem ref.fa reads.fq > out.sam".to_string(),
            description: Some("Align reads".to_string()),
            tags: vec!["alignment".to_string(), "bwa".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            schedule: Some("daily".to_string()),
            run_count: 5,
            last_run: Some(Utc::now()),
            last_exit_code: Some(0),
        };
        let toml = toml::to_string(&entry).unwrap();
        let back: JobEntry = toml::from_str(&toml).unwrap();
        assert_eq!(back.name, "align");
        assert_eq!(back.description.as_deref(), Some("Align reads"));
        assert_eq!(back.tags, vec!["alignment", "bwa"]);
        assert_eq!(back.schedule.as_deref(), Some("daily"));
        assert_eq!(back.run_count, 5);
        assert_eq!(back.last_exit_code, Some(0));
    }

    #[test]
    fn test_job_file_round_trip() {
        let mut jf = JobFile::default();
        jf.jobs.push(JobEntry {
            name: "test1".to_string(),
            command: "echo 1".to_string(),
            description: None,
            tags: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            schedule: None,
            run_count: 0,
            last_run: None,
            last_exit_code: None,
        });
        jf.jobs.push(JobEntry {
            name: "test2".to_string(),
            command: "echo 2".to_string(),
            description: Some("second job".to_string()),
            tags: vec!["test".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            schedule: None,
            run_count: 0,
            last_run: None,
            last_exit_code: None,
        });
        let toml = toml::to_string(&jf).unwrap();
        let back: JobFile = toml::from_str(&toml).unwrap();
        assert_eq!(back.jobs.len(), 2);
        assert_eq!(back.jobs[0].name, "test1");
        assert_eq!(back.jobs[1].name, "test2");
    }

    #[test]
    fn test_job_run_all_fields() {
        let run = JobRun {
            job_name: "align".to_string(),
            command: "bwa mem ref.fa reads.fq".to_string(),
            server: Some("hpc-cluster".to_string()),
            exit_code: 1,
            started_at: Utc::now(),
            duration_secs: 120.5,
        };
        let json = serde_json::to_string(&run).unwrap();
        let back: JobRun = serde_json::from_str(&json).unwrap();
        assert_eq!(back.server.as_deref(), Some("hpc-cluster"));
        assert_eq!(back.exit_code, 1);
        assert!((back.duration_secs - 120.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builtin_jobs_all_have_names() {
        for job in BUILTIN_JOBS {
            assert!(
                !job.name.is_empty(),
                "built-in job name should not be empty"
            );
            assert!(
                !job.command.is_empty(),
                "built-in job '{}' command should not be empty",
                job.name
            );
        }
    }

    #[test]
    fn test_builtin_jobs_unique_names() {
        let mut names: Vec<&str> = BUILTIN_JOBS.iter().map(|j| j.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(
            names.len(),
            BUILTIN_JOBS.len(),
            "built-in job names should be unique"
        );
    }

    // ─── New built-in job coverage tests ─────────────────────────────────────

    /// Verify every expected new built-in job name is present.
    #[test]
    fn test_new_builtin_jobs_exist() {
        let names: std::collections::HashSet<&str> = BUILTIN_JOBS.iter().map(|j| j.name).collect();
        for expected in &[
            "uptime",
            "load",
            "who",
            "inode",
            "tmp",
            "cpu-info",
            "find-recent",
            "find-bam",
            "find-fastq",
            "bam-index",
            "qstat-sge",
            "qhost",
            "qdel-me",
            "docker-images",
            "docker-all",
            "k8s-svc",
            "k8s-top",
            "git-status",
            "git-branch",
            "conda-envs",
            "screen-ls",
            "tmux-ls",
        ] {
            assert!(
                names.contains(*expected),
                "Expected built-in job '{expected}' to be present"
            );
        }
    }

    #[test]
    fn test_builtin_jobs_total_count_at_least_50() {
        assert!(
            BUILTIN_JOBS.len() >= 49,
            "Expected at least 49 built-in jobs, got {}",
            BUILTIN_JOBS.len()
        );
    }

    #[test]
    fn test_builtin_jobs_sge_tag_filter() {
        let sge = builtin_jobs(Some("sge"));
        assert!(!sge.is_empty(), "should have SGE built-in jobs");
        for j in &sge {
            assert!(
                j.tags.contains(&"sge"),
                "job '{}' should have 'sge' tag",
                j.name
            );
        }
        // spot-check at least qstat-sge and qdel-me
        let sge_names: Vec<&str> = sge.iter().map(|j| j.name).collect();
        assert!(
            sge_names.contains(&"qstat-sge"),
            "qstat-sge should be in sge filter"
        );
        assert!(
            sge_names.contains(&"qdel-me"),
            "qdel-me should be in sge filter"
        );
    }

    #[test]
    fn test_builtin_jobs_bioinformatics_tag_filter() {
        let bio = builtin_jobs(Some("bioinformatics"));
        assert!(!bio.is_empty(), "should have bioinformatics built-in jobs");
        let bio_names: Vec<&str> = bio.iter().map(|j| j.name).collect();
        assert!(
            bio_names.contains(&"find-bam"),
            "find-bam should be in bioinformatics filter"
        );
        assert!(
            bio_names.contains(&"find-fastq"),
            "find-fastq should be in bioinformatics filter"
        );
        assert!(
            bio_names.contains(&"bam-index"),
            "bam-index should be in bioinformatics filter"
        );
    }

    #[test]
    fn test_builtin_jobs_dev_tag_filter() {
        let dev = builtin_jobs(Some("dev"));
        assert!(!dev.is_empty(), "should have dev built-in jobs");
        let dev_names: Vec<&str> = dev.iter().map(|j| j.name).collect();
        assert!(
            dev_names.contains(&"git-status"),
            "git-status should be in dev filter"
        );
        assert!(
            dev_names.contains(&"git-branch"),
            "git-branch should be in dev filter"
        );
        assert!(
            dev_names.contains(&"conda-envs"),
            "conda-envs should be in dev filter"
        );
        assert!(
            dev_names.contains(&"screen-ls"),
            "screen-ls should be in dev filter"
        );
        assert!(
            dev_names.contains(&"tmux-ls"),
            "tmux-ls should be in dev filter"
        );
    }

    #[test]
    fn test_builtin_jobs_all_have_non_empty_descriptions() {
        for job in BUILTIN_JOBS {
            assert!(
                !job.description.is_empty(),
                "built-in job '{}' should have a description",
                job.name
            );
        }
    }

    #[test]
    fn test_builtin_jobs_all_have_at_least_one_tag() {
        for job in BUILTIN_JOBS {
            assert!(
                !job.tags.is_empty(),
                "built-in job '{}' should have at least one tag",
                job.name
            );
        }
    }

    #[test]
    fn test_builtin_job_uptime_command() {
        let uptime = BUILTIN_JOBS.iter().find(|j| j.name == "uptime").unwrap();
        assert_eq!(uptime.command, "uptime");
        assert!(uptime.tags.contains(&"system"));
    }

    #[test]
    fn test_builtin_job_find_bam_command() {
        let find_bam = BUILTIN_JOBS.iter().find(|j| j.name == "find-bam").unwrap();
        assert!(
            find_bam.command.contains("*.bam"),
            "find-bam should look for .bam files"
        );
        assert!(find_bam.tags.contains(&"bioinformatics"));
    }

    #[test]
    fn test_builtin_job_qstat_sge_command() {
        let qstat = BUILTIN_JOBS.iter().find(|j| j.name == "qstat-sge").unwrap();
        assert!(
            qstat.command.contains("qstat"),
            "qstat-sge should use qstat"
        );
        assert!(qstat.tags.contains(&"hpc"));
    }

    #[test]
    fn test_builtin_job_docker_all_command() {
        let docker_all = BUILTIN_JOBS
            .iter()
            .find(|j| j.name == "docker-all")
            .unwrap();
        assert!(
            docker_all.command.contains("docker ps -a"),
            "docker-all should use 'docker ps -a'"
        );
        assert!(docker_all.tags.contains(&"docker"));
    }

    #[test]
    fn test_builtin_job_git_status_command() {
        let git_status = BUILTIN_JOBS
            .iter()
            .find(|j| j.name == "git-status")
            .unwrap();
        assert!(
            git_status.command.contains("git status"),
            "git-status should use 'git status'"
        );
        assert!(git_status.tags.contains(&"git"));
    }

    #[test]
    fn test_builtin_job_k8s_svc_command() {
        let k8s_svc = BUILTIN_JOBS.iter().find(|j| j.name == "k8s-svc").unwrap();
        assert!(
            k8s_svc.command.contains("kubectl get svc"),
            "k8s-svc should use 'kubectl get svc'"
        );
        assert!(k8s_svc.tags.contains(&"k8s"));
    }

    #[test]
    fn test_job_run_store_with_server_field_round_trip() {
        // Verify that JobRun with server field round-trips through JSON.
        let run = JobRun {
            job_name: "my-job".to_string(),
            command: "echo test".to_string(),
            server: Some("remote-server".to_string()),
            exit_code: 0,
            started_at: Utc::now(),
            duration_secs: 1.5,
        };
        let json = serde_json::to_string(&run).unwrap();
        let back: JobRun = serde_json::from_str(&json).unwrap();
        assert_eq!(back.server.as_deref(), Some("remote-server"));
    }

    // ── interpolation ────────────────────────────────────────────────────────

    #[test]
    fn test_interpolate_item_and_line() {
        let vars = HashMap::new();
        let result = interpolate_command("echo {item} {line}", "hello.bam", 1, &vars);
        assert_eq!(result, "echo hello.bam hello.bam");
    }

    #[test]
    fn test_interpolate_nr() {
        let vars = HashMap::new();
        let result = interpolate_command("cmd --index {nr}", "file.txt", 3, &vars);
        assert_eq!(result, "cmd --index 3");
    }

    #[test]
    fn test_interpolate_path_parts() {
        let vars = HashMap::new();
        let result = interpolate_command("{dir}/{stem}.sorted.{ext}", "data/sample.bam", 1, &vars);
        assert_eq!(result, "data/sample.sorted.bam");
    }

    #[test]
    fn test_interpolate_basename() {
        let vars = HashMap::new();
        let result = interpolate_command("{basename}", "data/sample.bam", 1, &vars);
        assert_eq!(result, "sample.bam");
    }

    #[test]
    fn test_interpolate_user_vars() {
        let mut vars = HashMap::new();
        vars.insert("THREADS".to_string(), "8".to_string());
        vars.insert("REF".to_string(), "hg38.fa".to_string());
        let result = interpolate_command("bwa mem -t {THREADS} {REF} {item}", "reads.fq", 1, &vars);
        assert_eq!(result, "bwa mem -t 8 hg38.fa reads.fq");
    }

    #[test]
    fn test_interpolate_no_item_placeholders() {
        let mut vars = HashMap::new();
        vars.insert("KEY".to_string(), "value".to_string());
        let result = interpolate_command("echo {KEY}", "", 0, &vars);
        assert_eq!(result, "echo value");
    }

    #[test]
    fn test_parse_var_valid() {
        let (k, v) = parse_var("THREADS=8").unwrap();
        assert_eq!(k, "THREADS");
        assert_eq!(v, "8");
    }

    #[test]
    fn test_parse_var_value_contains_equals() {
        let (k, v) = parse_var("OPTS=-t 8 --flag=x").unwrap();
        assert_eq!(k, "OPTS");
        assert_eq!(v, "-t 8 --flag=x");
    }

    #[test]
    fn test_parse_var_missing_equals() {
        assert!(parse_var("NOEQUALS").is_err());
    }

    #[test]
    fn test_read_input_list_skips_blank_and_comments() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "sample1.bam").unwrap();
        writeln!(f, "# this is a comment").unwrap();
        #[allow(clippy::writeln_empty_string)]
        writeln!(f, "").unwrap();
        writeln!(f, "sample2.bam").unwrap();
        let items = read_input_list(f.path().to_str().unwrap()).unwrap();
        assert_eq!(items, vec!["sample1.bam", "sample2.bam"]);
    }

    #[test]
    fn test_interpolate_empty_item_leaves_built_ins_blank() {
        // When item is empty (vars-only run), built-in placeholders resolve to "".
        let vars = HashMap::new();
        let result = interpolate_command("echo {item} {line} {basename}", "", 0, &vars);
        // Empty item → basename of "" is "", dir is ".", stem is "", ext is "".
        assert_eq!(result, "echo   ");
    }

    #[test]
    fn test_interpolate_no_placeholders_unchanged() {
        let vars = HashMap::new();
        let cmd = "samtools flagstat input.bam";
        assert_eq!(interpolate_command(cmd, "ignored", 1, &vars), cmd);
    }

    #[test]
    fn test_interpolate_multiple_occurrences() {
        let vars = HashMap::new();
        let result = interpolate_command("cp {item} {item}.bak", "file.txt", 1, &vars);
        assert_eq!(result, "cp file.txt file.txt.bak");
    }

    #[test]
    fn test_interpolate_item_without_extension() {
        let vars = HashMap::new();
        let result = interpolate_command("{stem}.{ext}", "README", 1, &vars);
        // No extension → ext is empty → "README."
        assert_eq!(result, "README.");
    }

    #[test]
    fn test_interpolate_dir_for_root_item() {
        let vars = HashMap::new();
        // A bare filename has no parent dir → dir becomes "."
        let result = interpolate_command("{dir}", "file.bam", 1, &vars);
        assert_eq!(result, ".");
    }

    #[test]
    fn test_interpolate_dir_for_nested_item() {
        let vars = HashMap::new();
        let result = interpolate_command("{dir}", "data/reads/sample.fq.gz", 1, &vars);
        assert_eq!(result, "data/reads");
    }

    #[test]
    fn test_interpolate_stem_for_double_extension() {
        // Only the *last* extension is stripped (std behaviour).
        let vars = HashMap::new();
        let result = interpolate_command("{stem}", "sample.fq.gz", 1, &vars);
        assert_eq!(result, "sample.fq");
    }

    #[test]
    fn test_interpolate_vars_before_builtins_shadowing() {
        // A user var named "item" should override the built-in {item}.
        let mut vars = HashMap::new();
        vars.insert("item".to_string(), "CUSTOM".to_string());
        let result = interpolate_command("echo {item}", "real", 1, &vars);
        // User var applied first → "{item}" → "CUSTOM". Built-in then tries to replace
        // "{item}" again but it no longer appears in the string.
        assert_eq!(result, "echo CUSTOM");
    }

    #[test]
    fn test_parse_var_empty_value() {
        let (k, v) = parse_var("KEY=").unwrap();
        assert_eq!(k, "KEY");
        assert_eq!(v, "");
    }

    #[test]
    fn test_parse_var_empty_key() {
        // An empty key is now rejected with a clear error.
        let result = parse_var("=value");
        assert!(result.is_err(), "Expected error for empty key");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("key must not be empty"),
            "Expected empty-key error, got: {msg}"
        );
    }

    #[test]
    fn test_read_input_list_all_comments_returns_empty() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "# comment").unwrap();
        writeln!(f, "   # indented comment").unwrap();
        #[allow(clippy::writeln_empty_string)]
        writeln!(f, "").unwrap();
        let items = read_input_list(f.path().to_str().unwrap()).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_read_input_list_preserves_whitespace_within_items() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "file with spaces.bam").unwrap();
        let items = read_input_list(f.path().to_str().unwrap()).unwrap();
        assert_eq!(items, vec!["file with spaces.bam"]);
    }

    #[test]
    fn test_read_input_list_nonexistent_file_returns_error() {
        let result = read_input_list("/nonexistent/path/that/does/not/exist.txt");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("cannot open --input-list"),
            "Expected helpful error message, got: {msg}"
        );
    }

    // ── New features ─────────────────────────────────────────────────────────

    #[test]
    fn test_interpolate_curly_braces_alias_for_item() {
        // `{}` is the rush-compatible alias for `{item}`.
        let vars = HashMap::new();
        let result = interpolate_command("samtools view -bS {} > {}.bam", "input.sam", 1, &vars);
        assert_eq!(result, "samtools view -bS input.sam > input.sam.bam");
    }

    #[test]
    fn test_interpolate_curly_braces_does_not_conflict_with_nr() {
        // `{}` should not match `{nr}` — they are expanded independently.
        let vars = HashMap::new();
        let result = interpolate_command("{nr}: {}", "item", 3, &vars);
        assert_eq!(result, "3: item");
    }

    #[test]
    fn test_interpolate_curly_braces_with_vars() {
        // User vars are expanded before `{}`, so they can shadow it.
        let mut vars = HashMap::new();
        vars.insert("T".to_string(), "8".to_string());
        let result = interpolate_command("process -t {T} {}", "sample.bam", 1, &vars);
        assert_eq!(result, "process -t 8 sample.bam");
    }

    #[test]
    fn test_parse_var_still_allows_value_with_equals() {
        // KEY=a=b should split on the FIRST `=` only.
        let (k, v) = parse_var("KEY=a=b").unwrap();
        assert_eq!(k, "KEY");
        assert_eq!(v, "a=b");
    }

    #[test]
    fn test_read_input_list_io_error_propagates() {
        // Verify that read_input_list returns an error for unreadable paths,
        // not a silently-truncated empty vec.
        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::PermissionsExt;
            let mut f = tempfile::NamedTempFile::new().unwrap();
            writeln!(f, "item1").unwrap();
            std::fs::set_permissions(f.path(), std::fs::Permissions::from_mode(0o000)).unwrap();
            let result = read_input_list(f.path().to_str().unwrap());
            // Restore so the temp file can be cleaned up.
            let _ = std::fs::set_permissions(f.path(), std::fs::Permissions::from_mode(0o644));
            assert!(result.is_err(), "Expected error for unreadable file");
            let msg = result.unwrap_err().to_string();
            assert!(
                msg.contains("--input-list"),
                "Expected --input-list in error message, got: {msg}"
            );
        }
        // Non-Unix: just verify non-existent file error (already covered by
        // test_read_input_list_nonexistent_file_returns_error).
        #[cfg(not(unix))]
        {}
    }

    #[test]
    fn test_job_manager_add_and_find() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let entry = make_entry("test-add", "echo hello");
        JobManager::add(entry).unwrap();
        let found = JobManager::find("test-add").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().command, "echo hello");
    }

    #[test]
    fn test_job_manager_add_duplicate_fails() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        JobManager::add(make_entry("dup", "echo 1")).unwrap();
        let result = JobManager::add(make_entry("dup", "echo 2"));
        assert!(result.is_err());
    }

    #[test]
    fn test_job_manager_find_nonexistent() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let found = JobManager::find("no-such-job").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_job_manager_remove() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        JobManager::add(make_entry("to-remove", "echo rm")).unwrap();
        JobManager::remove("to-remove").unwrap();
        assert!(JobManager::find("to-remove").unwrap().is_none());
    }

    #[test]
    fn test_job_manager_remove_nonexistent_fails() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let result = JobManager::remove("ghost");
        assert!(result.is_err());
    }

    #[test]
    fn test_job_manager_list() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        JobManager::add(make_entry("job-a", "echo a")).unwrap();
        JobManager::add(make_entry("job-b", "echo b")).unwrap();
        let list = JobManager::list(None).unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_job_manager_list_with_tag_filter() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let mut entry = make_entry("tagged", "echo tagged");
        entry.tags = vec!["bio".to_string()];
        JobManager::add(entry).unwrap();
        JobManager::add(make_entry("untagged", "echo plain")).unwrap();
        let filtered = JobManager::list(Some("bio")).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "tagged");
    }

    #[test]
    fn test_job_manager_edit_command() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        JobManager::add(make_entry("edit-me", "echo old")).unwrap();
        JobManager::edit("edit-me", Some("echo new"), None, false, None, None, false).unwrap();
        let found = JobManager::find("edit-me").unwrap().unwrap();
        assert_eq!(found.command, "echo new");
    }

    #[test]
    fn test_job_manager_edit_description() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        JobManager::add(make_entry("desc-job", "echo hi")).unwrap();
        JobManager::edit("desc-job", None, Some("A description"), false, None, None, false).unwrap();
        let found = JobManager::find("desc-job").unwrap().unwrap();
        assert_eq!(found.description.as_deref(), Some("A description"));
    }

    #[test]
    fn test_job_manager_edit_clear_description() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let mut entry = make_entry("clear-desc", "echo hi");
        entry.description = Some("old desc".to_string());
        JobManager::add(entry).unwrap();
        JobManager::edit("clear-desc", None, None, true, None, None, false).unwrap();
        let found = JobManager::find("clear-desc").unwrap().unwrap();
        assert!(found.description.is_none());
    }

    #[test]
    fn test_job_manager_edit_tags() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        JobManager::add(make_entry("tag-job", "echo hi")).unwrap();
        JobManager::edit("tag-job", None, None, false, Some(vec!["new-tag".to_string()]), None, false).unwrap();
        let found = JobManager::find("tag-job").unwrap().unwrap();
        assert_eq!(found.tags, vec!["new-tag"]);
    }

    #[test]
    fn test_job_manager_edit_schedule() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        JobManager::add(make_entry("sched-job", "echo hi")).unwrap();
        JobManager::edit("sched-job", None, None, false, None, Some("0 * * * *"), false).unwrap();
        let found = JobManager::find("sched-job").unwrap().unwrap();
        assert_eq!(found.schedule.as_deref(), Some("0 * * * *"));
    }

    #[test]
    fn test_job_manager_edit_clear_schedule() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let mut entry = make_entry("clear-sched", "echo hi");
        entry.schedule = Some("0 * * * *".to_string());
        JobManager::add(entry).unwrap();
        JobManager::edit("clear-sched", None, None, false, None, None, true).unwrap();
        let found = JobManager::find("clear-sched").unwrap().unwrap();
        assert!(found.schedule.is_none());
    }

    #[test]
    fn test_job_manager_edit_nonexistent_fails() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let result = JobManager::edit("ghost", Some("echo"), None, false, None, None, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_job_manager_rename() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        JobManager::add(make_entry("old-name", "echo hi")).unwrap();
        JobManager::rename("old-name", "new-name").unwrap();
        assert!(JobManager::find("old-name").unwrap().is_none());
        assert!(JobManager::find("new-name").unwrap().is_some());
    }

    #[test]
    fn test_job_manager_rename_to_existing_fails() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        JobManager::add(make_entry("a", "echo a")).unwrap();
        JobManager::add(make_entry("b", "echo b")).unwrap();
        let result = JobManager::rename("a", "b");
        assert!(result.is_err());
    }

    #[test]
    fn test_job_manager_rename_nonexistent_fails() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let result = JobManager::rename("ghost", "new-name");
        assert!(result.is_err());
    }

    #[test]
    fn test_job_manager_record_run() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        JobManager::add(make_entry("run-job", "echo hi")).unwrap();
        JobManager::record_run("run-job", "echo hi", None, 0, Utc::now(), 0.5).unwrap();
        let found = JobManager::find("run-job").unwrap().unwrap();
        assert_eq!(found.run_count, 1);
        assert_eq!(found.last_exit_code, Some(0));
    }

    #[test]
    fn test_job_manager_set_schedule() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        JobManager::add(make_entry("sched-me", "echo hi")).unwrap();
        JobManager::set_schedule("sched-me", Some("0 */2 * * *")).unwrap();
        let found = JobManager::find("sched-me").unwrap().unwrap();
        assert_eq!(found.schedule.as_deref(), Some("0 */2 * * *"));
    }

    #[test]
    fn test_job_manager_set_schedule_clear() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let mut entry = make_entry("clear-sched2", "echo hi");
        entry.schedule = Some("0 * * * *".to_string());
        JobManager::add(entry).unwrap();
        JobManager::set_schedule("clear-sched2", None).unwrap();
        let found = JobManager::find("clear-sched2").unwrap().unwrap();
        assert!(found.schedule.is_none());
    }

    #[test]
    fn test_job_manager_scheduled_jobs() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        JobManager::add(make_entry("no-sched", "echo a")).unwrap();
        let mut entry = make_entry("has-sched", "echo b");
        entry.schedule = Some("0 * * * *".to_string());
        JobManager::add(entry).unwrap();
        let scheduled = JobManager::scheduled_jobs().unwrap();
        assert_eq!(scheduled.len(), 1);
        assert_eq!(scheduled[0].name, "has-sched");
    }

    #[test]
    fn test_job_run_store_append_and_load() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let run = JobRun {
            job_name: "store-test".to_string(),
            command: "echo test".to_string(),
            server: None,
            exit_code: 0,
            started_at: Utc::now(),
            duration_secs: 1.0,
        };
        JobRunStore::append(&run).unwrap();
        let loaded = JobRunStore::load(None).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].job_name, "store-test");
    }

    #[test]
    fn test_job_run_store_load_with_filter() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let run1 = JobRun {
            job_name: "job-a".to_string(),
            command: "echo a".to_string(),
            server: None,
            exit_code: 0,
            started_at: Utc::now(),
            duration_secs: 1.0,
        };
        let run2 = JobRun {
            job_name: "job-b".to_string(),
            command: "echo b".to_string(),
            server: None,
            exit_code: 1,
            started_at: Utc::now(),
            duration_secs: 2.0,
        };
        JobRunStore::append(&run1).unwrap();
        JobRunStore::append(&run2).unwrap();
        let filtered = JobRunStore::load(Some("job-a")).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].job_name, "job-a");
    }

    #[test]
    fn test_job_run_store_load_empty() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let loaded = JobRunStore::load(None).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_builtin_jobs_no_filter_returns_all() {
        let all = builtin_jobs(None);
        assert_eq!(all.len(), BUILTIN_JOBS.len());
    }
}
