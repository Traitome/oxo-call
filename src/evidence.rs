//! Evidence-graded prompt construction for oxo-call.
//!
//! The core architectural innovation: every command-generation prompt is built
//! from explicit, graded evidence blocks. The LLM receives clear authority
//! rules so it knows to prefer `--help` output over skill files over internet
//! knowledge.
//!
//! ## Evidence hierarchy
//!
//! | Level | Name   | Source                    | Authority |
//! |-------|--------|---------------------------|-----------|
//! | L0    | Live   | `--help` output (cached)  | Authoritative — ground truth |
//! | L1    | Curated| Skill file (.md)          | Expert-curated, verified    |
//! | L2    | Indexed| Bioconda/docs (vector KB) | Reference, may be stale     |
//! | L3    | Model  | LLM training data         | Fallback only               |
//! | L4    | Graph  | Tool relationship graph   | Context, not direct evidence|

use crate::doc_processor::StructuredDoc;
use crate::flag_extractor::FlagCatalog;
use crate::skill::Skill;

/// A single piece of evidence used in prompt construction.
#[derive(Debug, Clone)]
pub struct EvidenceBlock {
    /// L0, L1, L2, L3, or L4.
    pub level: String,
    /// Human-readable source label (e.g., "samtools v1.21 --help").
    pub title: String,
    /// The actual content to include in the prompt.
    pub content: String,
    /// How the LLM should treat this evidence.
    pub instruction: String,
}

impl EvidenceBlock {
    pub fn to_prompt_string(&self) -> String {
        format!(
            "<!-- EVIDENCE L{}: {} -->\n\
             <!-- {} -->\n\
             {}\n\
             <!-- END EVIDENCE -->\n",
            self.level, self.title, self.instruction, self.content
        )
    }
}

/// Build the evidence-graded system prompt.
///
/// This is the top-level system prompt injected before all evidence blocks.
/// It teaches the LLM the evidence hierarchy and how to resolve conflicts.
pub fn evidence_graded_system_prompt() -> &'static str {
    "You are a bioinformatics CLI assistant. Translate the task into command-line arguments \
     for the specified tool. Understand any language.\n\
     \n\
     FORMAT: Respond with EXACTLY two lines, nothing else:\n\
     ARGS: <subcommand then flags and values — NO tool name, NO markdown>\n\
     EXPLANATION: <one sentence in the task's language>\n\
     \n\
     EVIDENCE RULES (highest authority first):\n\
     L0 [AUTHORITATIVE]: The tool's live --help output. This is GROUND TRUTH. \
     Use ONLY flags that appear here. Match the exact flag format (--flag=value or --flag value).\n\
     L1 [CURATED]: Expert-written skill file with concepts, pitfalls, and examples. \
     Trust this over L2/L3. If L0 and L1 conflict, prefer L0 (the tool may have changed).\n\
     L2 [REFERENCE]: Internet documentation and bioconda recipes. Useful context but may be outdated.\n\
     L3 [FALLBACK]: General knowledge. Only use when L0/L1/L2 provide no information.\n\
     L4 [CONTEXT]: Related tools and common pipeline patterns. For background only.\n\
     \n\
     CORE RULES:\n\
     1. NEVER start ARGS with the tool name (auto-prepended by system).\n\
     2. First token = subcommand (sort, view, mem, index, etc), NEVER a flag.\n\
     3. Companion binaries (e.g. bowtie2-build) go as first token when indicated by L0/L1.\n\
     4. Use ONLY flags from L0 (--help) or L1 (skill examples). NEVER invent flags.\n\
     5. Default conventions: paired-end, coordinate-sorted BAM, hg38, gzipped FASTQ, Phred+33.\n\
     6. Include every file/path from the task. Add threads and output flags when applicable.\n\
     7. Generate one target-tool invocation. Do not invent dependent steps or a DAG.\n\
     8. If no arguments needed: ARGS: (none)."
}

/// Compact system prompt for small models.
pub fn evidence_graded_system_prompt_compact() -> &'static str {
    "Translate tasks into CLI args. Output EXACTLY two lines:\n\
     ARGS: <flags, NO tool name>\n\
     EXPLANATION: <one sentence>\n\
     Priority: L0 (--help) > L1 (skill) > L2 (docs) > L3 (knowledge). \
     Use only flags from L0/L1. Subcommand first."
}

