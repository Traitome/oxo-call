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
//! use oxo_call::history::{HistoryEntry, CommandProvenance};
//! use oxo_call::mcp::McpClient;
//! ```
//!
//! The binary entry point lives in `main.rs`; this file only surfaces the
//! library interface.

pub mod config;
pub mod docs;
pub mod engine;
pub mod error;
pub mod handlers;
pub mod history;
pub mod index;
pub mod license;
pub mod llm;
pub mod mcp;
pub mod runner;
pub mod sanitize;
pub mod skill;
pub mod workflow;
