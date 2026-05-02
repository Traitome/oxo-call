//! CLI Schema Module
//!
//! This module provides the Schema Intermediate Representation (IR) for
//! representing CLI tool interfaces in a unified formal model.
//!
//! ## Architecture Position
//!
//! The Schema system sits at Layer 2 of HDA (Hierarchical Deterministic Architecture):
//! - Layer 1: Intent parsing (deterministic)
//! - Layer 2: Schema parsing (deterministic, help → structured IR)
//! - Layer 3: Schema-guided generation (constrained LLM)
//! - Layer 4: Validation (deterministic)
//!
//! ## Key Components
//!
//! - `types`: Schema IR definitions (CliSchema, FlagSchema, etc.)
//! - `parser`: Help output parsers for different styles
//! - `generator`: Schema-guided prompt generation and validation
//! - `catalog`: Schema caching and version management (TODO)
//! - `validator`: Schema validation logic (integrated in types)

#[cfg(test)]
mod integration_test;

pub mod generator;
pub mod parser;
pub mod post_process;
pub mod types;
// TODO: Implement these modules
// pub mod catalog;

// Re-export main types for convenience
pub use generator::build_schema_prompt_section;
pub use generator::build_schema_prompt_section_compact;
pub use parser::parse_help;
pub use post_process::schema_post_process;
#[cfg(test)]
pub use types::ValidationError;
pub use types::{
    CliSchema, CliStyle, ConstraintRule, FlagSchema, ParamType, PositionalSchema, SubcommandSchema,
    ValidationResult,
};
