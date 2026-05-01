use crate::docs::DOMAIN_SUBCMD_MAP;
use crate::tool_doc::ToolDoc;
use crate::tool_resolver::ToolRecord;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCommandFill {
    pub subcommand: Option<String>,
    pub flags: BTreeMap<String, String>,
    pub positionals: Vec<String>,
}

pub struct IntentMapper;

impl Default for IntentMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl IntentMapper {
    pub fn new() -> Self {
        Self
    }

    pub fn map_intent(
        &self,
        record: &ToolRecord,
        doc: &ToolDoc,
        task: &str,
    ) -> (Option<String>, LlmCommandFill) {
        let subcommand = self.resolve_subcommand(record, doc, task);

        let fill = LlmCommandFill {
            subcommand: subcommand.clone(),
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };

        (subcommand, fill)
    }

    fn resolve_subcommand(&self, record: &ToolRecord, doc: &ToolDoc, task: &str) -> Option<String> {
        if doc.subcommands.is_empty() {
            return None;
        }

        let task_lower = task.to_lowercase();
        let task_words: Vec<&str> = task_lower.split_whitespace().collect();

        for subcmd in &doc.subcommands {
            if task_words.iter().any(|w| *w == subcmd.name) {
                return Some(subcmd.name.clone());
            }
        }

        if let Some(subcmd) = doc.select_subcommand(task) {
            return Some(subcmd.name.clone());
        }

        for &(tool, domain, subcmd) in DOMAIN_SUBCMD_MAP {
            if record.name == tool && task_lower.contains(domain) {
                return Some(subcmd.to_string());
            }
        }

        for companion in &record.companion_tools {
            let comp_lower = companion.to_lowercase();
            let comp_stem = comp_lower
                .trim_start_matches(&record.name.to_lowercase())
                .trim_start_matches('-')
                .trim_start_matches('_');
            if !comp_stem.is_empty() && task_lower.contains(comp_stem) {
                return None;
            }
        }

        None
    }

