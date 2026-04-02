//! Skill file parsing and benchmark scenario / usage-description generation.
//!
//! This module turns the built-in skill Markdown files (in `skills/`) into:
//!
//! 1. **Reference commands** — 10 scenarios per tool, each with a known-good
//!    `ARGS` string extracted (or derived) from the skill examples.
//! 2. **Usage descriptions** — 10 diverse English paraphrases per scenario,
//!    simulating users of different experience levels.
//!
//! Both artefacts are exported as CSV so that humans can review / tweak them
//! before running the actual LLM evaluation.

use std::io::Write;
use std::path::Path;

// ── Data types ───────────────────────────────────────────────────────────────

/// Parsed representation of a single skill Markdown file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillFile {
    pub name: String,
    pub category: String,
    pub description: String,
    pub tags: Vec<String>,
    pub source_url: String,
    pub examples: Vec<SkillExample>,
}

/// One example block from a skill file (`### heading` → `**Args:**` → `**Explanation:**`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillExample {
    pub task: String,
    pub args: String,
    pub explanation: String,
}

/// A benchmark scenario: one specific flag-combination for a tool with a
/// known-good reference command.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Scenario {
    pub tool: String,
    pub scenario_id: String,
    pub reference_args: String,
    pub task_description: String,
    pub category: String,
}

/// A single usage description — one of 10 paraphrases for a given scenario,
/// written at a particular user-experience level.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UsageDescription {
    pub tool: String,
    pub scenario_id: String,
    pub desc_id: String,
    pub user_level: String,
    pub description: String,
}

// ── Skill file parsing ───────────────────────────────────────────────────────

