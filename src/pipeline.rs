#![allow(dead_code)]

use crate::command_assembler::CommandAssembler;
use crate::config::Config;
use crate::doc_explorer::DocExplorer;
use crate::error::Result;
use crate::intent_mapper::{IntentMapper, LlmCommandFill};
use crate::llm::LlmClient;
use crate::tool_doc::ToolDoc;
use crate::tool_resolver::{ToolRecord, resolve_tool};
use crate::validator::{ValidationResult, Validator};
use crate::workflow_graph::WorkflowScenario;

pub struct Pipeline {
    config: Config,
    llm: LlmClient,
    scenario: WorkflowScenario,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PipelineResult {
    pub command: String,
    pub explanation: String,
    pub validation: ValidationResult,
    pub tool_doc: ToolDoc,
    pub tool_record: ToolRecord,
    pub effective_task: String,
}

impl Pipeline {
    pub fn new(config: Config, scenario: WorkflowScenario) -> Self {
        let llm = LlmClient::new(config.clone());
        Self {
            config,
            llm,
            scenario,
        }
    }

    pub async fn execute(&self, tool: &str, task: &str) -> Result<PipelineResult> {
        let record = self.stage1_resolve(tool)?;
        let doc = self.stage2_explore(&record, task).await?;
        let (subcommand, fill) = self.stage3_map(&record, &doc, task);
        let filled = self
            .stage3b_llm_fill(&record, &doc, task, subcommand.as_deref(), &fill)
            .await?;
        let command = self.stage4_assemble(&record, &filled, &doc);
        let validation = self.stage5_validate(&record, &filled, &doc);
        let explanation = self.build_explanation(&doc, &filled, &validation);

        Ok(PipelineResult {
            command,
            explanation,
            validation,
            tool_doc: doc,
            tool_record: record,
            effective_task: task.to_string(),
        })
    }

    fn stage1_resolve(&self, tool: &str) -> Result<ToolRecord> {
        resolve_tool(tool).map_err(|e| crate::error::OxoError::ToolNotFound(e.to_string()))
    }

    async fn stage2_explore(&self, record: &ToolRecord, task: &str) -> Result<ToolDoc> {
        match self.scenario {
            WorkflowScenario::Bare => Ok(ToolDoc {
                record: record.clone(),
                cli_style: crate::schema::CliStyle::FlagsFirst,
                description: String::new(),
                schema_source: "bare".to_string(),
                doc_quality: 0.0,
                subcommands: Vec::new(),
                global_flags: Vec::new(),
                flags: Vec::new(),
                positionals: Vec::new(),
                usage_patterns: Vec::new(),
                constraints: Vec::new(),
                examples: Vec::new(),
                concepts: Vec::new(),
                pitfalls: Vec::new(),
                raw_help: None,
                subcommand_helps: std::collections::HashMap::new(),
            }),
            WorkflowScenario::Doc | WorkflowScenario::Full => {
                let explorer = DocExplorer::new(self.config.clone());
                let result = explorer.explore(record, task).await.map_err(|e| {
                    crate::error::OxoError::DocFetchError(record.name.clone(), e.to_string())
                })?;
                Ok(result)
            }
        }
    }

    fn stage3_map(
        &self,
        record: &ToolRecord,
        doc: &ToolDoc,
        task: &str,
    ) -> (Option<String>, LlmCommandFill) {
        let mapper = IntentMapper::new();
        mapper.map_intent(record, doc, task)
    }

    async fn stage3b_llm_fill(
        &self,
        record: &ToolRecord,
        doc: &ToolDoc,
        task: &str,
        subcommand: Option<&str>,
        initial_fill: &LlmCommandFill,
    ) -> Result<LlmCommandFill> {
        match self.scenario {
            WorkflowScenario::Bare => {
                let prompt = format!(
                    "Generate command arguments for: {} {}\nOutput ONLY the arguments, nothing else.",
                    record.name, task
                );
                let response: String = self
                    .llm
                    .chat_completion(
                        "You are a CLI command generator. Output ONLY the command arguments.",
                        &prompt,
                        Some(256),
                        Some(0.0),
                    )
                    .await?;
                let args_str = response.trim();
                let mut fill = initial_fill.clone();
                parse_args_into_fill(args_str, &mut fill);
                Ok(fill)
            }
            WorkflowScenario::Doc | WorkflowScenario::Full => {
                let mapper = IntentMapper::new();
                let prompt = mapper.build_llm_prompt(record, doc, task, subcommand);

                let response: String = self
                    .llm
                    .chat_completion(
                        "You are a CLI command generator. Output ONLY valid JSON as specified.",
                        &prompt,
                        Some(512),
                        Some(0.0),
                    )
                    .await?;

                let mut fill = initial_fill.clone();

                let cleaned = response
                    .trim()
                    .trim_start_matches("```json")
                    .trim_start_matches("```")
                    .trim_end_matches("```")
                    .trim();

                match serde_json::from_str::<LlmCommandFill>(cleaned) {
                    Ok(parsed) => {
                        if parsed.subcommand.is_some() {
                            fill.subcommand = parsed.subcommand;
                        }
                        fill.flags.extend(parsed.flags);
                        fill.positionals = parsed.positionals;
                    }
                    Err(_) => {
                        parse_args_into_fill(&response, &mut fill);
                    }
                }

                Ok(fill)
            }
        }
    }

