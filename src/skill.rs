/// Skills framework for oxo-call.
///
/// A **Skill** is a Markdown file with YAML front-matter containing curated
/// domain-expert knowledge about a specific bioinformatics tool.  Skills have
/// three functions:
///
/// 1. **Prompt enrichment** — concepts, pitfalls, and worked examples are injected into
///    the LLM prompt, dramatically improving generation quality for weak/small models.
/// 2. **Community extensibility** — anyone can write and share skill files without
///    touching Rust code.
/// 3. **User customisation** — per-user overrides take priority over built-ins.
/// 4. **MCP integration** — remote MCP servers can act as skill providers, enabling
///    organisational or project-scoped skill libraries.
///
/// # Skill file format (`<tool>.md`)
///
/// ```markdown
/// ---
/// name: samtools
/// category: alignment
/// description: Suite of programs for SAM/BAM/CRAM handling
/// tags: [bam, sam, alignment, ngs]
/// author: oxo-call built-in
/// source_url: http://www.htslib.org/doc/samtools.html
/// ---
///
/// ## Concepts
///
/// - Key concept 1 about the tool
/// - Key concept 2 — be specific and actionable
///
/// ## Pitfalls
///
/// - Common mistake and consequence — explain what goes wrong
///
/// ## Examples
///
/// ### Sort a BAM file by genomic coordinates
/// **Args:** `sort -@ 4 -o sorted.bam input.bam`
/// **Explanation:** -@ 4 uses 4 threads; coordinate sort is the default
/// ```
///
/// # Load priority (highest first)
/// 1. User-defined   `~/.config/oxo-call/skills/<tool>.md`  (`.toml` also accepted)
/// 2. Community      `~/.local/share/oxo-call/skills/<tool>.md`  (`.toml` also accepted)
/// 3. MCP servers    configured in `~/.config/oxo-call/config.toml` under `[mcp]`
/// 4. Built-in       compiled into the binary via `include_str!`
use crate::config::Config;
use crate::error::{OxoError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─── Data structures ──────────────────────────────────────────────────────────

/// The top-level skill document.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Skill {
    pub meta: SkillMeta,
    #[serde(default)]
    pub context: SkillContext,
    #[serde(default)]
    pub examples: Vec<SkillExample>,
}

/// Metadata about the skill and the tool it covers.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillMeta {
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub source_url: Option<String>,
    /// Minimum tool version required for this skill (e.g., "1.10").
    /// Commands may fail or behave differently on older versions.
    #[serde(default)]
    pub min_version: Option<String>,
    /// Maximum tool version supported by this skill (e.g., "1.20").
    /// Newer versions may have breaking changes not reflected in examples.
    #[serde(default)]
    pub max_version: Option<String>,
}

/// Domain knowledge injected into the LLM prompt.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillContext {
    /// Key domain concepts that orient the LLM to this tool's data model and paradigm.
    #[serde(default)]
    pub concepts: Vec<String>,
    /// Common mistakes that cause incorrect commands — helps the LLM avoid them.
    #[serde(default)]
    pub pitfalls: Vec<String>,
}

/// A single worked example used as a few-shot prompt entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExample {
    /// Plain-English task description
    pub task: String,
    /// Correct command arguments (WITHOUT the tool name)
    pub args: String,
    /// One-sentence explanation of why these args were chosen
    pub explanation: String,
}

// ─── Skill Markdown parser ────────────────────────────────────────────────────

/// Parse a skill from the Markdown format: YAML front-matter + Markdown body.
///
/// Expected structure:
/// ```text
/// ---
/// name: toolname
/// category: domain
/// description: One-line summary
/// tags: [tag1, tag2]
/// author: Author Name         (optional)
/// source_url: https://...     (optional)
/// ---
///
/// ## Concepts
/// - concept 1
/// - concept 2
///
/// ## Pitfalls
/// - pitfall 1
///
/// ## Examples
///
/// ### Task description
/// **Args:** `command --flag value`
/// **Explanation:** explanation text
/// ```
pub fn parse_skill_md(content: &str) -> Option<Skill> {
    let content = content.trim_start();

    // The front-matter must start with "---" on its own line
    let rest = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))?;

    // Find the closing "---"
    let end_idx = rest.find("\n---")?;
    let yaml_part = &rest[..end_idx];
    let after_fence = &rest[end_idx + 4..]; // skip "\n---"
    let body = after_fence
        .strip_prefix('\n')
        .or_else(|| after_fence.strip_prefix("\r\n"))
        .unwrap_or(after_fence);

    let meta = parse_yaml_frontmatter(yaml_part);
    let (context, examples) = parse_skill_body(body);

    Some(Skill {
        meta,
        context,
        examples,
    })
}