    pub fn build_llm_prompt(
        &self,
        record: &ToolRecord,
        doc: &ToolDoc,
        task: &str,
        subcommand: Option<&str>,
    ) -> String {
        let mut parts = Vec::new();

        parts.push(format!("Tool: {}", record.name));
        parts.push(format!("Task: {}", task));

        if !doc.subcommands.is_empty() {
            let subcmd_list: Vec<String> = doc
                .subcommands
                .iter()
                .map(|s| {
                    if s.description.is_empty() {
                        s.name.clone()
                    } else {
                        format!("{} - {}", s.name, s.description)
                    }
                })
                .collect();
            parts.push(format!("Available subcommands: {}", subcmd_list.join(", ")));

            if let Some(name) = subcommand {
                parts.push(format!("Selected subcommand: {}", name));
            }
        }

        let flag_section = doc.build_flag_prompt_section(subcommand);
        if !flag_section.is_empty() {
            parts.push(format!(
                "Available flags for '{}':\n{}",
                subcommand.unwrap_or(&record.name),
                flag_section
            ));
        }

        parts.push(
            "Output JSON: {\"subcommand\":\"<name|null>\",\"flags\":{\"<flag>\":\"<value>\"},\"positionals\":[\"<value>\"]}"
                .to_string(),
        );

        parts.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::{CliStyle, ParamType};
    use crate::tool_doc::{FlagDoc, SubcommandDoc};
    use crate::tool_resolver::ToolRecord;
    use std::path::PathBuf;

    fn make_test_record() -> ToolRecord {
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

    fn make_test_doc() -> ToolDoc {
        ToolDoc {
            record: make_test_record(),
            cli_style: CliStyle::Subcommand,
            description: "Tools for dealing with SAM, BAM and CRAM".to_string(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: vec![
                SubcommandDoc {
                    name: "sort".to_string(),
                    description: "sort alignments by leftmost coordinates".to_string(),
                    usage_pattern: "sort [-@ INT] [-o FILE] INPUT".to_string(),
                    flags: vec![
                        FlagDoc {
                            name: "-@".to_string(),
                            aliases: vec!["--threads".to_string()],
                            param_type: ParamType::Int,
                            description: "Number of additional threads".to_string(),
                            default: Some("0".to_string()),
                            required: false,
                            category: crate::tool_doc::FlagCategory::Performance,
                        },
                        FlagDoc {
                            name: "-o".to_string(),
                            aliases: vec!["--output".to_string()],
                            param_type: ParamType::File,
                            description: "Output file".to_string(),
                            default: Some("stdout".to_string()),
                            required: false,
                            category: crate::tool_doc::FlagCategory::Output,
                        },
                    ],
                    positionals: vec![crate::tool_doc::PositionalDoc {
                        position: 0,
                        name: "INPUT".to_string(),
                        param_type: ParamType::File,
                        description: "Input BAM file".to_string(),
                        required: true,
                        default: None,
                    }],
                    constraints: Vec::new(),
                    task_keywords: vec!["sort".to_string(), "coordinate".to_string()],
                },
                SubcommandDoc {
                    name: "view".to_string(),
                    description: "view and convert SAM/BAM/CRAM".to_string(),
                    usage_pattern: "view [-h] [-b] [-o FILE] [INPUT]".to_string(),
                    flags: Vec::new(),
                    positionals: Vec::new(),
                    constraints: Vec::new(),
                    task_keywords: vec![
                        "view".to_string(),
                        "convert".to_string(),
                        "extract".to_string(),
                    ],
                },
            ],
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
        }
    }

    #[test]
    fn test_resolve_subcommand_direct_match() {
        let mapper = IntentMapper::new();
        let record = make_test_record();
        let doc = make_test_doc();
        let (subcmd, _) = mapper.map_intent(&record, &doc, "sort input.bam by coordinate");
        assert_eq!(subcmd, Some("sort".to_string()));
    }

    #[test]
    fn test_resolve_subcommand_keyword_match() {
        let mapper = IntentMapper::new();
        let record = make_test_record();
        let doc = make_test_doc();
        let (subcmd, _) = mapper.map_intent(&record, &doc, "convert BAM to SAM");
        assert_eq!(subcmd, Some("view".to_string()));
    }

    #[test]
    fn test_resolve_subcommand_domain_map() {
        let mapper = IntentMapper::new();
        let record = ToolRecord {
            name: "bwa".to_string(),
            resolved_path: PathBuf::from("/usr/bin/bwa"),
            interpreter: None,
            is_path_dependent: false,
            global_path: Some(PathBuf::from("/usr/bin/bwa")),
            version: None,
            companion_tools: Vec::new(),
        };
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.5,
            subcommands: vec![SubcommandDoc {
                name: "mem".to_string(),
                description: "BWA-MEM algorithm".to_string(),
                usage_pattern: String::new(),
                flags: Vec::new(),
                positionals: Vec::new(),
                constraints: Vec::new(),
                task_keywords: vec!["align".to_string(), "map".to_string()],
            }],
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
        };
        let (subcmd, _) = mapper.map_intent(&record, &doc, "align reads to reference");
        assert_eq!(subcmd, Some("mem".to_string()));
    }

    #[test]
    fn test_build_llm_prompt() {
        let mapper = IntentMapper::new();
        let record = make_test_record();
        let doc = make_test_doc();
        let prompt = mapper.build_llm_prompt(&record, &doc, "sort input.bam", Some("sort"));
        assert!(prompt.contains("Tool: samtools"));
        assert!(prompt.contains("sort"));
        assert!(prompt.contains("Output JSON"));
    }

    #[test]
    fn test_resolve_subcommand_no_subcommands() {
        let mapper = IntentMapper::new();
        let record = make_test_record();
        let mut doc = make_test_doc();
        doc.subcommands = Vec::new();
        let (subcmd, fill) = mapper.map_intent(&record, &doc, "sort input.bam");
        assert!(subcmd.is_none());
        assert!(fill.subcommand.is_none());
    }

    #[test]
    fn test_resolve_subcommand_no_match() {
        let mapper = IntentMapper::new();
        let record = make_test_record();
        let doc = make_test_doc();
        let (subcmd, _) = mapper.map_intent(&record, &doc, "compress the file");
        assert!(subcmd.is_none());
    }

    #[test]
    fn test_build_llm_prompt_no_subcommand() {
        let mapper = IntentMapper::new();
        let record = make_test_record();
        let doc = make_test_doc();
        let prompt = mapper.build_llm_prompt(&record, &doc, "do something", None);
        assert!(prompt.contains("Tool: samtools"));
        assert!(prompt.contains("Available subcommands"));
        assert!(prompt.contains("Output JSON"));
    }

    #[test]
    fn test_build_llm_prompt_no_subcommands_in_doc() {
        let mapper = IntentMapper::new();
        let record = make_test_record();
        let mut doc = make_test_doc();
        doc.subcommands = Vec::new();
        let prompt = mapper.build_llm_prompt(&record, &doc, "do something", None);
        assert!(prompt.contains("Tool: samtools"));
        assert!(!prompt.contains("Available subcommands"));
    }

    #[test]
    fn test_llm_command_fill_serialization() {
        let fill = LlmCommandFill {
            subcommand: Some("sort".to_string()),
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-@".to_string(), "4".to_string());
                m.insert("-o".to_string(), "out.bam".to_string());
                m
            },
            positionals: vec!["input.bam".to_string()],
        };
        let json = serde_json::to_string(&fill).unwrap();
        let parsed: LlmCommandFill = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.subcommand, Some("sort".to_string()));
        assert_eq!(parsed.flags.get("-@"), Some(&"4".to_string()));
        assert_eq!(parsed.positionals, vec!["input.bam"]);
    }

