//! Multi-stage LLM workflow executor for command generation.
//!
//! This module implements a LangGraph-inspired workflow with two modes:
//! - Fast mode: Single LLM call (existing behavior)
//! - Quality mode: Multi-stage pipeline (task standardization → doc cleaning → mini-skill generation → command generation)

#![allow(dead_code)]

use crate::config::Config;
use crate::doc_processor::DocProcessor;
use crate::error::{OxoError, Result};
use crate::llm::{
    LlmClient, LlmCommandSuggestion, build_mini_skill_prompt, mini_skill_generation_system_prompt,
};
use crate::mini_skill_cache::{CacheConfig, MiniSkill, MiniSkillCache};
use crate::skill::Skill;
// Re-export WorkflowMode from task_complexity for unified type
pub use crate::task_complexity::WorkflowMode;
use serde::Deserialize;
use sha2::Digest;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Result of a workflow execution
#[derive(Debug)]
pub struct WorkflowResult {
    /// Generated command suggestion
    pub suggestion: LlmCommandSuggestion,
    /// Whether mini-skill was generated in this run
    pub mini_skill_generated: bool,
    /// Whether cache was hit
    pub cache_hit: bool,
    /// Total LLM calls made
    pub llm_calls: usize,
    /// Total inference time (ms)
    pub total_inference_ms: f64,
}

/// Multi-stage LLM workflow executor
#[allow(dead_code)]
pub struct LlmWorkflowExecutor {
    llm_client: Arc<LlmClient>,
    mini_skill_cache: Arc<RwLock<MiniSkillCache>>,
    doc_processor: DocProcessor,
    mode: WorkflowMode,
}

impl LlmWorkflowExecutor {
    /// Create a new workflow executor
    #[allow(dead_code)]
    pub fn new(config: Config, mode: WorkflowMode) -> Result<Self> {
        let llm_client = Arc::new(LlmClient::new(config.clone()));

        // Setup mini-skill cache
        let cache_config = CacheConfig {
            memory_size: 100,
            persist_to_disk: true,
            max_age_days: 30,
        };
        let mini_skill_cache = MiniSkillCache::new(cache_config)?;

        Ok(Self {
            llm_client,
            mini_skill_cache: Arc::new(RwLock::new(mini_skill_cache)),
            doc_processor: DocProcessor::new(),
            mode,
        })
    }

    /// Execute the workflow to generate a command
    #[allow(dead_code)]
    pub async fn execute(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
    ) -> Result<WorkflowResult> {
        match self.mode {
            WorkflowMode::Fast => {
                self.execute_fast(tool, documentation, task, skill, no_prompt)
                    .await
            }
            WorkflowMode::Quality => {
                self.execute_quality(tool, documentation, task, skill, no_prompt)
                    .await
            }
        }
    }

    /// Fast mode: Single LLM call (existing behavior)
    #[allow(dead_code)]
    async fn execute_fast(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
    ) -> Result<WorkflowResult> {
        let suggestion = self
            .llm_client
            .suggest_command(tool, documentation, task, skill, no_prompt)
            .await?;

        let inference_ms = suggestion.inference_ms;
        Ok(WorkflowResult {
            suggestion,
            mini_skill_generated: false,
            cache_hit: false,
            llm_calls: 1,
            total_inference_ms: inference_ms,
        })
    }

