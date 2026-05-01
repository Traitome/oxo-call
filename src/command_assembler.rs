use crate::schema::types::CliStyle;
use crate::tool_doc::ToolDoc;
use crate::tool_resolver::ToolRecord;
use crate::intent_mapper::LlmCommandFill;

pub struct CommandAssembler;

impl CommandAssembler {
    pub fn new() -> Self {
        Self
    }

    pub fn assemble(
        &self,
        record: &ToolRecord,
        fill: &LlmCommandFill,
        doc: &ToolDoc,
    ) -> String {
        let mut parts = Vec::new();

        parts.push(record.effective_name().to_string());

        if let Some(subcmd) = &fill.subcommand {
            parts.push(subcmd.clone());
        }

        match doc.cli_style {
            CliStyle::Subcommand | CliStyle::FlagsFirst => {
                self.append_flags(&mut parts, fill, doc);
                self.append_positionals(&mut parts, fill);
            }
            CliStyle::Positional => {
                self.append_positionals(&mut parts, fill);
                self.append_flags(&mut parts, fill, doc);
            }
            CliStyle::Hybrid => {
                self.append_positionals(&mut parts, fill);
                self.append_flags(&mut parts, fill, doc);
            }
        }

        parts.join(" ")
    }

    fn append_flags(
        &self,
        parts: &mut Vec<String>,
        fill: &LlmCommandFill,
        doc: &ToolDoc,
    ) {
        let all_flags: Vec<&str> = doc.all_flag_names();

        let mut sorted_flags: Vec<(&String, &String)> = fill.flags.iter().collect();
        sorted_flags.sort_by(|a, b| {
            let a_idx = all_flags.iter().position(|n| *n == a.0.as_str()).unwrap_or(usize::MAX);
            let b_idx = all_flags.iter().position(|n| *n == b.0.as_str()).unwrap_or(usize::MAX);
            a_idx.cmp(&b_idx)
        });

        for (flag, value) in sorted_flags {
            if value.is_empty() {
                parts.push(flag.clone());
            } else if flag.starts_with("--") && flag.contains('=') {
                parts.push(format!("{}={}", flag, value));
            } else {
                parts.push(flag.clone());
                parts.push(value.clone());
            }
        }
    }

