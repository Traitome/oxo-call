//! Knowledge Enhancement Layer.
//!
//! This module provides knowledge for failure recovery:
//! - **Error Knowledge Base**: Learning from past command failures for auto-retry

pub mod error_db;

#[cfg(test)]
mod tests {
    use crate::knowledge::error_db::ErrorCategory;

    #[test]
    fn test_error_knowledge_db_functions() {
        let hint = ErrorCategory::MissingInput.recovery_hint();
        assert!(!hint.is_empty());
    }

    #[test]
    fn test_error_category_variants_classify() {
        let cases: Vec<(&str, ErrorCategory)> = vec![
            ("No such file or directory", ErrorCategory::MissingInput),
            ("illegal option", ErrorCategory::BadFlag),
            ("Permission denied", ErrorCategory::Permission),
            ("bad_alloc", ErrorCategory::Other),
            ("unexpected eof", ErrorCategory::FormatError),
        ];
        for (msg, expected) in cases {
            let result = ErrorCategory::classify(msg);
            assert_eq!(result, expected, "msg='{}' expected {:?}", msg, expected);
        }
    }
}
