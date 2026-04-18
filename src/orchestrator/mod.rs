//! AI Orchestration Layer (LangGraph-Inspired).
//!
//! Implements a multi-agent coordination system with four core agents:
//! - **Supervisor**: Routes tasks and decides orchestration strategy
//! - **Planner**: Decomposes complex tasks into steps
//! - **Executor**: Generates and runs commands
//! - **Validator**: Verifies results and provides feedback
//!
//! Supports adaptive single-call (Fast) and multi-agent (Quality) modes.

pub mod executor;
pub mod planner;
pub mod supervisor;
pub mod validator;