    fn stage4_assemble(&self, record: &ToolRecord, fill: &LlmCommandFill, doc: &ToolDoc) -> String {
        let assembler = CommandAssembler::new();
        assembler.assemble(record, fill, doc)
    }

    fn stage5_validate(
        &self,
        record: &ToolRecord,
        fill: &LlmCommandFill,
        doc: &ToolDoc,
    ) -> ValidationResult {
        let validator = Validator::new();
        validator.validate(record, fill, doc)
    }

    fn build_explanation(
        &self,
        doc: &ToolDoc,
        fill: &LlmCommandFill,
        validation: &ValidationResult,
    ) -> String {
        let mut parts = Vec::new();

        if let Some(ref subcmd) = fill.subcommand
            && let Some(s) = doc.get_subcommand(subcmd)
        {
            parts.push(format!("Using subcommand '{}': {}", subcmd, s.description));
        }

        for (flag, value) in &fill.flags {
            if value.is_empty() {
                parts.push(format!("Flag {} enabled", flag));
            } else {
                parts.push(format!("Flag {} = {}", flag, value));
            }
        }

        if !fill.positionals.is_empty() {
            parts.push(format!(
                "Positional arguments: {}",
                fill.positionals.join(", ")
            ));
        }

        if !validation.warnings.is_empty() {
            parts.push(String::new());
            parts.push("Warnings:".to_string());
            for w in &validation.warnings {
                parts.push(format!("  - {}", w));
            }
        }

        if !doc.examples.is_empty() {
            parts.push(String::new());
            parts.push("Similar examples:".to_string());
            for ex in doc.examples.iter().take(3) {
                parts.push(format!("  {} {}", doc.record.name, ex.args));
            }
        }

        parts.join("\n")
    }
}

fn parse_args_into_fill(args_str: &str, fill: &mut LlmCommandFill) {
    let tokens: Vec<String> = args_str.split_whitespace().map(String::from).collect();

    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        if token.starts_with('-') {
            if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                fill.flags.insert(token.clone(), tokens[i + 1].clone());
                i += 2;
            } else {
                fill.flags.insert(token.clone(), String::new());
                i += 1;
            }
        } else {
            fill.positionals.push(token.clone());
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::{CliStyle, ParamType};
    use crate::tool_doc::{FlagCategory, FlagDoc, PositionalDoc, SubcommandDoc};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn make_record() -> ToolRecord {
        ToolRecord {
            name: "samtools".to_string(),
            resolved_path: PathBuf::from("/usr/bin/samtools"),
            interpreter: None,
            is_path_dependent: false,
            global_path: Some(PathBuf::from("/usr/bin/samtools")),
            version: None,
            companion_tools: Vec::new(),
        }
    }

    fn make_doc() -> ToolDoc {
        ToolDoc {
            record: make_record(),
            cli_style: CliStyle::Subcommand,
            description: "SAMtools".to_string(),
            schema_source: "help".to_string(),
            doc_quality: 0.8,
            subcommands: vec![SubcommandDoc {
                name: "sort".to_string(),
                description: "Sort alignments".to_string(),
                usage_pattern: "samtools sort [-@ INT] [-o FILE] INPUT".to_string(),
                flags: vec![FlagDoc {
                    name: "-@".to_string(),
                    aliases: vec!["--threads".to_string()],
                    param_type: ParamType::Int,
                    description: "Number of threads".to_string(),
                    default: Some("1".to_string()),
                    required: false,
                    category: FlagCategory::Performance,
                }],
                positionals: vec![PositionalDoc {
                    position: 0,
                    name: "INPUT".to_string(),
                    param_type: ParamType::File,
                    description: "Input BAM".to_string(),
                    required: true,
                    default: None,
                }],
                constraints: Vec::new(),
                task_keywords: vec!["sort".to_string(), "order".to_string()],
            }],
            global_flags: Vec::new(),
            flags: Vec::new(),
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: vec![crate::tool_doc::CommandExample {
                args: "sort -@ 4 -o out.bam in.bam".to_string(),
                explanation: "Sort BAM with 4 threads".to_string(),
                source: crate::tool_doc::ExampleSource::HelpText,
            }],
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_parse_args_into_fill_flags_with_values() {
        let mut fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        parse_args_into_fill("-@ 4 -o out.bam", &mut fill);
        assert_eq!(fill.flags.get("-@"), Some(&"4".to_string()));
        assert_eq!(fill.flags.get("-o"), Some(&"out.bam".to_string()));
        assert!(fill.positionals.is_empty());
    }

    #[test]
    fn test_parse_args_into_fill_bool_flags() {
        let mut fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        parse_args_into_fill("-v -f", &mut fill);
        assert_eq!(fill.flags.get("-v"), Some(&String::new()));
        assert_eq!(fill.flags.get("-f"), Some(&String::new()));
    }

    #[test]
    fn test_parse_args_into_fill_positionals() {
        let mut fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        parse_args_into_fill("input.bam output.bam", &mut fill);
        assert!(fill.flags.is_empty());
        assert_eq!(fill.positionals, vec!["input.bam", "output.bam"]);
    }

    #[test]
    fn test_parse_args_into_fill_mixed() {
        let mut fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        parse_args_into_fill("-@ 4 input.bam -o out.bam", &mut fill);
        assert_eq!(fill.flags.get("-@"), Some(&"4".to_string()));
        assert_eq!(fill.flags.get("-o"), Some(&"out.bam".to_string()));
        assert_eq!(fill.positionals, vec!["input.bam"]);
    }

    #[test]
    fn test_parse_args_into_fill_empty() {
        let mut fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        parse_args_into_fill("", &mut fill);
        assert!(fill.flags.is_empty());
        assert!(fill.positionals.is_empty());
    }

    #[test]
    fn test_parse_args_into_fill_preserves_existing() {
        let mut fill = LlmCommandFill {
            subcommand: Some("sort".to_string()),
            flags: {
                let mut m = BTreeMap::new();
                m.insert("--existing".to_string(), "value".to_string());
                m
            },
            positionals: vec!["old.bam".to_string()],
        };
        parse_args_into_fill("-@ 8 new.bam", &mut fill);
        assert_eq!(fill.subcommand, Some("sort".to_string()));
        assert_eq!(fill.flags.get("--existing"), Some(&"value".to_string()));
        assert_eq!(fill.flags.get("-@"), Some(&"8".to_string()));
        assert_eq!(fill.positionals, vec!["old.bam", "new.bam"]);
    }

    #[test]
    fn test_parse_args_into_fill_long_flags() {
        let mut fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        parse_args_into_fill("--threads 4 --output out.bam", &mut fill);
        assert_eq!(fill.flags.get("--threads"), Some(&"4".to_string()));
        assert_eq!(fill.flags.get("--output"), Some(&"out.bam".to_string()));
    }

    #[test]
    fn test_parse_args_into_fill_equals_flag() {
        let mut fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        parse_args_into_fill("--output=out.bam", &mut fill);
        assert_eq!(fill.flags.get("--output=out.bam"), Some(&String::new()));
    }

    #[test]
    fn test_build_explanation_with_subcommand() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let doc = make_doc();
        let fill = LlmCommandFill {
            subcommand: Some("sort".to_string()),
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-@".to_string(), "4".to_string());
                m
            },
            positionals: vec!["input.bam".to_string()],
        };
        let validation = ValidationResult::valid();

        let explanation = pipeline.build_explanation(&doc, &fill, &validation);
        assert!(explanation.contains("sort"));
        assert!(explanation.contains("Sort alignments"));
        assert!(explanation.contains("-@"));
        assert!(explanation.contains("input.bam"));
    }

    #[test]
    fn test_build_explanation_with_warnings() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let doc = make_doc();
        let fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        let validation = ValidationResult::valid().with_warnings(vec!["Test warning".to_string()]);

        let explanation = pipeline.build_explanation(&doc, &fill, &validation);
        assert!(explanation.contains("Warnings"));
        assert!(explanation.contains("Test warning"));
    }

