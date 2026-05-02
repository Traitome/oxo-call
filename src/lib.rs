//! Programmatic API for oxo-call.
//!
//! This module re-exports the public types and key modules so that downstream
//! crates (including the benchmark suite) can depend on `oxo-call` as a library
//! without reaching into private internals.
//!
//! # Example
//!
//! ```rust,ignore
//! use oxo_call::skill::{Skill, SkillManager, validate_skill_depth};
//! use oxo_call::history::{HistoryEntry, CommandProvenance, WorkflowSuggestion};
//! use oxo_call::mcp::McpClient;
//! ```
//!
//! The binary entry point lives in `main.rs`; this file only surfaces the
//! library interface.

pub mod bench;
pub mod cache;
pub mod config;
pub mod copilot_auth;
pub mod doc_processor;
pub mod doc_summarizer;
pub mod docs;
pub mod engine;
pub mod error;
pub mod execution;
pub mod format;
pub mod handlers;
pub mod history;
pub mod index;
pub mod job;
pub mod knowledge;
pub mod license;
pub mod llm;
pub mod markdown;
pub mod mcp;
pub mod runner;
pub mod schema;
pub mod server;
pub mod skill;
pub mod streaming_display;
pub mod workflow;

// Re-export commonly used types for convenience
pub use history::{ArgCombo, CommandProvenance, HistoryEntry, WorkflowSuggestion};

/// A single crate-wide mutex that **all** test modules must acquire before
/// reading or writing `OXO_CALL_DATA_DIR` (or any other process-global
/// environment variable used by tests).  Using a shared instance prevents
/// the race conditions that arise when separate modules each define their
/// own `ENV_LOCK`.
#[cfg(test)]
pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