    fn append_positionals(
        &self,
        parts: &mut Vec<String>,
        fill: &LlmCommandFill,
    ) {
        for pos in &fill.positionals {
            parts.push(pos.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool_doc::{FlagDoc, SubcommandDoc};
    use crate::schema::types::ParamType;
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

    #[test]
    fn test_assemble_subcommand_style() {
        let assembler = CommandAssembler::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: vec![SubcommandDoc {
                name: "sort".to_string(),
                description: String::new(),
                usage_pattern: String::new(),
                flags: vec![
                    FlagDoc {
                        name: "-@".to_string(),
                        aliases: vec!["--threads".to_string()],
                        param_type: ParamType::Int,
                        description: "threads".to_string(),
                        default: None,
                        required: false,
                        category: crate::tool_doc::FlagCategory::Performance,
                    },
                    FlagDoc {
                        name: "-o".to_string(),
                        aliases: vec!["--output".to_string()],
                        param_type: ParamType::File,
                        description: "output".to_string(),
                        default: None,
                        required: false,
                        category: crate::tool_doc::FlagCategory::Output,
                    },
                ],
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

        let fill = LlmCommandFill {
            subcommand: Some("sort".to_string()),
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-@".to_string(), "4".to_string());
                m.insert("-o".to_string(), "sorted.bam".to_string());
                m
            },
            positionals: vec!["input.bam".to_string()],
        };

        let cmd = assembler.assemble(&record, &fill, &doc);
        assert!(cmd.starts_with("samtools sort"));
        assert!(cmd.contains("-@ 4"));
        assert!(cmd.contains("-o sorted.bam"));
        assert!(cmd.contains("input.bam"));
    }

    #[test]
    fn test_assemble_flags_first() {
        let assembler = CommandAssembler::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagDoc {
                name: "-i".to_string(),
                aliases: vec!["--input".to_string()],
                param_type: ParamType::File,
                description: "input".to_string(),
                default: None,
                required: false,
                category: crate::tool_doc::FlagCategory::Input,
            }],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-i".to_string(), "in.fq".to_string());
                m
            },
            positionals: vec!["out.fq".to_string()],
        };

        let cmd = assembler.assemble(&record, &fill, &doc);
        assert!(cmd.contains("-i in.fq"));
        assert!(cmd.contains("out.fq"));
        let i_pos = cmd.find("-i").unwrap();
        let o_pos = cmd.rfind("out.fq").unwrap();
        assert!(i_pos < o_pos, "flags should come before positionals in FlagsFirst style");
    }

    #[test]
    fn test_assemble_bool_flag() {
        let assembler = CommandAssembler::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagDoc {
                name: "--verbose".to_string(),
                aliases: vec!["-v".to_string()],
                param_type: ParamType::Bool,
                description: "verbose".to_string(),
                default: None,
                required: false,
                category: crate::tool_doc::FlagCategory::General,
            }],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("--verbose".to_string(), String::new());
                m
            },
            positionals: Vec::new(),
        };

        let cmd = assembler.assemble(&record, &fill, &doc);
        assert!(cmd.contains("--verbose"));
        assert!(!cmd.contains("--verbose "));
    }

    #[test]
    fn test_assemble_positional_style() {
        let assembler = CommandAssembler::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::Positional,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagDoc {
                name: "-o".to_string(),
                aliases: Vec::new(),
                param_type: ParamType::File,
                description: "output".to_string(),
                default: None,
                required: false,
                category: crate::tool_doc::FlagCategory::Output,
            }],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-o".to_string(), "out.txt".to_string());
                m
            },
            positionals: vec!["input.txt".to_string()],
        };

        let cmd = assembler.assemble(&record, &fill, &doc);
        let input_pos = cmd.find("input.txt").unwrap();
        let flag_pos = cmd.find("-o").unwrap();
        assert!(input_pos < flag_pos, "positionals should come before flags in Positional style");
    }

    #[test]
    fn test_assemble_hybrid_style() {
        let assembler = CommandAssembler::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::Hybrid,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagDoc {
                name: "-k".to_string(),
                aliases: Vec::new(),
                param_type: ParamType::Int,
                description: "k-mer size".to_string(),
                default: None,
                required: false,
                category: crate::tool_doc::FlagCategory::Algorithm,
            }],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-k".to_string(), "31".to_string());
                m
            },
            positionals: vec!["reads.fq".to_string()],
        };

        let cmd = assembler.assemble(&record, &fill, &doc);
        assert!(cmd.contains("reads.fq"));
        assert!(cmd.contains("-k 31"));
        let pos_pos = cmd.find("reads.fq").unwrap();
        let flag_pos = cmd.find("-k").unwrap();
        assert!(pos_pos < flag_pos, "positionals should come before flags in Hybrid style");
    }

    #[test]
    fn test_assemble_equals_style_flag() {
        let assembler = CommandAssembler::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagDoc {
                name: "--output=".to_string(),
                aliases: Vec::new(),
                param_type: ParamType::File,
                description: "output".to_string(),
                default: None,
                required: false,
                category: crate::tool_doc::FlagCategory::Output,
            }],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("--output=".to_string(), "out.txt".to_string());
                m
            },
            positionals: Vec::new(),
        };

        let cmd = assembler.assemble(&record, &fill, &doc);
        assert!(cmd.contains("--output==out.txt"));
    }

    #[test]
    fn test_assemble_empty_fill() {
        let assembler = CommandAssembler::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
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
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };

        let cmd = assembler.assemble(&record, &fill, &doc);
        assert_eq!(cmd, "samtools");
    }

    #[test]
    fn test_assemble_multiple_bool_flags() {
        let assembler = CommandAssembler::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![
                FlagDoc {
                    name: "-v".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::Bool,
                    description: "verbose".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
                FlagDoc {
                    name: "-f".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::Bool,
                    description: "force".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
            ],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-v".to_string(), String::new());
                m.insert("-f".to_string(), String::new());
                m
            },
            positionals: Vec::new(),
        };

        let cmd = assembler.assemble(&record, &fill, &doc);
        assert!(cmd.contains("-v"));
        assert!(cmd.contains("-f"));
        assert!(!cmd.contains("-v true"));
        assert!(!cmd.contains("-f true"));
    }

    #[test]
    fn test_assemble_flag_order_follows_doc() {
        let assembler = CommandAssembler::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![
                FlagDoc {
                    name: "-a".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::String,
                    description: "first".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
                FlagDoc {
                    name: "-b".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::String,
                    description: "second".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
                FlagDoc {
                    name: "-c".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::String,
                    description: "third".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
            ],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-c".to_string(), "3".to_string());
                m.insert("-a".to_string(), "1".to_string());
                m
            },
            positionals: Vec::new(),
        };

        let cmd = assembler.assemble(&record, &fill, &doc);
        let a_pos = cmd.find("-a").unwrap();
        let c_pos = cmd.find("-c").unwrap();
        assert!(a_pos < c_pos, "flags should follow doc order, not insertion order");
    }
}