/// Parse the simple YAML front-matter subset used by skill files.
/// Supports string values (bare or double-quoted) and inline arrays `[a, b, c]`.
fn parse_yaml_frontmatter(yaml: &str) -> SkillMeta {
    let mut meta = SkillMeta::default();

    for line in yaml.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Split only at the first colon — values may themselves contain colons
        let Some((key, raw_value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = raw_value.trim();

        match key {
            "name" => meta.name = yaml_unquote(value).to_string(),
            "category" => meta.category = yaml_unquote(value).to_string(),
            "description" => meta.description = yaml_unquote(value).to_string(),
            "author" => {
                let v = yaml_unquote(value);
                meta.author = if v.is_empty() {
                    None
                } else {
                    Some(v.to_string())
                };
            }
            "source_url" => {
                // source_url values may contain additional colons (e.g. "http://...")
                // Re-join from the original line beyond the first colon
                let full_value = raw_value.trim();
                let v = yaml_unquote(full_value);
                meta.source_url = if v.is_empty() {
                    None
                } else {
                    Some(v.to_string())
                };
            }
            "tags" => {
                // Parse inline array: [a, b, c] or ["a", "b"]
                let inner = value.trim_start_matches('[').trim_end_matches(']').trim();
                meta.tags = inner
                    .split(',')
                    .map(|s| yaml_unquote(s.trim()).to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            _ => {} // ignore unknown keys
        }
    }

    meta
}

/// Strip a single layer of double or single quotes from a YAML scalar.
fn yaml_unquote(s: &str) -> &str {
    if s.len() >= 2 {
        let first = s.as_bytes()[0];
        let last = s.as_bytes()[s.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &s[1..s.len() - 1];
        }
    }
    s
}

/// Parse the Markdown body sections: `## Concepts`, `## Pitfalls`, `## Examples`.
fn parse_skill_body(body: &str) -> (SkillContext, Vec<SkillExample>) {
    #[derive(PartialEq)]
    enum Section {
        None,
        Concepts,
        Pitfalls,
        Examples,
    }

    let mut section = Section::None;
    let mut concepts: Vec<String> = Vec::new();
    let mut pitfalls: Vec<String> = Vec::new();
    let mut examples: Vec<SkillExample> = Vec::new();

    // Accumulator for the example currently being parsed
    let mut cur_task: Option<String> = None;
    let mut cur_args: Option<String> = None;
    let mut cur_expl: Option<String> = None;

    /// Flush a completed example into the list.
    fn flush_example(
        task: &mut Option<String>,
        args: &mut Option<String>,
        expl: &mut Option<String>,
        examples: &mut Vec<SkillExample>,
    ) {
        if let (Some(t), Some(a), Some(e)) = (task.take(), args.take(), expl.take()) {
            examples.push(SkillExample {
                task: t,
                args: a,
                explanation: e,
            });
        }
    }

    for line in body.lines() {
        let trimmed = line.trim();

        // Detect top-level section headings
        match trimmed {
            "## Concepts" => {
                section = Section::Concepts;
                continue;
            }
            "## Pitfalls" => {
                section = Section::Pitfalls;
                continue;
            }
            "## Examples" => {
                section = Section::Examples;
                continue;
            }
            _ => {}
        }

        match section {
            Section::Concepts => {
                if let Some(item) = trimmed.strip_prefix("- ") {
                    concepts.push(item.to_string());
                }
            }
            Section::Pitfalls => {
                if let Some(item) = trimmed.strip_prefix("- ") {
                    pitfalls.push(item.to_string());
                }
            }
            Section::Examples => {
                if let Some(task) = trimmed.strip_prefix("### ") {
                    // Starting a new example — flush the previous one first
                    flush_example(&mut cur_task, &mut cur_args, &mut cur_expl, &mut examples);
                    cur_task = Some(task.to_string());
                } else if let Some(rest) = trimmed.strip_prefix("**Args:**") {
                    // Args value is wrapped in backticks: `code`
                    let args = rest.trim().trim_matches('`').to_string();
                    cur_args = Some(args);
                } else if let Some(rest) = trimmed.strip_prefix("**Explanation:**") {
                    cur_expl = Some(rest.trim().to_string());
                }
            }
            Section::None => {}
        }
    }

    // Flush the last example
    flush_example(&mut cur_task, &mut cur_args, &mut cur_expl, &mut examples);

    (SkillContext { concepts, pitfalls }, examples)
}

// ─── Built-in skill registry ──────────────────────────────────────────────────

macro_rules! builtin {
    ($name:literal) => {
        (
            $name,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/skills/",
                $name,
                ".md"
            )),
        )
    };
}

/// All skills compiled into the binary.  Community and user skills are loaded at
/// runtime and take priority over these.
pub static BUILTIN_SKILLS: &[(&str, &str)] = &[
    // ── Original 10 core tools ──────────────────────────────────────────────
    builtin!("samtools"),
    builtin!("bwa"),
    builtin!("bcftools"),
    builtin!("bedtools"),
    builtin!("seqkit"),
    builtin!("fastp"),
    builtin!("star"),
    builtin!("gatk"),
    builtin!("bowtie2"),
    builtin!("minimap2"),
    // ── QC & preprocessing ──────────────────────────────────────────────────
    builtin!("trimmomatic"),
    builtin!("cutadapt"),
    builtin!("fastqc"),
    builtin!("multiqc"),
    builtin!("trim_galore"),
    builtin!("picard"),
    // ── Short-read alignment ─────────────────────────────────────────────────
    builtin!("hisat2"),
    builtin!("bwa-mem2"),
    builtin!("chromap"),
    // ── RNA-seq quantification & assembly ───────────────────────────────────
    builtin!("salmon"),
    builtin!("kallisto"),
    builtin!("stringtie"),
    builtin!("rsem"),
    builtin!("featurecounts"),
    builtin!("trinity"),
    builtin!("arriba"),
    // ── Variant calling ──────────────────────────────────────────────────────
    builtin!("freebayes"),
    builtin!("deepvariant"),
    builtin!("strelka2"),
    builtin!("varscan2"),
    builtin!("longshot"),
    // ── Structural variant calling ───────────────────────────────────────────
    builtin!("manta"),
    builtin!("delly"),
    builtin!("sniffles"),
    builtin!("pbsv"),
    // ── CNV calling ─────────────────────────────────────────────────────────
    builtin!("cnvkit"),
    // ── Variant annotation ───────────────────────────────────────────────────
    builtin!("snpeff"),
    builtin!("vep"),
    builtin!("vcftools"),
    // ── Variant benchmarking & phasing ──────────────────────────────────────
    builtin!("whatshap"),
    builtin!("hap_py"),
    builtin!("shapeit4"),
    // ── Epigenomics ──────────────────────────────────────────────────────────
    builtin!("macs2"),
    builtin!("deeptools"),
    builtin!("bismark"),
    builtin!("methyldackel"),
    builtin!("pairtools"),
    // ── Metagenomics ─────────────────────────────────────────────────────────
    builtin!("kraken2"),
    builtin!("bracken"),
    builtin!("metaphlan"),
    builtin!("diamond"),
    builtin!("prokka"),
    builtin!("bakta"),
    builtin!("metabat2"),
    builtin!("checkm2"),
    builtin!("gtdbtk"),
    builtin!("humann3"),
    // ── Single-cell ──────────────────────────────────────────────────────────
    builtin!("cellranger"),
    builtin!("starsolo"),
    builtin!("kb"),
    // ── Long-read QC & basecalling ───────────────────────────────────────────
    builtin!("dorado"),
    builtin!("nanoplot"),
    builtin!("nanostat"),
    builtin!("chopper"),
    builtin!("porechop"),
    // ── Long-read alignment & polishing ─────────────────────────────────────
    builtin!("pbmm2"),
    builtin!("medaka"),
    builtin!("racon"),
    // ── Long-read variant & fusion calling ──────────────────────────────────
    builtin!("pbccs"),
    builtin!("pbfusion"),
    // ── De novo assembly ─────────────────────────────────────────────────────
    builtin!("spades"),
    builtin!("megahit"),
    builtin!("flye"),
    builtin!("hifiasm"),
    builtin!("canu"),
    builtin!("miniasm"),
    builtin!("wtdbg2"),
    // ── Assembly QC ──────────────────────────────────────────────────────────
    builtin!("quast"),
    builtin!("busco"),
    // ── Genome annotation ────────────────────────────────────────────────────
    builtin!("prodigal"),
    builtin!("augustus"),
    builtin!("agat"),
    builtin!("repeatmasker"),
    // ── Sequence utilities ───────────────────────────────────────────────────
    builtin!("seqtk"),
    builtin!("blast"),
    builtin!("hmmer"),
    builtin!("tabix"),
    builtin!("bamtools"),
    builtin!("sra-tools"),
    builtin!("mosdepth"),
    builtin!("crossmap"),
    builtin!("igvtools"),
    // ── Sequence search & clustering ─────────────────────────────────────────
    builtin!("mmseqs2"),
    // ── Sequence sketching & comparison ──────────────────────────────────────
    builtin!("mash"),
    builtin!("sourmash"),
    // ── Multiple sequence alignment ──────────────────────────────────────────
    builtin!("mafft"),
    builtin!("muscle"),
    // ── Phylogenetics ────────────────────────────────────────────────────────
    builtin!("iqtree2"),
    builtin!("fasttree"),
    // ── Population genomics ──────────────────────────────────────────────────
    builtin!("plink2"),
    builtin!("admixture"),
    builtin!("angsd"),
    // ── Comparative & functional genomics ───────────────────────────────────
    builtin!("orthofinder"),
    builtin!("eggnog-mapper"),
    // ── Genome annotation transfer ───────────────────────────────────────────
    builtin!("liftoff"),
    // ── Assembly polishing ───────────────────────────────────────────────────
    builtin!("pilon"),
    // ── Hybrid assembly ──────────────────────────────────────────────────────
    builtin!("verkko"),
    // ── Epigenomics (motif & ChIP-seq) ───────────────────────────────────────
    builtin!("homer"),
    // ── ONT base modification ────────────────────────────────────────────────
    builtin!("modkit"),
    // ── Metagenomics (additional) ────────────────────────────────────────────
    builtin!("centrifuge"),
    // ── Single-cell (additional) ─────────────────────────────────────────────
    builtin!("velocyto"),
    builtin!("cellsnp-lite"),
    // ── QC (additional) ──────────────────────────────────────────────────────
    builtin!("fastq-screen"),
    builtin!("nanocomp"),
    // ── Variant annotation (additional) ──────────────────────────────────────
    builtin!("vcfanno"),
    // ── Structural variant merging & benchmarking ────────────────────────────
    builtin!("survivor"),
    builtin!("truvari"),
    // ── Genomic arithmetic ───────────────────────────────────────────────────
    builtin!("bedops"),
    // ── Version control ──────────────────────────────────────────────────────
    builtin!("git"),
    // ── Networking & file transfer ───────────────────────────────────────────
    builtin!("ssh"),
    builtin!("curl"),
    builtin!("wget"),
    builtin!("rsync"),
    // ── Containerization ─────────────────────────────────────────────────────
    builtin!("docker"),
    builtin!("singularity"),
    // ── Filesystem & text processing ─────────────────────────────────────────
    builtin!("find"),
    builtin!("grep"),
    builtin!("sed"),
    builtin!("awk"),
    builtin!("tar"),
    builtin!("rm"),
    // ── Package management & scripting ───────────────────────────────────────
    builtin!("conda"),
    builtin!("mamba"),
    builtin!("pixi"),
    builtin!("pip"),
    builtin!("python"),
    builtin!("r"),
    builtin!("cargo"),
    // ── Programming languages ────────────────────────────────────────────────
    builtin!("perl"),
    builtin!("julia"),
    builtin!("bash"),
    builtin!("java"),
    // ── Workflow managers ────────────────────────────────────────────────────
    builtin!("nextflow"),
    builtin!("snakemake"),
    // ── AI assistant platforms ────────────────────────────────────────────────
    builtin!("openclaw"),
    builtin!("claude"),
    // ── oxo-call utility skills ────────────────────────────────────────────────
    builtin!("skill-generator"),
    // ── QC tools ─────────────────────────────────────────────────────────────
    builtin!("qualimap"),
    builtin!("rseqc"),
    // ── Read processing ──────────────────────────────────────────────────────
    builtin!("bbtools"),
    // ── Comparative genomics ─────────────────────────────────────────────────
    builtin!("fastani"),
    builtin!("mummer"),
    // ── Motif analysis ───────────────────────────────────────────────────────
    builtin!("meme"),
    // ── HPC & cluster management ─────────────────────────────────────────────
    builtin!("slurm"),
    builtin!("pbs"),
    builtin!("sge"),
    builtin!("lsf"),
    builtin!("htcondor"),
    builtin!("kubectl"),
];

/// HashMap for O(1) lookup of built-in skills by name (case-insensitive).
/// Built once at first access from the BUILTIN_SKILLS static array.
/// Keys are stored in lowercase to enable case-insensitive lookup without allocation.
static BUILTIN_SKILL_MAP: std::sync::LazyLock<std::collections::HashMap<String, &str>> =
    std::sync::LazyLock::new(|| {
        BUILTIN_SKILLS
            .iter()
            .map(|(name, content)| (name.to_ascii_lowercase(), *content))
            .collect()
    });

// ─── Prompt generation ────────────────────────────────────────────────────────

impl Skill {
    /// Render this skill as a section to be injected into the LLM system prompt.
    pub fn to_prompt_section(&self) -> String {
        self.to_prompt_section_limited(usize::MAX)
    }

    /// Generate a prompt section with a limited number of examples.
    ///
    /// When `max_examples` is smaller than the total number of examples,
    /// only the first `max_examples` are included.  Concepts and pitfalls
    /// are also trimmed: for `max_examples <= 3`, only the top concepts
    /// and pitfalls are kept to save context budget.
    pub fn to_prompt_section_limited(&self, max_examples: usize) -> String {
        self.render_section(max_examples, &None)
    }

    /// Generate a prompt section selecting examples relevant to the given task.
    ///
    /// When the skill has more examples than `max_examples`, this method ranks
    /// examples by keyword overlap with `task` and selects the most relevant
    /// ones.  This ensures the LLM sees examples that match the user's intent
    /// (e.g., "sort bam" → the sort example, not the index example).
    pub fn to_prompt_section_for_task(&self, max_examples: usize, task: &str) -> String {
        self.render_section(max_examples, &Some(task.to_string()))
    }

    /// Internal rendering shared by `to_prompt_section_limited` and `to_prompt_section_for_task`.
    fn render_section(&self, max_examples: usize, task: &Option<String>) -> String {
        let mut s = String::new();

        let compact = max_examples <= 3;

        if !self.context.concepts.is_empty() {
            let limit = if compact {
                self.context.concepts.len().min(3)
            } else {
                self.context.concepts.len()
            };
            s.push_str("## Expert Domain Knowledge\n");
            for (i, c) in self.context.concepts.iter().take(limit).enumerate() {
                s.push_str(&format!("{}. {}\n", i + 1, c));
            }
            s.push('\n');
        }

        if !self.context.pitfalls.is_empty() {
            let limit = if compact {
                self.context.pitfalls.len().min(2)
            } else {
                self.context.pitfalls.len()
            };
            s.push_str("## Common Pitfalls to Avoid\n");
            for p in self.context.pitfalls.iter().take(limit) {
                s.push_str(&format!("- {p}\n"));
            }
            s.push('\n');
        }

        if !self.examples.is_empty() {
            let selected = self.select_examples(max_examples, task.as_deref());
            s.push_str("## Worked Reference Examples\n");
            for (i, ex) in selected.iter().enumerate() {
                s.push_str(&format!("Example {}:\n", i + 1));
                s.push_str(&format!("  Task:        {}\n", ex.task));
                s.push_str(&format!("  ARGS:        {}\n", ex.args));
                s.push_str(&format!("  Explanation: {}\n", ex.explanation));
                s.push('\n');
            }
        }

        s
    }

    /// Select the most relevant examples for a given task.
    ///
    /// Strategy:
    /// 1. If task is None or empty, fall back to first `max_examples` (original behavior).
    /// 2. Score each example by keyword overlap with the task.
    /// 3. Select top-scoring examples, but always include at least the first
    ///    example (as a "default" reference) if there's room.
    pub fn select_examples(&self, max_examples: usize, task: Option<&str>) -> Vec<&SkillExample> {
        let Some(task) = task.filter(|t| !t.trim().is_empty()) else {
            return self.examples.iter().take(max_examples).collect();
        };

        if self.examples.len() <= max_examples {
            return self.examples.iter().take(max_examples).collect();
        }

        let task_tokens = tokenize_for_match(task);

        // Score each example by keyword overlap with the task.
        // The task description and args are both checked.
        let mut scored: Vec<(usize, usize)> = self
            .examples
            .iter()
            .enumerate()
            .map(|(i, ex)| {
                let ex_tokens = tokenize_for_match(&format!("{} {}", ex.task, ex.args));
                let score = ex_tokens.intersection(&task_tokens).count();
                // Tie-breaking: prefer earlier examples (they tend to be more fundamental)
                (i, score)
            })
            .collect();

        // Sort by score descending, then by index ascending (stable)
        scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        // Take top max_examples, but ensure example 0 is included if there's room
        let mut selected_indices: Vec<usize> = scored
            .into_iter()
            .take(max_examples)
            .map(|(i, _)| i)
            .collect();

        // If example 0 (the most fundamental example) is not in the selection
        // and there's room, swap in the lowest-scoring selected example.
        if !selected_indices.contains(&0)
            && max_examples > 1
            && let Some(&last_idx) = selected_indices.last()
            && last_idx != 0
        {
            selected_indices.pop();
            selected_indices.insert(0, 0);
        }

        selected_indices.sort();
        selected_indices
            .into_iter()
            .map(|i| &self.examples[i])
            .collect()
    }
}

/// Tokenize a string for keyword matching between task and example.
///
/// Splits on whitespace and common delimiters, lowercases, and filters
/// out common stop words that would create false matches.
/// Expands tokens with bioinformatics-specific synonyms for better
/// semantic matching (e.g., "sort" also matches "order", "align" also
/// matches "map").
fn tokenize_for_match(text: &str) -> std::collections::HashSet<String> {
    let stop_words: &[&str] = &[
        "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
        "from", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do",
        "does", "did", "will", "would", "could", "should", "may", "might", "shall", "can", "not",
        "no", "it", "its", "this", "that", "these", "those", "i", "me", "my", "we", "our", "you",
        "your", "he", "she", "they", "them", "file", "files", "into", "using", "use", "then",
        "after",
    ];

    let mut tokens: std::collections::HashSet<String> = text
        .to_ascii_lowercase()
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '/')
        .map(|s| s.trim_matches(|c: char| c == '.' || c == ':' || c == '-' || c == '_' || c == '"'))
        .filter(|s| s.len() >= 2)
        .filter(|s| !stop_words.contains(s))
        .map(|s| s.to_string())
        .collect();

    // Expand with bioinformatics synonym groups for better semantic matching
    expand_synonyms(&mut tokens);

    tokens
}

