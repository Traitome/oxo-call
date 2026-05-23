//! Unified prompt builder for the 3-stage pipeline.
//!
//! Single prompt format for all models. Cross-domain few-shot examples
//! teach general CLI structure. Chain-of-thought reduces hallucination.

use crate::doc_processor::{FlagEntry, StructuredDoc};
use crate::skill::Skill;

// ─── System prompt ────────────────────────────────────────────────────────────

/// Unified system prompt — all models, single format.
pub fn system_prompt_unified() -> &'static str {
    "You are a CLI expert. Convert tasks to exact command-line arguments.\n\
     Output format:\n\
     think: <brief reasoning>\n\
     ARGS: <arguments without tool name>"
}

// ─── Cross-domain few-shot examples ───────────────────────────────────────────

/// Cross-domain examples that teach general CLI structure.
/// Each entry: (task_description, correct_args_without_tool_name)
const CROSS_DOMAIN_EXAMPLES: &[(&str, &str)] = &[
    // File operations
    (
        "copy directory recursively to backup",
        "-r source_dir/ /backup/",
    ),
    ("move file to new location", "file.txt /new/path/"),
    // Text processing
    (
        "search for ERROR in all .log files recursively",
        "-rn ERROR *.log",
    ),
    (
        "sort CSV by second column numerically",
        "-t',' -k2,2 -n data.csv",
    ),
    ("count unique lines in a file", "-c input.txt | sort -rn"),
    // Version control
    (
        "clone a git repository",
        "clone https://github.com/user/repo.git",
    ),
    (
        "commit all changes with message",
        "commit -am 'fix: resolve bug'",
    ),
    ("show recent commit history", "log --oneline -10"),
    // System
    (
        "list processes sorted by memory usage",
        "aux --sort=-%mem | head -20",
    ),
    ("show disk usage of each subdirectory", "-sh */"),
    ("check available disk space", "-h /data/"),
    // Compression
    (
        "create tar.gz archive of a directory",
        "-czf archive.tar.gz /path/to/dir/",
    ),
    ("compress a file keeping the original", "-k input.fastq"),
    // Network
    (
        "download a file from URL",
        "-L -o output.tar.gz https://example.com/file.tar.gz",
    ),
    // Bioinformatics (minimal, structural examples only)
    (
        "align paired reads to reference with threads",
        "mem -t 4 reference.fa reads_1.fq reads_2.fq > out.sam",
    ),
    (
        "sort a BAM file by coordinate",
        "sort -@ 4 -o sorted.bam input.bam",
    ),
    ("index a sorted BAM file", "index sorted.bam"),
    (
        "call variants from BAM with reference",
        "HaplotypeCaller -R ref.fa -I input.bam -O output.vcf",
    ),
    (
        "filter VCF by quality threshold",
        "filter -i 'QUAL>30' -o output.vcf input.vcf",
    ),
    (
        "trim adapters from FASTQ",
        "-a AGATCGGAAGAG -o output.fq input.fq",
    ),
    (
        "quantify gene expression with index",
        "quant -i index -l A -1 reads_1.fq -2 reads_2.fq -o output_dir --threads 4",
    ),
    (
        "assemble genome from paired reads",
        "-t 4 -1 reads_1.fq -2 reads_2.fq -o output_dir",
    ),
    ("run quality control on FASTQ file", "input.fastq"),
    (
        "convert GFF3 to GTF format",
        "--gff input.gff3 -o output.gtf",
    ),
];

// ─── Prompt builder ───────────────────────────────────────────────────────────

