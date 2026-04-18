//! Knowledge Enhancement Layer (RAG-inspired).
//!
//! This module provides the knowledge foundation for grounding LLM calls:
//! - **Tool Knowledge Base**: Embedded bioconda tool metadata with similarity search
//! - **Error Knowledge Base**: Learning from past failures for error recovery
//! - **Best Practices**: Domain-specific bioinformatics best practices

pub mod best_practices;
pub mod error_db;
pub mod tool_knowledge;