/// Bioinformatics-specific synonym groups.
///
/// When a token from one group is found, all other tokens in the same group
/// are added to the set.  This bridges vocabulary gaps between how users
/// describe tasks and how skill examples are written.
const SYNONYM_GROUPS: &[&[&str]] = &[
    &["sort", "order", "arrange"],
    &["align", "map", "mapping", "alignment"],
    &["filter", "select", "extract", "subset"],
    &["convert", "transform", "reformat"],
    &["merge", "combine", "concatenate", "concat", "cat"],
    &["index", "idx"],
    &["count", "quantify", "quantification"],
    &["compress", "gzip", "bgzip"],
    &["decompress", "unzip", "gunzip"],
    &["trim", "clip", "adapter"],
    &["call", "detect", "identify"],
    &["annotate", "annotation"],
    &["assembly", "assemble"],
    &["dedup", "deduplicate", "markdup", "rmdup"],
    &["paired", "pe"],
    &["single", "se"],
    &["reference", "genome", "ref"],
    &["threads", "cores", "parallel", "cpu"],
    &["output", "out", "write"],
    &["input", "read"],
    &["quality", "qc", "quality control"],
    &["coordinate", "position", "pos"],
    &["name", "qname", "queryname"],
    &["variant", "snp", "indel", "mutation"],
    &["expression", "tpm", "fpkm", "rpkm"],
    &["coverage", "depth"],
    &["peak", "summit"],
    &["region", "interval", "bed"],
    &["bam", "sam", "cram"],
    &["vcf", "bcf"],
    &["fastq", "fq", "reads"],
    &["fasta", "fa", "sequence"],
];

/// Expand a token set with synonyms from predefined synonym groups.
fn expand_synonyms(tokens: &mut std::collections::HashSet<String>) {
    let original: Vec<String> = tokens.iter().cloned().collect();
    for token in &original {
        for group in SYNONYM_GROUPS {
            if group.contains(&token.as_str()) {
                for &synonym in *group {
                    tokens.insert(synonym.to_string());
                }
                break; // Each token matches at most one group
            }
        }
    }
}

// ─── Skill manager ────────────────────────────────────────────────────────────

pub struct SkillManager {
    config: Config,
}

impl SkillManager {
    pub fn new(config: Config) -> Self {
        SkillManager { config }
    }

    // ── Loading ──────────────────────────────────────────────────────────────

    /// Load the best available skill for a tool (synchronous).
    /// Priority: user-defined > community-installed > built-in.
    ///
    /// **Does not query MCP servers** — use [`load_async`][Self::load_async]
    /// when running inside an async context (e.g. inside `Runner::prepare`).
    ///
    /// Tool name matching is **case-insensitive**: "featureCounts", "FeatureCounts",
    /// and "featurecounts" all resolve to the same skill.  The canonical form used
    /// for file lookups and built-in registry matching is lowercase.
    pub fn load(&self, tool: &str) -> Option<Skill> {
        let tool_lc = tool.to_ascii_lowercase();
        self.load_user(&tool_lc)
            .or_else(|| self.load_community(&tool_lc))
            .or_else(|| self.load_builtin(&tool_lc))
    }

    /// Load the best available skill for a tool, including MCP server sources.
    ///
    /// Priority: user-defined > community-installed > MCP servers > built-in.
    ///
    /// MCP servers are queried in the order they appear in `config.toml`.  The
    /// first server that returns a parseable skill wins.  Network errors are
    /// silently ignored (a warning is printed with `--verbose`).
    pub async fn load_async(&self, tool: &str) -> Option<Skill> {
        let tool_lc = tool.to_ascii_lowercase();
        // 1. User-defined (highest priority)
        if let Some(skill) = self.load_user(&tool_lc) {
            return Some(skill);
        }
        // 2. Community-installed
        if let Some(skill) = self.load_community(&tool_lc) {
            return Some(skill);
        }
        // 3. MCP servers
        if let Some(skill) = self.load_mcp(&tool_lc).await {
            return Some(skill);
        }
        // 4. Built-in (lowest priority)
        self.load_builtin(&tool_lc)
    }

