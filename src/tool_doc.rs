use crate::schema::types::{
    CliStyle, ConstraintRule, FlagSchema, ParamType, PositionalSchema, SubcommandSchema,
};
use crate::tool_resolver::ToolRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDoc {
    pub record: ToolRecord,
    pub cli_style: CliStyle,
    pub description: String,
    pub schema_source: String,
    pub doc_quality: f32,
    pub subcommands: Vec<SubcommandDoc>,
    pub global_flags: Vec<FlagDoc>,
    pub flags: Vec<FlagDoc>,
    pub positionals: Vec<PositionalDoc>,
    pub usage_patterns: Vec<String>,
    pub constraints: Vec<ConstraintRule>,
    pub examples: Vec<CommandExample>,
    pub concepts: Vec<String>,
    pub pitfalls: Vec<String>,
    pub raw_help: Option<String>,
    pub subcommand_helps: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcommandDoc {
    pub name: String,
    pub description: String,
    pub usage_pattern: String,
    pub flags: Vec<FlagDoc>,
    pub positionals: Vec<PositionalDoc>,
    pub constraints: Vec<ConstraintRule>,
    pub task_keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagDoc {
    pub name: String,
    pub aliases: Vec<String>,
    pub param_type: ParamType,
    pub description: String,
    pub default: Option<String>,
    pub required: bool,
    pub category: FlagCategory,
}

impl FlagDoc {
    pub fn all_names(&self) -> Vec<&str> {
        let mut names = vec![self.name.as_str()];
        names.extend(self.aliases.iter().map(|s| s.as_str()));
        names
    }

    #[allow(dead_code)]
    pub fn matches_name(&self, name: &str) -> bool {
        self.name == name || self.aliases.iter().any(|a| a == name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FlagCategory {
    Input,
    Output,
    Performance,
    Quality,
    Format,
    Algorithm,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionalDoc {
    pub position: usize,
    pub name: String,
    pub param_type: ParamType,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExample {
    pub args: String,
    pub explanation: String,
    pub source: ExampleSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExampleSource {
    HelpText,
    SkillFile,
    LlmGenerated,
}

impl ToolDoc {
    pub fn all_flag_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = Vec::new();
        for f in &self.global_flags {
            names.push(&f.name);
            for a in &f.aliases {
                names.push(a);
            }
        }
        for f in &self.flags {
            names.push(&f.name);
            for a in &f.aliases {
                names.push(a);
            }
        }
        names
    }

    pub fn get_subcommand(&self, name: &str) -> Option<&SubcommandDoc> {
        self.subcommands.iter().find(|s| s.name == name)
    }

    pub fn select_subcommand(&self, task: &str) -> Option<&SubcommandDoc> {
        if self.subcommands.is_empty() {
            return None;
        }

        let task_lower = task.to_lowercase();
        let task_words: Vec<&str> = task_lower.split_whitespace().collect();

        let mut best: Option<(&SubcommandDoc, usize)> = None;

        for subcmd in &self.subcommands {
            let mut score = 0;

            if task_words.iter().any(|w| *w == subcmd.name) {
                score += 100;
            }

            for kw in &subcmd.task_keywords {
                let kw_lower = kw.to_lowercase();
                if task_words
                    .iter()
                    .any(|w| *w == kw_lower || w.contains(&kw_lower))
                {
                    score += 10;
                }
                if task_lower.contains(&kw_lower) {
                    score += 5;
                }
            }

            let desc_lower = subcmd.description.to_lowercase();
            for word in &task_words {
                if desc_lower.contains(word) {
                    score += 2;
                }
            }

            if score > 0 && best.is_none_or(|(_, s)| score > s) {
                best = Some((subcmd, score));
            }
        }

        best.map(|(s, _)| s)
    }

    pub fn build_flag_prompt_section(&self, subcommand: Option<&str>) -> String {
        let mut lines = Vec::new();

        if let Some(subcmd_name) = subcommand {
            if let Some(subcmd) = self.get_subcommand(subcmd_name) {
                for f in &subcmd.flags {
                    let names = if f.aliases.is_empty() {
                        f.name.clone()
                    } else {
                        format!("{}, {}", f.name, f.aliases.join(", "))
                    };
                    let type_hint = match &f.param_type {
                        ParamType::Bool => String::new(),
                        ParamType::Int => " INT".to_string(),
                        ParamType::Float => " FLOAT".to_string(),
                        ParamType::File => " FILE".to_string(),
                        ParamType::Enum(vals) => format!(" {{{}}}", vals.join("|")),
                        ParamType::String => " STR".to_string(),
                    };
                    let default_hint = f
                        .default
                        .as_ref()
                        .map(|d| format!(" [{}]", d))
                        .unwrap_or_default();
                    lines.push(format!(
                        "  {}{}{} - {}",
                        names, type_hint, default_hint, f.description
                    ));
                }
                for p in &subcmd.positionals {
                    lines.push(format!(
                        "  <{}> {} - {}",
                        p.name,
                        if p.required {
                            "(required)"
                        } else {
                            "(optional)"
                        },
                        p.description
                    ));
                }
            }
        } else {
            for f in &self.flags {
                let names = if f.aliases.is_empty() {
                    f.name.clone()
                } else {
                    format!("{}, {}", f.name, f.aliases.join(", "))
                };
                let type_hint = match &f.param_type {
                    ParamType::Bool => String::new(),
                    ParamType::Int => " INT".to_string(),
                    ParamType::Float => " FLOAT".to_string(),
                    ParamType::File => " FILE".to_string(),
                    ParamType::Enum(vals) => format!(" {{{}}}", vals.join("|")),
                    ParamType::String => " STR".to_string(),
                };
                let default_hint = f
                    .default
                    .as_ref()
                    .map(|d| format!(" [{}]", d))
                    .unwrap_or_default();
                lines.push(format!(
                    "  {}{}{} - {}",
                    names, type_hint, default_hint, f.description
                ));
            }
            for p in &self.positionals {
                lines.push(format!(
                    "  <{}> {} - {}",
                    p.name,
                    if p.required {
                        "(required)"
                    } else {
                        "(optional)"
                    },
                    p.description
                ));
            }
        }

        lines.join("\n")
    }
}

impl From<FlagSchema> for FlagDoc {
    fn from(f: FlagSchema) -> Self {
        FlagDoc {
            name: f.name,
            aliases: f.aliases,
            param_type: f.param_type,
            description: f.description,
            default: f.default,
            required: f.required,
            category: FlagCategory::General,
        }
    }
}

impl From<PositionalSchema> for PositionalDoc {
    fn from(p: PositionalSchema) -> Self {
        PositionalDoc {
            position: p.position,
            name: p.name,
            param_type: p.param_type,
            description: p.description,
            required: p.required,
            default: p.default,
        }
    }
}

impl From<SubcommandSchema> for SubcommandDoc {
    fn from(s: SubcommandSchema) -> Self {
        SubcommandDoc {
            name: s.name,
            description: s.description,
            usage_pattern: s.usage_pattern,
            flags: s.flags.into_iter().map(FlagDoc::from).collect(),
            positionals: s.positionals.into_iter().map(PositionalDoc::from).collect(),
            constraints: s.constraints,
            task_keywords: s.task_keywords,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::{CliStyle, ParamType};
    use crate::tool_resolver::ToolRecord;
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

    fn make_flag(name: &str, aliases: Vec<&str>, category: FlagCategory) -> FlagDoc {
        FlagDoc {
            name: name.to_string(),
            aliases: aliases.iter().map(|s| s.to_string()).collect(),
            param_type: ParamType::String,
            description: format!("desc for {}", name),
            default: None,
            required: false,
            category,
        }
    }

    fn make_subcommand(name: &str, keywords: Vec<&str>) -> SubcommandDoc {
        SubcommandDoc {
            name: name.to_string(),
            description: format!("{} description", name),
            usage_pattern: String::new(),
            flags: Vec::new(),
            positionals: Vec::new(),
            constraints: Vec::new(),
            task_keywords: keywords.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn make_doc() -> ToolDoc {
        ToolDoc {
            record: make_record(),
            cli_style: CliStyle::Subcommand,
            description: "SAM/BAM/CRAM tools".to_string(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: vec![
                make_subcommand("sort", vec!["sort", "coordinate"]),
                make_subcommand("view", vec!["view", "convert", "extract"]),
                make_subcommand("index", vec!["index", "bai"]),
            ],
            global_flags: vec![make_flag("--verbose", vec!["-v"], FlagCategory::General)],
            flags: vec![make_flag("-i", vec!["--input"], FlagCategory::Input)],
            positionals: vec![PositionalDoc {
                position: 0,
                name: "INPUT".to_string(),
                param_type: ParamType::File,
                description: "Input file".to_string(),
                required: true,
                default: None,
            }],
            usage_patterns: vec!["samtools sort [options] INPUT".to_string()],
            constraints: Vec::new(),
            examples: vec![CommandExample {
                args: "sort -@ 4 -o out.bam in.bam".to_string(),
                explanation: "Sort BAM with 4 threads".to_string(),
                source: ExampleSource::HelpText,
            }],
            concepts: vec!["BAM".to_string(), "SAM".to_string()],
            pitfalls: vec!["Do not use -h with -b".to_string()],
            raw_help: Some("Usage: samtools ...".to_string()),
            subcommand_helps: HashMap::new(),
        }
    }

    #[test]
    fn test_all_flag_names() {
        let doc = make_doc();
        let names = doc.all_flag_names();
        assert!(names.contains(&"--verbose"));
        assert!(names.contains(&"-v"));
        assert!(names.contains(&"-i"));
        assert!(names.contains(&"--input"));
    }

    #[test]
    fn test_all_flag_names_empty() {
        let doc = ToolDoc {
            record: make_record(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "none".to_string(),
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
            subcommand_helps: HashMap::new(),
        };
        assert!(doc.all_flag_names().is_empty());
    }

    #[test]
    fn test_get_subcommand() {
        let doc = make_doc();
        assert!(doc.get_subcommand("sort").is_some());
        assert!(doc.get_subcommand("view").is_some());
        assert!(doc.get_subcommand("nonexistent").is_none());
    }

    #[test]
    fn test_select_subcommand_direct_match() {
        let doc = make_doc();
        let result = doc.select_subcommand("sort the bam file");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "sort");
    }

    #[test]
    fn test_select_subcommand_keyword_match() {
        let doc = make_doc();
        let result = doc.select_subcommand("convert BAM to SAM");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "view");
    }

    #[test]
    fn test_select_subcommand_no_match() {
        let doc = make_doc();
        let result = doc.select_subcommand("compress the file");
        assert!(result.is_none());
    }

    #[test]
    fn test_select_subcommand_empty_subcommands() {
        let mut doc = make_doc();
        doc.subcommands = Vec::new();
        assert!(doc.select_subcommand("sort").is_none());
    }

    #[test]
    fn test_build_flag_prompt_section_no_subcommand() {
        let doc = make_doc();
        let section = doc.build_flag_prompt_section(None);
        assert!(section.contains("-i"));
        assert!(section.contains("--input"));
        assert!(section.contains("INPUT"));
    }

    #[test]
    fn test_build_flag_prompt_section_with_subcommand() {
        let mut doc = make_doc();
        doc.subcommands[0].flags = vec![make_flag(
            "-@",
            vec!["--threads"],
            FlagCategory::Performance,
        )];
        doc.subcommands[0].positionals = vec![PositionalDoc {
            position: 0,
            name: "INPUT".to_string(),
            param_type: ParamType::File,
            description: "Input BAM".to_string(),
            required: true,
            default: None,
        }];
        let section = doc.build_flag_prompt_section(Some("sort"));
        assert!(section.contains("-@"));
        assert!(section.contains("INPUT"));
    }

    #[test]
    fn test_flag_doc_all_names() {
        let flag = make_flag("-o", vec!["--output"], FlagCategory::Output);
        let names = flag.all_names();
        assert_eq!(names, vec!["-o", "--output"]);
    }

    #[test]
    fn test_flag_doc_matches_name() {
        let flag = make_flag("-o", vec!["--output"], FlagCategory::Output);
        assert!(flag.matches_name("-o"));
        assert!(flag.matches_name("--output"));
        assert!(!flag.matches_name("-i"));
    }

    #[test]
    fn test_from_flag_schema() {
        let fs = FlagSchema {
            name: "-t".to_string(),
            aliases: vec!["--threads".to_string()],
            param_type: ParamType::Int,
            description: "threads".to_string(),
            default: Some("1".to_string()),
            required: false,
            long_description: None,
        };
        let fd: FlagDoc = fs.into();
        assert_eq!(fd.name, "-t");
        assert_eq!(fd.aliases, vec!["--threads"]);
        assert_eq!(fd.category, FlagCategory::General);
    }

    #[test]
    fn test_from_positional_schema() {
        let ps = PositionalSchema {
            position: 0,
            name: "INPUT".to_string(),
            param_type: ParamType::File,
            description: "input file".to_string(),
            required: true,
            default: None,
        };
        let pd: PositionalDoc = ps.into();
        assert_eq!(pd.position, 0);
        assert_eq!(pd.name, "INPUT");
        assert!(pd.required);
    }

    #[test]
    fn test_from_subcommand_schema() {
        let ss = SubcommandSchema {
            name: "sort".to_string(),
            description: "sort alignments".to_string(),
            usage_pattern: "sort [opts]".to_string(),
            flags: vec![FlagSchema {
                name: "-@".to_string(),
                aliases: Vec::new(),
                param_type: ParamType::Int,
                description: "threads".to_string(),
                default: None,
                required: false,
                long_description: None,
            }],
            positionals: Vec::new(),
            constraints: Vec::new(),
            task_keywords: vec!["sort".to_string()],
        };
        let sd: SubcommandDoc = ss.into();
        assert_eq!(sd.name, "sort");
        assert_eq!(sd.flags.len(), 1);
        assert_eq!(sd.task_keywords, vec!["sort"]);
    }

    #[test]
    fn test_tool_doc_serialization() {
        let doc = make_doc();
        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: ToolDoc = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.record.name, "samtools");
        assert_eq!(deserialized.subcommands.len(), 3);
        assert_eq!(deserialized.doc_quality, 0.8);
    }

    #[test]
    fn test_command_example_sources() {
        let ex1 = CommandExample {
            args: "sort in.bam".to_string(),
            explanation: "sort".to_string(),
            source: ExampleSource::HelpText,
        };
        let ex2 = CommandExample {
            args: "view -b in.sam".to_string(),
            explanation: "convert".to_string(),
            source: ExampleSource::SkillFile,
        };
        let ex3 = CommandExample {
            args: "index in.bam".to_string(),
            explanation: "index".to_string(),
            source: ExampleSource::LlmGenerated,
        };
        let json = serde_json::to_string(&vec![ex1, ex2, ex3]).unwrap();
        let parsed: Vec<CommandExample> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 3);
        assert!(matches!(parsed[0].source, ExampleSource::HelpText));
        assert!(matches!(parsed[1].source, ExampleSource::SkillFile));
        assert!(matches!(parsed[2].source, ExampleSource::LlmGenerated));
    }

    #[test]
    fn test_build_flag_prompt_section_various_types() {
        let mut doc = make_doc();
        doc.subcommands[0].flags = vec![
            FlagDoc {
                name: "-@".to_string(),
                aliases: vec!["--threads".to_string()],
                param_type: ParamType::Int,
                description: "Number of threads".to_string(),
                default: Some("1".to_string()),
                required: false,
                category: FlagCategory::Performance,
            },
            FlagDoc {
                name: "--threshold".to_string(),
                aliases: vec![],
                param_type: ParamType::Float,
                description: "Threshold".to_string(),
                default: None,
                required: false,
                category: FlagCategory::Quality,
            },
            FlagDoc {
                name: "-o".to_string(),
                aliases: vec![],
                param_type: ParamType::File,
                description: "Output file".to_string(),
                default: None,
                required: true,
                category: FlagCategory::Output,
            },
            FlagDoc {
                name: "--format".to_string(),
                aliases: vec![],
                param_type: ParamType::Enum(vec![
                    "bam".to_string(),
                    "sam".to_string(),
                    "cram".to_string(),
                ]),
                description: "Output format".to_string(),
                default: Some("bam".to_string()),
                required: false,
                category: FlagCategory::Format,
            },
            FlagDoc {
                name: "-v".to_string(),
                aliases: vec![],
                param_type: ParamType::Bool,
                description: "Verbose".to_string(),
                default: None,
                required: false,
                category: FlagCategory::General,
            },
            FlagDoc {
                name: "-L".to_string(),
                aliases: vec![],
                param_type: ParamType::String,
                description: "Label".to_string(),
                default: None,
                required: false,
                category: FlagCategory::General,
            },
        ];
        doc.subcommands[0].positionals = vec![
            PositionalDoc {
                position: 0,
                name: "INPUT".to_string(),
                param_type: ParamType::File,
                description: "Input BAM".to_string(),
                required: true,
                default: None,
            },
            PositionalDoc {
                position: 1,
                name: "OUTPUT".to_string(),
                param_type: ParamType::File,
                description: "Output prefix".to_string(),
                required: false,
                default: None,
            },
        ];
        let section = doc.build_flag_prompt_section(Some("sort"));
        assert!(section.contains("INT"));
        assert!(section.contains("FLOAT"));
        assert!(section.contains("FILE"));
        assert!(section.contains("bam|sam|cram"));
        assert!(section.contains("[1]"));
        assert!(section.contains("[bam]"));
        assert!(section.contains("(required)"));
        assert!(section.contains("(optional)"));
    }

    #[test]
    fn test_build_flag_prompt_section_no_subcommand_various_types() {
        let mut doc = make_doc();
        doc.flags = vec![
            FlagDoc {
                name: "-t".to_string(),
                aliases: vec!["--threads".to_string()],
                param_type: ParamType::Int,
                description: "Threads".to_string(),
                default: Some("4".to_string()),
                required: false,
                category: FlagCategory::Performance,
            },
            FlagDoc {
                name: "--verbose".to_string(),
                aliases: vec![],
                param_type: ParamType::Bool,
                description: "Verbose".to_string(),
                default: None,
                required: false,
                category: FlagCategory::General,
            },
            FlagDoc {
                name: "-L".to_string(),
                aliases: vec![],
                param_type: ParamType::String,
                description: "Label".to_string(),
                default: None,
                required: false,
                category: FlagCategory::General,
            },
        ];
        doc.positionals = vec![PositionalDoc {
            position: 0,
            name: "INPUT".to_string(),
            param_type: ParamType::File,
            description: "Input".to_string(),
            required: true,
            default: None,
        }];
        let section = doc.build_flag_prompt_section(None);
        assert!(section.contains("INT"));
        assert!(section.contains("[4]"));
        assert!(section.contains("(required)"));
    }
}