    #[test]
    fn test_llm_command_fill_null_subcommand() {
        let json = r#"{"subcommand":null,"flags":{},"positionals":[]}"#;
        let fill: LlmCommandFill = serde_json::from_str(json).unwrap();
        assert!(fill.subcommand.is_none());
        assert!(fill.flags.is_empty());
        assert!(fill.positionals.is_empty());
    }

    #[test]
    fn test_map_intent_returns_empty_fill() {
        let mapper = IntentMapper::new();
        let record = make_test_record();
        let doc = make_test_doc();
        let (_, fill) = mapper.map_intent(&record, &doc, "sort input.bam");
        assert!(fill.flags.is_empty());
        assert!(fill.positionals.is_empty());
    }

    #[test]
    fn test_resolve_subcommand_companion_match() {
        let mapper = IntentMapper::new();
        let record = ToolRecord {
            name: "bowtie2".to_string(),
            resolved_path: PathBuf::from("/usr/bin/bowtie2"),
            interpreter: None,
            is_path_dependent: false,
            global_path: Some(PathBuf::from("/usr/bin/bowtie2")),
            version: None,
            companion_tools: vec!["bowtie2-build".to_string()],
        };
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.5,
            subcommands: vec![SubcommandDoc {
                name: "align".to_string(),
                description: "Align reads".to_string(),
                usage_pattern: String::new(),
                flags: Vec::new(),
                positionals: Vec::new(),
                constraints: Vec::new(),
                task_keywords: vec!["align".to_string()],
            }],
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
        };
        let (subcmd, _) = mapper.map_intent(&record, &doc, "build index for bowtie2");
        assert!(subcmd.is_none());
    }

    #[test]
    fn test_resolve_subcommand_domain_map_match() {
        let mapper = IntentMapper::new();
        let record = ToolRecord {
            name: "bwa".to_string(),
            resolved_path: PathBuf::from("/usr/bin/bwa"),
            interpreter: None,
            is_path_dependent: false,
            global_path: Some(PathBuf::from("/usr/bin/bwa")),
            version: None,
            companion_tools: Vec::new(),
        };
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.5,
            subcommands: vec![SubcommandDoc {
                name: "mem".to_string(),
                description: "BWA-MEM algorithm".to_string(),
                usage_pattern: String::new(),
                flags: Vec::new(),
                positionals: Vec::new(),
                constraints: Vec::new(),
                task_keywords: Vec::new(),
            }],
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
        };
        let (subcmd, _) = mapper.map_intent(&record, &doc, "align reads to reference");
        assert_eq!(subcmd, Some("mem".to_string()));
    }

    #[test]
    fn test_build_llm_prompt_subcommand_no_description() {
        let mapper = IntentMapper::new();
        let record = make_test_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.5,
            subcommands: vec![SubcommandDoc {
                name: "sort".to_string(),
                description: String::new(),
                usage_pattern: String::new(),
                flags: Vec::new(),
                positionals: Vec::new(),
                constraints: Vec::new(),
                task_keywords: Vec::new(),
            }],
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
        };
        let prompt = mapper.build_llm_prompt(&record, &doc, "sort data", Some("sort"));
        assert!(prompt.contains("sort"));
        assert!(prompt.contains("Selected subcommand: sort"));
    }
}
