//! Command runner module for oxo-call.
//!
//! This module provides the core execution pipeline:
//! - Fetch tool documentation
//! - Load skill files
//! - Call LLM to generate command arguments
//! - Execute the command (or show in dry-run mode)
//! - Verify results (optional)
//!
//! The module is organized into several sub-modules:
//! - `core`: Runner struct and core methods (prepare, run, dry_run)
//! - `batch`: Batch/parallel execution
//! - `retry`: Auto-retry and LLM verification
//! - `utils`: Helper functions (command building, risk assessment, etc.)
//! - `tests`: Unit and integration tests

mod batch;
mod core;
mod retry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public types
pub use core::Runner;
pub use utils::{is_companion_binary, is_script_executable, make_spinner};