/// Build the unified prompt for all models and modes.
pub fn build_unified_prompt(
    tool: &str,
    task: &str,
    sdoc: &StructuredDoc,
    skill: Option<&Skill>,
) -> String {
    let mut prompt = String::new();
    let task_values = super::task_values::extract_task_values(task);

    prompt.push_str(&format!("Tool: {tool}\n"));
    prompt.push_str(&format!("Task: {task}\n"));

    // ── Subcommand hint ──────────────────────────────────────────
    if sdoc.has_subcommands && !sdoc.subcommands.is_empty() {
        let best = find_best_subcommand_for_task(task, sdoc);
        if let Some(ref sub) = best {
            prompt.push_str(&format!("Subcommand: {sub}\n"));
        } else if sdoc.subcommands.len() <= 5 {
            prompt.push_str(&format!("Subcommand: {}\n", sdoc.subcommands.join(", ")));
        }
    } else if !sdoc.has_subcommands {
        prompt.push_str("(no subcommand)\n");
    }
    if !sdoc.companion_binaries.is_empty() {
        prompt.push_str(&format!("Binary: {}\n", sdoc.companion_binaries[0]));
    }

    // ── Flag list (max 5, values pre-assigned) ────────────────────
    let required: Vec<&FlagEntry> = sdoc
        .flag_catalog
        .iter()
        .filter(|e| e.required)
        .take(3)
        .collect();
    if !required.is_empty() {
        prompt.push_str("\nRequired:\n");
        for f in &required {
            let val = assign_value_to_flag(f, &task_values);
            prompt.push_str(&format!("  {}{}\n", f.flag, val));
        }
    }

    let task_lower = task.to_ascii_lowercase();
    let task_kw: Vec<&str> = task_lower
        .split_whitespace()
        .filter(|w| w.len() >= 2 && !w.contains('.'))
        .collect();
    let mut opt: Vec<&FlagEntry> = sdoc.flag_catalog.iter().filter(|e| !e.required).collect();
    opt.sort_by(|a, b| {
        let sa = flag_score(a, &task_kw, &task_lower);
        let sb = flag_score(b, &task_kw, &task_lower);
        sb.cmp(&sa)
    });

    let remaining = 5usize.saturating_sub(required.len());
    let top_opt: Vec<&FlagEntry> = opt.into_iter().take(remaining).collect();
    if !top_opt.is_empty() {
        if required.is_empty() {
            prompt.push_str("\nFlags:\n");
        } else {
            prompt.push_str("Optional:\n");
        }
        for f in &top_opt {
            let val = assign_value_to_flag(f, &task_values);
            prompt.push_str(&format!("  {}{}\n", f.flag, val));
        }
    }

    // ── Cross-domain example ──────────────────────────────────────
    let cross = pick_cross_domain_example(task);
    prompt.push_str(&format!(
        "\nExample:\n  Task: {}\n  ARGS: {}\n",
        cross.0, cross.1
    ));

    // ── Tool-specific example (skill or doc-extracted) ────────────
    let tool_example: Option<&str> = skill
        .and_then(|s| {
            s.select_examples(1, Some(task))
                .first()
                .map(|e| e.args.as_str())
        })
        .or_else(|| sdoc.extracted_examples.first().map(|s| s.as_str()));
    if let Some(args) = tool_example {
        prompt.push_str(&format!("\n{}-specific:\n  ARGS: {args}\n", tool));
    }

    prompt.push_str("\nThink step by step, then output:\nthink: <reasoning>\nARGS:");
    prompt
}

// ─── Subcommand matching ──────────────────────────────────────────────────────

