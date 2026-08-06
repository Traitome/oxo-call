//! Multi-stage command-generation pipeline.
//!
//! This module selects one of two generation strategies:
//! - Fast mode: Single LLM call (existing behavior)
//! - Quality mode: Multi-stage pipeline (task standardization → doc cleaning → mini-skill generation → command generation)

use crate::config::Config;
use crate::doc_processor::{DocProcessor, StructuredDoc};
use crate::error::{OxoError, Result};
use crate::llm::{
    LlmClient, LlmCommandSuggestion, build_mini_skill_prompt, mini_skill_generation_system_prompt,
};
use crate::mini_skill_cache::{CacheConfig, MiniSkill, MiniSkillCache};
use crate::skill::Skill;
pub use crate::task_complexity::GenerationMode;
use serde::Deserialize;
use sha2::Digest;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Result of a command-generation pipeline run.
#[derive(Debug)]
pub struct CommandGenerationResult {
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
    /// The effective task used for command generation (may be normalized)
    pub effective_task: String,
    /// Whether the task was actually normalized/standardized (changed from input)
    pub was_normalized: bool,
}

/// Multi-stage command-generation pipeline.
pub struct CommandGenerationPipeline {
    llm_client: Arc<LlmClient>,
    mini_skill_cache: Arc<RwLock<MiniSkillCache>>,
    doc_processor: DocProcessor,
    mode: GenerationMode,
}

impl CommandGenerationPipeline {
    /// Create a new command-generation pipeline.
    pub fn new(config: Config, mode: GenerationMode) -> Result<Self> {
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

    /// Generate one command.
    ///
    /// When `structured_doc` is provided, it is passed through to the LLM prompt
    /// builder, enabling doc-extracted few-shot examples and flag catalog injection.
    pub async fn execute(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
        structured_doc: Option<&StructuredDoc>,
    ) -> Result<CommandGenerationResult> {
        match self.mode {
            GenerationMode::Fast => {
                self.execute_fast(tool, documentation, task, skill, no_prompt, structured_doc)
                    .await
            }
            GenerationMode::Quality => {
                self.execute_quality(tool, documentation, task, skill, no_prompt, structured_doc)
                    .await
            }
        }
    }

    /// Fast mode: Single LLM call with doc-enriched prompt.
    ///
    /// This is the default mode. When `structured_doc` is provided, the prompt
    /// includes doc-extracted examples and a flag catalog — giving small models
    /// the grounding they need without the latency of multi-stage calls.
    async fn execute_fast(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
        structured_doc: Option<&StructuredDoc>,
    ) -> Result<CommandGenerationResult> {
        let suggestion = self
            .llm_client
            .suggest_command(tool, documentation, task, skill, no_prompt, structured_doc)
            .await?;

        let inference_ms = suggestion.inference_ms;
        Ok(CommandGenerationResult {
            suggestion,
            mini_skill_generated: false,
            cache_hit: false,
            llm_calls: 1,
            total_inference_ms: inference_ms,
            effective_task: task.to_string(),
            was_normalized: false,
        })
    }

    /// Quality mode: Multi-stage pipeline
    ///
    /// Stages 1 (task standardization) and 2 (mini-skill generation) are
    /// independent and run concurrently via `tokio::join!` when both are
    /// needed, cutting wall-clock latency by up to 50%.
    async fn execute_quality(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
        structured_doc: Option<&StructuredDoc>,
    ) -> Result<CommandGenerationResult> {
        let mut llm_calls = 0;
        let mut total_inference_ms = 0.0;
        let mut mini_skill_generated = false;
        let mut cache_hit = false;

        // Document processing (deterministic, no LLM)
        let owned_sdoc;
        let effective_sdoc = if let Some(sdoc) = structured_doc {
            sdoc
        } else {
            owned_sdoc = self.doc_processor.process(documentation);
            &owned_sdoc
        };
        let cleaned_doc = effective_sdoc.to_string();

        // Compute doc hash for cache key
        let doc_hash = hex::encode(sha2::Sha256::digest(documentation.as_bytes()));

        // Check mini-skill cache first (avoids unnecessary LLM call)
        let cached_mini_skill = {
            let mut cache = self.mini_skill_cache.write().await;
            cache.get(tool, &doc_hash)
        };

        if cached_mini_skill.is_some() {
            cache_hit = true;
        }

        // Determine what LLM calls are needed
        let needs_standardize = self.should_standardize_task(task);
        let needs_mini_skill = cached_mini_skill.is_none() && skill.is_none();

        // ── Run task standardization and mini-skill generation concurrently ──
        let (standardized_task, standardization_inference_ms, generated_mini_skill) =
            match (needs_standardize, needs_mini_skill) {
                (true, true) => {
                    // Both needed — run in parallel
                    let (std_result, ms_result) = tokio::join!(
                        self.llm_client.optimize_task_with_timing(tool, task),
                        self.generate_mini_skill(tool, &cleaned_doc, &doc_hash)
                    );
                    llm_calls += 2;
                    let (standardized_task, inference_ms) = std_result?;
                    (standardized_task, inference_ms, Some(ms_result?))
                }
                (true, false) => {
                    // Only standardization needed
                    llm_calls += 1;
                    let (standardized_task, inference_ms) = self
                        .llm_client
                        .optimize_task_with_timing(tool, task)
                        .await?;
                    (standardized_task, inference_ms, None)
                }
                (false, true) => {
                    // Only mini-skill generation needed
                    llm_calls += 1;
                    let generated = self
                        .generate_mini_skill(tool, &cleaned_doc, &doc_hash)
                        .await?;
                    (task.to_string(), 0.0, Some(generated))
                }
                (false, false) => {
                    // Neither needed
                    (task.to_string(), 0.0, None)
                }
            };
        total_inference_ms += standardization_inference_ms;

        // Insert generated mini-skill into cache
        let mini_skill = if let Some((generated, inference_ms)) = generated_mini_skill {
            let mut cache = self.mini_skill_cache.write().await;
            cache.insert(generated.clone());
            mini_skill_generated = true;
            total_inference_ms += inference_ms;
            Some(generated)
        } else {
            cached_mini_skill
        };

        // Final stage: Command generation with mini-skill + structured doc
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
                Some(effective_sdoc),
            )
            .await?;

        llm_calls += 1;
        let inference_ms = suggestion.inference_ms;
        total_inference_ms += inference_ms;
        let was_normalized = standardized_task != task;

        Ok(CommandGenerationResult {
            suggestion,
            mini_skill_generated,
            cache_hit,
            llm_calls,
            total_inference_ms,
            effective_task: standardized_task,
            was_normalized,
        })
    }

