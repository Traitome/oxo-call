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

    #[test]
    fn test_feedback_entry_serialize_roundtrip() {
        let entry = FeedbackEntry {
            tool: "samtools".to_string(),
            task: "sort a bam file".to_string(),
            generated_command: "samtools sort -o sorted.bam input.bam".to_string(),
            was_modified: false,
            modified_command: None,
            exit_code: 0,
            user_approved: true,
            model: "qwen2.5-coder:7b".to_string(),
            recorded_at: "2025-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let restored: FeedbackEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.tool, "samtools");
        assert_eq!(restored.exit_code, 0);
        assert!(restored.user_approved);
        assert!(!restored.was_modified);
        assert!(restored.modified_command.is_none());
    }

    #[test]
    fn test_feedback_entry_serialize_with_modification() {
        let entry = FeedbackEntry {
            tool: "bwa".to_string(),
            task: "align reads".to_string(),
            generated_command: "bwa mem ref.fa reads.fq".to_string(),
            was_modified: true,
            modified_command: Some("bwa mem -t 8 ref.fa reads.fq".to_string()),
            exit_code: 0,
            user_approved: true,
            model: "llama3:8b".to_string(),
            recorded_at: "2025-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let restored: FeedbackEntry = serde_json::from_str(&json).unwrap();
        assert!(restored.was_modified);
        assert_eq!(
            restored.modified_command.as_deref(),
            Some("bwa mem -t 8 ref.fa reads.fq")
        );
    }

    #[test]
    fn test_feedback_entry_with_failure() {
        let entry = FeedbackEntry {
            tool: "fastqc".to_string(),
            task: "check quality".to_string(),
            generated_command: "fastqc nonexistent.fq".to_string(),
            was_modified: false,
            modified_command: None,
            exit_code: 1,
            user_approved: false,
            model: "deepseek-coder:6.7b".to_string(),
            recorded_at: "2025-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(entry.exit_code, 1);
        assert!(!entry.user_approved);
    }

    #[test]
    fn test_feedback_stats_rates() {
        let stats = FeedbackStats {
            total: 10,
            successes: 8,
            modifications: 3,
            success_rate: 0.8,
            modification_rate: 0.3,
        };
        assert_eq!(stats.total, 10);
        assert_eq!(stats.successes, 8);
        assert!((stats.success_rate - 0.8).abs() < f32::EPSILON);
        assert!((stats.modification_rate - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_feedback_record_and_load() {
        // Use a temporary data dir to avoid clobbering real data.
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: This test is single-threaded within its own tempdir and
        // the env var is restored at the end.
        unsafe { std::env::set_var("OXO_CALL_DATA_DIR", tmp.path()) };

        let entry = FeedbackEntry {
            tool: "samtools".to_string(),
            task: "index a bam file".to_string(),
            generated_command: "samtools index input.bam".to_string(),
            was_modified: false,
            modified_command: None,
            exit_code: 0,
            user_approved: true,
            model: "qwen2.5-coder:7b".to_string(),
            recorded_at: "2025-06-01T12:00:00Z".to_string(),
        };

        FeedbackCollector::record(entry.clone()).unwrap();
        let loaded = FeedbackCollector::load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].tool, "samtools");

        // Record another and verify append behaviour.
        let entry2 = FeedbackEntry {
            tool: "bwa".to_string(),
            task: "align reads".to_string(),
            generated_command: "bwa mem ref.fa r1.fq r2.fq".to_string(),
            was_modified: true,
            modified_command: Some("bwa mem -t 4 ref.fa r1.fq r2.fq".to_string()),
            exit_code: 0,
            user_approved: true,
            model: "qwen2.5-coder:7b".to_string(),
            recorded_at: "2025-06-01T12:01:00Z".to_string(),
        };
        FeedbackCollector::record(entry2).unwrap();
        let loaded = FeedbackCollector::load_all().unwrap();
        assert_eq!(loaded.len(), 2);

        // Stats for samtools
        let stats = FeedbackCollector::stats_for_tool("samtools").unwrap();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.successes, 1);
        assert!((stats.success_rate - 1.0).abs() < f32::EPSILON);

        // Overall stats
        let overall = FeedbackCollector::overall_stats().unwrap();
        assert_eq!(overall.total, 2);
        assert_eq!(overall.successes, 2);
        assert_eq!(overall.modifications, 1);

        // Stats for unknown tool
        let empty_stats = FeedbackCollector::stats_for_tool("unknown_tool").unwrap();
        assert_eq!(empty_stats.total, 0);

        // SAFETY: Restoring the env var; single-threaded test context.
        unsafe { std::env::remove_var("OXO_CALL_DATA_DIR") };
    }
}