/// Find the best subcommand match for a task from the structured doc.
pub fn find_best_subcommand_for_task(task: &str, sdoc: &StructuredDoc) -> Option<String> {
    let task_lower = task.to_ascii_lowercase();
    let task_keywords: Vec<&str> = task_lower
        .split_whitespace()
        .filter(|w| w.len() >= 2 && !w.contains('.') && !w.starts_with('('))
        .collect();

    let mut best: Option<(String, i32)> = None;

    for sub in &sdoc.subcommands {
        let sub_lower = sub.to_ascii_lowercase();
        let mut score = 0i32;

        if task_keywords.iter().any(|w| *w == sub_lower) {
            score += 25;
        }
        if task_lower.contains(&sub_lower) {
            score += 20;
        }

        for part in sub_lower.split(|c: char| c == '_' || c == '-') {
            if part.len() < 2 {
                continue;
            }
            for kw in &task_keywords {
                if kw == &part {
                    score += 15;
                } else if kw.contains(part) && part.len() >= 3 {
                    score += 8;
                } else if part.contains(kw) && kw.len() >= 3 {
                    score += 8;
                } else if kw.len() >= 4
                    && part.len() >= 4
                    && (kw.starts_with(part) || part.starts_with(kw))
                {
                    score += 5;
                }
                // Stem matching
                if let Some(kw_stem) = kw.strip_suffix('s') {
                    if kw_stem.len() >= 2
                        && (kw_stem == part || part.contains(kw_stem) || kw_stem.contains(part))
                    {
                        score += 6;
                    }
                }
                if let Some(part_stem) = part.strip_suffix('s') {
                    if part_stem.len() >= 2
                        && (part_stem == *kw || kw.contains(part_stem) || part_stem.contains(kw))
                    {
                        score += 6;
                    }
                }
            }
        }

        for (desc_sub, desc) in &sdoc.subcommand_descriptions {
            if desc_sub == sub && !desc.is_empty() {
                let desc_lower = desc.to_ascii_lowercase();
                for kw in &task_keywords {
                    if kw.len() >= 3 && desc_lower.contains(kw) {
                        score += 10;
                    }
                }
            }
        }

        if score > 0 {
            match &best {
                Some((_, s)) if score <= *s => {}
                _ => best = Some((sub.clone(), score)),
            }
        }
    }

    best.map(|(sub, _)| sub)
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Assign a task value to a flag based on its type/description.
fn assign_value_to_flag(entry: &FlagEntry, tv: &super::task_values::TaskValues) -> String {
    let desc = entry.description.to_ascii_lowercase();
    let flag = entry.flag.to_ascii_lowercase();

    if desc.contains("output") || flag.contains("-o") || flag.contains("out") {
        if let Some(f) = tv.output_files.first() {
            return format!(" {}", f);
        }
    }
    if desc.contains("thread") || desc.contains("cpu") || flag == "-@" || flag == "-p" {
        for n in &tv.numbers {
            if let Ok(v) = n.parse::<u32>() {
                if v <= 128 {
                    return format!(" {n}");
                }
            }
        }
    }
    if desc.contains("input") || flag == "-i" || flag == "-f" || flag == "-1" {
        if let Some(f) = tv.input_files.first() {
            return format!(" {f}");
        }
    }
    if desc.contains("reference") || desc.contains("genome") || flag == "-x" || flag == "-r" {
        if let Some(f) = tv.reference_files.first() {
            return format!(" {f}");
        }
        for f in &tv.input_files {
            let fl = f.to_ascii_lowercase();
            if fl.ends_with(".fa") || fl.ends_with(".fasta") || fl.ends_with(".fna") {
                return format!(" {f}");
            }
        }
    }
    if let Some(ref vt) = entry.value_type {
        format!(" <{vt}>")
    } else {
        String::new()
    }
}

/// Score a flag's relevance to the task.
fn flag_score(entry: &FlagEntry, kw: &[&str], task_lower: &str) -> i32 {
    let dl = entry.description.to_ascii_lowercase();
    let fl = entry.flag.to_ascii_lowercase();
    let mut s = 0i32;
    for k in kw {
        if dl.contains(k) {
            s += 2;
        }
        if fl.contains(k) {
            s += 1;
        }
    }
    if dl.contains("output")
        && (task_lower.contains("output")
            || task_lower.contains("save")
            || task_lower.contains("write"))
    {
        s += 3;
    }
    if (dl.contains("thread") || dl.contains("cpu"))
        && (task_lower.contains("thread") || task_lower.contains("cpu"))
    {
        s += 3;
    }
    s
}

/// Pick a cross-domain example structurally different from the current task.
fn pick_cross_domain_example(task: &str) -> (&'static str, &'static str) {
    let tl = task.to_ascii_lowercase();
    if tl.contains("copy") || tl.contains("move") || tl.contains("rename") {
        CROSS_DOMAIN_EXAMPLES[2] // text processing
    } else if tl.contains("search") || tl.contains("find") || tl.contains("grep") {
        CROSS_DOMAIN_EXAMPLES[0] // file ops
    } else if tl.contains("commit") || tl.contains("clone") || tl.contains("push") {
        CROSS_DOMAIN_EXAMPLES[6] // system
    } else if tl.contains("download") || tl.contains("curl") || tl.contains("fetch") {
        CROSS_DOMAIN_EXAMPLES[1] // file ops
    } else if tl.contains("compress") || tl.contains("archive") || tl.contains("tar") {
        CROSS_DOMAIN_EXAMPLES[14] // network
    } else {
        // Default: pick one that's structurally different from bioinformatics
        CROSS_DOMAIN_EXAMPLES[0] // "copy directory recursively to backup"
    }
}

// ─── Backward compatibility wrappers ───────────────────────────────────────────

/// Legacy wrapper — delegates to unified prompt builder.
pub fn build_prompt(
    tool: &str,
    _documentation: &str,
    task: &str,
    skill: Option<&Skill>,
    _no_prompt: bool,
    _context_window: u32,
    _tier: super::types::PromptTier,
    structured_doc: Option<&StructuredDoc>,
) -> String {
    match structured_doc {
        Some(sdoc) => build_unified_prompt(tool, task, sdoc, skill),
        None => format!("Tool: {tool}\nTask: {task}\n\nARGS:"),
    }
}

/// Legacy — kept for compilation.
pub fn system_prompt() -> &'static str {
    system_prompt_unified()
}
pub fn system_prompt_medium() -> &'static str {
    system_prompt_unified()
}
pub fn system_prompt_compact() -> &'static str {
    system_prompt_unified()
}

/// Legacy — uses unified prompt.
pub fn build_retry_prompt(
    tool: &str,
    _documentation: &str,
    task: &str,
    skill: Option<&Skill>,
    _prev_raw: &str,
    _no_prompt: bool,
    _context_window: u32,
    _tier: super::types::PromptTier,
) -> String {
    format!(
        "Tool: {tool}\nTask: {task}\n\nYour previous output was invalid. Output ONLY:\nARGS: <arguments>"
    )
}

