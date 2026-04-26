//! Multi-stage LLM workflow executor for command generation.
//!
//! This module implements a LangGraph-inspired workflow with two modes:
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
    /// The effective task used for command generation (may be normalized)
    pub effective_task: String,
    /// Whether the task was actually normalized/standardized (changed from input)
    pub was_normalized: bool,
}

/// Multi-stage LLM workflow executor
pub struct LlmWorkflowExecutor {
    llm_client: Arc<LlmClient>,
    mini_skill_cache: Arc<RwLock<MiniSkillCache>>,
    doc_processor: DocProcessor,
    mode: WorkflowMode,
}

impl LlmWorkflowExecutor {
    /// Create a new workflow executor
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

    /// Execute the workflow to generate a command.
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
    ) -> Result<WorkflowResult> {
        match self.mode {
            WorkflowMode::Fast => {
                self.execute_fast(tool, documentation, task, skill, no_prompt, structured_doc)
                    .await
            }
            WorkflowMode::Quality => {
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
    ) -> Result<WorkflowResult> {
        let suggestion = self
            .llm_client
            .suggest_command(tool, documentation, task, skill, no_prompt, structured_doc)
            .await?;

        let inference_ms = suggestion.inference_ms;
        Ok(WorkflowResult {
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
    ) -> Result<WorkflowResult> {
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
        // Mini-skill generation can fail (JSON parsing issues); on failure, fallback to None
        let (standardized_task, generated_mini_skill) = match (needs_standardize, needs_mini_skill)
        {
            (true, true) => {
                // Both needed — run in parallel
                let (std_result, ms_result) = tokio::join!(
                    self.llm_client.optimize_task(tool, task),
                    self.generate_mini_skill(tool, &cleaned_doc, &doc_hash)
                );
                llm_calls += 2;
                total_inference_ms += 50.0; // Rough estimate for standardization
                // Mini-skill failure → continue without it (documentation is still used)
                let std = std_result?;
                let ms = ms_result.ok(); // Use ok() to convert error to None
                if ms.is_none() {
                    tracing::warn!("Mini-skill generation failed, using documentation only");
                }
                (std, ms)
            }
            (true, false) => {
                // Only standardization needed
                llm_calls += 1;
                total_inference_ms += 50.0;
                let result = self.llm_client.optimize_task(tool, task).await?;
                (result, None)
            }
            (false, true) => {
                // Only mini-skill generation needed
                llm_calls += 1;
                let generated = self
                    .generate_mini_skill(tool, &cleaned_doc, &doc_hash)
                    .await;
                // Mini-skill failure → continue without it
                let ms = generated.ok();
                if ms.is_none() {
                    tracing::warn!("Mini-skill generation failed, using documentation only");
                }
                (task.to_string(), ms)
            }
            (false, false) => {
                // Neither needed
                (task.to_string(), None)
            }
        };

        // Insert generated mini-skill into cache
        let mini_skill = if let Some(generated) = generated_mini_skill {
            let mut cache = self.mini_skill_cache.write().await;
            cache.insert(generated.clone());
            mini_skill_generated = true;
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

        Ok(WorkflowResult {
            suggestion,
            mini_skill_generated,
            cache_hit,
            llm_calls,
            total_inference_ms,
            effective_task: standardized_task,
            was_normalized: needs_standardize,
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

    /// Extract subcommand from USAGE line in documentation.
    /// For multi-subcommand tools like bwa, samtools, bcftools.
    /// Returns the subcommand if USAGE shows: "tool subcmd [options]"
    fn extract_subcmd_from_usage(doc: &str, tool: &str) -> Option<String> {
        // Find the first USAGE line
        for line in doc.lines().take(20) {
            let line_lower = line.to_lowercase();
            if line_lower.contains("usage:") || line_lower.starts_with("usage:") {
                // Parse: "usage: tool subcmd [options]" or "Usage: bwa mem [options]"
                let parts: Vec<&str> = line.split_whitespace().collect();
                // Look for pattern: usage tool subcmd
                // Skip "usage:" or "Usage:", then find tool name, then subcmd
                let mut found_tool = false;
                for part in parts.iter().skip(1) { // Skip "usage:"
                    let part = part.trim_start_matches(':');
                    if part == tool || part.to_lowercase() == tool.to_lowercase() {
                        found_tool = true;
                    } else if found_tool {
                        // This should be the subcommand
                        // Check it's not a flag or placeholder
                        if !part.starts_with('-')
                            && !part.starts_with('[')
                            && !part.starts_with('<')
                            && part.len() >= 2
                        {
                            return Some(part.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    /// Generate a mini-skill from documentation
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
        let task_hash = hex::encode(sha2::Sha256::digest(task_pattern.as_bytes()));

        // Flatten StringOrArray fields into plain strings
        let concepts = flatten_string_or_array(parsed.concepts);
        let pitfalls = flatten_string_or_array(parsed.pitfalls);

        // Extract subcommand from USAGE if present (for multi-subcommand tools)
        // e.g., "bwa mem [options]" → subcmd = "mem"
        let subcmd_from_usage = Self::extract_subcmd_from_usage(documentation, tool);
        if subcmd_from_usage.is_some() {
            tracing::info!("Detected subcmd from USAGE for {}: {:?}", tool, subcmd_from_usage);
        }

        // Process examples: prepend subcommand if args doesn't start with it
        let examples = parsed
            .examples
            .into_iter()
            .map(|ex| {
                let args = if let Some(ref subcmd) = subcmd_from_usage {
                    // Check if args already starts with subcmd
                    if ex.args.trim().starts_with(subcmd) {
                        tracing::debug!("Example args '{}' already has subcmd {}", ex.args, subcmd);
                        ex.args
                    } else {
                        // Prepend subcmd to args
                        tracing::info!("Prepending subcmd {} to args '{}'", subcmd, ex.args);
                        format!("{} {}", subcmd, ex.args.trim())
                    }
                } else {
                    ex.args
                };
                crate::mini_skill_cache::MiniSkillExample {
                    task: ex.task,
                    args,
                    explanation: ex.explanation.to_string_vec().join(" "),
                }
            })
            .collect();

        Ok(MiniSkill {
            tool: tool.to_string(),
            task_hash,
            doc_hash: doc_hash.to_string(),
            concepts,
            pitfalls,
            examples,
            created_at: chrono::Utc::now(),
            hit_count: 0,
        })
    }
}

/// Intermediate JSON structure for mini-skill parsing
///
/// Uses StringOrArray to handle LLM occasionally generating arrays
/// where strings are expected (a common issue with small models).
#[derive(Debug, Deserialize)]
struct MiniSkillJson {
    #[serde(default)]
    concepts: Vec<StringOrArray>,
    #[serde(default)]
    pitfalls: Vec<StringOrArray>,
    #[serde(default)]
    examples: Vec<ExampleJson>,
}

/// Helper type to parse either a string or an array of strings.
/// LLMs sometimes generate `["point1", "point2"]` when a single string is expected.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StringOrArray {
    String(String),
    Array(Vec<String>),
}

impl Default for StringOrArray {
    fn default() -> Self {
        Self::String(String::new())
    }
}

impl StringOrArray {
    /// Convert to a flat list of strings.
    /// Single strings become one-element vectors; arrays are returned directly.
    fn to_string_vec(&self) -> Vec<String> {
        match self {
            Self::String(s) => {
                if s.is_empty() {
                    vec![]
                } else {
                    vec![s.clone()]
                }
            }
            Self::Array(arr) => arr.clone(),
        }
    }
}

/// Flatten a Vec<StringOrArray> into a Vec<String>.
fn flatten_string_or_array(items: Vec<StringOrArray>) -> Vec<String> {
    items.iter().flat_map(|item| item.to_string_vec()).collect()
}

#[derive(Debug, Deserialize)]
struct ExampleJson {
    task: String,
    args: String,
    #[serde(default)]
    explanation: StringOrArray,
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

    #[test]
    fn test_should_standardize_short_task() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        // Tasks shorter than 10 chars should always be standardized
        assert!(executor.should_standardize_task("sort"));
        assert!(executor.should_standardize_task("align"));
        assert!(executor.should_standardize_task("view"));
        // Exactly 10 chars - NOT short
        assert!(!executor.should_standardize_task("sort files"));
    }

    #[test]
    fn test_should_standardize_vague_keywords() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        assert!(executor.should_standardize_task("simply sort the bam file by coordinate"));
        assert!(executor.should_standardize_task("basically align the reads to the genome"));
        assert!(executor.should_standardize_task("do something with the vcf file please"));
        assert!(executor.should_standardize_task("call some variants from the BAM file"));
    }

    #[test]
    fn test_should_not_standardize_clear_task() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        assert!(
            !executor.should_standardize_task(
                "Sort BAM file input.bam by coordinate, output sorted.bam"
            )
        );
        assert!(
            !executor.should_standardize_task(
                "Align paired-end reads R1.fq.gz R2.fq.gz to hg38 reference"
            )
        );
        assert!(
            !executor.should_standardize_task(
                "Call variants from aligned.bam using hg38 reference genome"
            )
        );
    }

    #[test]
    fn test_workflow_executor_new_fast_mode() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast);
        assert!(executor.is_ok());
    }

    #[test]
    fn test_workflow_executor_new_quality_mode() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Quality);
        assert!(executor.is_ok());
    }

    #[test]
    fn test_mini_skill_json_deserializes() {
        let json = r#"{
            "concepts": ["BAM format", "Coordinate sorting"],
            "pitfalls": ["Always sort before indexing"],
            "examples": [
                {
                    "task": "Sort a BAM file",
                    "args": "sort -@ 4 -o sorted.bam input.bam",
                    "explanation": "Sort by coordinate with 4 threads"
                }
            ]
        }"#;
        let parsed: MiniSkillJson = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.concepts.len(), 2);
        assert_eq!(parsed.pitfalls.len(), 1);
        assert_eq!(parsed.examples.len(), 1);
        assert_eq!(parsed.examples[0].task, "Sort a BAM file");
        assert_eq!(parsed.examples[0].args, "sort -@ 4 -o sorted.bam input.bam");
    }

    #[test]
    fn test_example_json_deserializes() {
        let json = r#"{"task": "Index BAM", "args": "index sorted.bam", "explanation": "Create BAI index"}"#;
        let ex: ExampleJson = serde_json::from_str(json).unwrap();
        assert_eq!(ex.task, "Index BAM");
        assert_eq!(ex.args, "index sorted.bam");
        // explanation is StringOrArray, check it converts to string correctly
        assert_eq!(ex.explanation.to_string_vec().join(" "), "Create BAI index");
    }

    #[test]
    fn test_example_json_array_explanation() {
        // Test that array explanations are handled correctly
        let json = r#"{"task": "Index BAM", "args": "index sorted.bam", "explanation": ["Create BAI index", "Fast operation"]}"#;
        let ex: ExampleJson = serde_json::from_str(json).unwrap();
        assert_eq!(ex.explanation.to_string_vec().join(" "), "Create BAI index Fast operation");
    }

    #[test]
    fn test_workflow_mode_variants() {
        // WorkflowMode is re-exported from task_complexity
        let _fast = WorkflowMode::Fast;
        let _quality = WorkflowMode::Quality;
        // Just verify the enum variants exist and can be used
        assert!(matches!(WorkflowMode::Fast, WorkflowMode::Fast));
        assert!(matches!(WorkflowMode::Quality, WorkflowMode::Quality));
    }
}
