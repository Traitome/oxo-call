use crate::config::Config;
use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Provenance metadata recorded alongside each command to enable reproducibility.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandProvenance {
    /// Version string reported by the tool (e.g. from `tool --version`), if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_version: Option<String>,
    /// SHA-256 hash of the documentation text used to build the LLM prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_hash: Option<String>,
    /// Name of the skill file used (e.g. "samtools"), if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,
    /// LLM model identifier that generated the command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub tool: String,
    pub task: String,
    pub command: String,
    pub exit_code: i32,
    pub executed_at: DateTime<Utc>,
    pub dry_run: bool,
    /// Remote server name when the command was executed via SSH (None for local runs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    /// Command provenance for reproducibility (tool version, docs hash, skill, model).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<CommandProvenance>,
}

pub struct HistoryStore;

impl HistoryStore {
    fn history_path() -> Result<PathBuf> {
        Ok(Config::data_dir()?.join("history.jsonl"))
    }

    pub fn append(entry: HistoryEntry) -> Result<()> {
        let path = Self::history_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let line = serde_json::to_string(&entry)?;
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(file, "{line}")?;
        Ok(())
    }

    pub fn load_all() -> Result<Vec<HistoryEntry>> {
        let path = Self::history_path()?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let mut entries = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<HistoryEntry>(line) {
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    pub fn clear() -> Result<()> {
        let path = Self::history_path()?;
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

// ─── User preference learning ─────────────────────────────────────────────────

/// Preferences learned from the user's command history.
///
/// By analyzing past successful commands for a tool, we can extract patterns
/// like preferred thread counts, output directories, and reference genomes.
/// These are injected into the LLM prompt as soft defaults.
#[derive(Debug, Clone, Default)]
pub struct UserPreferences {
    /// Most commonly used thread count (from -@ / -t / --threads flags).
    pub preferred_threads: Option<String>,
    /// Most commonly used output directory pattern.
    pub preferred_output_dir: Option<String>,
    /// Most commonly used reference genome path.
    pub preferred_reference: Option<String>,
}

impl UserPreferences {
    /// Generate a hint string for LLM prompt injection.
    pub fn to_prompt_hint(&self) -> String {
        let mut hints = Vec::new();
        if let Some(ref t) = self.preferred_threads {
            hints.push(format!("preferred threads: {t}"));
        }
        if let Some(ref d) = self.preferred_output_dir {
            hints.push(format!("preferred output dir: {d}"));
        }
        if let Some(ref r) = self.preferred_reference {
            hints.push(format!("preferred reference: {r}"));
        }
        if hints.is_empty() {
            String::new()
        } else {
            format!("[User preferences: {}]", hints.join(", "))
        }
    }
}

/// Learn user preferences from command history for a specific tool.
///
/// Analyzes the last N successful (exit_code == 0) commands for the tool
/// to extract common patterns.
pub fn learn_user_preferences(tool: &str, history: &[HistoryEntry]) -> UserPreferences {
    let relevant: Vec<&HistoryEntry> = history
        .iter()
        .filter(|e| e.tool == tool && e.exit_code == 0 && !e.dry_run)
        .collect();

    if relevant.is_empty() {
        return UserPreferences::default();
    }

    UserPreferences {
        preferred_threads: most_common_extracted(&relevant, extract_threads),
        preferred_output_dir: most_common_extracted(&relevant, extract_output_dir),
        preferred_reference: most_common_extracted(&relevant, extract_reference),
    }
}

/// Extract the most common value from a set of entries using an extractor function.
fn most_common_extracted(
    entries: &[&HistoryEntry],
    extractor: fn(&str) -> Option<String>,
) -> Option<String> {
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for entry in entries {
        if let Some(val) = extractor(&entry.command) {
            *counts.entry(val).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .filter(|(_, count)| *count >= 2) // Require at least 2 occurrences
        .map(|(val, _)| val)
}

/// Extract thread count from a command string.
fn extract_threads(command: &str) -> Option<String> {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    for (i, token) in tokens.iter().enumerate() {
        if (*token == "-@"
            || *token == "-t"
            || *token == "--threads"
            || *token == "-p"
            || *token == "--cores")
            && i + 1 < tokens.len()
        {
            let val = tokens[i + 1];
            if val.parse::<u32>().is_ok() {
                return Some(val.to_string());
            }
        }
        // Handle -@4, -t8 form
        for prefix in &["-@", "-t", "-p"] {
            if let Some(num) = token.strip_prefix(prefix)
                && num.parse::<u32>().is_ok()
            {
                return Some(num.to_string());
            }
        }
    }
    None
}

/// Extract output directory from a command string.
fn extract_output_dir(command: &str) -> Option<String> {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    for (i, token) in tokens.iter().enumerate() {
        if (*token == "-o" || *token == "--output" || *token == "--outdir" || *token == "--out")
            && i + 1 < tokens.len()
        {
            let val = tokens[i + 1];
            // Extract directory part of the path
            if let Some(parent) = std::path::Path::new(val).parent() {
                let dir = parent.to_string_lossy().to_string();
                if !dir.is_empty() && dir != "." {
                    return Some(dir);
                }
            }
        }
    }
    None
}

/// Extract reference genome path from a command string.
fn extract_reference(command: &str) -> Option<String> {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    for (i, token) in tokens.iter().enumerate() {
        if (*token == "--ref"
            || *token == "--reference"
            || *token == "-x"
            || *token == "--genome"
            || *token == "--genomeDir"
            || *token == "--genome-dir")
            && i + 1 < tokens.len()
        {
            return Some(tokens[i + 1].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // All tests that mutate OXO_CALL_DATA_DIR use the crate-wide ENV_LOCK to
    // prevent races with docs.rs, config.rs, and skill.rs tests.
    use crate::ENV_LOCK;

    fn make_entry(id: &str, tool: &str, dry_run: bool) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            tool: tool.to_string(),
            task: format!("do something with {tool}"),
            command: format!("{tool} --help"),
            exit_code: 0,
            executed_at: Utc::now(),
            dry_run,
            server: None,
            provenance: None,
        }
    }

    fn make_entry_with_provenance(id: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            tool: "samtools".to_string(),
            task: "sort bam".to_string(),
            command: "samtools sort -o out.bam in.bam".to_string(),
            exit_code: 0,
            executed_at: Utc::now(),
            dry_run: false,
            server: None,
            provenance: Some(CommandProvenance {
                tool_version: Some("1.17".to_string()),
                docs_hash: Some("abc123".to_string()),
                skill_name: Some("samtools".to_string()),
                model: Some("gpt-4o".to_string()),
            }),
        }
    }

    #[test]
    fn test_append_and_load_all() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded access guaranteed by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        HistoryStore::clear().unwrap();
        assert!(HistoryStore::load_all().unwrap().is_empty());

        let e1 = make_entry("id-1", "samtools", false);
        let e2 = make_entry("id-2", "bwa", true);
        HistoryStore::append(e1).unwrap();
        HistoryStore::append(e2).unwrap();

        let entries = HistoryStore::load_all().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "id-1");
        assert_eq!(entries[0].tool, "samtools");
        assert!(!entries[0].dry_run);
        assert_eq!(entries[1].id, "id-2");
        assert!(entries[1].dry_run);
    }

    #[test]
    fn test_clear_removes_entries() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded access guaranteed by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        HistoryStore::append(make_entry("id-x", "gatk", false)).unwrap();
        assert!(!HistoryStore::load_all().unwrap().is_empty());

        HistoryStore::clear().unwrap();
        assert!(HistoryStore::load_all().unwrap().is_empty());
    }

    #[test]
    fn test_clear_idempotent_on_empty() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded access guaranteed by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        // Clear on a non-existent file should not error
        HistoryStore::clear().unwrap();
        HistoryStore::clear().unwrap();
    }

    #[test]
    fn test_provenance_round_trip() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded access guaranteed by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        HistoryStore::clear().unwrap();
        HistoryStore::append(make_entry_with_provenance("prov-1")).unwrap();

        let entries = HistoryStore::load_all().unwrap();
        assert_eq!(entries.len(), 1);
        let prov = entries[0].provenance.as_ref().unwrap();
        assert_eq!(prov.tool_version.as_deref(), Some("1.17"));
        assert_eq!(prov.skill_name.as_deref(), Some("samtools"));
        assert_eq!(prov.model.as_deref(), Some("gpt-4o"));
    }

    #[test]
    fn test_command_provenance_default() {
        let p = CommandProvenance::default();
        assert!(p.tool_version.is_none());
        assert!(p.docs_hash.is_none());
        assert!(p.skill_name.is_none());
        assert!(p.model.is_none());
    }

    #[test]
    fn test_history_entry_serializes_without_null_provenance() {
        let entry = make_entry("no-prov", "bwa", false);
        let json = serde_json::to_string(&entry).unwrap();
        // provenance should be omitted (skip_serializing_if = "Option::is_none")
        assert!(!json.contains("provenance"));
    }

    #[test]
    fn test_load_all_empty_when_no_file() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded access guaranteed by ENV_LOCK
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        // Don't create the file; just load — should return empty vec
        let entries = HistoryStore::load_all().unwrap();
        assert!(entries.is_empty());
    }

