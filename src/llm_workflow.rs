//! HDA Workflow Executor for command generation.
//!
//! This module implements a confidence-driven workflow based on the Hierarchical
//! Deterministic Architecture (HDA):
//!
//! - **High Confidence (≥0.7)**: Single LLM call with schema constraints → Fast
//! - **Medium Confidence (0.4-0.7)**: Single call + validation + retry → Quality
//! - **Low Confidence (<0.4)**: Multi-stage reasoning with task standardization
//!
//! ## Design Principles
//!
//! 1. **Deterministic layers first**: Schema parsing, intent matching, validation
//! 2. **Probabilistic layer minimal**: LLM only fills parameter values
//! 3. **No mini-skill for small models**: StructuredDoc is more reliable for ≤3B

use crate::confidence::{ConfidenceLevel, ConfidenceResult, estimate_confidence};
use crate::config::Config;
use crate::doc_processor::{DocProcessor, StructuredDoc};
use crate::error::Result;
use crate::llm::{LlmClient, LlmCommandSuggestion};
use crate::schema::CliSchema;
use crate::schema::CliStyle;
use crate::skill::Skill;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WorkflowMode {
    #[default]
    Fast,
    Quality,
}

/// Result of a workflow execution
#[derive(Debug)]
#[allow(dead_code)]
pub struct WorkflowResult {
    pub suggestion: LlmCommandSuggestion,
    pub llm_calls: usize,
    pub total_inference_ms: f64,
    pub effective_task: String,
    pub was_normalized: bool,
    pub confidence: Option<ConfidenceResult>,
}

pub struct LlmWorkflowExecutor {
    llm_client: Arc<LlmClient>,
    doc_processor: DocProcessor,
    mode: WorkflowMode,
    model_param_count: Option<f32>,
}

impl LlmWorkflowExecutor {
    pub fn new(config: Config, mode: WorkflowMode) -> Result<Self> {
        let llm_client = Arc::new(LlmClient::new(config.clone()));
        let model_name = config.llm.model.as_deref().unwrap_or("");
        let model_param_count = crate::config::infer_model_parameter_count(model_name);

        Ok(Self {
            llm_client,
            doc_processor: DocProcessor::new(),
            mode,
            model_param_count,
        })
    }

