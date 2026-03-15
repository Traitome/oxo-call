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
}
