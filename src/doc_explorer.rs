use crate::doc_processor::DocProcessor;
use crate::docs::DocsFetcher;
use crate::schema::parse_help;
use crate::skill::SkillManager;
use crate::tool_doc::{
    CommandExample, ExampleSource, FlagCategory, FlagDoc, PositionalDoc, SubcommandDoc, ToolDoc,
};
use crate::tool_resolver::ToolRecord;
use color_eyre::Result;
use std::collections::HashMap;

const _MAX_SUBCOMMAND_DEPTH: usize = 3;

#[allow(dead_code)]
pub struct DocExplorer {
    fetcher: DocsFetcher,
    skill_manager: SkillManager,
}

#[allow(dead_code)]
impl DocExplorer {
    pub fn new(config: crate::config::Config) -> Self {
        let fetcher = DocsFetcher::new(config.clone());
        let skill_manager = SkillManager::new(config);
        Self {
            fetcher,
            skill_manager,
        }
    }

    pub async fn explore(&self, record: &ToolRecord, task: &str) -> Result<ToolDoc> {
        let tool = &record.name;

        let raw_help = self.fetch_root_help(tool).await;

        let (cli_schema, structured_doc) = if let Some(ref help) = raw_help {
            let schema = parse_help(tool, help);
            let processor = DocProcessor::new();
            let sdoc = processor.clean_and_structure(help);
            (Some(schema), Some(sdoc))
        } else {
            (None, None)
        };

        let mut subcommand_helps = HashMap::new();

        if let Some(ref schema) = cli_schema {
            if !schema.subcommands.is_empty() {
                let selected = schema.select_subcommand(task);
                let subcmds_to_fetch = if let Some(sel) = selected {
                    vec![sel.name.clone()]
                } else {
                    schema
                        .subcommands
                        .iter()
                        .take(10)
                        .map(|s| s.name.clone())
                        .collect()
                };

                for subcmd_name in subcmds_to_fetch {
                    if let Some(help) = self.fetch_subcommand_help(tool, &subcmd_name).await {
                        subcommand_helps.insert(subcmd_name.clone(), help);
                    }
                }
            }
        }

        let skill = self.skill_manager.load(tool);

        Ok(build_tool_doc(
            record,
            &raw_help,
            &cli_schema,
            &structured_doc,
            &subcommand_helps,
            &skill,
        ))
    }

    async fn fetch_root_help(&self, tool: &str) -> Option<String> {
        let help_strategies = [
            "--help",
            "-h",
            "-H",
            "help",
            "--usage",
            "--help-all",
            "",
            "-help",
            "--h",
            "-?",
        ];

        for flag in &help_strategies {
            let result = if flag.is_empty() {
                self.run_no_args(tool).await
            } else if *flag == "help" {
                self.run_help_subcommand(tool).await
            } else if *flag == "-?" {
                self.run_with_flag(tool, "-?").await
            } else {
                self.run_with_flag(tool, flag).await
            };

            if let Some(output) = result {
                if is_valid_help(&output) {
                    return Some(output);
                }
            }
        }

        None
    }

    async fn fetch_subcommand_help(&self, tool: &str, subcmd: &str) -> Option<String> {
        let strategies = [
            format!("{} --help", subcmd),
            format!("{} -h", subcmd),
            format!("help {}", subcmd),
            subcmd.to_string(),
            format!("--help {}", subcmd),
            format!("{} --help-all", subcmd),
        ];

        for strategy in &strategies {
            let parts: Vec<&str> = strategy.split_whitespace().collect();
            let result = if parts.len() == 1 && parts[0] == subcmd {
                self.run_no_args_subcmd(tool, subcmd).await
            } else {
                self.run_with_args(tool, &parts).await
            };

            if let Some(output) = result {
                if is_valid_help(&output) {
                    return Some(output);
                }
            }
        }

        None
    }

    async fn run_with_flag(&self, tool: &str, flag: &str) -> Option<String> {
        let output = tokio::process::Command::new(tool)
            .arg(flag)
            .output()
            .await
            .ok()?;

        extract_useful(&output.stdout, &output.stderr)
    }

    async fn run_no_args(&self, tool: &str) -> Option<String> {
        let output = tokio::process::Command::new(tool)
            .output()
            .await
            .ok()?;

        extract_useful(&output.stdout, &output.stderr)
    }