    // ─── New field: server ────────────────────────────────────────────────────

    /// Helper that creates an entry tagged with a server name.
    fn make_server_entry(id: &str, server: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            tool: "samtools".to_string(),
            task: "sort bam".to_string(),
            command: "samtools sort -o out.bam in.bam".to_string(),
            exit_code: 0,
            executed_at: Utc::now(),
            dry_run: false,
            server: Some(server.to_string()),
            provenance: None,
        }
    }

    #[test]
    fn test_server_field_round_trip_via_store() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        HistoryStore::clear().unwrap();
        HistoryStore::append(make_server_entry("srv-1", "hpc-cluster")).unwrap();

        let entries = HistoryStore::load_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].server.as_deref(), Some("hpc-cluster"));
    }

    #[test]
    fn test_server_field_is_none_for_local_runs() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        HistoryStore::clear().unwrap();
        // local entry has server = None
        HistoryStore::append(make_entry("local-1", "bwa", false)).unwrap();

        let entries = HistoryStore::load_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert!(
            entries[0].server.is_none(),
            "local entry should have server = None"
        );
    }

    #[test]
    fn test_server_field_omitted_in_json_when_none() {
        let entry = make_entry("no-server", "gatk", false);
        let json = serde_json::to_string(&entry).unwrap();
        // server is None → should be omitted (skip_serializing_if)
        assert!(
            !json.contains("\"server\""),
            "server field should be omitted when None, got: {json}"
        );
    }

    #[test]
    fn test_server_field_present_in_json_when_set() {
        let entry = make_server_entry("srv-json", "my-server");
        let json = serde_json::to_string(&entry).unwrap();
        assert!(
            json.contains("\"server\":\"my-server\""),
            "server field should be present when set, got: {json}"
        );
    }

    #[test]
    fn test_old_history_entry_without_server_still_deserializes() {
        // Simulate an old-format JSON line that has no "server" key.
        let old_json = r#"{"id":"old-1","tool":"samtools","task":"sort","command":"samtools sort -o out.bam in.bam","exit_code":0,"executed_at":"2024-01-01T00:00:00Z","dry_run":false}"#;
        let entry: HistoryEntry = serde_json::from_str(old_json).unwrap();
        assert_eq!(entry.id, "old-1");
        assert!(
            entry.server.is_none(),
            "old entries without server should deserialize with server = None"
        );
    }

    #[test]
    fn test_dry_run_entry_serialization() {
        let entry = make_entry("dry-1", "samtools", true);
        let json = serde_json::to_string(&entry).unwrap();
        let back: HistoryEntry = serde_json::from_str(&json).unwrap();
        assert!(back.dry_run, "dry_run should survive round-trip");
        assert!(
            back.server.is_none(),
            "server should be None for local dry-run"
        );
    }

    #[test]
    fn test_server_dry_run_entry() {
        let entry = HistoryEntry {
            id: "sdr-1".to_string(),
            tool: "ls".to_string(),
            task: "list files".to_string(),
            command: "ls -la".to_string(),
            exit_code: 0,
            executed_at: Utc::now(),
            dry_run: true,
            server: Some("remote-box".to_string()),
            provenance: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: HistoryEntry = serde_json::from_str(&json).unwrap();
        assert!(back.dry_run);
        assert_eq!(back.server.as_deref(), Some("remote-box"));
    }

    #[test]
    fn test_mixed_local_and_server_entries() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }

        HistoryStore::clear().unwrap();
        // local run
        HistoryStore::append(make_entry("local-mix", "bwa", false)).unwrap();
        // server run
        HistoryStore::append(make_server_entry("srv-mix", "hpc1")).unwrap();
        // dry-run
        HistoryStore::append(make_entry("dry-mix", "samtools", true)).unwrap();

        let entries = HistoryStore::load_all().unwrap();
        assert_eq!(entries.len(), 3);
        assert!(entries[0].server.is_none());
        assert_eq!(entries[1].server.as_deref(), Some("hpc1"));
        assert!(entries[2].dry_run);
    }

    // ─── User preference learning tests ──────────────────────────────────

    #[test]
    fn test_extract_threads() {
        assert_eq!(
            extract_threads("sort -@ 8 -o out.bam in.bam"),
            Some("8".to_string())
        );
        assert_eq!(
            extract_threads("sort -t 4 -o out.bam"),
            Some("4".to_string())
        );
        assert_eq!(
            extract_threads("sort --threads 16 in.bam"),
            Some("16".to_string())
        );
        assert_eq!(extract_threads("sort -@12 in.bam"), Some("12".to_string()));
        assert_eq!(extract_threads("sort -o out.bam"), None);
    }

    #[test]
    fn test_extract_output_dir() {
        assert_eq!(
            extract_output_dir("run -o /data/results/out.bam in.bam"),
            Some("/data/results".to_string())
        );
        assert_eq!(extract_output_dir("run -o out.bam in.bam"), None);
    }

    #[test]
    fn test_extract_reference() {
        assert_eq!(
            extract_reference("mem --ref /genomes/hg38.fa reads.fq"),
            Some("/genomes/hg38.fa".to_string())
        );
        assert_eq!(
            extract_reference("mem -x /idx/hg38 reads.fq"),
            Some("/idx/hg38".to_string())
        );
    }

    #[test]
    fn test_learn_preferences_empty_history() {
        let prefs = learn_user_preferences("samtools", &[]);
        assert!(prefs.preferred_threads.is_none());
        assert!(prefs.preferred_output_dir.is_none());
        assert!(prefs.preferred_reference.is_none());
    }

    #[test]
    fn test_learn_preferences_from_entries() {
        let entries = vec![
            HistoryEntry {
                id: "test1".to_string(),
                tool: "samtools".to_string(),
                task: "sort".to_string(),
                command: "sort -@ 8 -o /results/out1.bam in1.bam".to_string(),
                exit_code: 0,
                executed_at: Utc::now(),
                dry_run: false,
                server: None,
                provenance: None,
            },
            HistoryEntry {
                id: "test2".to_string(),
                tool: "samtools".to_string(),
                task: "sort".to_string(),
                command: "sort -@ 8 -o /results/out2.bam in2.bam".to_string(),
                exit_code: 0,
                executed_at: Utc::now(),
                dry_run: false,
                server: None,
                provenance: None,
            },
        ];
        let prefs = learn_user_preferences("samtools", &entries);
        assert_eq!(prefs.preferred_threads, Some("8".to_string()));
        assert_eq!(prefs.preferred_output_dir, Some("/results".to_string()));
    }

    #[test]
    fn test_preferences_prompt_hint() {
        let prefs = UserPreferences {
            preferred_threads: Some("8".to_string()),
            preferred_output_dir: None,
            preferred_reference: Some("/genomes/hg38.fa".to_string()),
        };
        let hint = prefs.to_prompt_hint();
        assert!(hint.contains("preferred threads: 8"));
        assert!(hint.contains("preferred reference"));
        assert!(!hint.contains("preferred output dir"));
    }
}