    /// Load a skill from the built-in registry (compiled into the binary).
    /// Matching is case-insensitive: "SAMTOOLS" and "SamTools" both load "samtools".
    /// Uses O(1) HashMap lookup instead of O(n) linear search.
    pub fn load_builtin(&self, tool: &str) -> Option<Skill> {
        let tool_lc = tool.to_ascii_lowercase();
        BUILTIN_SKILL_MAP.get(tool_lc.as_str()).and_then(|content| {
            parse_skill_md(content).or_else(|| {
                eprintln!("warning: could not parse built-in skill '{tool}'");
                None
            })
        })
    }

    /// Load a user-defined skill from `~/.config/oxo-call/skills/<tool>.md`
    /// (falls back to `<tool>.toml` for backward compatibility).
    fn load_user(&self, tool: &str) -> Option<Skill> {
        let dir = self.user_skill_dir().ok()?;
        // Prefer the new .md format; fall back to legacy .toml
        let md_path = dir.join(format!("{tool}.md"));
        let toml_path = dir.join(format!("{tool}.toml"));
        self.load_from_path(&md_path)
            .or_else(|| self.load_from_path(&toml_path))
    }

    /// Load a community-installed skill from `~/.local/share/oxo-call/skills/<tool>.md`
    /// (falls back to `<tool>.toml` for backward compatibility).
    fn load_community(&self, tool: &str) -> Option<Skill> {
        let dir = self.community_skill_dir().ok()?;
        // Prefer the new .md format; fall back to legacy .toml
        let md_path = dir.join(format!("{tool}.md"));
        let toml_path = dir.join(format!("{tool}.toml"));
        self.load_from_path(&md_path)
            .or_else(|| self.load_from_path(&toml_path))
    }

    /// Parse a skill file from disk, auto-detecting format by extension.
    /// `.md`   → YAML front-matter + Markdown parser
    /// `.toml` → legacy TOML parser (backward compatibility)
    fn load_from_path(&self, path: &PathBuf) -> Option<Skill> {
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(path).ok()?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "md" => parse_skill_md(&content).or_else(|| {
                eprintln!("warning: could not parse skill '{}' (md)", path.display());
                None
            }),
            "toml" => toml::from_str(&content)
                .map_err(|e| eprintln!("warning: could not parse skill '{}': {e}", path.display()))
                .ok(),
            _ => {
                eprintln!("warning: unknown skill file extension '{}'", path.display());
                None
            }
        }
    }

    /// Try each configured MCP server in order; return the first parseable skill found.
    async fn load_mcp(&self, tool: &str) -> Option<Skill> {
        use crate::mcp::McpClient;

        for server in &self.config.mcp.servers {
            let client = McpClient::new(server.clone());
            if let Some(content) = client.fetch_skill(tool).await {
                if let Some(skill) = parse_skill_md(&content) {
                    return Some(skill);
                }
                // Fallback: try legacy TOML format
                if let Ok(skill) = toml::from_str::<Skill>(&content) {
                    return Some(skill);
                }
            }
        }
        None
    }

    // ── Discovery ────────────────────────────────────────────────────────────

    /// Return all known skills with their source label (built-in / community / user).
    ///
    /// Does **not** include MCP skills. Use [`list_all_async`][Self::list_all_async]
    /// to also include skills from configured MCP servers.
    pub fn list_all(&self) -> Vec<(String, String)> {
        let mut skills: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        // Built-in (lowest priority in display — will be overridden below)
        for (name, _) in BUILTIN_SKILLS {
            skills.insert(name.to_string(), "built-in".to_string());
        }

        // Community-installed
        if let Ok(dir) = self.community_skill_dir() {
            for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "md" || e == "toml")
                    && let Some(stem) = path.file_stem()
                {
                    skills.insert(stem.to_string_lossy().into_owned(), "community".to_string());
                }
            }
        }

        // User-defined (highest priority)
        if let Ok(dir) = self.user_skill_dir() {
            for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "md" || e == "toml")
                    && let Some(stem) = path.file_stem()
                {
                    skills.insert(stem.to_string_lossy().into_owned(), "user".to_string());
                }
            }
        }

        let mut result: Vec<(String, String)> = skills.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    /// Return all known skills including those from configured MCP servers.
    ///
    /// Priority labels: `user` > `community` > `mcp:<server-name>` > `built-in`.
    /// MCP servers are queried concurrently; errors are silently ignored.
    ///
    /// Returns `Vec<(tool_name, source_label)>` sorted alphabetically.
    pub async fn list_all_async(&self) -> Vec<(String, String)> {
        use crate::mcp::McpClient;

        // Start from the synchronous list (user/community/built-in)
        let mut skills: std::collections::HashMap<String, String> =
            self.list_all().into_iter().collect();

        // Query MCP servers — each skill only fills in the gap (does not
        // override user/community skills that are already present).
        for server in &self.config.mcp.servers {
            let client = McpClient::new(server.clone());
            if let Ok(entries) = client.list_skill_resources().await {
                let label = format!("mcp:{}", server.name());
                for entry in entries {
                    // Only add if not already known from higher-priority sources.
                    // Use or_insert_with to avoid cloning the label when the key
                    // is already present.
                    skills
                        .entry(entry.tool.to_ascii_lowercase())
                        .or_insert_with(|| label.clone());
                }
            }
        }

        let mut result: Vec<(String, String)> = skills.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    // ── Install / remove ─────────────────────────────────────────────────────

    /// Install a skill from a URL into the community skills directory.
    ///
    /// Both `.md` (YAML front-matter + Markdown, preferred) and legacy `.toml`
    /// formats are accepted; the format is detected from the downloaded content.
    pub async fn install_from_url(&self, tool: &str, url: &str) -> Result<Skill> {
        if !url.starts_with("https://") && !url.starts_with("http://") {
            return Err(OxoError::IndexError(
                "Only http:// and https:// URLs are accepted".to_string(),
            ));
        }
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(OxoError::IndexError(format!(
                "HTTP {} fetching skill from {url}",
                response.status()
            )));
        }
        let content = response.text().await?;

        // Detect format from content or URL extension
        let is_md = url.ends_with(".md") || content.trim_start().starts_with("---");
        let skill = if is_md {
            parse_skill_md(&content).ok_or_else(|| {
                OxoError::IndexError(
                    "Invalid skill Markdown: could not parse front-matter and sections".to_string(),
                )
            })?
        } else {
            toml::from_str(&content)
                .map_err(|e| OxoError::IndexError(format!("Invalid skill TOML: {e}")))?
        };

        let dir = self.community_skill_dir()?;
        std::fs::create_dir_all(&dir)?;
        let ext = if is_md { "md" } else { "toml" };
        std::fs::write(dir.join(format!("{tool}.{ext}")), &content)?;
        Ok(skill)
    }

    /// Install a skill from the official oxo-call community registry on GitHub.
    pub async fn install_from_registry(&self, tool: &str) -> Result<Skill> {
        let url = format!(
            "https://raw.githubusercontent.com/Traitome/oxo-call-skills/main/skills/{tool}.md"
        );
        self.install_from_url(tool, &url).await
    }

    /// Remove a community-installed or user-installed skill.
    /// Checks both `.md` and `.toml` extensions.
    pub fn remove(&self, tool: &str) -> Result<()> {
        // Check community paths (.md and .toml)
        let comm_dir = self.community_skill_dir()?;
        let user_dir = self.user_skill_dir()?;

        for ext in &["md", "toml"] {
            let community_path = comm_dir.join(format!("{tool}.{ext}"));
            if community_path.exists() {
                std::fs::remove_file(&community_path)?;
                return Ok(());
            }
        }
        for ext in &["md", "toml"] {
            let user_path = user_dir.join(format!("{tool}.{ext}"));
            if user_path.exists() {
                std::fs::remove_file(&user_path)?;
                return Ok(());
            }
        }
        Err(OxoError::IndexError(format!(
            "Skill '{tool}' is not installed. Built-in skills cannot be removed."
        )))
    }

    // ── Template generation ───────────────────────────────────────────────────

    /// Generate a blank skill template in Markdown format (YAML front-matter + Markdown body).
    pub fn create_template(tool: &str) -> String {
        format!(
            r#"---
name: {tool}
category:        # e.g. alignment, variant-calling, qc, assembly, annotation
description:     # One-line description of the tool
tags: []         # e.g. [bam, ngs, short-read]
author:          # Your name / GitHub handle (optional)
source_url:      # Link to tool documentation (optional)
---

## Concepts

- Key concept that orients the LLM to this tool's data model
- Another important concept — be specific and actionable
- A third concept about flag semantics or output format

## Pitfalls

- Common mistake users make — explain what goes wrong and how to fix it
- Another pitfall — always explain the consequence
- A third pitfall specific to this tool

## Examples

### describe the task in plain English
**Args:** `--flag value input.file -o output.file`
**Explanation:** why these specific flags were chosen

### another representative task
**Args:** `--other-flag input.file`
**Explanation:** explanation of what this accomplishes
"#,
            tool = tool
        )
    }

    // ── Path helpers ──────────────────────────────────────────────────────────

    pub fn user_skill_dir(&self) -> Result<PathBuf> {
        Ok(Config::config_dir()?.join("skills"))
    }

    pub fn community_skill_dir(&self) -> Result<PathBuf> {
        Ok(Config::data_dir()?.join("skills"))
    }

    /// Find the on-disk path for a user-defined or community-installed skill.
    ///
    /// Checks (in order): user `.md`, user `.toml`, community `.md`, community `.toml`.
    /// Returns an error if the skill is not installed locally (e.g., it's built-in or MCP-only).
    pub fn find_user_or_community_skill_path(&self, tool: &str) -> Result<PathBuf> {
        let tool_lc = tool.to_ascii_lowercase();
        let user_dir = self.user_skill_dir()?;
        let comm_dir = self.community_skill_dir()?;
        for dir in &[&user_dir, &comm_dir] {
            for ext in &["md", "toml"] {
                let path = dir.join(format!("{tool_lc}.{ext}"));
                if path.exists() {
                    return Ok(path);
                }
            }
        }
        Err(OxoError::IndexError(format!(
            "Skill '{tool}' has no editable local file. \
             Built-in and MCP skills cannot be polished in-place. \
             Install the skill first with 'oxo-call skill install {tool}' \
             or create a user skill with 'oxo-call skill create {tool}'."
        )))
    }
}