/// Build the evidence-graded user prompt for command generation.
///
/// Evidence blocks are ordered L0 → L1 → L2 → L4 (L3 is implicit — the LLM's own knowledge).
/// Each block is clearly marked with its authority level and conflict-resolution instruction.
#[allow(clippy::too_many_arguments)]
pub fn build_evidence_graded_prompt(
    tool: &str,
    task: &str,
    // L0: Live --help output (or empty if not available)
    help_output: &str,
    // L1: Curated skill (or None)
    skill: Option<&Skill>,
    // L2: Indexed documentation from vector KB (or empty)
    indexed_docs: &str,
    // L4: Knowledge graph context — related tools and patterns (or empty)
    knowledge_context: &str,
    // Structured doc with flag catalog and extracted examples
    structured_doc: Option<&StructuredDoc>,
    // Flag catalog from --help parsing
    flag_catalog: Option<&FlagCatalog>,
) -> String {
    let mut sections: Vec<EvidenceBlock> = Vec::new();

    // ── L0: Live --help output ───────────────────────────────────────────
    if !help_output.is_empty() {
        sections.push(EvidenceBlock {
            level: "0".to_string(),
            title: format!("{tool} --help output (LIVE)"),
            content: truncate_help_for_prompt(help_output, 3000),
            instruction: "AUTHORITATIVE. This is the tool's actual --help output. \
                          Use ONLY flags that appear here. Match exact flag format. \
                          If any other evidence below conflicts with this, TRUST THIS."
                .to_string(),
        });
    }

    // ── L1: Curated skill ────────────────────────────────────────────────
    if let Some(s) = skill {
        let skill_text = s.to_prompt_section();
        if !skill_text.is_empty() {
            sections.push(EvidenceBlock {
                level: "1".to_string(),
                title: format!("Skill: {} ({})", s.meta.name, s.meta.category),
                content: skill_text,
                instruction: "CURATED. Expert-written guidance. Trust over L2/L3. \
                              If conflicts with L0 above, prefer L0 (the tool may have been updated)."
                    .to_string(),
            });
        }
    }

    // ── L2: Indexed documentation ────────────────────────────────────────
    if !indexed_docs.is_empty() {
        sections.push(EvidenceBlock {
            level: "2".to_string(),
            title: "Indexed documentation (bioconda, manuals)".to_string(),
            content: truncate_help_for_prompt(indexed_docs, 2000),
            instruction: "REFERENCE. May be outdated or for a different version. \
                          Prefer L0 and L1 over this. Use for context and understanding tool behavior."
                .to_string(),
        });
    }

    // ── Flag catalog (derived from L0) ───────────────────────────────────
    if let Some(catalog) = flag_catalog
        && !catalog.flags.is_empty()
    {
        let flag_list: Vec<String> = catalog
            .flags
            .iter()
            .take(30)
            .map(|f| {
                let name = f.long.as_deref().unwrap_or("?");
                format!("--{name}")
            })
            .collect();
        let subcmd_hint = if !catalog.subcommands.is_empty() {
            format!(
                "\nValid subcommands: {}",
                catalog
                    .subcommands
                    .iter()
                    .take(10)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            String::new()
        };
        sections.push(EvidenceBlock {
            level: "0".to_string(),
            title: "Flag catalog (extracted from --help)".to_string(),
            content: format!("Flags from --help: {}{}", flag_list.join(", "), subcmd_hint),
            instruction:
                "CONSTRAINT. These are the ONLY valid flags. Do not use any flag not listed here."
                    .to_string(),
        });
    }

    // ── Structured doc examples ──────────────────────────────────────────
    if let Some(sdoc) = structured_doc
        && !sdoc.extracted_examples.is_empty()
    {
        let examples: Vec<String> = sdoc
            .extracted_examples
            .iter()
            .take(5)
            .map(|ex| format!("  {tool} {ex}"))
            .collect();
        sections.push(EvidenceBlock {
            level: "0".to_string(),
            title: "Example commands from documentation".to_string(),
            content: format!("Real usage examples:\n{}", examples.join("\n")),
            instruction: "LEARN THE PATTERN. Match the flag order and format from these examples."
                .to_string(),
        });
    }

    // ── L4: Knowledge graph context ──────────────────────────────────────
    if !knowledge_context.is_empty() {
        sections.push(EvidenceBlock {
            level: "4".to_string(),
            title: "Tool relationship context".to_string(),
            content: knowledge_context.to_string(),
            instruction:
                "CONTEXT. Related tools and common patterns. For background understanding only."
                    .to_string(),
        });
    }

    // ── Assemble the prompt ──────────────────────────────────────────────
    let mut prompt = String::new();

    // Tool header
    prompt.push_str(&format!(
        "# Tool: `{tool}`\n\
         Generate command-line arguments to accomplish the task.\n\
         Evidence sources below are ordered by authority: L0 > L1 > L2 > L3 > L4.\n\n"
    ));

    // Evidence blocks
    for block in &sections {
        prompt.push_str(&block.to_prompt_string());
        prompt.push('\n');
    }

    // Task
    prompt.push_str(&format!(
        "## Task\n{task}\n\n\
         ## Output\n\
         ARGS: <subcommand then flags, NO tool name prefix>\n\
         EXPLANATION: <one sentence in the task's language>\n"
    ));

    prompt
}

/// Truncate help output to fit within prompt budget, preserving key sections.
fn truncate_help_for_prompt(help: &str, max_chars: usize) -> String {
    if help.len() <= max_chars {
        return help.to_string();
    }

    // Prioritize: USAGE line, then flag definitions, then examples
    let mut result = String::new();
    let mut remaining = max_chars.saturating_sub(50); // reserve for truncation marker

    // Always include USAGE
    for line in help.lines() {
        let trimmed = line.trim().to_lowercase();
        if trimmed.starts_with("usage:") || trimmed.starts_with("usage ") {
            if result.len() + line.len() < remaining {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(line);
                remaining = remaining.saturating_sub(line.len() + 1);
            }
            break;
        }
    }

    // Add flag definitions
    let mut flags_section = String::new();
    let mut in_options = false;
    for line in help.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Options:")
            || trimmed.starts_with("OPTIONS:")
            || trimmed.starts_with("Optional arguments:")
        {
            in_options = true;
            if flags_section.len() + line.len() < remaining {
                flags_section.push_str(line);
                flags_section.push('\n');
                remaining = remaining.saturating_sub(line.len() + 1);
            }
            continue;
        }
        if in_options && trimmed.is_empty() {
            break;
        }
        if in_options
            && (trimmed.starts_with('-') || trimmed.starts_with("--"))
            && flags_section.len() + line.len() < remaining
        {
            flags_section.push_str(line);
            flags_section.push('\n');
            remaining = remaining.saturating_sub(line.len() + 1);
        }
    }

    if !result.is_empty() && !flags_section.is_empty() {
        result.push('\n');
    }
    result.push_str(&flags_section);

    if result.len() < help.len() {
        result.push_str("\n[...truncated]");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_evidence() {
        let prompt = build_evidence_graded_prompt(
            "samtools",
            "sort input.bam",
            "",   // no help
            None, // no skill
            "",   // no indexed docs
            "",   // no knowledge context
            None, // no structured doc
            None, // no flag catalog
        );
        assert!(prompt.contains("Tool: `samtools`"));
        assert!(prompt.contains("sort input.bam"));
        assert!(prompt.contains("ARGS:"));
        // No L0 block since help is empty
        assert!(!prompt.contains("EVIDENCE L0"));
    }

    #[test]
    fn test_help_included_as_l0() {
        let prompt = build_evidence_graded_prompt(
            "samtools",
            "sort input.bam",
            "Usage: samtools sort [options]\n  -@ INT  threads\n  -o FILE output",
            None,
            "",
            "",
            None,
            None,
        );
        assert!(prompt.contains("EVIDENCE L0"));
        assert!(prompt.contains("AUTHORITATIVE"));
        assert!(prompt.contains("samtools sort"));
        assert!(prompt.contains("-@ INT"));
    }

    #[test]
    fn test_help_truncation() {
        let long_help = "Usage: tool\n".to_string()
            + &"Options:\n".to_string()
            + &(0..100)
                .map(|i| format!("  --flag-{}  description of flag {}\n", i, i))
                .collect::<Vec<_>>()
                .join("");
        let truncated = truncate_help_for_prompt(&long_help, 500);
        assert!(truncated.len() <= 520); // some slack for truncation marker
        assert!(truncated.contains("Usage:"));
        assert!(truncated.contains("[...truncated]"));
    }

    #[test]
    fn test_flag_catalog_in_prompt() {
        let catalog = FlagCatalog {
            flags: vec![crate::flag_extractor::FlagInfo {
                short: Some("o".into()),
                long: Some("output".into()),
                value_type: "FILE".into(),
                required: false,
                description: "output file".into(),
            }],
            subcommands: vec!["sort".into(), "index".into()],
            usage_line: String::new(),
        };
        let prompt = build_evidence_graded_prompt(
            "samtools",
            "sort input.bam",
            "",
            None,
            "",
            "",
            None,
            Some(&catalog),
        );
        assert!(prompt.contains("--output"));
        assert!(prompt.contains("Valid subcommands: sort, index"));
    }
}