    /// Quality mode: Multi-stage pipeline
    #[allow(dead_code)]
    async fn execute_quality(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
    ) -> Result<WorkflowResult> {
        let mut llm_calls = 0;
        let mut total_inference_ms = 0.0;
        let mut mini_skill_generated = false;
        let mut cache_hit = false;

        // Stage 1: Task standardization (optional, only if task is vague)
        let standardized_task = if self.should_standardize_task(task) {
            llm_calls += 1;
            let result = self.llm_client.optimize_task(tool, task).await?;
            // Note: optimize_task doesn't return inference time, so we estimate
            total_inference_ms += 50.0; // Rough estimate
            result
        } else {
            task.to_string()
        };

        // Stage 2: Document processing (intelligent lossless cleaning)
        let structured_doc = self.doc_processor.process(documentation);
        let cleaned_doc = structured_doc.to_string();

        // Compute doc hash for cache key
        let doc_hash = format!("{:x}", sha2::Sha256::digest(documentation.as_bytes()));

        // Stage 3: Mini-skill generation (with cache)
        let mini_skill = {
            let mut cache = self.mini_skill_cache.write().await;

            // Try cache lookup
            if let Some(cached_skill) = cache.get(tool, &standardized_task, &doc_hash) {
                cache_hit = true;
                Some(cached_skill)
            } else if skill.is_none() {
                // No existing skill, generate mini-skill
                llm_calls += 1;
                let generated = self
                    .generate_mini_skill(tool, &cleaned_doc, &doc_hash)
                    .await?;
                cache.insert(generated.clone());
                mini_skill_generated = true;
                Some(generated)
            } else {
                None
            }
        };

        // Stage 4: Command generation with mini-skill
        let mini_skill_ref = mini_skill.as_ref();
        let mini_skill_converted: Option<Skill> = mini_skill_ref.map(|ms| ms.clone().into());

        let suggestion = self
            .llm_client
            .suggest_command(
                tool,
                &cleaned_doc,
                &standardized_task,
                mini_skill_converted.as_ref().or(skill),
                no_prompt,
            )
            .await?;

        llm_calls += 1;
        let inference_ms = suggestion.inference_ms;
        total_inference_ms += inference_ms;

        Ok(WorkflowResult {
            suggestion,
            mini_skill_generated,
            cache_hit,
            llm_calls,
            total_inference_ms,
        })
    }

    /// Check if task needs standardization
    #[allow(dead_code)]
    fn should_standardize_task(&self, task: &str) -> bool {
        // Heuristics for vague tasks
        let task_lower = task.to_lowercase();

        // Too short
        if task.len() < 10 {
            return true;
        }

        // Vague keywords
        let vague_keywords = ["just", "simply", "basically", "something", "some"];
        if vague_keywords.iter().any(|kw| task_lower.contains(kw)) {
            return true;
        }

        // Non-English (simple heuristic: check for non-ASCII characters)
        if !task.is_ascii() {
            return true;
        }

        false
    }

    /// Generate a mini-skill from documentation
    #[allow(dead_code)]
    async fn generate_mini_skill(
        &self,
        tool: &str,
        documentation: &str,
        doc_hash: &str,
    ) -> Result<MiniSkill> {
        let system = mini_skill_generation_system_prompt();
        let user_prompt = build_mini_skill_prompt(tool, documentation);

        let raw_response = self
            .llm_client
            .chat_completion(system, &user_prompt, Some(1024), Some(0.3))
            .await?;

        // Parse JSON response
        let json_str = raw_response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let parsed: MiniSkillJson = serde_json::from_str(json_str).map_err(|e| {
            OxoError::LlmError(format!(
                "Failed to parse mini-skill JSON: {}\nJSON: {}",
                e, json_str
            ))
        })?;

        // Compute task hash from the first example task (or use empty string if no examples)
        let task_pattern = parsed
            .examples
            .first()
            .map(|ex| ex.task.as_str())
            .unwrap_or("");
        let task_hash = format!("{:x}", sha2::Sha256::digest(task_pattern.as_bytes()));

        Ok(MiniSkill {
            tool: tool.to_string(),
            task_hash,
            doc_hash: doc_hash.to_string(),
            concepts: parsed.concepts,
            pitfalls: parsed.pitfalls,
            examples: parsed
                .examples
                .into_iter()
                .map(|ex| crate::mini_skill_cache::MiniSkillExample {
                    task: ex.task,
                    args: ex.args,
                    explanation: ex.explanation,
                })
                .collect(),
            created_at: chrono::Utc::now(),
            hit_count: 0,
        })
    }
}

/// Intermediate JSON structure for mini-skill parsing
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MiniSkillJson {
    concepts: Vec<String>,
    pitfalls: Vec<String>,
    examples: Vec<ExampleJson>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ExampleJson {
    task: String,
    args: String,
    explanation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_standardize_task() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();

        // Should standardize
        assert!(executor.should_standardize_task("sort"));
        assert!(executor.should_standardize_task("just sort the bam"));
        assert!(executor.should_standardize_task("排序BAM文件"));

        // Should not standardize
        assert!(!executor.should_standardize_task("Sort BAM file by read names"));
        assert!(
            !executor.should_standardize_task("Convert SAM to BAM format with compression level 9")
        );
    }
}
