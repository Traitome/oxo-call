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
    builtin!("annot8r"),
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

// ─── Prompt generation ────────────────────────────────────────────────────────

impl Skill {
    /// Render this skill as a section to be injected into the LLM system prompt.
    pub fn to_prompt_section(&self) -> String {
        let mut s = String::new();

        if !self.context.concepts.is_empty() {
            s.push_str("## Expert Domain Knowledge\n");
            for (i, c) in self.context.concepts.iter().enumerate() {
                s.push_str(&format!("{}. {}\n", i + 1, c));
            }
            s.push('\n');
        }

        if !self.context.pitfalls.is_empty() {
            s.push_str("## Common Pitfalls to Avoid\n");
            for p in &self.context.pitfalls {
                s.push_str(&format!("- {p}\n"));
            }
            s.push('\n');
        }

        if !self.examples.is_empty() {
            s.push_str("## Worked Reference Examples\n");
            for (i, ex) in self.examples.iter().enumerate() {
                s.push_str(&format!("Example {}:\n", i + 1));
                s.push_str(&format!("  Task:        {}\n", ex.task));
                s.push_str(&format!("  ARGS:        {}\n", ex.args));
                s.push_str(&format!("  Explanation: {}\n", ex.explanation));
                s.push('\n');
            }
        }

        s
    }
}

// ─── Skill manager ────────────────────────────────────────────────────────────

pub struct SkillManager {
    #[allow(dead_code)]
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
    #[cfg(not(target_arch = "wasm32"))]
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

    /// Wasm32-compatible stub: falls back to synchronous load (MCP not available).
    #[cfg(target_arch = "wasm32")]
    pub async fn load_async(&self, tool: &str) -> Option<Skill> {
        self.load(tool)
    }

    /// Load a skill from the built-in registry (compiled into the binary).
    /// Matching is case-insensitive: "SAMTOOLS" and "SamTools" both load "samtools".
    pub fn load_builtin(&self, tool: &str) -> Option<Skill> {
        let tool_lc = tool.to_ascii_lowercase();
        BUILTIN_SKILLS
            .iter()
            .find(|(name, _)| *name == tool_lc.as_str())
            .and_then(|(_, content)| {
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
    #[cfg(not(target_arch = "wasm32"))]
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
    #[cfg(not(target_arch = "wasm32"))]
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

    /// Wasm32-compatible stub: falls back to synchronous list (MCP not available).
    #[cfg(target_arch = "wasm32")]
    pub async fn list_all_async(&self) -> Vec<(String, String)> {
        self.list_all()
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
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::IndexError(
            "Skill installation from URL is not supported in WebAssembly".to_string(),
        ));
        #[cfg(not(target_arch = "wasm32"))]
        {
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
                        "Invalid skill Markdown: could not parse front-matter and sections"
                            .to_string(),
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
#[allow(dead_code)]
pub const MIN_EXAMPLES: usize = 5;
#[allow(dead_code)]
pub const MIN_CONCEPTS: usize = 3;
#[allow(dead_code)]
pub const MIN_PITFALLS: usize = 3;

/// Validate that a parsed skill meets the minimum quality thresholds.
/// Returns a list of human-readable issues; an empty list means the skill passes.
#[allow(dead_code)]
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
}
