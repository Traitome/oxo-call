//! User-defined custom commands ("job" library).
//!
//! `cmd` lets users store named shell command shortcuts — similar to
//! shell aliases but richer: each entry carries a description, tags,
//! and timestamps, and can be executed locally or on a registered
//! remote server via SSH.
//!
//! Commands are stored in `~/.local/share/oxo-call/cmds.toml`.

use crate::config::Config;
use crate::error::{OxoError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─── Data structures ──────────────────────────────────────────────────────────

/// A single user-defined command entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmdEntry {
    /// Short name used to invoke this command (must be unique).
    pub name: String,
    /// The shell command string to execute.
    pub command: String,
    /// Optional human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional tags for filtering and organization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// When the entry was first created.
    pub created_at: DateTime<Utc>,
    /// When the entry was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Top-level document stored in `cmds.toml`.
#[derive(Debug, Default, Serialize, Deserialize)]
struct CmdFile {
    #[serde(default)]
    cmds: Vec<CmdEntry>,
}

// ─── Manager ─────────────────────────────────────────────────────────────────

/// Manages the user's personal command library.
pub struct CmdManager;

impl CmdManager {
    fn store_path() -> Result<PathBuf> {
        Ok(Config::data_dir()?.join("cmds.toml"))
    }

    fn load_file() -> Result<CmdFile> {
        let path = Self::store_path()?;
        if !path.exists() {
            return Ok(CmdFile::default());
        }
        let content = std::fs::read_to_string(&path)?;
        toml::from_str(&content)
            .map_err(|e| OxoError::ConfigError(format!("failed to parse cmds.toml: {e}")))
    }

    fn save_file(file: &CmdFile) -> Result<()> {
        let path = Self::store_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(file)
            .map_err(|e| OxoError::ConfigError(format!("failed to serialize cmds: {e}")))?;
        // Atomic write via temp file
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, &content)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// List all commands, optionally filtered by tag.
    pub fn list(tag_filter: Option<&str>) -> Result<Vec<CmdEntry>> {
        let file = Self::load_file()?;
        let entries = if let Some(tag) = tag_filter {
            file.cmds
                .into_iter()
                .filter(|e| e.tags.iter().any(|t| t == tag))
                .collect()
        } else {
            file.cmds
        };
        Ok(entries)
    }

    /// Find a command by name.
    pub fn find(name: &str) -> Result<Option<CmdEntry>> {
        let file = Self::load_file()?;
        Ok(file.cmds.into_iter().find(|e| e.name == name))
    }

    /// Add a new command entry.  Fails if the name is already taken.
    pub fn add(entry: CmdEntry) -> Result<()> {
        let mut file = Self::load_file()?;
        if file.cmds.iter().any(|e| e.name == entry.name) {
            return Err(OxoError::ConfigError(format!(
                "Command '{}' already exists. Use 'cmd edit' to update it.",
                entry.name
            )));
        }
        file.cmds.push(entry);
        Self::save_file(&file)
    }

    /// Remove a command by name.
    pub fn remove(name: &str) -> Result<()> {
        let mut file = Self::load_file()?;
        let before = file.cmds.len();
        file.cmds.retain(|e| e.name != name);
        if file.cmds.len() == before {
            return Err(OxoError::ConfigError(format!(
                "No command found with name '{name}'"
            )));
        }
        Self::save_file(&file)
    }

    /// Edit an existing command entry in place.
    ///
    /// Only fields that are `Some` are updated; `None` leaves the field
    /// unchanged.  Pass `clear_description = true` to explicitly erase
    /// the description.
    pub fn edit(
        name: &str,
        new_command: Option<&str>,
        new_description: Option<&str>,
        clear_description: bool,
        new_tags: Option<Vec<String>>,
    ) -> Result<()> {
        let mut file = Self::load_file()?;
        let entry = file
            .cmds
            .iter_mut()
            .find(|e| e.name == name)
            .ok_or_else(|| OxoError::ConfigError(format!("No command found with name '{name}'")))?;
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
        entry.updated_at = Utc::now();
        Self::save_file(&file)
    }

    /// Rename a command entry.
    pub fn rename(old_name: &str, new_name: &str) -> Result<()> {
        let mut file = Self::load_file()?;
        if file.cmds.iter().any(|e| e.name == new_name) {
            return Err(OxoError::ConfigError(format!(
                "Command '{new_name}' already exists."
            )));
        }
        let entry = file
            .cmds
            .iter_mut()
            .find(|e| e.name == old_name)
            .ok_or_else(|| {
                OxoError::ConfigError(format!("No command found with name '{old_name}'"))
            })?;
        entry.name = new_name.to_string();
        entry.updated_at = Utc::now();
        Self::save_file(&file)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(name: &str, command: &str) -> CmdEntry {
        let now = Utc::now();
        CmdEntry {
            name: name.to_string(),
            command: command.to_string(),
            description: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_cmd_entry_serialization_round_trip() {
        let entry = make_entry("gpu-check", "nvidia-smi");
        let serialized = toml::to_string_pretty(&CmdFile {
            cmds: vec![entry.clone()],
        })
        .unwrap();
        let deserialized: CmdFile = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.cmds.len(), 1);
        assert_eq!(deserialized.cmds[0].name, "gpu-check");
        assert_eq!(deserialized.cmds[0].command, "nvidia-smi");
    }

    #[test]
    fn test_cmd_entry_with_tags_round_trip() {
        let mut entry = make_entry("job1", "squeue -u $USER");
        entry.tags = vec!["slurm".to_string(), "hpc".to_string()];
        entry.description = Some("Show my SLURM jobs".to_string());

        let file = CmdFile { cmds: vec![entry] };
        let s = toml::to_string_pretty(&file).unwrap();
        let back: CmdFile = toml::from_str(&s).unwrap();
        assert_eq!(back.cmds[0].tags, vec!["slurm", "hpc"]);
        assert_eq!(
            back.cmds[0].description.as_deref(),
            Some("Show my SLURM jobs")
        );
    }

    #[test]
    fn test_cmd_file_default_is_empty() {
        let file = CmdFile::default();
        assert!(file.cmds.is_empty());
    }

    #[test]
    fn test_cmd_entry_empty_tags_skip_serialized() {
        let entry = make_entry("empty-label", "echo hi");
        let s = toml::to_string_pretty(&CmdFile { cmds: vec![entry] }).unwrap();
        // tags field should be omitted when the vec is empty
        assert!(
            !s.contains("tags = "),
            "empty tags should not appear in TOML, got: {s}"
        );
    }

    #[test]
    fn test_cmd_entry_no_description_skip_serialized() {
        let entry = make_entry("no-desc", "echo hi");
        let s = toml::to_string_pretty(&CmdFile { cmds: vec![entry] }).unwrap();
        assert!(
            !s.contains("description ="),
            "absent description should not appear in TOML, got: {s}"
        );
    }
}