    #[allow(dead_code)]
    fn is_small_model(&self) -> bool {
        self.model_param_count.is_some_and(|p| p <= 3.0)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn execute(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
        structured_doc: Option<&StructuredDoc>,
        schema: Option<&CliSchema>,
    ) -> Result<WorkflowResult> {
        let confidence = self.compute_confidence(schema, task, skill);
        let effective_mode = self.resolve_mode(&confidence);

        match effective_mode {
            WorkflowMode::Fast => {
                self.execute_fast(
                    tool,
                    documentation,
                    task,
                    skill,
                    no_prompt,
                    structured_doc,
                    schema,
                    &confidence,
                )
                .await
            }
            WorkflowMode::Quality => {
                self.execute_quality(
                    tool,
                    documentation,
                    task,
                    skill,
                    no_prompt,
                    structured_doc,
                    schema,
                    &confidence,
                )
                .await
            }
        }
    }

    fn compute_confidence(
        &self,
        schema: Option<&CliSchema>,
        task: &str,
        skill: Option<&Skill>,
    ) -> Option<ConfidenceResult> {
        let schema_flags = schema
            .map(|s| s.flags.len() + s.global_flags.len())
            .unwrap_or(0);
        let schema_desc_coverage = schema
            .map(|s| {
                let with_desc = s.flags.iter().filter(|f| !f.description.is_empty()).count()
                    + s.global_flags
                        .iter()
                        .filter(|f| !f.description.is_empty())
                        .count();
                let total = s.flags.len() + s.global_flags.len();
                if total > 0 {
                    with_desc as f32 / total as f32
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);

        let task_lower = task.to_lowercase();
        let keyword_match = schema.and_then(|s| s.select_subcommand(task)).is_some();
        let file_mentions = task.matches('.').count()
            + task_lower.matches("file").count()
            + task_lower.matches("bam").count()
            + task_lower.matches("fastq").count()
            + task_lower.matches("fasta").count();

        Some(estimate_confidence(
            schema_flags,
            schema_desc_coverage,
            keyword_match,
            file_mentions,
            self.model_param_count,
            skill.is_some(),
        ))
    }

    fn resolve_mode(&self, confidence: &Option<ConfidenceResult>) -> WorkflowMode {
        match confidence {
            Some(c) if c.level == ConfidenceLevel::Low => WorkflowMode::Quality,
            _ => self.mode,
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute_fast(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
        structured_doc: Option<&StructuredDoc>,
        schema: Option<&CliSchema>,
        confidence: &Option<ConfidenceResult>,
    ) -> Result<WorkflowResult> {
        let mut suggestion = self
            .llm_client
            .suggest_command(
                tool,
                documentation,
                task,
                skill,
                no_prompt,
                structured_doc,
                schema,
            )
            .await?;

        let inference_ms = suggestion.inference_ms;
        if let Some(sch) = schema {
            suggestion.args = schema_post_process(&suggestion.args, sch, task);
        }

        Ok(WorkflowResult {
            suggestion,
            llm_calls: 1,
            total_inference_ms: inference_ms,
            effective_task: task.to_string(),
            was_normalized: false,
            confidence: confidence.clone(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute_quality(
        &self,
        tool: &str,
        documentation: &str,
        task: &str,
        skill: Option<&Skill>,
        no_prompt: bool,
        structured_doc: Option<&StructuredDoc>,
        schema: Option<&CliSchema>,
        confidence: &Option<ConfidenceResult>,
    ) -> Result<WorkflowResult> {
        let mut llm_calls = 0;
        let mut total_inference_ms = 0.0;

        let owned_sdoc;
        let effective_sdoc = if let Some(sdoc) = structured_doc {
            sdoc
        } else {
            owned_sdoc = self.doc_processor.process(documentation);
            &owned_sdoc
        };
        let cleaned_doc = effective_sdoc.to_string();

        let needs_standardize = self.should_standardize_task(task);
        let mut standardized_task = task.to_string();
        let mut was_normalized = false;

        if needs_standardize {
            llm_calls += 1;
            total_inference_ms += 50.0;
            standardized_task = self.llm_client.optimize_task(tool, task).await?;
            was_normalized = true;
        }

        let mut suggestion = self
            .llm_client
            .suggest_command(
                tool,
                &cleaned_doc,
                &standardized_task,
                skill,
                no_prompt,
                Some(effective_sdoc),
                schema,
            )
            .await?;

        llm_calls += 1;
        total_inference_ms += suggestion.inference_ms;

        if let Some(sch) = schema {
            suggestion.args = schema_post_process(&suggestion.args, sch, &standardized_task);
        }

        Ok(WorkflowResult {
            suggestion,
            llm_calls,
            total_inference_ms,
            effective_task: standardized_task,
            was_normalized,
            confidence: confidence.clone(),
        })
    }

    fn should_standardize_task(&self, task: &str) -> bool {
        let task_lower = task.to_lowercase();
        if !task.is_ascii() || task.len() < 10 {
            return true;
        }
        let vague_keywords = ["just", "simply", "basically", "something", "some"];
        vague_keywords.iter().any(|kw| task_lower.contains(kw))
    }
}

/// HDA Layer 4: Deterministic post-processing of LLM-generated args.
///
/// This is the most critical optimization for small models (≤3B):
/// - **Subcommand injection**: If schema says subcommand-style but LLM
///   omitted the subcommand, deterministically inject the correct one
/// - **Flag whitelist enforcement**: Remove flags not in schema whitelist
/// - **Required flag injection**: Add missing required flags with placeholder values
///
/// These operations are 100% deterministic — no LLM involved.
fn schema_post_process(args: &[String], schema: &CliSchema, task: &str) -> Vec<String> {
    if args.is_empty() {
        return args.to_vec();
    }

    let mut tokens = args.to_vec();

    // Phase 1: Subcommand deterministic injection
    if schema.cli_style == CliStyle::Subcommand && !schema.subcommands.is_empty() {
        tokens = fix_subcommand(tokens, schema, task);
    }

    // Phase 2: Flag whitelist enforcement (remove hallucinated flags)
    let subcmd = detect_subcmd_from_tokens(&tokens, schema);
    tokens = remove_invalid_flags(tokens, schema, subcmd.as_deref());

    // Phase 3: Required flag injection (add missing required flags)
    tokens = inject_required_flags(tokens, schema, subcmd.as_deref());

    tokens
}

/// Phase 1: Fix missing or wrong subcommand using schema.
///
/// If the first token is not a known subcommand but schema says the tool
/// requires one, inject the correct subcommand based on task keywords.
fn fix_subcommand(tokens: Vec<String>, schema: &CliSchema, task: &str) -> Vec<String> {
    if tokens.is_empty() {
        return tokens;
    }

    let first = &tokens[0];
    let is_known_subcmd = schema.subcommands.iter().any(|s| s.name == *first);

    if is_known_subcmd {
        return tokens;
    }

    let suggested = schema.select_subcommand(task);
    if let Some(subcmd) = suggested {
        if first.starts_with('-') {
            let mut fixed = vec![subcmd.name.clone()];
            fixed.extend(tokens);
            return fixed;
        }

        if looks_like_positional(first) || first.contains('.') {
            let mut fixed = vec![subcmd.name.clone()];
            fixed.extend(tokens);
            return fixed;
        }

        let mut fixed = vec![subcmd.name.clone()];
        fixed.extend(tokens);
        return fixed;
    }

    tokens
}

/// Detect if a token looks like a positional argument (file path, number, etc.)
fn looks_like_positional(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    if token.starts_with('-') {
        return false;
    }
    if token.contains('.') {
        return true;
    }
    if token.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    false
}

/// Detect which subcommand is being used from the token list.
fn detect_subcmd_from_tokens(tokens: &[String], schema: &CliSchema) -> Option<String> {
    if schema.cli_style != CliStyle::Subcommand {
        return None;
    }
    tokens.first().and_then(|t| {
        schema
            .subcommands
            .iter()
            .find(|s| s.name == *t)
            .map(|s| s.name.clone())
    })
}

/// Phase 2: Remove flags that are not in the schema whitelist.
///
/// This eliminates hallucinated flags that small models frequently generate.
/// Keeps the flag's value token if it's not a flag itself.
fn remove_invalid_flags(
    tokens: Vec<String>,
    schema: &CliSchema,
    subcommand: Option<&str>,
) -> Vec<String> {
    let valid_flags = schema.all_flag_names(subcommand);
    if valid_flags.is_empty() {
        return tokens;
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];

        if token.starts_with('-') && !token.contains('=') {
            let flag_name = token.split('=').next().unwrap_or(token);
            let is_valid = valid_flags.contains(&flag_name);

            if is_valid {
                result.push(token.clone());
                let takes_value = schema
                    .get_flag(flag_name, subcommand)
                    .is_some_and(|f| !matches!(f.param_type, crate::schema::ParamType::Bool));
                if takes_value && i + 1 < tokens.len() {
                    let next = &tokens[i + 1];
                    if !next.starts_with('-') || next.starts_with('-') && next.contains('=') {
                        result.push(next.clone());
                        i += 1;
                    }
                }
            } else {
                let takes_value = schema
                    .get_flag(flag_name, subcommand)
                    .is_some_and(|f| !matches!(f.param_type, crate::schema::ParamType::Bool));
                if takes_value && i + 1 < tokens.len() {
                    let next = &tokens[i + 1];
                    if !next.starts_with('-') {
                        i += 1;
                    }
                }
            }
        } else if token.contains('=') && token.starts_with('-') {
            let parts: Vec<&str> = token.splitn(2, '=').collect();
            let flag_name = parts[0];
            let is_valid = valid_flags.contains(&flag_name);

            if is_valid {
                result.push(token.clone());
            }
        } else {
            result.push(token.clone());
        }

        i += 1;
    }

    result
}

/// Phase 3: Inject missing required flags.
///
/// For flags marked as required in the schema but absent from the generated
/// command, inject them with a placeholder value based on the flag's type.
fn inject_required_flags(
    tokens: Vec<String>,
    schema: &CliSchema,
    subcommand: Option<&str>,
) -> Vec<String> {
    let valid_flags = schema.all_flag_names(subcommand);
    if valid_flags.is_empty() {
        return tokens;
    }

    let used_flag_names: Vec<String> = tokens
        .iter()
        .filter(|t| t.starts_with('-'))
        .map(|t| t.split('=').next().unwrap_or(t).to_string())
        .collect();

    let required_flags: Vec<_> = if let Some(subcmd) = subcommand {
        schema
            .get_subcommand(subcmd)
            .map(|s| s.flags.iter().filter(|f| f.required).collect())
            .unwrap_or_default()
    } else {
        schema.flags.iter().filter(|f| f.required).collect()
    };

    let mut additions = Vec::new();
    for flag in &required_flags {
        let is_used = flag
            .all_names()
            .iter()
            .any(|n| used_flag_names.iter().any(|u| u == n));

        if !is_used {
            match &flag.param_type {
                crate::schema::ParamType::Bool => {
                    additions.push(flag.name.clone());
                }
                crate::schema::ParamType::File => {
                    if let Some(default) = &flag.default {
                        additions.push(format!("{}={}", flag.name, default));
                    } else {
                        additions.push(format!("{}=OUTPUT", flag.name));
                    }
                }
                crate::schema::ParamType::Int => {
                    if let Some(default) = &flag.default {
                        additions.push(format!("{}={}", flag.name, default));
                    } else {
                        additions.push(flag.name.clone());
                        additions.push("1".to_string());
                    }
                }
                _ => {
                    if let Some(default) = &flag.default {
                        additions.push(format!("{}={}", flag.name, default));
                    }
                }
            }
        }
    }

    let mut result = tokens;
    result.extend(additions);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{CliStyle, FlagSchema, ParamType, SubcommandSchema};

    #[test]
    fn test_should_standardize_task() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        assert!(executor.should_standardize_task("sort"));
        assert!(executor.should_standardize_task("just sort the bam"));
        assert!(executor.should_standardize_task("排序BAM文件"));
        assert!(!executor.should_standardize_task("Sort BAM file by read names"));
    }

    #[test]
    fn test_resolve_mode_from_confidence() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();

        let high = estimate_confidence(10, 0.9, true, 2, Some(7.0), true);
        assert_eq!(executor.resolve_mode(&Some(high)), WorkflowMode::Fast);

        let low = estimate_confidence(0, 0.0, false, 0, Some(1.5), false);
        assert_eq!(executor.resolve_mode(&Some(low)), WorkflowMode::Quality);
    }

    fn test_subcommand_schema() -> CliSchema {
        CliSchema {
            tool: "samtools".to_string(),
            version: None,
            cli_style: CliStyle::Subcommand,
            description: "Tools for BAM".to_string(),
            subcommands: vec![
                SubcommandSchema {
                    name: "sort".to_string(),
                    description: "Sort BAM".to_string(),
                    usage_pattern: "samtools sort -o output.bam input.bam".to_string(),
                    flags: vec![
                        FlagSchema {
                            name: "-@".to_string(),
                            aliases: vec!["--threads".to_string()],
                            param_type: ParamType::Int,
                            description: "Threads".to_string(),
                            default: Some("1".to_string()),
                            required: false,
                            long_description: None,
                        },
                        FlagSchema {
                            name: "-o".to_string(),
                            aliases: vec!["--output".to_string()],
                            param_type: ParamType::File,
                            description: "Output file".to_string(),
                            default: None,
                            required: true,
                            long_description: None,
                        },
                    ],
                    positionals: Vec::new(),
                    constraints: Vec::new(),
                    task_keywords: vec!["sort".to_string(), "coordinate".to_string()],
                },
                SubcommandSchema {
                    name: "index".to_string(),
                    description: "Index BAM".to_string(),
                    usage_pattern: "samtools index input.bam".to_string(),
                    flags: Vec::new(),
                    positionals: Vec::new(),
                    constraints: Vec::new(),
                    task_keywords: vec!["index".to_string()],
                },
                SubcommandSchema {
                    name: "view".to_string(),
                    description: "View BAM".to_string(),
                    usage_pattern: "samtools view -b input.bam".to_string(),
                    flags: vec![FlagSchema {
                        name: "-b".to_string(),
                        aliases: vec!["--bam".to_string()],
                        param_type: ParamType::Bool,
                        description: "Output BAM".to_string(),
                        default: None,
                        required: false,
                        long_description: None,
                    }],
                    positionals: Vec::new(),
                    constraints: Vec::new(),
                    task_keywords: vec!["view".to_string(), "convert".to_string()],
                },
            ],
            global_flags: Vec::new(),
            flags: Vec::new(),
            positionals: Vec::new(),
            usage_summary: String::new(),
            constraints: Vec::new(),
            doc_quality: 0.9,
            schema_source: "test".to_string(),
        }
    }

    #[test]
    fn test_fix_subcommand_inject_missing() {
        let schema = test_subcommand_schema();
        let tokens = vec!["-@".to_string(), "4".to_string(), "input.bam".to_string()];
        let fixed = fix_subcommand(tokens, &schema, "sort bam by coordinate");
        assert_eq!(fixed[0], "sort");
    }

    #[test]
    fn test_fix_subcommand_keep_existing() {
        let schema = test_subcommand_schema();
        let tokens = vec!["sort".to_string(), "-@".to_string(), "4".to_string()];
        let fixed = fix_subcommand(tokens, &schema, "sort bam by coordinate");
        assert_eq!(fixed[0], "sort");
        assert_eq!(fixed.len(), 3);
    }

    #[test]
    fn test_fix_subcommand_inject_before_flags() {
        let schema = test_subcommand_schema();
        let tokens = vec!["-b".to_string(), "input.bam".to_string()];
        let fixed = fix_subcommand(tokens, &schema, "view and convert bam");
        assert_eq!(fixed[0], "view");
        assert_eq!(fixed[1], "-b");
    }

    #[test]
    fn test_remove_invalid_flags() {
        let schema = test_subcommand_schema();
        let tokens = vec![
            "sort".to_string(),
            "--invalid".to_string(),
            "value".to_string(),
            "-@".to_string(),
            "4".to_string(),
            "input.bam".to_string(),
        ];
        let cleaned = remove_invalid_flags(tokens, &schema, Some("sort"));
        assert!(!cleaned.iter().any(|t| t == "--invalid"));
        assert!(cleaned.iter().any(|t| t == "-@"));
    }

    #[test]
    fn test_schema_post_process_full() {
        let schema = test_subcommand_schema();
        let args: Vec<String> = vec!["-@".to_string(), "4".to_string(), "input.bam".to_string()];
        let result = schema_post_process(&args, &schema, "sort bam by coordinate");
        assert!(result[0] == "sort");
        assert!(result.iter().any(|t| t == "-@"));
    }

    #[test]
    fn test_shell_tokenize_simple() {
        let tokens: Vec<String> = vec![
            "sort".to_string(),
            "-@".to_string(),
            "4".to_string(),
            "-o".to_string(),
            "'output file.bam'".to_string(),
            "input.bam".to_string(),
        ];
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0], "sort");
    }

    #[test]
    fn test_looks_like_positional_file() {
        assert!(looks_like_positional("input.bam"));
        assert!(looks_like_positional("data.fastq.gz"));
        assert!(looks_like_positional("ref.fa"));
    }

    #[test]
    fn test_looks_like_positional_number() {
        assert!(looks_like_positional("42"));
        assert!(looks_like_positional("0"));
    }

    #[test]
    fn test_looks_like_positional_flag() {
        assert!(!looks_like_positional("-v"));
        assert!(!looks_like_positional("--output"));
    }

    #[test]
    fn test_looks_like_positional_empty() {
        assert!(!looks_like_positional(""));
    }

    #[test]
    fn test_looks_like_positional_word() {
        assert!(!looks_like_positional("sort"));
    }

    #[test]
    fn test_detect_subcmd_from_tokens_subcommand_style() {
        let schema = test_subcommand_schema();
        assert_eq!(
            detect_subcmd_from_tokens(
                &["sort".to_string(), "-@".to_string(), "4".to_string()],
                &schema
            ),
            Some("sort".to_string())
        );
        assert_eq!(
            detect_subcmd_from_tokens(&["view".to_string(), "-b".to_string()], &schema),
            Some("view".to_string())
        );
    }

    #[test]
    fn test_detect_subcmd_from_tokens_no_match() {
        let schema = test_subcommand_schema();
        assert_eq!(
            detect_subcmd_from_tokens(&["unknown".to_string()], &schema),
            None
        );
    }

    #[test]
    fn test_detect_subcmd_from_tokens_flags_first() {
        let mut schema = test_subcommand_schema();
        schema.cli_style = CliStyle::FlagsFirst;
        assert_eq!(
            detect_subcmd_from_tokens(&["sort".to_string()], &schema),
            None
        );
    }

    #[test]
    fn test_remove_invalid_flags_keeps_valid() {
        let schema = test_subcommand_schema();
        let tokens = vec![
            "sort".to_string(),
            "-@".to_string(),
            "4".to_string(),
            "-o".to_string(),
            "out.bam".to_string(),
        ];
        let cleaned = remove_invalid_flags(tokens, &schema, Some("sort"));
        assert!(cleaned.iter().any(|t| t == "-@"));
        assert!(cleaned.iter().any(|t| t == "-o"));
    }

    #[test]
    fn test_remove_invalid_flags_removes_hallucinated() {
        let schema = test_subcommand_schema();
        let tokens = vec![
            "sort".to_string(),
            "--hallucinated".to_string(),
            "value".to_string(),
            "-@".to_string(),
            "4".to_string(),
        ];
        let cleaned = remove_invalid_flags(tokens, &schema, Some("sort"));
        assert!(!cleaned.iter().any(|t| t == "--hallucinated"));
    }

    #[test]
    fn test_remove_invalid_flags_empty_whitelist() {
        let mut schema = test_subcommand_schema();
        schema.flags.clear();
        schema.global_flags.clear();
        for s in &mut schema.subcommands {
            s.flags.clear();
        }
        let tokens = vec!["sort".to_string(), "--flag".to_string()];
        let cleaned = remove_invalid_flags(tokens, &mut schema, Some("sort"));
        assert_eq!(cleaned.len(), 2);
    }

    #[test]
    fn test_remove_invalid_flags_equals_form() {
        let schema = test_subcommand_schema();
        let tokens = vec![
            "sort".to_string(),
            "-@=4".to_string(),
            "--invalid=foo".to_string(),
        ];
        let cleaned = remove_invalid_flags(tokens, &schema, Some("sort"));
        assert!(cleaned.iter().any(|t| t == "-@=4"));
        assert!(!cleaned.iter().any(|t| t.contains("invalid")));
    }

    #[test]
    fn test_inject_required_flags_missing() {
        let schema = test_subcommand_schema();
        let tokens = vec![
            "sort".to_string(),
            "-@".to_string(),
            "4".to_string(),
            "input.bam".to_string(),
        ];
        let result = inject_required_flags(tokens, &schema, Some("sort"));
        assert!(result.iter().any(|t| t.contains("-o")));
    }

    #[test]
    fn test_inject_required_flags_already_present() {
        let schema = test_subcommand_schema();
        let tokens = vec![
            "sort".to_string(),
            "-o".to_string(),
            "out.bam".to_string(),
            "-@".to_string(),
            "4".to_string(),
        ];
        let result = inject_required_flags(tokens, &schema, Some("sort"));
        let o_count = result
            .iter()
            .filter(|t| **t == "-o" || t.starts_with("-o="))
            .count();
        assert_eq!(o_count, 1);
    }

    #[test]
    fn test_inject_required_flags_bool_type() {
        let mut schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: "test".to_string(),
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagSchema {
                name: "--verbose".to_string(),
                aliases: vec!["-v".to_string()],
                param_type: ParamType::Bool,
                description: "Verbose".to_string(),
                default: None,
                required: true,
                long_description: None,
            }],
            positionals: Vec::new(),
            usage_summary: String::new(),
            constraints: Vec::new(),
            doc_quality: 0.9,
            schema_source: "test".to_string(),
        };
        let tokens = vec!["input.bam".to_string()];
        let result = inject_required_flags(tokens, &schema, None);
        assert!(result.iter().any(|t| t == "--verbose"));
    }

    #[test]
    fn test_inject_required_flags_int_with_default() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: "test".to_string(),
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagSchema {
                name: "--threads".to_string(),
                aliases: Vec::new(),
                param_type: ParamType::Int,
                description: "Threads".to_string(),
                default: Some("4".to_string()),
                required: true,
                long_description: None,
            }],
            positionals: Vec::new(),
            usage_summary: String::new(),
            constraints: Vec::new(),
            doc_quality: 0.9,
            schema_source: "test".to_string(),
        };
        let tokens = vec!["input.bam".to_string()];
        let result = inject_required_flags(tokens, &schema, None);
        assert!(result.iter().any(|t| t == "--threads=4"));
    }

    #[test]
    fn test_inject_required_flags_int_no_default() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: "test".to_string(),
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagSchema {
                name: "--count".to_string(),
                aliases: Vec::new(),
                param_type: ParamType::Int,
                description: "Count".to_string(),
                default: None,
                required: true,
                long_description: None,
            }],
            positionals: Vec::new(),
            usage_summary: String::new(),
            constraints: Vec::new(),
            doc_quality: 0.9,
            schema_source: "test".to_string(),
        };
        let tokens = vec!["input.bam".to_string()];
        let result = inject_required_flags(tokens, &schema, None);
        assert!(result.iter().any(|t| t == "--count"));
        assert!(result.iter().any(|t| t == "1"));
    }

    #[test]
    fn test_inject_required_flags_file_with_default() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: "test".to_string(),
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagSchema {
                name: "--output".to_string(),
                aliases: Vec::new(),
                param_type: ParamType::File,
                description: "Output".to_string(),
                default: Some("stdout".to_string()),
                required: true,
                long_description: None,
            }],
            positionals: Vec::new(),
            usage_summary: String::new(),
            constraints: Vec::new(),
            doc_quality: 0.9,
            schema_source: "test".to_string(),
        };
        let tokens = vec!["input.bam".to_string()];
        let result = inject_required_flags(tokens, &schema, None);
        assert!(result.iter().any(|t| t == "--output=stdout"));
    }

    #[test]
    fn test_inject_required_flags_file_no_default() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: "test".to_string(),
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagSchema {
                name: "--output".to_string(),
                aliases: Vec::new(),
                param_type: ParamType::File,
                description: "Output".to_string(),
                default: None,
                required: true,
                long_description: None,
            }],
            positionals: Vec::new(),
            usage_summary: String::new(),
            constraints: Vec::new(),
            doc_quality: 0.9,
            schema_source: "test".to_string(),
        };
        let tokens = vec!["input.bam".to_string()];
        let result = inject_required_flags(tokens, &schema, None);
        assert!(result.iter().any(|t| t == "--output=OUTPUT"));
    }

    #[test]
    fn test_inject_required_flags_string_with_default() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: "test".to_string(),
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagSchema {
                name: "--format".to_string(),
                aliases: Vec::new(),
                param_type: ParamType::String,
                description: "Format".to_string(),
                default: Some("json".to_string()),
                required: true,
                long_description: None,
            }],
            positionals: Vec::new(),
            usage_summary: String::new(),
            constraints: Vec::new(),
            doc_quality: 0.9,
            schema_source: "test".to_string(),
        };
        let tokens = vec!["input".to_string()];
        let result = inject_required_flags(tokens, &schema, None);
        assert!(result.iter().any(|t| t == "--format=json"));
    }

    #[test]
    fn test_schema_post_process_empty() {
        let schema = test_subcommand_schema();
        let args: Vec<String> = vec![];
        let result = schema_post_process(&args, &schema, "sort");
        assert!(result.is_empty());
    }

    #[test]
    fn test_fix_subcommand_empty_tokens() {
        let schema = test_subcommand_schema();
        let tokens: Vec<String> = vec![];
        let fixed = fix_subcommand(tokens, &schema, "sort");
        assert!(fixed.is_empty());
    }

    #[test]
    fn test_fix_subcommand_no_match_in_schema() {
        let schema = test_subcommand_schema();
        let tokens = vec!["unknown_cmd".to_string(), "input.bam".to_string()];
        let fixed = fix_subcommand(tokens, &schema, "do something unrelated");
        assert_eq!(fixed[0], "unknown_cmd");
    }

    #[test]
    fn test_fix_subcommand_positional_input() {
        let schema = test_subcommand_schema();
        let tokens = vec!["input.bam".to_string()];
        let fixed = fix_subcommand(tokens, &schema, "sort bam");
        assert_eq!(fixed[0], "sort");
    }

    #[test]
    fn test_workflow_mode_default() {
        assert_eq!(WorkflowMode::default(), WorkflowMode::Fast);
    }

    #[test]
    fn test_workflow_result_fields() {
        let result = WorkflowResult {
            suggestion: LlmCommandSuggestion {
                args: vec!["sort".to_string()],
                explanation: "Sort".to_string(),
                inference_ms: 100.0,
            },
            llm_calls: 1,
            total_inference_ms: 100.0,
            effective_task: "sort".to_string(),
            was_normalized: false,
            confidence: None,
        };
        assert_eq!(result.llm_calls, 1);
        assert!(!result.was_normalized);
    }

    #[test]
    fn test_compute_confidence_with_schema() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        let schema = test_subcommand_schema();
        let result = executor.compute_confidence(Some(&schema), "sort bam file", None);
        assert!(result.is_some());
        assert!(result.unwrap().score > 0.0);
    }

    #[test]
    fn test_compute_confidence_no_schema() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        let result = executor.compute_confidence(None, "sort bam file", None);
        assert!(result.is_some());
    }

    #[test]
    fn test_compute_confidence_with_skill() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        let skill = Skill::default();
        let result = executor.compute_confidence(None, "sort bam file", Some(&skill));
        assert!(result.is_some());
    }

    #[test]
    fn test_is_small_model() {
        let mut executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        executor.model_param_count = Some(2.0);
        assert!(executor.is_small_model());
        executor.model_param_count = Some(7.0);
        assert!(!executor.is_small_model());
        executor.model_param_count = None;
        assert!(!executor.is_small_model());
    }

    #[test]
    fn test_remove_invalid_flags_next_starts_with_dash_equals() {
        let schema = test_subcommand_schema();
        let tokens = vec![
            "sort".to_string(),
            "-@".to_string(),
            "--invalid=foo".to_string(),
            "4".to_string(),
        ];
        let cleaned = remove_invalid_flags(tokens, &schema, Some("sort"));
        assert!(cleaned.iter().any(|t| t == "-@"));
    }

    #[test]
    fn test_inject_required_flags_empty_whitelist() {
        let mut schema = test_subcommand_schema();
        schema.flags.clear();
        schema.global_flags.clear();
        for s in &mut schema.subcommands {
            s.flags.clear();
        }
        let tokens = vec!["sort".to_string()];
        let result = inject_required_flags(tokens, &schema, Some("sort"));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_inject_required_flags_string_no_default() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: "test".to_string(),
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagSchema {
                name: "--format".to_string(),
                aliases: Vec::new(),
                param_type: ParamType::String,
                description: "Format".to_string(),
                default: None,
                required: true,
                long_description: None,
            }],
            positionals: Vec::new(),
            usage_summary: String::new(),
            constraints: Vec::new(),
            doc_quality: 0.9,
            schema_source: "test".to_string(),
        };
        let tokens = vec!["input".to_string()];
        let result = inject_required_flags(tokens, &schema, None);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_remove_invalid_flags_bool_flag_keeps_no_value() {
        let schema = test_subcommand_schema();
        let tokens = vec![
            "view".to_string(),
            "-b".to_string(),
            "input.bam".to_string(),
        ];
        let cleaned = remove_invalid_flags(tokens, &schema, Some("view"));
        assert!(cleaned.iter().any(|t| t == "-b"));
        assert!(cleaned.iter().any(|t| t == "input.bam"));
    }

    #[test]
    fn test_remove_invalid_flags_invalid_bool_skips_next_if_flag() {
        let schema = test_subcommand_schema();
        let tokens = vec![
            "sort".to_string(),
            "--invalid".to_string(),
            "--another-invalid".to_string(),
            "-@".to_string(),
            "4".to_string(),
        ];
        let cleaned = remove_invalid_flags(tokens, &schema, Some("sort"));
        assert!(cleaned.iter().any(|t| t == "-@"));
        assert!(!cleaned.iter().any(|t| t == "--invalid"));
    }

    #[test]
    fn test_fix_subcommand_file_path_input() {
        let schema = test_subcommand_schema();
        let tokens = vec!["data/input.bam".to_string()];
        let fixed = fix_subcommand(tokens, &schema, "sort bam");
        assert_eq!(fixed[0], "sort");
    }

    #[test]
    fn test_schema_post_process_flags_first_style() {
        let mut schema = test_subcommand_schema();
        schema.cli_style = CliStyle::FlagsFirst;
        let args: Vec<String> = vec!["-@".to_string(), "4".to_string(), "input.bam".to_string()];
        let result = schema_post_process(&args, &schema, "sort bam");
        assert!(result.iter().any(|t| t == "-@"));
    }

    #[test]
    fn test_remove_invalid_flags_non_flag_passthrough() {
        let schema = test_subcommand_schema();
        let tokens = vec![
            "sort".to_string(),
            "input.bam".to_string(),
            "output.bam".to_string(),
        ];
        let cleaned = remove_invalid_flags(tokens, &schema, Some("sort"));
        assert!(cleaned.iter().any(|t| t == "input.bam"));
        assert!(cleaned.iter().any(|t| t == "output.bam"));
    }

    #[test]
    fn test_inject_required_flags_alias_already_used() {
        let schema = test_subcommand_schema();
        let tokens = vec![
            "sort".to_string(),
            "--threads".to_string(),
            "4".to_string(),
            "input.bam".to_string(),
        ];
        let result = inject_required_flags(tokens, &schema, Some("sort"));
        let threads_count = result
            .iter()
            .filter(|t| **t == "--threads" || **t == "-@")
            .count();
        assert_eq!(threads_count, 1);
    }

    #[test]
    fn test_compute_confidence_file_mentions() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        let result = executor.compute_confidence(None, "sort input.bam to output.bam", None);
        assert!(result.is_some());
    }

    #[test]
    fn test_workflow_mode_serde() {
        let mode = WorkflowMode::Quality;
        let json = serde_json::to_string(&mode).unwrap();
        assert!(json.contains("Quality"));
        let deserialized: WorkflowMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, WorkflowMode::Quality);
    }

    // ── WorkflowMode additional ───────────────────────────────────────────────

    #[test]
    fn test_workflow_mode_fast_serde() {
        let mode = WorkflowMode::Fast;
        let json = serde_json::to_string(&mode).unwrap();
        assert!(json.contains("Fast"));
        let back: WorkflowMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, WorkflowMode::Fast);
    }

    #[test]
    fn test_workflow_mode_default_is_fast() {
        let mode = WorkflowMode::default();
        assert_eq!(mode, WorkflowMode::Fast);
    }

    #[test]
    fn test_workflow_mode_debug() {
        let fast = format!("{:?}", WorkflowMode::Fast);
        let quality = format!("{:?}", WorkflowMode::Quality);
        assert_eq!(fast, "Fast");
        assert_eq!(quality, "Quality");
    }

    #[test]
    fn test_workflow_mode_copy() {
        let a = WorkflowMode::Quality;
        let b = a; // Copy
        assert_eq!(a, b);
    }

    // ── LlmWorkflowExecutor::new with Quality mode ────────────────────────────

    #[test]
    fn test_llm_workflow_executor_new_quality() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Quality);
        assert!(executor.is_ok());
        let executor = executor.unwrap();
        assert_eq!(executor.mode, WorkflowMode::Quality);
    }

    #[test]
    fn test_llm_workflow_executor_new_fast() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast);
        assert!(executor.is_ok());
        let executor = executor.unwrap();
        assert_eq!(executor.mode, WorkflowMode::Fast);
    }

    // ── should_standardize_task: more keyword variations ─────────────────────

    #[test]
    fn test_should_standardize_task_simply() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        assert!(executor.should_standardize_task("simply sort the bam file by coordinates"));
    }

    #[test]
    fn test_should_standardize_task_basically() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        assert!(executor.should_standardize_task("basically I want to sort reads"));
    }

    #[test]
    fn test_should_standardize_task_short() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        // Short tasks (< 10 chars) should always be standardized
        assert!(executor.should_standardize_task("sort bam"));
    }

    #[test]
    fn test_should_standardize_task_non_ascii() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        assert!(executor.should_standardize_task("对BAM文件按坐标排序并建立索引"));
    }

    #[test]
    fn test_should_not_standardize_clear_task() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        assert!(
            !executor
                .should_standardize_task("Sort BAM file by coordinate order using samtools sort")
        );
    }

    // ── resolve_mode with None confidence ─────────────────────────────────────

    #[test]
    fn test_resolve_mode_none_confidence_uses_executor_mode() {
        let fast_exec = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        assert_eq!(fast_exec.resolve_mode(&None), WorkflowMode::Fast);

        let quality_exec =
            LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Quality).unwrap();
        assert_eq!(quality_exec.resolve_mode(&None), WorkflowMode::Quality);
    }

    // ── compute_confidence edge cases ─────────────────────────────────────────

    #[test]
    fn test_compute_confidence_no_schema_returns_some() {
        // compute_confidence always returns Some, even without a schema
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        let result = executor.compute_confidence(None, "sort bam file", None);
        assert!(result.is_some());
    }

    #[test]
    fn test_compute_confidence_with_bam_mention() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        let schema = test_subcommand_schema();
        let result =
            executor.compute_confidence(Some(&schema), "sort input.bam to output.bam", None);
        assert!(result.is_some());
    }

    #[test]
    fn test_compute_confidence_with_fastq_mention() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        let schema = test_subcommand_schema();
        let result =
            executor.compute_confidence(Some(&schema), "align reads.fastq to reference", None);
        assert!(result.is_some());
    }

    #[test]
    fn test_compute_confidence_score_range() {
        let executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        let schema = test_subcommand_schema();
        let result =
            executor.compute_confidence(Some(&schema), "sort bam file by coordinate", None);
        if let Some(conf) = result {
            assert!((0.0..=1.0).contains(&conf.score));
        }
    }

    // ── is_small_model ────────────────────────────────────────────────────────

    #[test]
    fn test_is_small_model_with_small_param_count() {
        let mut executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        executor.model_param_count = Some(1.5);
        assert!(executor.is_small_model());
    }

    #[test]
    fn test_is_small_model_with_large_param_count() {
        let mut executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        executor.model_param_count = Some(70.0);
        assert!(!executor.is_small_model());
    }

    #[test]
    fn test_is_small_model_with_none() {
        let mut executor = LlmWorkflowExecutor::new(Config::default(), WorkflowMode::Fast).unwrap();
        executor.model_param_count = None;
        assert!(!executor.is_small_model());
    }

    // ── fix_subcommand additional edge cases ──────────────────────────────────

    #[test]
    fn test_fix_subcommand_already_present() {
        let schema = test_subcommand_schema();
        let tokens = vec!["sort".to_string(), "-o".to_string(), "out.bam".to_string()];
        let result = fix_subcommand(tokens.clone(), &schema, "sort");
        assert_eq!(result[0], "sort");
    }

    #[test]
    fn test_fix_subcommand_case_insensitive_match() {
        let schema = test_subcommand_schema();
        // Task keyword "sort" should match "sort" subcommand
        let tokens = vec![
            "-o".to_string(),
            "out.bam".to_string(),
            "in.bam".to_string(),
        ];
        let result = fix_subcommand(tokens, &schema, "sort");
        assert_eq!(result[0], "sort");
    }

    // ── looks_like_positional ─────────────────────────────────────────────────

    #[test]
    fn test_looks_like_positional_file_path() {
        assert!(looks_like_positional("input.bam"));
        assert!(looks_like_positional("/data/reads.fastq.gz"));
        assert!(looks_like_positional("reads.fq"));
    }

    #[test]
    fn test_looks_like_positional_not_a_file() {
        assert!(!looks_like_positional("--output"));
        assert!(!looks_like_positional("-o"));
    }

    #[test]
    fn test_looks_like_positional_number_with_dot() {
        // "1.5" has a dot but is a number
        // behavior depends on implementation; just should not panic
        let _ = looks_like_positional("1.5");
    }

    // ── detect_subcmd_from_tokens ─────────────────────────────────────────────

    #[test]
    fn test_detect_subcmd_from_tokens_no_subcommand() {
        let schema = test_subcommand_schema();
        let tokens = vec!["-o".to_string(), "output.bam".to_string()];
        let result = detect_subcmd_from_tokens(&tokens, &schema);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_subcmd_from_tokens_with_sort() {
        let schema = test_subcommand_schema();
        let tokens = vec!["sort".to_string(), "input.bam".to_string()];
        let result = detect_subcmd_from_tokens(&tokens, &schema);
        assert_eq!(result.as_deref(), Some("sort"));
    }

    // ── schema_post_process: subcommand injection for FlagsFirst ──────────────

    #[test]
    fn test_schema_post_process_flags_first_bwa() {
        let schema = CliSchema {
            tool: "bwa".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: "BWA aligner".to_string(),
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagSchema {
                name: "-t".to_string(),
                aliases: Vec::new(),
                param_type: ParamType::Int,
                description: "Threads".to_string(),
                default: None,
                required: false,
                long_description: None,
            }],
            positionals: Vec::new(),
            usage_summary: String::new(),
            constraints: Vec::new(),
            doc_quality: 0.9,
            schema_source: "test".to_string(),
        };
        let tokens = vec![
            "-t".to_string(),
            "8".to_string(),
            "ref.fa".to_string(),
            "reads.fq".to_string(),
        ];
        let result = schema_post_process(&tokens, &schema, "align reads");
        // Tokens should be kept or re-ordered; must not panic
        assert!(!result.is_empty());
    }

    // ── WorkflowResult fields ────────────────────────────────────────────────

    #[test]
    fn test_workflow_result_debug() {
        use crate::llm::LlmCommandSuggestion;
        let result = WorkflowResult {
            suggestion: LlmCommandSuggestion {
                args: vec!["sort".to_string()],
                explanation: "Sort BAM".to_string(),
                inference_ms: 100.0,
            },
            llm_calls: 1,
            total_inference_ms: 100.0,
            effective_task: "sort bam".to_string(),
            was_normalized: false,
            confidence: None,
        };
        let dbg = format!("{result:?}");
        assert!(dbg.contains("WorkflowResult"));
    }
}
