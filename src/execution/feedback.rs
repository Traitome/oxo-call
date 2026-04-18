//! Feedback Collector — records user actions for self-evolution.
//!
//! Tracks whether LLM-generated commands succeeded/failed and whether
//! the user modified them, feeding this data back into the knowledge layer.

use crate::config::Config;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A feedback entry recording the outcome of a command generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEntry {
    /// Tool name.
    pub tool: String,
    /// Original task.
    pub task: String,
    /// Generated command.
    pub generated_command: String,
    /// Whether the command was executed as-is or modified.
    pub was_modified: bool,
    /// Modified command (if user changed it).
    pub modified_command: Option<String>,
    /// Exit code of the executed command.
    pub exit_code: i32,
    /// Whether the user considered the result correct.
    pub user_approved: bool,
    /// Model used.
    pub model: String,
    /// Timestamp.
    pub recorded_at: String,
}

/// Aggregated feedback statistics for a tool.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct FeedbackStats {
    /// Total executions tracked.
    pub total: usize,
    /// Successful executions (exit code 0).
    pub successes: usize,
    /// Times the user modified the generated command.
    pub modifications: usize,
    /// Success rate (0.0–1.0).
    pub success_rate: f32,
    /// Modification rate (0.0–1.0).
    pub modification_rate: f32,
}

/// Feedback collection and aggregation.
pub struct FeedbackCollector;

impl FeedbackCollector {
    fn feedback_path() -> Result<PathBuf> {
        Ok(Config::data_dir()?.join("feedback.jsonl"))
    }

    /// Record a feedback entry.
    pub fn record(entry: FeedbackEntry) -> Result<()> {
        let path = Self::feedback_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let line = serde_json::to_string(&entry)?;
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(f, "{line}")?;
        Ok(())
    }

    /// Load all feedback entries.
    #[allow(dead_code)]
    pub fn load_all() -> Result<Vec<FeedbackEntry>> {
        let path = Self::feedback_path()?;
        if !path.exists() {
            return Ok(vec![]);
        }
        let content = std::fs::read_to_string(&path)?;
        let entries: Vec<FeedbackEntry> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        Ok(entries)
    }

    /// Get aggregated statistics for a specific tool.
    #[allow(dead_code)]
    pub fn stats_for_tool(tool: &str) -> Result<FeedbackStats> {
        let entries = Self::load_all()?;
        let tool_entries: Vec<&FeedbackEntry> = entries
            .iter()
            .filter(|e| e.tool.to_lowercase() == tool.to_lowercase())
            .collect();

        let total = tool_entries.len();
        if total == 0 {
            return Ok(FeedbackStats::default());
        }

        let successes = tool_entries.iter().filter(|e| e.exit_code == 0).count();
        let modifications = tool_entries.iter().filter(|e| e.was_modified).count();

        Ok(FeedbackStats {
            total,
            successes,
            modifications,
            success_rate: successes as f32 / total as f32,
            modification_rate: modifications as f32 / total as f32,
        })
    }

    /// Get overall statistics across all tools.
    #[allow(dead_code)]
    pub fn overall_stats() -> Result<FeedbackStats> {
        let entries = Self::load_all()?;
        let total = entries.len();
        if total == 0 {
            return Ok(FeedbackStats::default());
        }

        let successes = entries.iter().filter(|e| e.exit_code == 0).count();
        let modifications = entries.iter().filter(|e| e.was_modified).count();

        Ok(FeedbackStats {
            total,
            successes,
            modifications,
            success_rate: successes as f32 / total as f32,
            modification_rate: modifications as f32 / total as f32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_stats_default() {
        let stats = FeedbackStats::default();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.success_rate, 0.0);
    }
}