/// Parse a single skill Markdown file into a [`SkillFile`].
///
/// Expected layout:
/// ```text
/// ---
/// name: samtools
/// category: alignment
/// description: ...
/// tags: [bam, sam]
/// author: ...
/// source_url: "..."
/// ---
/// ## Concepts
/// ...
/// ## Examples
/// ### task description
/// **Args:** `flags`
/// **Explanation:** text
/// ```
pub fn parse_skill_file(content: &str) -> anyhow::Result<SkillFile> {
    // ── 1. Extract YAML front-matter ──────────────────────────────────────
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        anyhow::bail!("Missing YAML front-matter delimiters");
    }
    let front = parts[1].trim();
    let body = parts[2];

    let name = extract_yaml_value(front, "name").unwrap_or_default();
    let category = extract_yaml_value(front, "category").unwrap_or_default();
    let description = extract_yaml_value(front, "description").unwrap_or_default();
    let source_url = extract_yaml_value(front, "source_url")
        .unwrap_or_default()
        .trim_matches('"')
        .to_string();

    let tags = extract_yaml_value(front, "tags")
        .map(|s| {
            s.trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();

    // ── 2. Parse examples ─────────────────────────────────────────────────
    let examples = parse_examples(body);

    Ok(SkillFile {
        name,
        category,
        description,
        tags,
        source_url,
        examples,
    })
}

/// Extract a simple `key: value` from YAML-like text.
fn extract_yaml_value(yaml: &str, key: &str) -> Option<String> {
    for line in yaml.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(value) = rest.strip_prefix(':') {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

/// Parse the `## Examples` section of a skill file body.
fn parse_examples(body: &str) -> Vec<SkillExample> {
    let mut examples = Vec::new();
    let mut in_examples = false;
    let mut current_task: Option<String> = None;
    let mut current_args: Option<String> = None;
    let mut current_explanation: Option<String> = None;

    for line in body.lines() {
        let trimmed = line.trim();

        // Detect ## Examples section.
        if trimmed.starts_with("## ") {
            if trimmed.eq_ignore_ascii_case("## examples")
                || trimmed.eq_ignore_ascii_case("## Examples")
                || trimmed.starts_with("## Example")
            {
                in_examples = true;
            } else if in_examples {
                // Another ## section after examples → stop.
                break;
            }
            continue;
        }

        if !in_examples {
            continue;
        }

        // ### heading → new example.
        if let Some(heading) = trimmed.strip_prefix("### ") {
            // Flush previous example.
            if let (Some(task), Some(args)) = (current_task.take(), current_args.take()) {
                examples.push(SkillExample {
                    task,
                    args,
                    explanation: current_explanation.take().unwrap_or_default(),
                });
            }
            current_task = Some(heading.to_string());
            current_args = None;
            current_explanation = None;
            continue;
        }

        // **Args:** `...`
        if let Some(rest) = trimmed
            .strip_prefix("**Args:**")
            .or_else(|| trimmed.strip_prefix("**Args: **"))
        {
            let args = rest
                .trim()
                .trim_start_matches('`')
                .trim_end_matches('`')
                .to_string();
            current_args = Some(args);
            continue;
        }

        // **Explanation:** ...
        if let Some(rest) = trimmed
            .strip_prefix("**Explanation:**")
            .or_else(|| trimmed.strip_prefix("**Explanation: **"))
        {
            current_explanation = Some(rest.trim().to_string());
        }
    }

    // Flush final example.
    if let (Some(task), Some(args)) = (current_task, current_args) {
        examples.push(SkillExample {
            task,
            args,
            explanation: current_explanation.unwrap_or_default(),
        });
    }

    examples
}

/// Load all skill files from a directory (files matching `*.md`).
pub fn load_skills_from_dir(dir: &Path) -> anyhow::Result<Vec<SkillFile>> {
    let mut skills = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let content = std::fs::read_to_string(entry.path())?;
        match parse_skill_file(&content) {
            Ok(skill) if !skill.examples.is_empty() => skills.push(skill),
            Ok(_) => {
                eprintln!(
                    "  warning: {} has no examples, skipping",
                    entry.path().display()
                );
            }
            Err(e) => {
                eprintln!("  warning: failed to parse {}: {e}", entry.path().display());
            }
        }
    }
    Ok(skills)
}

// ── Scenario generation ──────────────────────────────────────────────────────

/// Number of scenarios to generate per tool.
pub const SCENARIOS_PER_TOOL: usize = 10;

/// Generate exactly [`SCENARIOS_PER_TOOL`] scenarios from a parsed skill file.
///
/// If the skill has fewer than 10 examples the remaining slots are filled with
/// synthesised variants that recombine flags from existing examples.
pub fn generate_scenarios(skill: &SkillFile) -> Vec<Scenario> {
    let mut scenarios: Vec<Scenario> = skill
        .examples
        .iter()
        .take(SCENARIOS_PER_TOOL)
        .enumerate()
        .map(|(i, ex)| Scenario {
            tool: skill.name.clone(),
            scenario_id: format!("{}_{:02}", skill.name, i + 1),
            reference_args: ex.args.clone(),
            task_description: ex.task.clone(),
            category: skill.category.clone(),
        })
        .collect();

    // Pad to SCENARIOS_PER_TOOL with synthesised variants.
    if scenarios.len() < SCENARIOS_PER_TOOL && !skill.examples.is_empty() {
        let originals = scenarios.clone();
        let mut idx = scenarios.len();
        let mut source_idx = 0;

        while scenarios.len() < SCENARIOS_PER_TOOL {
            let base = &originals[source_idx % originals.len()];
            let variant = synthesise_variant(base, idx);
            scenarios.push(variant);
            idx += 1;
            source_idx += 1;
        }
    }

    scenarios
}

/// Synthetic variant types used to pad scenarios to [`SCENARIOS_PER_TOOL`].
const VARIANT_VERBOSE: usize = 0;
const VARIANT_THREADS: usize = 1;
const VARIANT_OUTPUT: usize = 2;
const VARIANT_QUIET: usize = 3;
const VARIANT_DEFAULT: usize = 4;
const NUM_VARIANT_TYPES: usize = 5;

/// Return `true` when appending an extra flag to `args` would be unsafe,
/// e.g. the command already uses a shell pipe (`|`) or output redirection
/// (`>`), so an appended flag would land after the pipe/redirect rather than
/// on the intended tool.
fn has_shell_operator(args: &str) -> bool {
    args.contains('|') || args.contains('>')
}

/// Create a synthetic variant of a scenario by adding/changing common flags.
///
/// The function is conservative: it never appends flags to commands that
/// contain shell operators (`|`, `>`), and it avoids appending `-t 4` when
/// the command already uses `-t` or `--threads` (since `-t` has different
/// semantics across tools — threads in some, tag/reference in others).
fn synthesise_variant(base: &Scenario, idx: usize) -> Scenario {
    let variant_type = idx % NUM_VARIANT_TYPES;

    // If the base command contains shell operators, we never append flags
    // because they would land after the pipe/redirect, not on the tool.
    let shell_op = has_shell_operator(&base.reference_args);

    let suffix = match variant_type {
        VARIANT_VERBOSE if !shell_op => " with verbose output",
        VARIANT_THREADS if !shell_op => " using multiple threads",
        VARIANT_OUTPUT if !shell_op => " and write output to a file",
        VARIANT_QUIET if !shell_op => " in quiet mode",
        VARIANT_DEFAULT => " with default parameters",
        _ => "",
    };

    let args_suffix = match variant_type {
        _ if shell_op => "",
        VARIANT_VERBOSE => " --verbose",
        // Only add -t 4 if the command does not already contain -t (which
        // has tool-specific semantics), -@, -j, -p, or --threads.
        VARIANT_THREADS
            if base.reference_args.contains("-t ")
                || base.reference_args.contains("-t\t")
                || base.reference_args.contains("-@ ")
                || base.reference_args.contains("-j ")
                || base.reference_args.contains("-p ")
                || base.reference_args.contains("--threads") =>
        {
            ""
        }
        VARIANT_THREADS => " -t 4",
        // Only add -o if the command doesn't already write to a file.
        VARIANT_OUTPUT
            if base.reference_args.contains("-o ")
                || base.reference_args.contains("-o\t")
                || base.reference_args.contains("--output") =>
        {
            ""
        }
        VARIANT_OUTPUT => " -o output.txt",
        VARIANT_QUIET => " --quiet",
        _ => "",
    };

    let new_args = format!("{}{}", base.reference_args, args_suffix);
    let new_task = format!("{}{}", base.task_description, suffix);

    Scenario {
        tool: base.tool.clone(),
        scenario_id: format!("{}_{:02}", base.tool, idx + 1),
        reference_args: new_args,
        task_description: new_task,
        category: base.category.clone(),
    }
}

// ── Description generation ───────────────────────────────────────────────────

/// Number of descriptions to generate per scenario.
pub const DESCRIPTIONS_PER_SCENARIO: usize = 10;

/// User-level labels (one per generated description).
pub const USER_LEVELS: [&str; DESCRIPTIONS_PER_SCENARIO] = [
    "original",
    "beginner",
    "student",
    "polite",
    "sysadmin",
    "goal_oriented",
    "expert",
    "detailed",
    "informal",
    "alternative",
];

/// Generate [`DESCRIPTIONS_PER_SCENARIO`] diverse English descriptions for a
/// single [`Scenario`], simulating users of different experience levels.
pub fn generate_descriptions(scenario: &Scenario) -> Vec<UsageDescription> {
    let task = &scenario.task_description;
    let tool = &scenario.tool;
    let lc = lowercase_first_char(task);

    let variants: Vec<String> = vec![
        // 1  original — skill-file wording
        task.to_string(),
        // 2  beginner — question form
        format!("How do I {}?", lc),
        // 3  student — simple request
        format!("I need to {}", lc),
        // 4  polite — imperative with please
        format!("Please {}", lc),
        // 5  sysadmin — mentions tool
        format!("Use {} to {}", tool, lc),
        // 6  goal-oriented — focus on outcome
        format!("I want to {}", lc),
        // 7  expert — terse
        make_terse(task),
        // 8  detailed — adds extra context
        make_detailed(task, &scenario.reference_args),
        // 9  informal — casual wording
        make_casual(task),
        // 10 alternative — rephrased
        make_alternative(task),
    ];

    variants
        .into_iter()
        .enumerate()
        .map(|(i, desc)| UsageDescription {
            tool: tool.clone(),
            scenario_id: scenario.scenario_id.clone(),
            desc_id: format!("{}_{:02}", scenario.scenario_id, i + 1),
            user_level: USER_LEVELS[i].to_string(),
            description: desc,
        })
        .collect()
}

/// Lower-case the first character of a string (for embedding in sentences).
fn lowercase_first_char(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_lowercase().to_string() + chars.as_str(),
    }
}

/// Expert-level terse description: remove articles and filler words.
fn make_terse(task: &str) -> String {
    let stopwords: std::collections::HashSet<&str> = [
        "a", "an", "the", "for", "with", "and", "in", "to", "from", "using", "into",
    ]
    .into_iter()
    .collect();
    let result: Vec<&str> = task
        .split_whitespace()
        .filter(|w| !stopwords.contains(&w.to_lowercase().as_str()))
        .collect();
    result.join(" ")
}

/// Detailed description: adds explanatory context from the reference args.
fn make_detailed(task: &str, reference_args: &str) -> String {
    let has_threads = reference_args.contains("-t ")
        || reference_args.contains("-@ ")
        || reference_args.contains("--threads")
        || reference_args.contains("-p ");
    let has_output = reference_args.contains("-o ") || reference_args.contains("--output");

    let mut desc = format!("I have data that I need to process: {}", task);
    if has_threads {
        desc.push_str(", utilizing multiple CPU threads for speed");
    }
    if has_output {
        desc.push_str(", saving the result to a specified output file");
    }
    desc
}

/// Casual / informal description.
fn make_casual(task: &str) -> String {
    let lc = lowercase_first_char(task);
    format!("Hey, can you help me {}?", lc)
}

/// Alternative phrasing: swap clause order or use synonyms.
fn make_alternative(task: &str) -> String {
    // Try to split on common conjunctions and swap.
    if let Some(pos) = task.find(" and ") {
        let (first, second) = task.split_at(pos);
        let second = &second[5..]; // skip " and "
        return format!("{} after {}", capitalize_first(second.trim()), first.trim());
    }
    if let Some(pos) = task.find(" with ") {
        let (first, second) = task.split_at(pos);
        let second = &second[6..]; // skip " with "
        return format!(
            "With {}, {}",
            second.trim(),
            lowercase_first_char(first.trim())
        );
    }
    if let Some(pos) = task.find(" to ") {
        let (first, second) = task.split_at(pos);
        let second = &second[4..]; // skip " to "
        return format!(
            "Output {} by performing: {}",
            second.trim(),
            lowercase_first_char(first.trim())
        );
    }
    // Fallback: prepend "Perform: ".
    format!("Perform: {}", task)
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

// ── CSV export ───────────────────────────────────────────────────────────────

/// Write reference-command scenarios to CSV.
///
/// Columns: `tool,scenario_id,reference_args,task_description,category`
pub fn write_scenarios_csv<W: Write>(
    writer: &mut W,
    scenarios: &[Scenario],
) -> std::io::Result<()> {
    writeln!(
        writer,
        "tool,scenario_id,reference_args,task_description,category"
    )?;
    for s in scenarios {
        let args_esc = csv_escape(&s.reference_args);
        let task_esc = csv_escape(&s.task_description);
        writeln!(
            writer,
            "{},{},{},{},{}",
            s.tool, s.scenario_id, args_esc, task_esc, s.category
        )?;
    }
    Ok(())
}

/// Write usage descriptions to CSV.
///
/// Columns: `tool,scenario_id,desc_id,user_level,description`
pub fn write_descriptions_csv<W: Write>(
    writer: &mut W,
    descriptions: &[UsageDescription],
) -> std::io::Result<()> {
    writeln!(writer, "tool,scenario_id,desc_id,user_level,description")?;
    for d in descriptions {
        let desc_esc = csv_escape(&d.description);
        writeln!(
            writer,
            "{},{},{},{},{}",
            d.tool, d.scenario_id, d.desc_id, d.user_level, desc_esc
        )?;
    }
    Ok(())
}

/// RFC 4180 CSV field escaping: double any internal quotes and wrap in quotes.
fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SKILL: &str = r#"---
name: testtool
category: testing
description: A test tool for unit tests
tags: [test, unit]
author: oxo-call built-in
source_url: "https://example.com"
---

## Concepts

- Concept one for testing.
- Concept two for testing.

## Pitfalls

- Pitfall one.

## Examples

### run basic analysis on input.txt
**Args:** `analyze -i input.txt -o output.txt`
**Explanation:** basic analysis with input/output

### run analysis with 4 threads
**Args:** `analyze -i input.txt -o output.txt -t 4`
**Explanation:** multi-threaded analysis

### run verbose analysis
**Args:** `analyze -v -i input.txt`
**Explanation:** verbose mode shows detailed progress

### generate summary report
**Args:** `report --summary input.txt`
**Explanation:** creates a concise summary

### merge two files
**Args:** `merge file1.txt file2.txt -o merged.txt`
**Explanation:** combine two input files
"#;

    #[test]
    fn test_parse_skill_file_metadata() {
        let skill = parse_skill_file(SAMPLE_SKILL).unwrap();
        assert_eq!(skill.name, "testtool");
        assert_eq!(skill.category, "testing");
        assert_eq!(skill.description, "A test tool for unit tests");
        assert_eq!(skill.tags, vec!["test", "unit"]);
        assert_eq!(skill.source_url, "https://example.com");
    }

    #[test]
    fn test_parse_skill_file_examples() {
        let skill = parse_skill_file(SAMPLE_SKILL).unwrap();
        assert_eq!(skill.examples.len(), 5);
        assert_eq!(skill.examples[0].task, "run basic analysis on input.txt");
        assert_eq!(skill.examples[0].args, "analyze -i input.txt -o output.txt");
        assert_eq!(
            skill.examples[0].explanation,
            "basic analysis with input/output"
        );
    }

    #[test]
    fn test_generate_scenarios_pads_to_ten() {
        let skill = parse_skill_file(SAMPLE_SKILL).unwrap();
        let scenarios = generate_scenarios(&skill);
        assert_eq!(
            scenarios.len(),
            SCENARIOS_PER_TOOL,
            "should always produce exactly {} scenarios",
            SCENARIOS_PER_TOOL
        );
        // First 5 come from the original examples.
        assert_eq!(
            scenarios[0].reference_args,
            "analyze -i input.txt -o output.txt"
        );
        // All scenario IDs are unique.
        let ids: std::collections::HashSet<&str> =
            scenarios.iter().map(|s| s.scenario_id.as_str()).collect();
        assert_eq!(ids.len(), SCENARIOS_PER_TOOL);
    }

    #[test]
    fn test_generate_scenarios_truncates_to_ten() {
        // Build a skill with 14 examples.
        let mut skill = parse_skill_file(SAMPLE_SKILL).unwrap();
        for i in 6..=14 {
            skill.examples.push(SkillExample {
                task: format!("extra task {i}"),
                args: format!("extra --arg {i}"),
                explanation: format!("explanation {i}"),
            });
        }
        assert_eq!(skill.examples.len(), 14);
        let scenarios = generate_scenarios(&skill);
        assert_eq!(scenarios.len(), SCENARIOS_PER_TOOL);
    }

    #[test]
    fn test_generate_descriptions_count() {
        let scenario = Scenario {
            tool: "testtool".to_string(),
            scenario_id: "testtool_01".to_string(),
            reference_args: "analyze -i input.txt -o output.txt".to_string(),
            task_description: "run basic analysis on input.txt".to_string(),
            category: "testing".to_string(),
        };
        let descs = generate_descriptions(&scenario);
        assert_eq!(descs.len(), DESCRIPTIONS_PER_SCENARIO);
        // All desc IDs are unique.
        let ids: std::collections::HashSet<&str> =
            descs.iter().map(|d| d.desc_id.as_str()).collect();
        assert_eq!(ids.len(), DESCRIPTIONS_PER_SCENARIO);
    }

    #[test]
    fn test_generate_descriptions_user_levels() {
        let scenario = Scenario {
            tool: "testtool".to_string(),
            scenario_id: "testtool_01".to_string(),
            reference_args: "analyze -i input.txt".to_string(),
            task_description: "run basic analysis on input.txt".to_string(),
            category: "testing".to_string(),
        };
        let descs = generate_descriptions(&scenario);
        let levels: Vec<&str> = descs.iter().map(|d| d.user_level.as_str()).collect();
        assert_eq!(levels, USER_LEVELS);
    }

    #[test]
    fn test_generate_descriptions_diversity() {
        let scenario = Scenario {
            tool: "samtools".to_string(),
            scenario_id: "samtools_01".to_string(),
            reference_args: "sort -@ 4 -o sorted.bam input.bam".to_string(),
            task_description: "sort a BAM file by genomic coordinates".to_string(),
            category: "alignment".to_string(),
        };
        let descs = generate_descriptions(&scenario);
        // All descriptions should be non-empty.
        for d in &descs {
            assert!(!d.description.is_empty(), "description should be non-empty");
        }
        // Descriptions should be diverse (not all identical).
        let unique: std::collections::HashSet<&str> =
            descs.iter().map(|d| d.description.as_str()).collect();
        assert!(
            unique.len() >= 8,
            "at least 8 unique descriptions expected, got {}",
            unique.len()
        );
    }

    #[test]
    fn test_write_scenarios_csv() {
        let scenarios = vec![Scenario {
            tool: "testtool".to_string(),
            scenario_id: "testtool_01".to_string(),
            reference_args: "analyze -i input.txt".to_string(),
            task_description: "run analysis".to_string(),
            category: "testing".to_string(),
        }];
        let mut buf = Vec::new();
        write_scenarios_csv(&mut buf, &scenarios).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("tool,scenario_id,reference_args,task_description,category"));
        assert!(text.contains("testtool,testtool_01"));
    }

    #[test]
    fn test_write_descriptions_csv() {
        let descs = vec![UsageDescription {
            tool: "testtool".to_string(),
            scenario_id: "testtool_01".to_string(),
            desc_id: "testtool_01_01".to_string(),
            user_level: "beginner".to_string(),
            description: "How do I run analysis?".to_string(),
        }];
        let mut buf = Vec::new();
        write_descriptions_csv(&mut buf, &descs).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("tool,scenario_id,desc_id,user_level,description"));
        assert!(text.contains("testtool_01_01"));
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(csv_escape("simple"), "simple");
        assert_eq!(csv_escape("has, comma"), "\"has, comma\"");
        assert_eq!(csv_escape("has \"quote\""), "\"has \"\"quote\"\"\"");
    }

    #[test]
    fn test_make_terse() {
        assert_eq!(
            make_terse("sort a BAM file by genomic coordinates"),
            "sort BAM file by genomic coordinates"
        );
    }

    #[test]
    fn test_make_alternative_with_and() {
        let alt = make_alternative("sort a BAM file and create an index");
        assert!(alt.contains("Create"));
        assert!(alt.contains("sort"));
    }

    #[test]
    fn test_make_alternative_with_with() {
        let alt = make_alternative("align reads with 8 threads");
        assert!(alt.starts_with("With 8 threads"));
    }

    #[test]
    fn test_make_alternative_with_to() {
        let alt = make_alternative("convert SAM to BAM format");
        assert!(alt.contains("BAM format"));
    }

    #[test]
    fn test_make_alternative_fallback() {
        let alt = make_alternative("index sorted.bam");
        assert!(alt.starts_with("Perform:"));
    }

    #[test]
    fn test_full_pipeline_skill_to_descriptions() {
        let skill = parse_skill_file(SAMPLE_SKILL).unwrap();
        let scenarios = generate_scenarios(&skill);
        assert_eq!(scenarios.len(), SCENARIOS_PER_TOOL);

        let mut all_descs = Vec::new();
        for scenario in &scenarios {
            let descs = generate_descriptions(scenario);
            assert_eq!(descs.len(), DESCRIPTIONS_PER_SCENARIO);
            all_descs.extend(descs);
        }
        assert_eq!(
            all_descs.len(),
            SCENARIOS_PER_TOOL * DESCRIPTIONS_PER_SCENARIO,
            "should produce exactly {} descriptions per tool",
            SCENARIOS_PER_TOOL * DESCRIPTIONS_PER_SCENARIO
        );
    }

    #[test]
    fn test_load_skills_from_repo_dir() {
        // This test uses the real skills/ directory from the repository.
        let skills_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("skills");
        if !skills_dir.exists() {
            return; // skip if not in the repo
        }
        let skills = load_skills_from_dir(&skills_dir).unwrap();
        assert!(
            skills.len() >= 100,
            "expected at least 100 skills from repo, got {}",
            skills.len()
        );
        // Every skill should have at least 1 example.
        for skill in &skills {
            assert!(
                !skill.examples.is_empty(),
                "skill '{}' should have examples",
                skill.name
            );
        }
    }

    #[test]
    fn test_synthesise_variant_no_append_after_pipe() {
        let skill = SkillFile {
            name: "bcftools".to_string(),
            category: "variant-calling".to_string(),
            description: "".to_string(),
            tags: vec![],
            source_url: "".to_string(),
            examples: vec![],
        };
        let base = Scenario {
            tool: "bcftools".to_string(),
            scenario_id: "bcftools_01".to_string(),
            reference_args: "mpileup -f ref.fa input.bam | bcftools call -mv -o out.vcf"
                .to_string(),
            task_description: "call variants".to_string(),
            category: "variant-calling".to_string(),
        };
        // All variant types except DEFAULT should produce no args suffix
        // when the base command contains a pipe.
        for idx in 0..NUM_VARIANT_TYPES {
            let v = synthesise_variant(&base, idx);
            // The args should NOT have extra flags appended after the pipe.
            assert!(
                !v.reference_args.ends_with("--verbose")
                    && !v.reference_args.ends_with("-t 4")
                    && !v.reference_args.ends_with("-o output.txt")
                    && !v.reference_args.ends_with("--quiet"),
                "variant idx={idx} should not append flags after pipe: {}",
                v.reference_args
            );
        }
    }

    #[test]
    fn test_synthesise_variant_no_append_after_redirect() {
        let skill = SkillFile {
            name: "samtools".to_string(),
            category: "alignment".to_string(),
            description: "".to_string(),
            tags: vec![],
            source_url: "".to_string(),
            examples: vec![],
        };
        let base = Scenario {
            tool: "samtools".to_string(),
            scenario_id: "samtools_01".to_string(),
            reference_args: "stats input.bam > stats.txt".to_string(),
            task_description: "get stats".to_string(),
            category: "alignment".to_string(),
        };
        for idx in 0..NUM_VARIANT_TYPES {
            let v = synthesise_variant(&base, idx);
            assert!(
                !v.reference_args.ends_with("--verbose")
                    && !v.reference_args.ends_with("-t 4")
                    && !v.reference_args.ends_with("-o output.txt")
                    && !v.reference_args.ends_with("--quiet"),
                "variant idx={idx} should not append flags after redirect: {}",
                v.reference_args
            );
        }
    }

    #[test]
    fn test_synthesise_variant_skips_threads_when_t_present() {
        let skill = SkillFile {
            name: "samtools".to_string(),
            category: "alignment".to_string(),
            description: "".to_string(),
            tags: vec![],
            source_url: "".to_string(),
            examples: vec![],
        };
        let base = Scenario {
            tool: "samtools".to_string(),
            scenario_id: "samtools_01".to_string(),
            reference_args: "sort -@ 4 -o sorted.bam input.bam".to_string(),
            task_description: "sort BAM".to_string(),
            category: "alignment".to_string(),
        };
        // VARIANT_THREADS (idx=1) should not add -t 4 because -@ is present.
        let v = synthesise_variant(&base, VARIANT_THREADS);
        assert!(
            !v.reference_args.contains("-t 4"),
            "should not add -t 4 when -@ is already present"
        );
    }
}
