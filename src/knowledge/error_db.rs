//! Error Knowledge Database for learning from past failures.
//!
//! Records execution failures with their context (tool, task, error message,
//! stderr) and uses pattern matching to suggest fixes for recurring errors.
//! This implements the "Self-Evolution Engine" concept from the architecture.

use crate::config::Config;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A recorded error with its resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    /// Tool that failed.
    pub tool: String,
    /// Original task description.
    pub task: String,
    /// The command that failed.
    pub failed_command: String,
    /// Exit code from the failed command.
    pub exit_code: i32,
    /// Stderr output (truncated to 2000 chars).
    pub stderr_snippet: String,
    /// Category of the error (e.g. "missing_file", "bad_flag", "permission").
    pub error_category: ErrorCategory,
    /// The corrected command (if auto-retry succeeded), or a manual fix hint.
    pub resolution: Option<String>,
    /// Timestamp of the error.
    pub recorded_at: String,
}

/// Broad error categories for pattern matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Input file not found or inaccessible.
    MissingInput,
    /// Invalid or unknown flag/option.
    BadFlag,
    /// Permission denied.
    Permission,
    /// Out of memory.
    OutOfMemory,
    /// Invalid file format.
    FormatError,
    /// Tool not installed or not in PATH.
    ToolMissing,
    /// Reference genome / index not found.
    MissingReference,
    /// General / uncategorised.
    Other,
}

impl ErrorCategory {
    /// Classify stderr text into an error category.
    ///
    /// Uses broad keyword matching — longer / more-specific patterns are tested
    /// first to avoid mis-classification (e.g., "reference not found" before
    /// generic "not found").
    pub fn classify(stderr: &str) -> Self {
        let s = stderr.to_lowercase();

        // Reference-specific (must precede generic "not found").
        if (s.contains("reference") || s.contains("genome") || s.contains("index"))
            && (s.contains("not found") || s.contains("missing") || s.contains("no such"))
        {
            return Self::MissingReference;
        }

        // Missing input.
        if s.contains("no such file")
            || s.contains("not found")
            || s.contains("cannot open")
            || s.contains("file not found")
            || s.contains("does not exist")
            || s.contains("failed to open")
        {
            return Self::MissingInput;
        }

        // Bad flags.
        if s.contains("unknown option")
            || s.contains("unrecognized option")
            || s.contains("invalid option")
            || s.contains("bad flag")
            || s.contains("unrecognised option")
            || s.contains("illegal option")
            || s.contains("unknown argument")
        {
            return Self::BadFlag;
        }

        // Permission.
        if s.contains("permission denied")
            || s.contains("access denied")
            || s.contains("operation not permitted")
        {
            return Self::Permission;
        }

        // Out of memory.
        if s.contains("out of memory")
            || s.contains("cannot allocate")
            || s.contains("oom")
            || s.contains("memory allocation failed")
            || s.contains("std::bad_alloc")
            || s.contains("killed")
        {
            return Self::OutOfMemory;
        }

        // Format errors.
        if s.contains("invalid format")
            || s.contains("not a bam")
            || s.contains("truncated file")
            || s.contains("is not a")
            || s.contains("unexpected eof")
            || s.contains("corrupted")
            || s.contains("magic number")
            || s.contains("failed to parse")
        {
            return Self::FormatError;
        }

        // Tool missing.
        if s.contains("command not found")
            || s.contains("no such command")
            || s.contains("not recognized")
            || s.contains("not installed")
        {
            return Self::ToolMissing;
        }

        Self::Other
    }

    /// Return a human-readable recovery hint for this error category.
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::MissingInput => "Check that the input file exists and the path is correct.",
            Self::BadFlag => {
                "One or more flags may be invalid for this tool version. Check `tool --help` for supported options."
            }
            Self::Permission => {
                "Check file/directory permissions. You may need to run with elevated privileges or change ownership."
            }
            Self::OutOfMemory => {
                "The system ran out of memory. Try reducing thread count, using a smaller chunk size, or running on a machine with more RAM."
            }
            Self::FormatError => {
                "The input file format may be incorrect or the file may be corrupted. Verify the file type matches what the tool expects."
            }
            Self::ToolMissing => {
                "The tool is not installed or not in your PATH. Install it via conda/bioconda or check your PATH."
            }
            Self::MissingReference => {
                "A reference genome or index file is missing. Ensure you have built the required index."
            }
            Self::Other => "An unexpected error occurred. Check the stderr output for details.",
        }
    }
}

/// Error knowledge database backed by a JSONL file.
pub struct ErrorKnowledgeDb;

impl ErrorKnowledgeDb {
    fn db_path() -> Result<PathBuf> {
        Ok(Config::data_dir()?.join("error_knowledge.jsonl"))
    }

    /// Record an error for future learning.
    pub fn record(record: ErrorRecord) -> Result<()> {
        let path = Self::db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let line = serde_json::to_string(&record)?;
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(f, "{line}")?;
        Ok(())
    }

    /// Search for similar past errors for the given tool and error category.
    /// Returns up to `limit` matching records, newest first.
    pub fn search(tool: &str, category: ErrorCategory, limit: usize) -> Result<Vec<ErrorRecord>> {
        let path = Self::db_path()?;
        if !path.exists() {
            return Ok(vec![]);
        }

        let content = std::fs::read_to_string(&path)?;
        let mut matches: Vec<ErrorRecord> = content
            .lines()
            .filter_map(|line| serde_json::from_str::<ErrorRecord>(line).ok())
            .filter(|r| {
                r.tool.to_lowercase() == tool.to_lowercase() && r.error_category == category
            })
            .collect();

        // Newest first (reverse order since file is append-only).
        matches.reverse();
        matches.truncate(limit);
        Ok(matches)
    }

