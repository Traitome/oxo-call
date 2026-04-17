//! Validator Agent — verifies execution results and provides feedback.
//!
//! The validator inspects command output (exit code, stderr, output files)
//! and uses the error knowledge DB to provide recovery suggestions.

use crate::knowledge::error_db::{ErrorCategory, ErrorKnowledgeDb};
use serde::{Deserialize, Serialize};

/// Validation result for a command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the command succeeded.
    pub success: bool,
    /// Validation summary.
    pub summary: String,
    /// Detected issues.
    pub issues: Vec<String>,
    /// Recovery suggestions (from error DB + heuristics).
    pub suggestions: Vec<String>,
    /// Error category (if failed).
    pub error_category: Option<ErrorCategory>,
}

impl ValidationResult {
    /// Create a success result.
    pub fn success(summary: &str) -> Self {
        Self {
            success: true,
            summary: summary.to_string(),
            issues: vec![],
            suggestions: vec![],
            error_category: None,
        }
    }

    /// Create a failure result.
    pub fn failure(summary: &str, issues: Vec<String>, suggestions: Vec<String>) -> Self {
        Self {
            success: false,
            summary: summary.to_string(),
            issues,
            suggestions,
            error_category: None,
        }
    }
}

/// The Validator Agent.
pub struct ValidatorAgent;

impl Default for ValidatorAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidatorAgent {
    pub fn new() -> Self {
        Self
    }

    /// Validate a command execution result.
    pub fn validate(
        &self,
        tool: &str,
        _task: &str,
        _command: &str,
        exit_code: i32,
        stderr: &str,
    ) -> ValidationResult {
        if exit_code == 0 && !self.has_warning_patterns(stderr) {
            return ValidationResult::success("Command completed successfully");
        }

        let error_category = ErrorCategory::classify(stderr);
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        // Collect issues.
        if exit_code != 0 {
            issues.push(format!("Command exited with code {exit_code}"));
        }

        // Extract key error lines from stderr.
        let error_lines: Vec<&str> = stderr
            .lines()
            .filter(|l| {
                let lower = l.to_lowercase();
                lower.contains("error")
                    || lower.contains("fatal")
                    || lower.contains("fail")
                    || lower.contains("abort")
                    || lower.starts_with("[e::")
            })
            .take(5)
            .collect();

        for line in &error_lines {
            issues.push((*line).to_string());
        }

        // Get recovery suggestion from error DB.
        let recovery = ErrorKnowledgeDb::suggest_recovery(tool, stderr);
        suggestions.push(recovery);

        // Note: Error recording is done in the runner (runner/core.rs) to avoid
        // duplication.  The validator only reads from the error DB.

        let summary =
            format!("Command failed (exit code {exit_code}, category: {error_category:?})");

        let mut result = ValidationResult::failure(&summary, issues, suggestions);
        result.error_category = Some(error_category);
        result
    }

    /// Check if stderr contains warning patterns even when exit code is 0.
    fn has_warning_patterns(&self, stderr: &str) -> bool {
        let lower = stderr.to_lowercase();
        lower.contains("[warning]")
            || lower.contains("warn:")
            || (lower.contains("error") && !lower.contains("error rate"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_success() {
        let validator = ValidatorAgent::new();
        let result = validator.validate(
            "samtools",
            "sort input.bam",
            "samtools sort input.bam",
            0,
            "",
        );
        assert!(result.success);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_validate_failure_missing_file() {
        let validator = ValidatorAgent::new();
        let result = validator.validate(
            "samtools",
            "sort input.bam",
            "samtools sort input.bam",
            1,
            "samtools sort: No such file or directory: 'input.bam'",
        );
        assert!(!result.success);
        assert_eq!(result.error_category, Some(ErrorCategory::MissingInput));
        assert!(!result.suggestions.is_empty());
    }

    #[test]
    fn test_validate_failure_bad_flag() {
        let validator = ValidatorAgent::new();
        let result = validator.validate(
            "samtools",
            "sort bam",
            "samtools sort --invalid-flag",
            1,
            "samtools sort: unrecognized option '--invalid-flag'",
        );
        assert!(!result.success);
        assert_eq!(result.error_category, Some(ErrorCategory::BadFlag));
    }

    #[test]
    fn test_validate_success_with_benign_stderr() {
        let validator = ValidatorAgent::new();
        let result = validator.validate(
            "samtools",
            "flagstat input.bam",
            "samtools flagstat input.bam",
            0,
            "0 + 0 mapped\n1000 + 0 total\nerror rate: 0.01",
        );
        // "error rate" should not trigger a warning.
        assert!(result.success);
    }
}