    /// Check if task needs standardization
    fn should_standardize_task(&self, task: &str) -> bool {
        let task_lower = task.to_lowercase();

        // Non-English input always benefits from standardization.
        if !task.is_ascii() {
            return true;
        }

        // Too short — ambiguous by definition
        if task.len() < 10 {
            return true;
        }

        // Vague keywords that indicate an unclear request
        let vague_keywords = ["just", "simply", "basically", "something", "some"];
        if vague_keywords.iter().any(|kw| task_lower.contains(kw)) {
            return true;
        }

        false
    }

    /// Generate a mini-skill from documentation
    async fn generate_mini_skill(
        &self,
        tool: &str,
        documentation: &str,
        doc_hash: &str,
    ) -> Result<(MiniSkill, f64)> {
        let system = mini_skill_generation_system_prompt();
        let user_prompt = build_mini_skill_prompt(tool, documentation);

        let (raw_response, inference_ms) = self
            .llm_client
            .chat_completion_with_timing(system, &user_prompt, Some(1024), Some(0.3))
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
        let task_hash = hex::encode(sha2::Sha256::digest(task_pattern.as_bytes()));

        Ok((
            MiniSkill {
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
            },
            inference_ms,
        ))
    }
}

/// Intermediate JSON structure for mini-skill parsing
#[derive(Debug, Deserialize)]
struct MiniSkillJson {
    concepts: Vec<String>,
    pitfalls: Vec<String>,
    examples: Vec<ExampleJson>,
}

#[derive(Debug, Deserialize)]
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
        let executor =
            CommandGenerationPipeline::new(Config::default(), GenerationMode::Fast).unwrap();

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