    /// Get a recovery suggestion based on past errors.
    /// First checks the error DB for tool-specific fixes, then falls back to generic hints.
    pub fn suggest_recovery(tool: &str, stderr: &str) -> String {
        let category = ErrorCategory::classify(stderr);

        // Try to find a past resolution for this tool + category.
        if let Ok(past_errors) = Self::search(tool, category, 3) {
            for record in &past_errors {
                if let Some(ref resolution) = record.resolution {
                    return format!(
                        "Based on a similar past error: {resolution}\n(Category: {category:?})"
                    );
                }
            }
        }

        // Fall back to generic hint.
        format!(
            "{}\n(Error category: {:?})",
            category.recovery_hint(),
            category
        )
    }

    /// Count total recorded errors.
    #[allow(dead_code)]
    pub fn count() -> Result<usize> {
        let path = Self::db_path()?;
        if !path.exists() {
            return Ok(0);
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(content.lines().filter(|l| !l.trim().is_empty()).count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_classify_missing_input() {
        assert_eq!(
            ErrorCategory::classify("samtools: No such file or directory"),
            ErrorCategory::MissingInput
        );
    }

    #[test]
    fn test_error_category_classify_bad_flag() {
        assert_eq!(
            ErrorCategory::classify("samtools: unrecognized option '--xyz'"),
            ErrorCategory::BadFlag
        );
    }

    #[test]
    fn test_error_category_classify_permission() {
        assert_eq!(
            ErrorCategory::classify("Permission denied: /data/output.bam"),
            ErrorCategory::Permission
        );
    }

    #[test]
    fn test_error_category_classify_oom() {
        assert_eq!(
            ErrorCategory::classify("fatal: out of memory"),
            ErrorCategory::OutOfMemory
        );
    }

    #[test]
    fn test_error_category_classify_format() {
        assert_eq!(
            ErrorCategory::classify("[E::sam_parse1] truncated file"),
            ErrorCategory::FormatError
        );
    }

    #[test]
    fn test_error_category_classify_other() {
        assert_eq!(
            ErrorCategory::classify("some random error"),
            ErrorCategory::Other
        );
    }

    #[test]
    fn test_recovery_hint_not_empty() {
        for cat in &[
            ErrorCategory::MissingInput,
            ErrorCategory::BadFlag,
            ErrorCategory::Permission,
            ErrorCategory::OutOfMemory,
            ErrorCategory::FormatError,
            ErrorCategory::ToolMissing,
            ErrorCategory::MissingReference,
            ErrorCategory::Other,
        ] {
            assert!(!cat.recovery_hint().is_empty());
        }
    }

    // ── Expanded error classification tests ──────────────────────────────────

    #[test]
    fn test_classify_does_not_exist() {
        assert_eq!(
            ErrorCategory::classify("error: file 'input.bam' does not exist"),
            ErrorCategory::MissingInput
        );
    }

    #[test]
    fn test_classify_failed_to_open() {
        assert_eq!(
            ErrorCategory::classify("[E::hts_open_format] Failed to open input.bam"),
            ErrorCategory::MissingInput
        );
    }

    #[test]
    fn test_classify_illegal_option() {
        assert_eq!(
            ErrorCategory::classify("illegal option -- z"),
            ErrorCategory::BadFlag
        );
    }

    #[test]
    fn test_classify_unknown_argument() {
        assert_eq!(
            ErrorCategory::classify("error: unknown argument '--xyz'"),
            ErrorCategory::BadFlag
        );
    }

    #[test]
    fn test_classify_operation_not_permitted() {
        assert_eq!(
            ErrorCategory::classify("operation not permitted: /etc/hosts"),
            ErrorCategory::Permission
        );
    }

    #[test]
    fn test_classify_bad_alloc() {
        assert_eq!(
            ErrorCategory::classify("terminate called after throwing 'std::bad_alloc'"),
            ErrorCategory::OutOfMemory
        );
    }

    #[test]
    fn test_classify_killed() {
        assert_eq!(
            ErrorCategory::classify("Killed"),
            ErrorCategory::OutOfMemory
        );
    }

    #[test]
    fn test_classify_corrupted() {
        assert_eq!(
            ErrorCategory::classify("error: input file is corrupted"),
            ErrorCategory::FormatError
        );
    }

    #[test]
    fn test_classify_unexpected_eof() {
        assert_eq!(
            ErrorCategory::classify("unexpected EOF in gzip stream"),
            ErrorCategory::FormatError
        );
    }

    #[test]
    fn test_classify_not_recognized() {
        assert_eq!(
            ErrorCategory::classify("'bwa' is not recognized as a command"),
            ErrorCategory::ToolMissing
        );
    }

    #[test]
    fn test_classify_reference_missing() {
        assert_eq!(
            ErrorCategory::classify("reference genome not found at /data/ref.fa"),
            ErrorCategory::MissingReference
        );
    }

    #[test]
    fn test_classify_index_missing() {
        assert_eq!(
            ErrorCategory::classify("index not found for reference.fa"),
            ErrorCategory::MissingReference
        );
    }

    #[test]
    fn test_suggest_recovery_returns_hint() {
        let suggestion =
            ErrorKnowledgeDb::suggest_recovery("samtools", "No such file or directory");
        assert!(suggestion.contains("Check"));
        assert!(suggestion.contains("MissingInput"));
    }
}
