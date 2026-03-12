use crate::config::Config;
use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub tool: String,
    pub task: String,
    pub command: String,
    pub exit_code: i32,
    pub executed_at: DateTime<Utc>,
    pub dry_run: bool,
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