    async fn run_help_subcommand(&self, tool: &str) -> Option<String> {
        let output = tokio::process::Command::new(tool)
            .arg("help")
            .output()
            .await
            .ok()?;

        extract_useful(&output.stdout, &output.stderr)
    }

    async fn run_no_args_subcmd(&self, tool: &str, subcmd: &str) -> Option<String> {
        let output = tokio::process::Command::new(tool)
            .arg(subcmd)
            .output()
            .await
            .ok()?;

        extract_useful(&output.stdout, &output.stderr)
    }

    async fn run_with_args(&self, tool: &str, args: &[&str]) -> Option<String> {
        let output = tokio::process::Command::new(tool)
            .args(args)
            .output()
            .await
            .ok()?;

        extract_useful(&output.stdout, &output.stderr)
    }
}

fn is_valid_help(output: &str) -> bool {
    let lower = output.to_lowercase();
    let has_help_section = lower.contains("usage")
        || lower.contains("option")
        || lower.contains("flag")
        || lower.contains("command")
        || lower.contains("argument")
        || lower.contains("syntax");
    let is_pure_error =
        (lower.contains("error:") || lower.contains("fatal:")) && !has_help_section;
    let long_enough = output.trim().len() >= 40;
    has_help_section && !is_pure_error && long_enough
}

fn extract_useful(stdout: &[u8], stderr: &[u8]) -> Option<String> {
    let stdout_str = String::from_utf8_lossy(stdout).to_string();
    let stderr_str = String::from_utf8_lossy(stderr).to_string();

    if !stdout_str.trim().is_empty() {
        Some(stdout_str)
    } else if !stderr_str.trim().is_empty() {
        Some(stderr_str)
    } else {
        None
    }
}

fn categorize_flag(name: &str, description: &str) -> FlagCategory {
    let lower = format!("{} {}", name, description).to_lowercase();

    if lower.contains("format")
        || lower.contains("--format")
        || lower.contains("--sam")
        || lower.contains("--cram")
        || lower.contains("--json")
        || lower.contains("--tsv")
        || lower.contains("--csv")
    {
        return FlagCategory::Format;
    }
    if lower.contains("input")
        || lower.contains("-i ")
        || lower.contains("--input")
        || lower.contains("--read")
        || lower.contains("--ref")
        || lower.contains("--bam")
        || lower.contains("--fastq")
        || lower.contains("--vcf")
        || lower.contains("--bed")
    {
        return FlagCategory::Input;
    }
    if lower.contains("output")
        || lower.contains("-o ")
        || lower.contains("--output")
        || lower.contains("--out")
        || lower.contains("--write")
    {
        return FlagCategory::Output;
    }
    if lower.contains("thread")
        || lower.contains("-@ ")
        || lower.contains("--thread")
        || lower.contains("--process")
        || lower.contains("--memory")
        || lower.contains("--mem")
        || lower.contains("cpu")
        || lower.contains("parallel")
    {
        return FlagCategory::Performance;
    }
    if lower.contains("quality")
        || lower.contains("--min-quality")
        || lower.contains("--min-mapq")
        || lower.contains("-q ")
        || lower.contains("--qual")
        || lower.contains("--filter")
    {
        return FlagCategory::Quality;
    }
    if lower.contains("algorithm")
        || lower.contains("--algorithm")
        || lower.contains("--strategy")
        || lower.contains("--mode")
        || lower.contains("--method")
    {
        return FlagCategory::Algorithm;
    }

    FlagCategory::General
}

fn get_skill_subcmd_keywords(_tool: &str, _subcmd: &str) -> Option<Vec<String>> {
    None
}

