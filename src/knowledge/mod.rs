//! Knowledge Enhancement Layer (RAG-inspired).
//!
//! This module provides the knowledge foundation for grounding LLM calls:
//! - **Tool Knowledge Base**: Embedded bioconda tool metadata with similarity search
//! - **Error Knowledge Base**: Learning from past failures for error recovery
//! - **Best Practices**: Domain-specific bioinformatics best practices

pub mod best_practices;
pub mod error_db;
pub mod tool_knowledge;

#[cfg(test)]
mod tests {
    use crate::knowledge::best_practices::BestPracticesDb;
    use crate::knowledge::error_db::{ErrorCategory, ErrorKnowledgeDb};
    use crate::knowledge::tool_knowledge::ToolKnowledgeBase;

    #[test]
    fn test_best_practices_db_new() {
        let db = BestPracticesDb::new();
        assert!(!db.is_empty());
    }

    #[test]
    fn test_error_knowledge_db_functions() {
        let hint = ErrorCategory::MissingInput.recovery_hint();
        assert!(!hint.is_empty());
    }

    #[test]
    fn test_tool_knowledge_base_new() {
        let kb = ToolKnowledgeBase::new();
        assert!(!kb.is_empty());
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

    #[test]
    fn test_best_practices_for_samtools() {
        let db = BestPracticesDb::new();
        let practices = db.for_tool("samtools");
        assert!(!practices.is_empty(), "samtools should have best practices");
    }

    #[test]
    fn test_tool_knowledge_lookup() {
        let kb = ToolKnowledgeBase::new();
        let entry = kb.lookup("samtools");
        assert!(entry.is_some(), "samtools should be in knowledge base");
    }
}
