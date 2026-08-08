//! Command-generation pipeline — single LLM call with evidence-graded prompting.
//!
//! Simplified in 0.20.0: always uses Fast mode (one LLM call with docs + skill).
//! Quality mode (multi-stage mini-skill generation) removed as unnecessary overhead.

use crate::config::Config;
use crate::error::Result;
use crate::llm::{LlmClient, LlmCommandSuggestion};
use crate::skill::Skill;
use std::sync::Arc;

/// Result of a command-generation pipeline run.
#[derive(Debug)]
pub struct CommandGenerationResult {
    pub suggestion: LlmCommandSuggestion,
    pub total_inference_ms: f64,
}

/// Simple single-call pipeline — one LLM invocation with evidence-graded context.
pub struct CommandGenerationPipeline {
    llm_client: Arc<LlmClient>,
}

impl CommandGenerationPipeline {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            llm_client: Arc::new(LlmClient::new(config)),
        })
    }

    /// Generate a command: one LLM call with docs + skill as evidence.
    pub async fn generate(
        &self,
        tool: &str,
        task: &str,
        docs: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
    ) -> Result<CommandGenerationResult> {
        let suggestion = self
            .llm_client
            .suggest_command(tool, docs, task, skill, no_prompt, None)
            .await?;
        Ok(CommandGenerationResult {
            total_inference_ms: suggestion.inference_ms,
            suggestion,
        })
    }
}