pub fn build_tool_doc(
    record: &ToolRecord,
    raw_help: &Option<String>,
    cli_schema: &Option<crate::schema::CliSchema>,
    structured_doc: &Option<crate::doc_processor::StructuredDoc>,
    subcommand_helps: &HashMap<String, String>,
    skill: &Option<crate::skill::Skill>,
) -> ToolDoc {
    let tool = &record.name;

    let mut subcommand_docs = Vec::new();

    if let Some(schema) = cli_schema {
        if !schema.subcommands.is_empty() {
            subcommand_docs = schema
                .subcommands
                .iter()
                .map(|s| {
                    let mut doc = SubcommandDoc::from(s.clone());
                    if let Some(help) = subcommand_helps.get(&s.name) {
                        let sub_schema = parse_help(
                            &format!("{} {}", tool, s.name),
                            help,
                        );
                        if !sub_schema.flags.is_empty() {
                            doc.flags = sub_schema
                                .flags
                                .into_iter()
                                .map(|f| {
                                    let mut fd = FlagDoc::from(f);
                                    fd.category = categorize_flag(&fd.name, &fd.description);
                                    fd
                                })
                                .collect();
                        }
                        if !sub_schema.positionals.is_empty() {
                            doc.positionals = sub_schema
                                .positionals
                                .into_iter()
                                .map(|p| PositionalDoc::from(p.clone()))
                                .collect();
                        }
                        if !sub_schema.usage_summary.is_empty() && doc.usage_pattern.is_empty() {
                            doc.usage_pattern = sub_schema.usage_summary;
                        }
                    }
                    doc
                })
                .collect();
        }
    }

    let (cli_style, description, schema_source, doc_quality) = if let Some(schema) = cli_schema
    {
        (
            schema.cli_style,
            schema.description.clone(),
            schema.schema_source.clone(),
            schema.doc_quality,
        )
    } else {
        (
            crate::schema::CliStyle::FlagsFirst,
            String::new(),
            "none".to_string(),
            0.0,
        )
    };

    let mut examples = Vec::new();
    if let Some(sdoc) = structured_doc {
        for ex in &sdoc.extracted_examples {
            examples.push(CommandExample {
                args: ex.clone(),
                explanation: String::new(),
                source: ExampleSource::HelpText,
            });
        }
    }

    let mut concepts = Vec::new();
    let mut pitfalls = Vec::new();

    if let Some(s) = skill {
        for ex in &s.examples {
            examples.push(CommandExample {
                args: ex.args.clone(),
                explanation: ex.explanation.clone(),
                source: ExampleSource::SkillFile,
            });
        }
        concepts = s.context.concepts.clone();
        pitfalls = s.context.pitfalls.clone();

        for subcmd in &mut subcommand_docs {
            if let Some(skill_subcmd_keywords) = get_skill_subcmd_keywords(&s.meta.name, &subcmd.name) {
                for kw in skill_subcmd_keywords {
                    if !subcmd.task_keywords.contains(&kw) {
                        subcmd.task_keywords.push(kw);
                    }
                }
            }
        }
    }

    let global_flags = cli_schema
        .as_ref()
        .map(|s| {
            s.global_flags
                .iter()
                .map(|f| {
                    let mut fd = FlagDoc::from(f.clone());
                    fd.category = categorize_flag(&fd.name, &fd.description);
                    fd
                })
                .collect()
        })
        .unwrap_or_default();

    let flags = cli_schema
        .as_ref()
        .map(|s| {
            s.flags
                .iter()
                .map(|f| {
                    let mut fd = FlagDoc::from(f.clone());
                    fd.category = categorize_flag(&fd.name, &fd.description);
                    fd
                })
                .collect()
        })
        .unwrap_or_default();

    let positionals = cli_schema
        .as_ref()
        .map(|s| {
            s.positionals
                .iter()
                .map(|p| PositionalDoc::from(p.clone()))
                .collect()
        })
        .unwrap_or_default();

    let constraints = cli_schema
        .as_ref()
        .map(|s| s.constraints.clone())
        .unwrap_or_default();

    let usage_patterns = if let Some(sdoc) = structured_doc {
        if !sdoc.usage.is_empty() {
            vec![sdoc.usage.clone()]
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    ToolDoc {
        record: record.clone(),
        cli_style,
        description,
        schema_source,
        doc_quality,
        subcommands: subcommand_docs,
        global_flags,
        flags,
        positionals,
        usage_patterns,
        constraints,
        examples,
        concepts,
        pitfalls,
        raw_help: raw_help.clone(),
        subcommand_helps: subcommand_helps.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_help() {
        assert!(is_valid_help("Usage: tool [options]\nOptions:\n  -h  Show help"));
        assert!(is_valid_help("SYNTAX: tool input output\nArguments:\n  input  Input file"));
        assert!(!is_valid_help("Error: command not found"));
        assert!(!is_valid_help("hi"));
        assert!(!is_valid_help(""));
    }

    #[test]
    fn test_categorize_flag() {
        assert_eq!(categorize_flag("-i", "input file"), FlagCategory::Input);
        assert_eq!(categorize_flag("-o", "output file"), FlagCategory::Output);
        assert_eq!(
            categorize_flag("-@", "number of threads"),
            FlagCategory::Performance
        );
        assert_eq!(
            categorize_flag("-q", "minimum quality"),
            FlagCategory::Quality
        );
        assert_eq!(
            categorize_flag("--format", "output format"),
            FlagCategory::Format
        );
        assert_eq!(
            categorize_flag("--algorithm", "alignment algorithm"),
            FlagCategory::Algorithm
        );
        assert_eq!(categorize_flag("-h", "show help"), FlagCategory::General);
    }

    #[test]
    fn test_categorize_flag_aliases() {
        assert_eq!(categorize_flag("--input", "read file"), FlagCategory::Input);
        assert_eq!(categorize_flag("--output", "write file"), FlagCategory::Output);
        assert_eq!(categorize_flag("--threads", "parallel threads"), FlagCategory::Performance);
        assert_eq!(categorize_flag("--min-quality", "quality threshold"), FlagCategory::Quality);
        assert_eq!(categorize_flag("--sam", "output SAM format"), FlagCategory::Format);
        assert_eq!(categorize_flag("--mode", "running mode"), FlagCategory::Algorithm);
    }

    #[test]
    fn test_categorize_flag_combined() {
        assert_eq!(categorize_flag("--format", "output format type"), FlagCategory::Format);
        assert_eq!(categorize_flag("-o", "output file path"), FlagCategory::Output);
        assert_eq!(categorize_flag("--read1", "input read1 fastq"), FlagCategory::Input);
    }

    #[test]
    fn test_is_valid_help_various() {
        assert!(is_valid_help("Usage: mytool [options]\nOptions:\n  -v  Verbose mode\n  -o FILE  Output"));
        assert!(is_valid_help("usage: tool input output\nCommands:\n  run  Run analysis"));
        assert!(is_valid_help("SYNTAX: tool [flags]\nFlags:\n  --help  Show help"));
        assert!(is_valid_help("Arguments:\n  INPUT   Input file\n  OUTPUT  Output file\nThis tool processes files."));
        assert!(!is_valid_help("Error: command not found"));
        assert!(!is_valid_help("fatal: cannot open file"));
        assert!(!is_valid_help("hi"));
        assert!(!is_valid_help(""));
        assert!(!is_valid_help("option"));
    }

    #[test]
    fn test_is_valid_help_error_with_help() {
        assert!(is_valid_help("Error: missing argument\nUsage: tool [options]\nOptions:\n  -h  Help"));
    }

    #[test]
    fn test_extract_useful_stdout() {
        let stdout: &[u8] = b"Usage: tool [options]";
        let stderr: &[u8] = b"";
        let result = extract_useful(stdout, stderr);
        assert!(result.is_some());
        assert!(result.unwrap().contains("Usage"));
    }

    #[test]
    fn test_extract_useful_stderr() {
        let stdout: &[u8] = b"";
        let stderr: &[u8] = b"Usage: tool [options]";
        let result = extract_useful(stdout, stderr);
        assert!(result.is_some());
        assert!(result.unwrap().contains("Usage"));
    }

    #[test]
    fn test_extract_useful_both() {
        let stdout: &[u8] = b"output text";
        let stderr: &[u8] = b"error text";
        let result = extract_useful(stdout, stderr);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "output text");
    }

    #[test]
    fn test_extract_useful_neither() {
        let stdout: &[u8] = b"";
        let stderr: &[u8] = b"";
        let result = extract_useful(stdout, stderr);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_useful_whitespace_only() {
        let stdout: &[u8] = b"   \n  ";
        let stderr: &[u8] = b"";
        let result = extract_useful(stdout, stderr);
        assert!(result.is_none());
    }

    #[test]
    fn test_categorize_flag_performance_variants() {
        assert_eq!(categorize_flag("--memory", "memory limit"), FlagCategory::Performance);
        assert_eq!(categorize_flag("--mem", "memory usage"), FlagCategory::Performance);
        assert_eq!(categorize_flag("--process", "number of processes"), FlagCategory::Performance);
        assert_eq!(categorize_flag("-p", "cpu parallel"), FlagCategory::Performance);
    }

    #[test]
    fn test_categorize_flag_quality_variants() {
        assert_eq!(categorize_flag("--min-mapq", "minimum mapping quality"), FlagCategory::Quality);
        assert_eq!(categorize_flag("--qual", "quality score"), FlagCategory::Quality);
        assert_eq!(categorize_flag("--filter", "quality filter"), FlagCategory::Quality);
    }

    #[test]
    fn test_categorize_flag_format_variants() {
        assert_eq!(categorize_flag("--json", "JSON output"), FlagCategory::Format);
        assert_eq!(categorize_flag("--tsv", "TSV output"), FlagCategory::Format);
        assert_eq!(categorize_flag("--csv", "CSV output"), FlagCategory::Format);
        assert_eq!(categorize_flag("--cram", "CRAM format"), FlagCategory::Format);
    }

    #[test]
    fn test_categorize_flag_input_variants() {
        assert_eq!(categorize_flag("--read", "read file"), FlagCategory::Input);
        assert_eq!(categorize_flag("--ref", "reference file"), FlagCategory::Input);
        assert_eq!(categorize_flag("--bam", "BAM input"), FlagCategory::Input);
        assert_eq!(categorize_flag("--fastq", "FASTQ input"), FlagCategory::Input);
        assert_eq!(categorize_flag("--vcf", "VCF input"), FlagCategory::Input);
        assert_eq!(categorize_flag("--bed", "BED input"), FlagCategory::Input);
    }

    #[test]
    fn test_categorize_flag_output_variants() {
        assert_eq!(categorize_flag("--out", "output prefix"), FlagCategory::Output);
        assert_eq!(categorize_flag("--write", "write output"), FlagCategory::Output);
    }

    #[test]
    fn test_categorize_flag_algorithm_variants() {
        assert_eq!(categorize_flag("--strategy", "search strategy"), FlagCategory::Algorithm);
        assert_eq!(categorize_flag("--method", "alignment method"), FlagCategory::Algorithm);
    }

    fn make_test_record() -> ToolRecord {
        ToolRecord {
            name: "testtool".to_string(),
            resolved_path: std::path::PathBuf::from("/usr/bin/testtool"),
            interpreter: None,
            is_path_dependent: false,
            global_path: Some(std::path::PathBuf::from("/usr/bin/testtool")),
            version: None,
            companion_tools: Vec::new(),
        }
    }

    #[test]
    fn test_build_tool_doc_no_schema_no_skill() {
        let record = make_test_record();
        let doc = build_tool_doc(
            &record,
            &None,
            &None,
            &None,
            &HashMap::new(),
            &None,
        );
        assert_eq!(doc.record.name, "testtool");
        assert_eq!(doc.schema_source, "none");
        assert_eq!(doc.doc_quality, 0.0);
        assert!(doc.subcommands.is_empty());
        assert!(doc.flags.is_empty());
        assert!(doc.global_flags.is_empty());
        assert!(doc.positionals.is_empty());
        assert!(doc.examples.is_empty());
        assert!(doc.concepts.is_empty());
        assert!(doc.pitfalls.is_empty());
        assert!(doc.usage_patterns.is_empty());
        assert!(doc.constraints.is_empty());
        assert!(doc.raw_help.is_none());
        assert!(doc.subcommand_helps.is_empty());
    }

    #[test]
    fn test_build_tool_doc_with_raw_help() {
        let record = make_test_record();
        let help = "Usage: testtool [options]".to_string();
        let doc = build_tool_doc(
            &record,
            &Some(help.clone()),
            &None,
            &None,
            &HashMap::new(),
            &None,
        );
        assert_eq!(doc.raw_help, Some(help));
    }

    #[test]
    fn test_build_tool_doc_with_schema() {
        let record = make_test_record();
        let schema = crate::schema::CliSchema::minimal("testtool", crate::schema::CliStyle::FlagsFirst);
        let doc = build_tool_doc(
            &record,
            &None,
            &Some(schema),
            &None,
            &HashMap::new(),
            &None,
        );
        assert_eq!(doc.schema_source, "minimal");
        assert_eq!(doc.cli_style, crate::schema::CliStyle::FlagsFirst);
    }

    #[test]
    fn test_build_tool_doc_with_structured_doc_examples() {
        let record = make_test_record();
        let sdoc = crate::doc_processor::StructuredDoc {
            usage: "testtool [options] INPUT".to_string(),
            examples: String::new(),
            options: String::new(),
            commands: String::new(),
            other: String::new(),
            quick_flags: Vec::new(),
            flag_catalog: Vec::new(),
            extracted_examples: vec!["testtool -v input.txt".to_string()],
            quality_score: 0.8,
            command_pattern: String::new(),
            detected_subcommand: None,
            all_subcommands: Vec::new(),
        };
        let doc = build_tool_doc(
            &record,
            &None,
            &None,
            &Some(sdoc),
            &HashMap::new(),
            &None,
        );
        assert_eq!(doc.examples.len(), 1);
        assert_eq!(doc.examples[0].args, "testtool -v input.txt");
        assert_eq!(doc.examples[0].source, ExampleSource::HelpText);
        assert_eq!(doc.usage_patterns, vec!["testtool [options] INPUT"]);
    }

    #[test]
    fn test_build_tool_doc_with_structured_doc_no_usage() {
        let record = make_test_record();
        let sdoc = crate::doc_processor::StructuredDoc {
            usage: String::new(),
            examples: String::new(),
            options: String::new(),
            commands: String::new(),
            other: String::new(),
            quick_flags: Vec::new(),
            flag_catalog: Vec::new(),
            extracted_examples: Vec::new(),
            quality_score: 0.0,
            command_pattern: String::new(),
            detected_subcommand: None,
            all_subcommands: Vec::new(),
        };
        let doc = build_tool_doc(
            &record,
            &None,
            &None,
            &Some(sdoc),
            &HashMap::new(),
            &None,
        );
        assert!(doc.usage_patterns.is_empty());
    }

    #[test]
    fn test_build_tool_doc_with_skill() {
        let record = make_test_record();
        let skill = crate::skill::Skill {
            meta: crate::skill::SkillMeta {
                name: "testtool".to_string(),
                category: "testing".to_string(),
                description: "A test tool".to_string(),
                tags: Vec::new(),
                author: None,
                source_url: None,
                min_version: None,
                max_version: None,
            },
            context: crate::skill::SkillContext {
                concepts: vec!["alignment".to_string()],
                pitfalls: vec!["wrong order".to_string()],
            },
            examples: vec![crate::skill::SkillExample {
                task: "run test".to_string(),
                args: "-v input.txt".to_string(),
                explanation: "verbose mode".to_string(),
            }],
        };
        let doc = build_tool_doc(
            &record,
            &None,
            &None,
            &None,
            &HashMap::new(),
            &Some(skill),
        );
        assert_eq!(doc.concepts, vec!["alignment"]);
        assert_eq!(doc.pitfalls, vec!["wrong order"]);
        assert_eq!(doc.examples.len(), 1);
        assert_eq!(doc.examples[0].args, "-v input.txt");
        assert_eq!(doc.examples[0].source, ExampleSource::SkillFile);
    }

    #[test]
    fn test_build_tool_doc_with_subcommand_helps() {
        let record = make_test_record();
        let mut schema = crate::schema::CliSchema::minimal("testtool", crate::schema::CliStyle::Subcommand);
        schema.subcommands = vec![crate::schema::types::SubcommandSchema {
            name: "sort".to_string(),
            description: "Sort data".to_string(),
            usage_pattern: "testtool sort INPUT".to_string(),
            flags: Vec::new(),
            positionals: Vec::new(),
            constraints: Vec::new(),
            task_keywords: vec!["sort".to_string()],
        }];
        let mut subcmd_helps = HashMap::new();
        subcmd_helps.insert("sort".to_string(), "Usage: testtool sort [options] INPUT\nOptions:\n  -t INT    Number of threads\n  -o FILE   Output file".to_string());

        let doc = build_tool_doc(
            &record,
            &None,
            &Some(schema),
            &None,
            &subcmd_helps,
            &None,
        );
        assert_eq!(doc.subcommands.len(), 1);
        assert_eq!(doc.subcommands[0].name, "sort");
        assert!(!doc.subcommands[0].flags.is_empty());
        assert_eq!(doc.subcommands[0].flags[0].name, "-t");
        assert_eq!(doc.subcommands[0].flags[0].category, FlagCategory::Performance);
    }

    #[test]
    fn test_build_tool_doc_schema_with_flags() {
        let record = make_test_record();
        let mut schema = crate::schema::CliSchema::minimal("testtool", crate::schema::CliStyle::FlagsFirst);
        schema.flags = vec![crate::schema::types::FlagSchema {
            name: "-o".to_string(),
            aliases: vec!["--output".to_string()],
            param_type: crate::schema::types::ParamType::File,
            description: "output file".to_string(),
            default: None,
            required: false,
            long_description: None,
        }];
        schema.global_flags = vec![crate::schema::types::FlagSchema {
            name: "-v".to_string(),
            aliases: vec!["--verbose".to_string()],
            param_type: crate::schema::types::ParamType::Bool,
            description: "verbose mode".to_string(),
            default: None,
            required: false,
            long_description: None,
        }];

        let doc = build_tool_doc(
            &record,
            &None,
            &Some(schema),
            &None,
            &HashMap::new(),
            &None,
        );
        assert_eq!(doc.flags.len(), 1);
        assert_eq!(doc.flags[0].name, "-o");
        assert_eq!(doc.flags[0].category, FlagCategory::Output);
        assert_eq!(doc.global_flags.len(), 1);
        assert_eq!(doc.global_flags[0].name, "-v");
        assert_eq!(doc.global_flags[0].category, FlagCategory::General);
    }

    #[test]
    fn test_build_tool_doc_schema_with_positionals() {
        let record = make_test_record();
        let mut schema = crate::schema::CliSchema::minimal("testtool", crate::schema::CliStyle::Positional);
        schema.positionals = vec![crate::schema::types::PositionalSchema {
            name: "INPUT".to_string(),
            position: 0,
            param_type: crate::schema::types::ParamType::File,
            description: "input file".to_string(),
            required: true,
            default: None,
        }];

        let doc = build_tool_doc(
            &record,
            &None,
            &Some(schema),
            &None,
            &HashMap::new(),
            &None,
        );
        assert_eq!(doc.positionals.len(), 1);
        assert_eq!(doc.positionals[0].name, "INPUT");
    }

    #[test]
    fn test_build_tool_doc_schema_with_constraints() {
        let record = make_test_record();
        let mut schema = crate::schema::CliSchema::minimal("testtool", crate::schema::CliStyle::FlagsFirst);
        schema.constraints = vec![crate::schema::types::ConstraintRule::MutuallyExclusive(
            "-a".to_string(), "-b".to_string()
        )];

        let doc = build_tool_doc(
            &record,
            &None,
            &Some(schema),
            &None,
            &HashMap::new(),
            &None,
        );
        assert_eq!(doc.constraints.len(), 1);
    }

    #[test]
    fn test_build_tool_doc_subcommand_no_help() {
        let record = make_test_record();
        let mut schema = crate::schema::CliSchema::minimal("testtool", crate::schema::CliStyle::Subcommand);
        schema.subcommands = vec![crate::schema::types::SubcommandSchema {
            name: "index".to_string(),
            description: "Build index".to_string(),
            usage_pattern: "testtool index REF".to_string(),
            flags: Vec::new(),
            positionals: Vec::new(),
            constraints: Vec::new(),
            task_keywords: vec!["index".to_string()],
        }];

        let doc = build_tool_doc(
            &record,
            &None,
            &Some(schema),
            &None,
            &HashMap::new(),
            &None,
        );
        assert_eq!(doc.subcommands.len(), 1);
        assert_eq!(doc.subcommands[0].name, "index");
        assert!(doc.subcommands[0].flags.is_empty());
    }

    #[test]
    fn test_build_tool_doc_combined_skill_and_help_examples() {
        let record = make_test_record();
        let sdoc = crate::doc_processor::StructuredDoc {
            usage: String::new(),
            examples: String::new(),
            options: String::new(),
            commands: String::new(),
            other: String::new(),
            quick_flags: Vec::new(),
            flag_catalog: Vec::new(),
            extracted_examples: vec!["testtool -h".to_string()],
            quality_score: 0.5,
            command_pattern: String::new(),
            detected_subcommand: None,
            all_subcommands: Vec::new(),
        };
        let skill = crate::skill::Skill {
            meta: crate::skill::SkillMeta {
                name: "testtool".to_string(),
                category: "testing".to_string(),
                description: String::new(),
                tags: Vec::new(),
                author: None,
                source_url: None,
                min_version: None,
                max_version: None,
            },
            context: crate::skill::SkillContext {
                concepts: Vec::new(),
                pitfalls: Vec::new(),
            },
            examples: vec![crate::skill::SkillExample {
                task: "run".to_string(),
                args: "-v input".to_string(),
                explanation: "verbose".to_string(),
            }],
        };
        let doc = build_tool_doc(
            &record,
            &None,
            &None,
            &Some(sdoc),
            &HashMap::new(),
            &Some(skill),
        );
        assert_eq!(doc.examples.len(), 2);
        assert_eq!(doc.examples[0].source, ExampleSource::HelpText);
        assert_eq!(doc.examples[1].source, ExampleSource::SkillFile);
    }

    #[test]
    fn test_build_tool_doc_subcommand_help_with_positionals() {
        let record = make_test_record();
        let mut schema = crate::schema::CliSchema::minimal("testtool", crate::schema::CliStyle::Subcommand);
        schema.subcommands = vec![crate::schema::types::SubcommandSchema {
            name: "view".to_string(),
            description: "View data".to_string(),
            usage_pattern: String::new(),
            flags: Vec::new(),
            positionals: Vec::new(),
            constraints: Vec::new(),
            task_keywords: vec!["view".to_string()],
        }];
        let mut subcmd_helps = HashMap::new();
        subcmd_helps.insert("view".to_string(), "Usage: testtool view INPUT OUTPUT\nPositional arguments:\n  INPUT   Input file\n  OUTPUT  Output file".to_string());

        let doc = build_tool_doc(
            &record,
            &None,
            &Some(schema),
            &None,
            &subcmd_helps,
            &None,
        );
        assert_eq!(doc.subcommands.len(), 1);
        assert!(!doc.subcommands[0].positionals.is_empty());
    }

    #[test]
    fn test_build_tool_doc_subcommand_help_with_usage() {
        let record = make_test_record();
        let mut schema = crate::schema::CliSchema::minimal("testtool", crate::schema::CliStyle::Subcommand);
        schema.subcommands = vec![crate::schema::types::SubcommandSchema {
            name: "sort".to_string(),
            description: "Sort data".to_string(),
            usage_pattern: String::new(),
            flags: Vec::new(),
            positionals: Vec::new(),
            constraints: Vec::new(),
            task_keywords: vec!["sort".to_string()],
        }];
        let mut subcmd_helps = HashMap::new();
        subcmd_helps.insert("sort".to_string(), "Usage: testtool sort [options] INPUT\nOptions:\n  -@ INT  Threads".to_string());

        let doc = build_tool_doc(
            &record,
            &None,
            &Some(schema),
            &None,
            &subcmd_helps,
            &None,
        );
        assert!(!doc.subcommands[0].usage_pattern.is_empty() || !doc.subcommands[0].flags.is_empty());
    }

    #[test]
    fn test_build_tool_doc_skill_with_subcommands() {
        let record = make_test_record();
        let mut schema = crate::schema::CliSchema::minimal("testtool", crate::schema::CliStyle::Subcommand);
        schema.subcommands = vec![crate::schema::types::SubcommandSchema {
            name: "run".to_string(),
            description: "Run analysis".to_string(),
            usage_pattern: String::new(),
            flags: Vec::new(),
            positionals: Vec::new(),
            constraints: Vec::new(),
            task_keywords: vec!["run".to_string()],
        }];
        let skill = crate::skill::Skill {
            meta: crate::skill::SkillMeta {
                name: "testtool".to_string(),
                category: "testing".to_string(),
                description: String::new(),
                tags: Vec::new(),
                author: None,
                source_url: None,
                min_version: None,
                max_version: None,
            },
            context: crate::skill::SkillContext {
                concepts: vec!["analysis".to_string()],
                pitfalls: vec!["wrong input".to_string()],
            },
            examples: vec![crate::skill::SkillExample {
                task: "run".to_string(),
                args: "run -v input".to_string(),
                explanation: "verbose run".to_string(),
            }],
        };
        let doc = build_tool_doc(
            &record,
            &None,
            &Some(schema),
            &None,
            &HashMap::new(),
            &Some(skill),
        );
        assert_eq!(doc.subcommands.len(), 1);
        assert_eq!(doc.concepts, vec!["analysis"]);
        assert_eq!(doc.pitfalls, vec!["wrong input"]);
    }
}