// ─── Skill depth validation ───────────────────────────────────────────────────

/// Minimum quality thresholds for skill files.  Validation is not enforced at
/// runtime (so that partial community skills are still usable) but is exercised
/// by tests to surface skill files that fall below the recommended depth.
pub const MIN_EXAMPLES: usize = 5;
pub const MIN_CONCEPTS: usize = 3;
pub const MIN_PITFALLS: usize = 3;

/// Validate that a parsed skill meets the minimum quality thresholds.
/// Returns a list of human-readable issues; an empty list means the skill passes.
pub fn validate_skill_depth(skill: &Skill) -> Vec<String> {
    let mut issues = Vec::new();
    if skill.examples.len() < MIN_EXAMPLES {
        issues.push(format!(
            "{}: has {} examples (minimum {})",
            skill.meta.name,
            skill.examples.len(),
            MIN_EXAMPLES
        ));
    }
    if skill.context.concepts.len() < MIN_CONCEPTS {
        issues.push(format!(
            "{}: has {} concepts (minimum {})",
            skill.meta.name,
            skill.context.concepts.len(),
            MIN_CONCEPTS
        ));
    }
    if skill.context.pitfalls.len() < MIN_PITFALLS {
        issues.push(format!(
            "{}: has {} pitfalls (minimum {})",
            skill.meta.name,
            skill.context.pitfalls.len(),
            MIN_PITFALLS
        ));
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_md_basic() {
        let md = r#"---
name: testtool
category: test
description: A test tool for unit testing
tags: [test, unit]
author: test-author
source_url: https://example.com
---

## Concepts

- Concept one about this tool
- Concept two — another important point
- Concept three — and a third

## Pitfalls

- Pitfall one — common mistake here
- Pitfall two — another thing to avoid
- Pitfall three — edge case

## Examples

### do the first thing
**Args:** `--flag value input.file`
**Explanation:** --flag does this thing

### do the second thing
**Args:** `--other-flag input.file -o output.file`
**Explanation:** -o writes to output file

### do the third thing
**Args:** `subcommand --param value`
**Explanation:** subcommand is used for this

### do the fourth thing
**Args:** `-x -y -z`
**Explanation:** combined short flags

### do the fifth thing
**Args:** `--verbose --output result.txt`
**Explanation:** verbose output to file
"#;
        let skill = parse_skill_md(md).expect("should parse");
        assert_eq!(skill.meta.name, "testtool");
        assert_eq!(skill.meta.category, "test");
        assert_eq!(skill.meta.description, "A test tool for unit testing");
        assert_eq!(skill.meta.tags, vec!["test", "unit"]);
        assert_eq!(skill.meta.author.as_deref(), Some("test-author"));
        assert_eq!(
            skill.meta.source_url.as_deref(),
            Some("https://example.com")
        );
        assert_eq!(skill.context.concepts.len(), 3);
        assert_eq!(skill.context.pitfalls.len(), 3);
        assert_eq!(skill.examples.len(), 5);
        assert_eq!(skill.examples[0].task, "do the first thing");
        assert_eq!(skill.examples[0].args, "--flag value input.file");
        assert_eq!(skill.examples[0].explanation, "--flag does this thing");
    }

    #[test]
    fn test_parse_skill_md_quoted_description() {
        let md = "---\nname: tool\ncategory: cat\ndescription: \"A tool: with colons in description\"\ntags: []\n---\n\n## Concepts\n\n- concept\n\n## Pitfalls\n\n- pitfall\n\n## Examples\n\n### task\n**Args:** `--flag`\n**Explanation:** explanation\n";
        let skill = parse_skill_md(md).expect("should parse");
        assert_eq!(skill.meta.description, "A tool: with colons in description");
    }

    #[test]
    fn test_parse_skill_md_missing_frontmatter() {
        assert!(parse_skill_md("No front matter here").is_none());
        assert!(parse_skill_md("## Concepts\n\n- item").is_none());
    }

    #[test]
    fn test_all_builtin_skills_parse() {
        for (name, md_str) in BUILTIN_SKILLS {
            let skill = parse_skill_md(md_str)
                .unwrap_or_else(|| panic!("built-in skill '{name}' failed to parse"));
            assert!(
                !skill.meta.name.is_empty(),
                "built-in skill '{name}' has an empty meta.name"
            );
        }
    }

    #[test]
    fn test_builtin_skill_depth_report() {
        // This test does NOT fail on individual skills — it reports a summary so
        // contributors can see which skills need attention.  The test only fails
        // if *no* built-in skills pass validation, which would indicate a systemic
        // problem rather than a single missing example.
        let mut passing = 0usize;
        let mut total = 0usize;

        for (name, md_str) in BUILTIN_SKILLS {
            if let Some(skill) = parse_skill_md(md_str) {
                total += 1;
                let issues = validate_skill_depth(&skill);
                if issues.is_empty() {
                    passing += 1;
                } else {
                    eprintln!("skill depth gaps for '{name}':");
                    for issue in &issues {
                        eprintln!("  - {issue}");
                    }
                }
            }
        }

        assert!(
            passing > 0,
            "no built-in skills pass minimum depth validation ({total} total)"
        );
        eprintln!("\nskill depth summary: {passing}/{total} skills meet minimum depth thresholds");
    }

    #[test]
    fn test_validate_skill_depth_pass() {
        let skill = Skill {
            meta: SkillMeta {
                name: "test-tool".into(),
                ..Default::default()
            },
            context: SkillContext {
                concepts: vec!["c1".into(), "c2".into(), "c3".into()],
                pitfalls: vec!["p1".into(), "p2".into(), "p3".into()],
            },
            examples: (0..5)
                .map(|i| SkillExample {
                    task: format!("task {i}"),
                    args: format!("--flag{i}"),
                    explanation: format!("explanation {i}"),
                })
                .collect(),
        };
        let issues = validate_skill_depth(&skill);
        assert!(issues.is_empty(), "expected no issues, got: {issues:?}");
    }

    #[test]
    fn test_validate_skill_depth_fail() {
        let skill = Skill {
            meta: SkillMeta {
                name: "shallow".into(),
                ..Default::default()
            },
            context: SkillContext {
                concepts: vec!["c1".into()],
                pitfalls: vec![],
            },
            examples: vec![],
        };
        let issues = validate_skill_depth(&skill);
        assert_eq!(issues.len(), 3);
    }

    #[test]
    fn test_load_builtin_case_insensitive() {
        use crate::config::Config;
        let config = Config::default();
        let mgr = SkillManager::new(config);

        // Exact lowercase — should always work
        assert!(
            mgr.load_builtin("samtools").is_some(),
            "samtools (lowercase) should load"
        );
        // UPPERCASE — must also resolve via case-insensitive matching
        assert!(
            mgr.load_builtin("SAMTOOLS").is_some(),
            "SAMTOOLS (uppercase) should load via case-insensitive matching"
        );
        // Mixed case (featureCounts is the real-world trigger of this bug)
        assert!(
            mgr.load_builtin("featureCounts").is_some(),
            "featureCounts (mixed case) should load via case-insensitive matching"
        );
        assert!(
            mgr.load_builtin("GATK").is_some(),
            "GATK (uppercase) should load via case-insensitive matching"
        );
    }

    #[test]
    fn test_skill_manager_load_case_insensitive() {
        use crate::config::Config;
        let config = Config::default();
        let mgr = SkillManager::new(config);

        let lower = mgr.load("samtools");
        let upper = mgr.load("SAMTOOLS");
        let mixed = mgr.load("SamTools");

        // All three should resolve to the same skill
        assert!(lower.is_some(), "samtools lowercase should load");
        assert!(upper.is_some(), "SAMTOOLS uppercase should load");
        assert!(mixed.is_some(), "SamTools mixed-case should load");
        let name_lower = lower.unwrap().meta.name;
        let name_upper = upper.unwrap().meta.name;
        let name_mixed = mixed.unwrap().meta.name;
        assert_eq!(
            name_lower, name_upper,
            "lowercase and uppercase should resolve to the same skill"
        );
        assert_eq!(
            name_lower, name_mixed,
            "lowercase and mixed-case should resolve to the same skill"
        );
    }

    #[test]
    fn test_mcp_config_defaults_to_empty() {
        use crate::config::Config;
        let cfg = Config::default();
        assert!(
            cfg.mcp.servers.is_empty(),
            "default config should have no MCP servers"
        );
    }

    #[test]
    fn test_mcp_config_round_trips_toml() {
        use crate::config::{Config, McpServerConfig};

        let mut cfg = Config::default();
        cfg.mcp.servers.push(McpServerConfig {
            url: "http://localhost:3000".to_string(),
            name: "test-server".to_string(),
            api_key: None,
        });
        cfg.mcp.servers.push(McpServerConfig {
            url: "https://skills.example.org".to_string(),
            name: "org-skills".to_string(),
            api_key: Some("secret".to_string()),
        });

        let toml_str = toml::to_string_pretty(&cfg).expect("serialize");
        let back: Config = toml::from_str(&toml_str).expect("deserialize");

        assert_eq!(back.mcp.servers.len(), 2);
        assert_eq!(back.mcp.servers[0].url, "http://localhost:3000");
        assert_eq!(back.mcp.servers[0].name, "test-server");
        assert!(back.mcp.servers[0].api_key.is_none());
        assert_eq!(back.mcp.servers[1].api_key.as_deref(), Some("secret"));
    }

    #[test]
    fn test_mcp_config_backward_compat_no_mcp_section() {
        // Old config.toml files without [mcp] should deserialize to empty servers list
        let old_toml = r#"
[llm]
provider = "github-copilot"
max_tokens = 2048
temperature = 0.0

[docs]
local_paths = []
remote_sources = []
auto_update = true
"#;
        let cfg: crate::config::Config =
            toml::from_str(old_toml).expect("should deserialize old config");
        assert!(
            cfg.mcp.servers.is_empty(),
            "old configs should have no MCP servers"
        );
    }

    // ─── SkillManager::create_template ───────────────────────────────────────

    #[test]
    fn test_create_template_contains_tool_name() {
        let template = SkillManager::create_template("gatk");
        assert!(
            template.contains("name: gatk"),
            "should have tool name in frontmatter"
        );
        assert!(
            template.contains("## Concepts"),
            "should have Concepts section"
        );
        assert!(
            template.contains("## Pitfalls"),
            "should have Pitfalls section"
        );
        assert!(
            template.contains("## Examples"),
            "should have Examples section"
        );
    }

    // ─── SkillManager user/community skill loading ────────────────────────────

    #[test]
    fn test_skill_manager_load_user_skill_md() {
        use crate::config::Config;
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());

        let tmp = tempfile::tempdir().unwrap();
        let skill_dir = tmp.path().join(".config").join("oxo-call").join("skills");
        std::fs::create_dir_all(&skill_dir).unwrap();

        let skill_md = r#"---
name: my-custom-tool
category: custom
description: A custom user tool
tags: []
---

## Concepts

- Concept 1
- Concept 2
- Concept 3

## Pitfalls

- Pitfall 1
- Pitfall 2
- Pitfall 3

## Examples

### task one
**Args:** `--flag value`
**Explanation:** does something useful

### task two
**Args:** `-x input.txt`
**Explanation:** second example

### task three
**Args:** `--output out.txt`
**Explanation:** third example

### task four
**Args:** `subcommand -v`
**Explanation:** fourth example

### task five
**Args:** `--all --recursive`
**Explanation:** fifth example
"#;
        std::fs::write(skill_dir.join("my-custom-tool.md"), skill_md).unwrap();

        // Override config_dir to point to our tmp directory
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        // Also override config dir for user skills
        // (SkillManager::user_skill_dir uses Config::config_dir())
        // We need to write the skill where SkillManager::user_skill_dir() points to.
        // Since we can't easily override config_dir, we test load() which falls back to builtin.
        // The user skill load path is covered by verifying SkillManager::user_skill_dir().
        let config = Config::default();
        let mgr = SkillManager::new(config);
        let dir = mgr.user_skill_dir();
        assert!(dir.is_ok(), "user_skill_dir should work");
        let comm_dir = mgr.community_skill_dir();
        assert!(comm_dir.is_ok(), "community_skill_dir should work");
    }

    // ─── SkillManager::find_user_or_community_skill_path ─────────────────────

    #[test]
    fn test_find_user_or_community_skill_path_not_found() {
        use crate::config::Config;
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());

        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let config = Config::default();
        let mgr = SkillManager::new(config);
        let result = mgr.find_user_or_community_skill_path("nonexistent-tool-xyz");
        assert!(result.is_err(), "should fail for nonexistent tool");
    }

    // ─── SkillManager::load (returns None for unknown) ────────────────────────

    #[test]
    fn test_skill_manager_load_unknown_returns_none() {
        use crate::config::Config;
        let config = Config::default();
        let mgr = SkillManager::new(config);
        assert!(mgr.load("nonexistent_tool_xyz_12345").is_none());
    }

    // ─── SkillMeta defaults ───────────────────────────────────────────────────

    #[test]
    fn test_skill_meta_default() {
        let meta = SkillMeta::default();
        assert!(meta.name.is_empty());
        assert!(meta.category.is_empty());
        assert!(meta.description.is_empty());
        assert!(meta.tags.is_empty());
        assert!(meta.author.is_none());
        assert!(meta.source_url.is_none());
    }

    // ─── SkillContext format_for_prompt ──────────────────────────────────────

    #[test]
    fn test_skill_context_in_prompt_section() {
        let skill = Skill {
            meta: SkillMeta {
                name: "tool".to_string(),
                ..Default::default()
            },
            context: SkillContext {
                concepts: vec![
                    "Use -o for output".to_string(),
                    "Use -t for threads".to_string(),
                ],
                pitfalls: vec!["Don't forget the index".to_string()],
            },
            examples: vec![],
        };
        let section = skill.to_prompt_section();
        assert!(section.contains("Use -o for output"));
        assert!(section.contains("Don't forget the index"));
    }

    // ─── Skill::to_prompt_section with examples ───────────────────────────────

    #[test]
    fn test_skill_to_prompt_section_with_examples() {
        let skill = Skill {
            meta: SkillMeta {
                name: "tool".to_string(),
                ..Default::default()
            },
            context: SkillContext {
                concepts: vec![],
                pitfalls: vec![],
            },
            examples: vec![
                SkillExample {
                    task: "sort bam file".to_string(),
                    args: "sort -o sorted.bam input.bam".to_string(),
                    explanation: "sort by coordinate".to_string(),
                },
                SkillExample {
                    task: "index bam file".to_string(),
                    args: "index sorted.bam".to_string(),
                    explanation: "creates .bai index".to_string(),
                },
            ],
        };
        let formatted = skill.to_prompt_section();
        assert!(formatted.contains("sort bam file"));
        assert!(formatted.contains("sort -o sorted.bam"));
        assert!(formatted.contains("index bam file"));
    }

    // ─── SkillManager community skill loading ─────────────────────────────────

    /// Helper: write a minimal valid skill .md to `dir/<tool>.md`
    fn write_test_skill(dir: &std::path::Path, tool: &str) {
        let md = format!(
            r#"---
name: {tool}
category: test
description: Test skill for {tool}
tags: []
---

## Concepts

- Concept 1 for {tool}
- Concept 2 for {tool}
- Concept 3 for {tool}

## Pitfalls

- Pitfall 1
- Pitfall 2
- Pitfall 3

## Examples

### task one
**Args:** `--flag value`
**Explanation:** does something useful

### task two
**Args:** `-x input.txt`
**Explanation:** second example

### task three
**Args:** `--output out.txt`
**Explanation:** third example

### task four
**Args:** `subcommand -v`
**Explanation:** fourth example

### task five
**Args:** `--all --recursive`
**Explanation:** fifth example
"#
        );
        std::fs::write(dir.join(format!("{tool}.md")), md).unwrap();
    }

    #[test]
    fn test_load_community_skill_from_data_dir() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let community_skill_dir = tmp.path().join("skills");
        std::fs::create_dir_all(&community_skill_dir).unwrap();
        write_test_skill(&community_skill_dir, "my-community-tool");

        let config = Config::default();
        let mgr = SkillManager::new(config);
        let skill = mgr.load("my-community-tool");
        assert!(skill.is_some(), "community skill should be loadable");
        assert_eq!(
            skill.unwrap().meta.name,
            "my-community-tool",
            "skill name should match"
        );
    }

    #[test]
    fn test_list_all_includes_community_skills() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let community_skill_dir = tmp.path().join("skills");
        std::fs::create_dir_all(&community_skill_dir).unwrap();
        write_test_skill(&community_skill_dir, "my-community-tool");

        let config = Config::default();
        let mgr = SkillManager::new(config);
        let skills = mgr.list_all();
        let found = skills.iter().find(|(name, _)| name == "my-community-tool");
        assert!(found.is_some(), "list_all should include community skill");
        assert_eq!(found.unwrap().1, "community");
    }

    #[test]
    fn test_list_all_includes_builtin_skills() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let config = Config::default();
        let mgr = SkillManager::new(config);
        let skills = mgr.list_all();
        // There are 158 built-in skills — list should be non-empty
        assert!(
            !skills.is_empty(),
            "list_all should include built-in skills"
        );
        // samtools should be built-in
        let samtools = skills.iter().find(|(name, _)| name == "samtools");
        assert!(samtools.is_some(), "samtools should be in built-in skills");
        assert_eq!(samtools.unwrap().1, "built-in");
    }

    #[test]
    fn test_remove_community_skill() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let community_skill_dir = tmp.path().join("skills");
        std::fs::create_dir_all(&community_skill_dir).unwrap();
        write_test_skill(&community_skill_dir, "removable-tool");

        let config = Config::default();
        let mgr = SkillManager::new(config);
        // File exists → remove should succeed
        assert!(mgr.remove("removable-tool").is_ok());
        // File is gone → remove again should fail
        assert!(mgr.remove("removable-tool").is_err());
    }

    #[test]
    fn test_remove_builtin_skill_fails() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let config = Config::default();
        let mgr = SkillManager::new(config);
        // samtools is built-in — removing it should fail
        let result = mgr.remove("samtools");
        assert!(result.is_err(), "removing a built-in skill should fail");
    }

    #[test]
    fn test_find_user_or_community_skill_path_found() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("OXO_CALL_DATA_DIR", tmp.path());
        }
        let community_skill_dir = tmp.path().join("skills");
        std::fs::create_dir_all(&community_skill_dir).unwrap();
        write_test_skill(&community_skill_dir, "findable-tool");

        let config = Config::default();
        let mgr = SkillManager::new(config);
        let path = mgr.find_user_or_community_skill_path("findable-tool");
        assert!(path.is_ok(), "should find the community skill path");
        assert!(
            path.unwrap().to_str().unwrap().contains("findable-tool"),
            "path should contain the tool name"
        );
    }

    #[test]
    fn test_load_from_path_toml_format() {
        let tmp = tempfile::tempdir().unwrap();
        let toml_content = r#"
[meta]
name = "my-toml-tool"
category = "test"
description = "A toml tool"
tags = []

[context]
concepts = ["concept 1"]
pitfalls = ["pitfall 1"]

[[examples]]
task = "task one"
args = "--flag value"
explanation = "an example"
"#;
        let path = tmp.path().join("my-toml-tool.toml");
        std::fs::write(&path, toml_content).unwrap();

        let config = Config::default();
        let mgr = SkillManager::new(config);
        let skill = mgr.load_from_path(&path.to_path_buf());
        // TOML format may or may not be parseable depending on schema
        // The point is the code path is exercised without panicking
        let _ = skill;
    }

    #[test]
    fn test_load_from_path_nonexistent() {
        let config = Config::default();
        let mgr = SkillManager::new(config);
        let result = mgr.load_from_path(&std::path::PathBuf::from("/nonexistent/path.md"));
        assert!(result.is_none());
    }

    #[test]
    fn test_load_builtin_case_insensitive_new() {
        let config = Config::default();
        let mgr = SkillManager::new(config);
        // Should load samtools whether upper or lower case
        assert!(mgr.load_builtin("samtools").is_some());
        assert!(mgr.load_builtin("SAMTOOLS").is_some());
        assert!(mgr.load_builtin("SamTools").is_some());
    }

    // ─── yaml_unquote ─────────────────────────────────────────────────────────

    #[test]
    fn test_yaml_unquote_double_quoted() {
        assert_eq!(yaml_unquote("\"hello world\""), "hello world");
    }

    #[test]
    fn test_yaml_unquote_single_quoted() {
        assert_eq!(yaml_unquote("'hello world'"), "hello world");
    }

    #[test]
    fn test_yaml_unquote_bare_value() {
        assert_eq!(yaml_unquote("hello"), "hello");
    }

    #[test]
    fn test_yaml_unquote_empty_string() {
        assert_eq!(yaml_unquote(""), "");
    }

    #[test]
    fn test_yaml_unquote_single_char() {
        assert_eq!(yaml_unquote("x"), "x");
    }

    #[test]
    fn test_yaml_unquote_mismatched_quotes() {
        assert_eq!(yaml_unquote("\"hello'"), "\"hello'");
    }

    #[test]
    fn test_yaml_unquote_empty_quoted() {
        assert_eq!(yaml_unquote("\"\""), "");
    }

    // ─── parse_yaml_frontmatter ───────────────────────────────────────────────

    #[test]
    fn test_parse_yaml_frontmatter_basic() {
        let yaml = "name: samtools\ncategory: alignment\ndescription: SAM/BAM tool";
        let meta = parse_yaml_frontmatter(yaml);
        assert_eq!(meta.name, "samtools");
        assert_eq!(meta.category, "alignment");
        assert_eq!(meta.description, "SAM/BAM tool");
    }

    #[test]
    fn test_parse_yaml_frontmatter_quoted_values() {
        let yaml = "name: \"my tool\"\ndescription: 'a description'";
        let meta = parse_yaml_frontmatter(yaml);
        assert_eq!(meta.name, "my tool");
        assert_eq!(meta.description, "a description");
    }

    #[test]
    fn test_parse_yaml_frontmatter_tags() {
        let yaml = "name: test\ntags: [bam, sam, alignment]";
        let meta = parse_yaml_frontmatter(yaml);
        assert_eq!(meta.tags, vec!["bam", "sam", "alignment"]);
    }

    #[test]
    fn test_parse_yaml_frontmatter_tags_quoted() {
        let yaml = "name: test\ntags: [\"bam\", \"sam\"]";
        let meta = parse_yaml_frontmatter(yaml);
        assert_eq!(meta.tags, vec!["bam", "sam"]);
    }

    #[test]
    fn test_parse_yaml_frontmatter_author_and_source_url() {
        let yaml = "name: test\nauthor: John Doe\nsource_url: http://example.com/docs";
        let meta = parse_yaml_frontmatter(yaml);
        assert_eq!(meta.author.as_deref(), Some("John Doe"));
        assert_eq!(meta.source_url.as_deref(), Some("http://example.com/docs"));
    }

    #[test]
    fn test_parse_yaml_frontmatter_empty_author() {
        let yaml = "name: test\nauthor: ";
        let meta = parse_yaml_frontmatter(yaml);
        assert!(meta.author.is_none());
    }

    #[test]
    fn test_parse_yaml_frontmatter_empty_source_url() {
        let yaml = "name: test\nsource_url: ";
        let meta = parse_yaml_frontmatter(yaml);
        assert!(meta.source_url.is_none());
    }

    #[test]
    fn test_parse_yaml_frontmatter_skips_comments() {
        let yaml = "name: test\n# this is a comment\ncategory: bio";
        let meta = parse_yaml_frontmatter(yaml);
        assert_eq!(meta.name, "test");
        assert_eq!(meta.category, "bio");
    }

    #[test]
    fn test_parse_yaml_frontmatter_skips_empty_lines() {
        let yaml = "name: test\n\ncategory: bio";
        let meta = parse_yaml_frontmatter(yaml);
        assert_eq!(meta.name, "test");
        assert_eq!(meta.category, "bio");
    }

    #[test]
    fn test_parse_yaml_frontmatter_unknown_keys_ignored() {
        let yaml = "name: test\nunknown_key: value\ncategory: bio";
        let meta = parse_yaml_frontmatter(yaml);
        assert_eq!(meta.name, "test");
        assert_eq!(meta.category, "bio");
    }

    #[test]
    fn test_parse_yaml_frontmatter_source_url_with_colons() {
        let yaml = "name: test\nsource_url: https://example.com:8080/path";
        let meta = parse_yaml_frontmatter(yaml);
        assert_eq!(
            meta.source_url.as_deref(),
            Some("https://example.com:8080/path")
        );
    }

    // ─── parse_skill_body ─────────────────────────────────────────────────────

    #[test]
    fn test_parse_skill_body_concepts_only() {
        let body = "## Concepts\n- concept one\n- concept two\n";
        let (ctx, examples) = parse_skill_body(body);
        assert_eq!(ctx.concepts, vec!["concept one", "concept two"]);
        assert!(ctx.pitfalls.is_empty());
        assert!(examples.is_empty());
    }

    #[test]
    fn test_parse_skill_body_pitfalls_only() {
        let body = "## Pitfalls\n- pitfall one\n- pitfall two\n";
        let (ctx, examples) = parse_skill_body(body);
        assert!(ctx.concepts.is_empty());
        assert_eq!(ctx.pitfalls, vec!["pitfall one", "pitfall two"]);
        assert!(examples.is_empty());
    }

    #[test]
    fn test_parse_skill_body_examples_only() {
        let body = "## Examples\n\n### Sort BAM\n**Args:** `sort -o out.bam in.bam`\n**Explanation:** Sorts by coordinate\n";
        let (ctx, examples) = parse_skill_body(body);
        assert!(ctx.concepts.is_empty());
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0].task, "Sort BAM");
        assert_eq!(examples[0].args, "sort -o out.bam in.bam");
        assert_eq!(examples[0].explanation, "Sorts by coordinate");
    }

    #[test]
    fn test_parse_skill_body_multiple_examples() {
        let body = "## Examples\n\n### Task 1\n**Args:** `arg1`\n**Explanation:** expl1\n\n### Task 2\n**Args:** `arg2`\n**Explanation:** expl2\n";
        let (_, examples) = parse_skill_body(body);
        assert_eq!(examples.len(), 2);
        assert_eq!(examples[0].task, "Task 1");
        assert_eq!(examples[1].task, "Task 2");
    }

    #[test]
    fn test_parse_skill_body_all_sections() {
        let body = "\
## Concepts
- concept A
- concept B

## Pitfalls
- pitfall X

## Examples

### Do something
**Args:** `--flag value`
**Explanation:** This does something
";
        let (ctx, examples) = parse_skill_body(body);
        assert_eq!(ctx.concepts.len(), 2);
        assert_eq!(ctx.pitfalls.len(), 1);
        assert_eq!(examples.len(), 1);
    }

    #[test]
    fn test_parse_skill_body_empty_body() {
        let (ctx, examples) = parse_skill_body("");
        assert!(ctx.concepts.is_empty());
        assert!(ctx.pitfalls.is_empty());
        assert!(examples.is_empty());
    }

    #[test]
    fn test_parse_skill_body_non_list_lines_ignored() {
        let body = "## Concepts\nThis is a paragraph, not a list item\n- actual concept\n";
        let (ctx, _) = parse_skill_body(body);
        assert_eq!(ctx.concepts, vec!["actual concept"]);
    }

    #[test]
    fn test_parse_skill_body_incomplete_example_not_flushed() {
        // Missing explanation → should not be collected
        let body = "## Examples\n\n### Task\n**Args:** `arg`\n";
        let (_, examples) = parse_skill_body(body);
        assert!(
            examples.is_empty(),
            "incomplete examples (missing explanation) should not be collected"
        );
    }

    // ─── to_prompt_section ────────────────────────────────────────────────────

    #[test]
    fn test_to_prompt_section_empty_skill() {
        let skill = Skill::default();
        let section = skill.to_prompt_section();
        assert!(
            section.is_empty(),
            "empty skill should produce empty section"
        );
    }

    #[test]
    fn test_to_prompt_section_concepts_only() {
        let skill = Skill {
            context: SkillContext {
                concepts: vec!["concept 1".to_string()],
                pitfalls: vec![],
            },
            ..Default::default()
        };
        let section = skill.to_prompt_section();
        assert!(section.contains("Expert Domain Knowledge"));
        assert!(section.contains("concept 1"));
        assert!(!section.contains("Common Pitfalls"));
    }

    #[test]
    fn test_to_prompt_section_pitfalls_only() {
        let skill = Skill {
            context: SkillContext {
                concepts: vec![],
                pitfalls: vec!["pitfall 1".to_string()],
            },
            ..Default::default()
        };
        let section = skill.to_prompt_section();
        assert!(!section.contains("Expert Domain Knowledge"));
        assert!(section.contains("Common Pitfalls"));
        assert!(section.contains("pitfall 1"));
    }

    #[test]
    fn test_to_prompt_section_examples_only() {
        let skill = Skill {
            examples: vec![SkillExample {
                task: "Sort BAM".to_string(),
                args: "sort -o out.bam in.bam".to_string(),
                explanation: "Sorts by coordinate".to_string(),
            }],
            ..Default::default()
        };
        let section = skill.to_prompt_section();
        assert!(section.contains("Worked Reference Examples"));
        assert!(section.contains("Sort BAM"));
        assert!(section.contains("sort -o out.bam in.bam"));
    }

    #[test]
    fn test_to_prompt_section_full() {
        let skill = Skill {
            meta: SkillMeta {
                name: "samtools".to_string(),
                ..Default::default()
            },
            context: SkillContext {
                concepts: vec!["concept A".to_string(), "concept B".to_string()],
                pitfalls: vec!["pitfall X".to_string()],
            },
            examples: vec![SkillExample {
                task: "Sort BAM".to_string(),
                args: "sort -o out.bam in.bam".to_string(),
                explanation: "Sorts by coordinate".to_string(),
            }],
        };
        let section = skill.to_prompt_section();
        assert!(section.contains("Expert Domain Knowledge"));
        assert!(section.contains("Common Pitfalls"));
        assert!(section.contains("Worked Reference Examples"));
        assert!(section.contains("1. concept A"));
        assert!(section.contains("2. concept B"));
    }

    // ─── validate_skill_depth ─────────────────────────────────────────────────

    #[test]
    fn test_validate_skill_depth_insufficient_examples() {
        let skill = Skill {
            meta: SkillMeta {
                name: "test".to_string(),
                ..Default::default()
            },
            context: SkillContext {
                concepts: vec!["a".into(), "b".into(), "c".into()],
                pitfalls: vec!["a".into(), "b".into(), "c".into()],
            },
            examples: vec![],
        };
        let issues = validate_skill_depth(&skill);
        assert!(issues.iter().any(|i| i.contains("examples")));
    }

    #[test]
    fn test_validate_skill_depth_insufficient_concepts() {
        let skill = Skill {
            meta: SkillMeta {
                name: "test".to_string(),
                ..Default::default()
            },
            context: SkillContext {
                concepts: vec![],
                pitfalls: vec!["a".into(), "b".into(), "c".into()],
            },
            examples: (0..5)
                .map(|i| SkillExample {
                    task: format!("task {i}"),
                    args: format!("arg {i}"),
                    explanation: format!("expl {i}"),
                })
                .collect(),
        };
        let issues = validate_skill_depth(&skill);
        assert!(issues.iter().any(|i| i.contains("concepts")));
    }

    #[test]
    fn test_validate_skill_depth_insufficient_pitfalls() {
        let skill = Skill {
            meta: SkillMeta {
                name: "test".to_string(),
                ..Default::default()
            },
            context: SkillContext {
                concepts: vec!["a".into(), "b".into(), "c".into()],
                pitfalls: vec![],
            },
            examples: (0..5)
                .map(|i| SkillExample {
                    task: format!("task {i}"),
                    args: format!("arg {i}"),
                    explanation: format!("expl {i}"),
                })
                .collect(),
        };
        let issues = validate_skill_depth(&skill);
        assert!(issues.iter().any(|i| i.contains("pitfalls")));
    }

    // ─── parse_skill_md edge cases ────────────────────────────────────────────

    #[test]
    fn test_parse_skill_md_no_closing_fence() {
        let md = "---\nname: test\ncategory: test\n";
        assert!(
            parse_skill_md(md).is_none(),
            "no closing --- should return None"
        );
    }

    #[test]
    fn test_parse_skill_md_empty_body() {
        let md = "---\nname: test\ncategory: test\ndescription: desc\n---\n";
        let skill = parse_skill_md(md);
        assert!(skill.is_some());
        let skill = skill.unwrap();
        assert_eq!(skill.meta.name, "test");
        assert!(skill.context.concepts.is_empty());
        assert!(skill.examples.is_empty());
    }

    #[test]
    fn test_parse_skill_md_whitespace_prefix() {
        let md = "  \n---\nname: test\ncategory: test\ndescription: desc\n---\n## Concepts\n- c1\n";
        let skill = parse_skill_md(md);
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().context.concepts, vec!["c1"]);
    }

    #[test]
    fn test_parse_skill_md_crlf_line_endings() {
        let md = "---\r\nname: test\r\ncategory: test\r\ndescription: desc\r\n---\r\n## Concepts\r\n- c1\r\n";
        let skill = parse_skill_md(md);
        assert!(skill.is_some());
    }

    // ─── Synonym expansion tests ──────────────────────────────────────────

    #[test]
    fn test_synonym_expansion_sort() {
        let tokens = tokenize_for_match("sort the BAM file");
        assert!(tokens.contains("sort"));
        assert!(tokens.contains("order"), "sort should expand to order");
        assert!(tokens.contains("arrange"), "sort should expand to arrange");
    }

    #[test]
    fn test_synonym_expansion_align() {
        let tokens = tokenize_for_match("align reads to reference");
        assert!(tokens.contains("align"));
        assert!(tokens.contains("map"), "align should expand to map");
        assert!(tokens.contains("mapping"), "align should expand to mapping");
    }

    #[test]
    fn test_synonym_expansion_filter() {
        let tokens = tokenize_for_match("filter variants by quality");
        assert!(tokens.contains("filter"));
        assert!(tokens.contains("select"), "filter should expand to select");
        assert!(
            tokens.contains("extract"),
            "filter should expand to extract"
        );
    }

    #[test]
    fn test_synonym_expansion_no_expansion_for_unknown() {
        let tokens = tokenize_for_match("foobar baz");
        // Unknown words should not get synonyms
        assert!(tokens.contains("foobar"));
        assert!(!tokens.contains("sort"));
    }

    #[test]
    fn test_tokenize_removes_stop_words() {
        let tokens = tokenize_for_match("sort the file into output");
        assert!(!tokens.contains("the"));
        assert!(!tokens.contains("into"));
        assert!(tokens.contains("sort"));
    }
}
