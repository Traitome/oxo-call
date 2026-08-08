//! LLM client module for oxo-call.
//!
//! This module provides the core LLM interaction capabilities, including:
//! - Command generation from natural language task descriptions
//! - Task optimization and refinement
//! - Result verification and validation
//! - Skill file generation and review
//!
//! The module is organized into several sub-modules:
//! - `types`: Core data structures and traits
//! - `prompt`: Prompt building functions for different LLM roles
//! - `response`: Response parsing and validation
//! - `provider`: LLM client implementation and HTTP handling
//! - `streaming`: Shared SSE streaming utilities
//! - `tests`: Unit and integration tests

mod prompt;
mod provider;
mod response;
pub(crate) mod streaming;
pub(crate) mod types;

#[cfg(test)]
mod tests;

// Re-export public types used by other modules
pub use prompt::prompt_tier;
pub use provider::LlmClient;
pub use types::{LlmCommandSuggestion, PromptTier};
