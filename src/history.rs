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
}