    #[test]
    fn test_build_explanation_with_examples() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let doc = make_doc();
        let fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        let validation = ValidationResult::valid();

        let explanation = pipeline.build_explanation(&doc, &fill, &validation);
        assert!(explanation.contains("Similar examples"));
        assert!(explanation.contains("sort -@ 4 -o out.bam in.bam"));
    }

    #[test]
    fn test_build_explanation_bool_flag() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let doc = make_doc();
        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-v".to_string(), String::new());
                m
            },
            positionals: Vec::new(),
        };
        let validation = ValidationResult::valid();

        let explanation = pipeline.build_explanation(&doc, &fill, &validation);
        assert!(explanation.contains("Flag -v enabled"));
    }

    #[test]
    fn test_pipeline_result_fields() {
        let record = make_record();
        let doc = make_doc();
        let result = PipelineResult {
            command: "samtools sort -@ 4 input.bam".to_string(),
            explanation: "Using subcommand sort".to_string(),
            validation: ValidationResult::valid(),
            tool_doc: doc,
            tool_record: record,
            effective_task: "sort BAM file".to_string(),
        };
        assert_eq!(result.command, "samtools sort -@ 4 input.bam");
        assert_eq!(result.effective_task, "sort BAM file");
        assert!(result.validation.is_valid);
    }

    #[test]
    fn test_stage1_resolve_valid_tool() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Bare);
        let result = pipeline.stage1_resolve("ls");
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.name, "ls");
    }

    #[test]
    fn test_stage1_resolve_empty_tool() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Bare);
        let result = pipeline.stage1_resolve("");
        assert!(result.is_err());
    }

    #[test]
    fn test_stage4_assemble() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let record = make_record();
        let doc = make_doc();
        let fill = LlmCommandFill {
            subcommand: Some("sort".to_string()),
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-@".to_string(), "4".to_string());
                m
            },
            positionals: vec!["input.bam".to_string()],
        };
        let command = pipeline.stage4_assemble(&record, &fill, &doc);
        assert!(command.starts_with("samtools sort"));
        assert!(command.contains("-@ 4"));
        assert!(command.contains("input.bam"));
    }

    #[test]
    fn test_stage5_validate() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let record = make_record();
        let doc = make_doc();
        let fill = LlmCommandFill {
            subcommand: Some("sort".to_string()),
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-@".to_string(), "4".to_string());
                m
            },
            positionals: vec!["input.bam".to_string()],
        };
        let validation = pipeline.stage5_validate(&record, &fill, &doc);
        assert!(validation.is_valid);
    }

    #[test]
    fn test_stage3_map() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let record = make_record();
        let doc = make_doc();
        let (subcmd, fill) = pipeline.stage3_map(&record, &doc, "sort the BAM file");
        assert_eq!(subcmd, Some("sort".to_string()));
        assert!(fill.flags.is_empty());
    }

    #[test]
    fn test_pipeline_new() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Bare);
        assert!(matches!(pipeline.scenario, WorkflowScenario::Bare));
    }

    #[test]
    fn test_stage2_explore_bare() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Bare);
        let record = make_record();
        let doc = pipeline.stage2_explore(&record, "test task");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let doc = rt.block_on(doc).unwrap();
        assert_eq!(doc.record.name, "samtools");
        assert_eq!(doc.schema_source, "bare");
        assert_eq!(doc.doc_quality, 0.0);
        assert!(doc.subcommands.is_empty());
        assert!(doc.flags.is_empty());
        assert!(doc.raw_help.is_none());
    }

    #[test]
    fn test_build_explanation_empty_fill() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let doc = make_doc();
        let fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        let validation = ValidationResult::valid();
        let explanation = pipeline.build_explanation(&doc, &fill, &validation);
        assert!(!explanation.contains("Using subcommand"));
        assert!(!explanation.contains("Flag"));
        assert!(!explanation.contains("Positional"));
    }

    #[test]
    fn test_build_explanation_no_examples() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let mut doc = make_doc();
        doc.examples = Vec::new();
        let fill = LlmCommandFill {
            subcommand: Some("sort".to_string()),
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        let validation = ValidationResult::valid();
        let explanation = pipeline.build_explanation(&doc, &fill, &validation);
        assert!(!explanation.contains("Similar examples"));
    }

    #[test]
    fn test_build_explanation_flag_with_value() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let doc = make_doc();
        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-o".to_string(), "out.bam".to_string());
                m
            },
            positionals: Vec::new(),
        };
        let validation = ValidationResult::valid();
        let explanation = pipeline.build_explanation(&doc, &fill, &validation);
        assert!(explanation.contains("Flag -o = out.bam"));
    }

    #[test]
    fn test_build_explanation_positionals() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let doc = make_doc();
        let fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: vec!["input.bam".to_string(), "output.bam".to_string()],
        };
        let validation = ValidationResult::valid();
        let explanation = pipeline.build_explanation(&doc, &fill, &validation);
        assert!(explanation.contains("Positional arguments: input.bam, output.bam"));
    }

    #[test]
    fn test_parse_args_into_fill_multiple_positionals_after_flags() {
        let mut fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        parse_args_into_fill("-t 4 in1.bam in2.bam", &mut fill);
        assert_eq!(fill.flags.get("-t"), Some(&"4".to_string()));
        assert_eq!(fill.positionals, vec!["in1.bam", "in2.bam"]);
    }

    #[test]
    fn test_parse_args_into_fill_only_whitespace() {
        let mut fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        parse_args_into_fill("   ", &mut fill);
        assert!(fill.flags.is_empty());
        assert!(fill.positionals.is_empty());
    }

    #[test]
    fn test_stage3_map_no_subcommand_match() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let record = make_record();
        let doc = make_doc();
        let (subcmd, fill) = pipeline.stage3_map(&record, &doc, "calculate depth");
        assert!(subcmd.is_none());
        assert!(fill.flags.is_empty());
    }

    #[test]
    fn test_stage5_validate_with_unknown_flag() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let record = make_record();
        let doc = make_doc();
        let fill = LlmCommandFill {
            subcommand: Some("sort".to_string()),
            flags: {
                let mut m = BTreeMap::new();
                m.insert("--nonexistent".to_string(), "value".to_string());
                m
            },
            positionals: vec!["input.bam".to_string()],
        };
        let validation = pipeline.stage5_validate(&record, &fill, &doc);
        assert!(
            !validation.is_valid
                || !validation.errors.is_empty()
                || !validation.warnings.is_empty()
        );
    }

    #[test]
    fn test_pipeline_result_debug() {
        let record = make_record();
        let doc = make_doc();
        let result = PipelineResult {
            command: "test".to_string(),
            explanation: "test explanation".to_string(),
            validation: ValidationResult::valid(),
            tool_doc: doc,
            tool_record: record,
            effective_task: "test task".to_string(),
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_parse_args_into_fill_flag_at_end() {
        let mut fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        parse_args_into_fill("input.bam -v", &mut fill);
        assert_eq!(fill.positionals, vec!["input.bam"]);
        assert_eq!(fill.flags.get("-v"), Some(&String::new()));
    }

    #[test]
    fn test_parse_args_into_fill_flag_before_positional() {
        let mut fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        parse_args_into_fill("-o out.bam input.bam", &mut fill);
        assert_eq!(fill.flags.get("-o"), Some(&"out.bam".to_string()));
        assert_eq!(fill.positionals, vec!["input.bam"]);
    }

    #[test]
    fn test_build_explanation_subcommand_no_doc_match() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let doc = make_doc();
        let fill = LlmCommandFill {
            subcommand: Some("nonexistent".to_string()),
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        let validation = ValidationResult::valid();
        let explanation = pipeline.build_explanation(&doc, &fill, &validation);
        // When subcommand has no matching doc entry, no subcommand description is added
        assert!(!explanation.contains("Using subcommand"));
    }

    #[test]
    fn test_stage1_resolve_path_dependent() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Bare);
        let result = pipeline.stage1_resolve("/bin/ls");
        assert!(result.is_ok());
    }

    #[test]
    fn test_stage4_assemble_no_subcommand() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let record = make_record();
        let doc = make_doc();
        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-v".to_string(), String::new());
                m
            },
            positionals: vec!["input.bam".to_string()],
        };
        let command = pipeline.stage4_assemble(&record, &fill, &doc);
        assert!(command.starts_with("samtools"));
        assert!(command.contains("-v"));
        assert!(command.contains("input.bam"));
    }

    #[test]
    fn test_build_explanation_multiple_flags() {
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Doc);
        let doc = make_doc();
        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-o".to_string(), "out.bam".to_string());
                m.insert("-@".to_string(), "4".to_string());
                m
            },
            positionals: Vec::new(),
        };
        let validation = ValidationResult::valid();
        let explanation = pipeline.build_explanation(&doc, &fill, &validation);
        assert!(explanation.contains("-o"));
        assert!(explanation.contains("-@"));
    }

    #[test]
    fn test_stage1_resolve_known_tool() {
        // stage1_resolve delegates to resolve_tool; verify it returns something for a known binary
        // (e.g. "sh" which is always present) and does not panic.
        let config = Config::default();
        let pipeline = Pipeline::new(config, WorkflowScenario::Bare);
        // We only assert the call completes without panicking; the returned Result may be Ok or Err
        // depending on whether resolve_tool can find the binary.
        let _ = pipeline.stage1_resolve("sh");
    }
}