/// Legacy — kept for compilation.  
pub fn prompt_tier(_context_window: u32, _model: &str) -> super::types::PromptTier {
    super::types::PromptTier::Slim
}

/// Legacy — kept for workflow module.
pub fn verification_system_prompt() -> &'static str {
    "You are an expert bioinformatics QC analyst."
}
pub fn skill_reviewer_system_prompt() -> &'static str {
    "You are an expert bioinformatics skill author."
}
pub fn build_verification_prompt(
    tool: &str,
    task: &str,
    command: &str,
    exit_code: i32,
    stderr: &str,
    output_files: &[(String, Option<u64>)],
) -> String {
    format!("Verify: {tool} {task}\nCommand: {command}\nExit: {exit_code}\nStderr: {stderr}")
}
pub fn build_skill_verify_prompt(tool: &str, skill_content: &str) -> String {
    format!("Verify skill for {tool}:\n{skill_content}")
}
pub fn build_skill_polish_prompt(tool: &str, skill_content: &str) -> String {
    format!("Polish skill for {tool}:\n{skill_content}")
}
pub fn build_skill_generate_prompt(tool: &str) -> String {
    format!("Generate skill for {tool}")
}

/// Legacy — kept for tests.
pub fn split_into_sections(docs: &str) -> Vec<&str> {
    docs.split("\n\n")
        .filter(|s| !s.trim().is_empty())
        .collect()
}

pub fn estimate_tokens(text: &str) -> usize {
    text.len().div_ceil(4)
}

// ─── Skill-to-Schema Extraction ──────────────────────────────────────────────

/// Build a minimal StructuredDoc from skill file content when --help output
/// is unavailable. Extracts flags from skill examples and concepts.
pub fn build_schema_from_skill(skill: &Skill) -> StructuredDoc {
    let mut flags: Vec<FlagEntry> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Extract flags from skill examples
    for ex in &skill.examples {
        let parts: Vec<&str> = ex.args.split_whitespace().collect();
        let mut i = 0;
        while i < parts.len() {
            let part = parts[i];
            if part.starts_with('-') {
                let flag = part.split('=').next().unwrap_or(part).to_string();
                if seen.insert(flag.clone()) {
                    let value_type = if part.contains('=')
                        || (i + 1 < parts.len() && !parts[i + 1].starts_with('-'))
                    {
                        Some("VALUE".to_string())
                    } else {
                        None
                    };
                    let mut desc = String::new();
                    if !ex.explanation.is_empty() {
                        let expl_lower = ex.explanation.to_ascii_lowercase();
                        if let Some(pos) = expl_lower.find(&flag.to_ascii_lowercase()) {
                            let end = expl_lower[pos..]
                                .find('.')
                                .unwrap_or(expl_lower.len() - pos);
                            desc = expl_lower[pos..pos + end.min(80)].to_string();
                        }
                    }
                    flags.push(FlagEntry {
                        flag,
                        value_type,
                        description: desc,
                        required: false,
                        default: None,
                        alt_form: None,
                        enum_values: Vec::new(),
                    });
                }
            }
            i += 1;
        }
    }

    // Extract subcommands from skill pitfalls (first line mentions subcommands)
    let mut subcommands = Vec::new();
    let mut has_subcommands = false;
    for pitfall in &skill.context.pitfalls {
        if pitfall.contains("must start with a subcommand")
            || pitfall.contains("ALWAYS comes first")
        {
            has_subcommands = true;
            // Try to extract subcommand list from parentheses
            if let Some(start) = pitfall.find('(') {
                if let Some(end) = pitfall[start..].find(')') {
                    let subs_str = &pitfall[start + 1..start + end];
                    for sub in subs_str.split([',', ' ']) {
                        let sub = sub.trim();
                        if !sub.is_empty() && sub.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            subcommands.push(sub.to_string());
                        }
                    }
                }
            }
        }
    }

    // Mark common required flags
    for f in &mut flags {
        let fl = f.flag.to_ascii_lowercase();
        if fl == "-o" || fl.contains("output") || fl == "-@" || fl.contains("thread") {
            f.required = true;
        }
    }

    StructuredDoc {
        has_subcommands,
        subcommands,
        flag_catalog: flags,
        usage_pattern: Default::default(),
        format_hint: None,
        companion_binaries: Vec::new(),
        extracted_examples: skill.examples.iter().map(|e| e.args.clone()).collect(),
        ..Default::default()
    }
}
