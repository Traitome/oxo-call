//! Execution & Monitoring Layer.
//!
//! Provides result analysis, feedback collection, and self-evolution
//! capabilities for continuous improvement of command generation.

pub mod feedback;
pub mod result_analyzer;

#[cfg(test)]
mod tests {
    use crate::execution::feedback::{FeedbackCollector, FeedbackEntry};
    use crate::execution::result_analyzer::ResultAnalyzer;

    #[test]
    fn test_result_analyzer_new() {
        // ResultAnalyzer can be constructed; no fields to inspect.
        let _analyzer = ResultAnalyzer::new();
    }

    #[test]
    fn test_feedback_entry_roundtrip() {
        let cases: Vec<(&str, i32, bool)> =
            vec![("samtools", 0, true), ("bwa", 1, false), ("gatk", 2, false)];
        for (tool, exit_code, user_approved) in cases {
            let entry = FeedbackEntry {
                tool: tool.to_string(),
                task: "test task".to_string(),
                generated_command: format!("{tool} test"),
                was_modified: false,
                modified_command: None,
                exit_code,
                user_approved,
                model: "test-model".to_string(),
                recorded_at: "2026-01-01T00:00:00Z".to_string(),
            };
            assert_eq!(entry.tool, tool);
            assert_eq!(entry.exit_code, exit_code);
            assert_eq!(entry.user_approved, user_approved);
        }
    }

    #[test]
    fn test_feedback_collector_is_unit_struct() {
        // Verify FeedbackCollector can be used as a unit struct.
        let _fc = FeedbackCollector;
    }
}
