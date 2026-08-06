//! AI Orchestration Layer (LangGraph-Inspired).
//!
//! Implements a command-generation coordination system with three core agents:
//! - **Supervisor**: Routes tasks and decides orchestration strategy
//! - **Executor**: Generates and runs commands
//! - **Validator**: Verifies results and provides feedback
//!
//! Supports adaptive single-call (Fast) and multi-agent (Quality) modes.

pub mod executor;
pub mod supervisor;
pub mod validator;
