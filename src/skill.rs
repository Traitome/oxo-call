/// Skills framework for oxo-call.
///
/// A **Skill** is a TOML file containing curated domain-expert knowledge about a
/// specific bioinformatics tool.  Skills have three functions:
///
/// 1. **Prompt enrichment** — concepts, pitfalls, and worked examples are injected into
///    the LLM prompt, dramatically improving generation quality for weak/small models.
/// 2. **Community extensibility** — anyone can write and share skill files without
///    touching Rust code.
/// 3. **User customisation** — per-user overrides take priority over built-ins.
///
/// # Load priority (highest first)
/// 1. User-defined   `~/.config/oxo-call/skills/<tool>.toml`
/// 2. Community      `~/.local/share/oxo-call/skills/<tool>.toml`
/// 3. Built-in       compiled into the binary via `include_str!`
use crate::config::Config;
use crate::error::{OxoError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─── Data structures ──────────────────────────────────────────────────────────

/// The top-level skill document parsed from a TOML file.
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

// ─── Built-in skill registry ──────────────────────────────────────────────────

macro_rules! builtin {
    ($name:literal) => {
        (
            $name,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/skills/",
                $name,
                ".toml"
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
    // ── Filesystem & text processing ─────────────────────────────────────────
    builtin!("find"),
    builtin!("grep"),
    builtin!("sed"),
    builtin!("awk"),
    builtin!("tar"),
    builtin!("rm"),
    // ── Package management & scripting ───────────────────────────────────────
    builtin!("conda"),
    builtin!("pip"),
    builtin!("python"),
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

    /// Load the best available skill for a tool.
    /// Priority: user-defined > community-installed > built-in.
    pub fn load(&self, tool: &str) -> Option<Skill> {
        self.load_user(tool)
            .or_else(|| self.load_community(tool))
            .or_else(|| self.load_builtin(tool))
    }

    /// Load a skill from the built-in registry (compiled into the binary).
    pub fn load_builtin(&self, tool: &str) -> Option<Skill> {
        BUILTIN_SKILLS
            .iter()
            .find(|(name, _)| *name == tool)
            .and_then(|(_, content)| {
                toml::from_str(content)
                    .map_err(|e| eprintln!("warning: could not parse built-in skill '{tool}': {e}"))
                    .ok()
            })
    }

    /// Load a user-defined skill from `~/.config/oxo-call/skills/<tool>.toml`.
    fn load_user(&self, tool: &str) -> Option<Skill> {
        let path = self.user_skill_dir().ok()?.join(format!("{tool}.toml"));
        self.load_from_path(&path)
    }

    /// Load a community-installed skill from `~/.local/share/oxo-call/skills/<tool>.toml`.
    fn load_community(&self, tool: &str) -> Option<Skill> {
        let path = self
            .community_skill_dir()
            .ok()?
            .join(format!("{tool}.toml"));
        self.load_from_path(&path)
    }

    fn load_from_path(&self, path: &PathBuf) -> Option<Skill> {
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str(&content)
            .map_err(|e| eprintln!("warning: could not parse skill '{}': {e}", path.display()))
            .ok()
    }

    // ── Discovery ────────────────────────────────────────────────────────────

    /// Return all known skills with their source label (built-in / community / user).
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
                if path.extension().is_some_and(|e| e == "toml")
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
                if path.extension().is_some_and(|e| e == "toml")
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

    // ── Install / remove ─────────────────────────────────────────────────────

    /// Install a skill from a URL into the community skills directory.
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
            let skill: Skill = toml::from_str(&content)
                .map_err(|e| OxoError::IndexError(format!("Invalid skill TOML: {e}")))?;

            let dir = self.community_skill_dir()?;
            std::fs::create_dir_all(&dir)?;
            std::fs::write(dir.join(format!("{tool}.toml")), &content)?;
            Ok(skill)
        }
    }

    /// Install a skill from the official oxo-call community registry on GitHub.
    pub async fn install_from_registry(&self, tool: &str) -> Result<Skill> {
        let url = format!(
            "https://raw.githubusercontent.com/Traitome/oxo-call-skills/main/skills/{tool}.toml"
        );
        self.install_from_url(tool, &url).await
    }

    /// Remove a community-installed skill.
    pub fn remove(&self, tool: &str) -> Result<()> {
        let community_path = self.community_skill_dir()?.join(format!("{tool}.toml"));
        let user_path = self.user_skill_dir()?.join(format!("{tool}.toml"));

        if community_path.exists() {
            std::fs::remove_file(&community_path)?;
            return Ok(());
        }
        if user_path.exists() {
            std::fs::remove_file(&user_path)?;
            return Ok(());
        }
        Err(OxoError::IndexError(format!(
            "Skill '{tool}' is not installed. Built-in skills cannot be removed."
        )))
    }

    // ── Template generation ───────────────────────────────────────────────────

    /// Generate a blank TOML skill template for a new tool.
    pub fn create_template(tool: &str) -> String {
        format!(
            r#"[meta]
name = "{tool}"
category = ""        # e.g. alignment, variant-calling, qc, assembly, annotation
description = ""     # One-line description of the tool
tags = []            # e.g. ["bam", "ngs", "short-read"]
author = ""          # Your name / GitHub handle  (optional)
source_url = ""      # Link to tool documentation (optional)

[context]
# 3–6 essential concepts that orient the LLM to this tool's data model and paradigm.
# Good concepts prevent the LLM from confusing this tool with similar ones.
concepts = [
    "",
]

# Common mistakes that produce wrong commands — helps the LLM avoid them.
pitfalls = [
    "",
]

# Worked examples — the single most important section for guiding weak LLMs.
# Aim for 5+ examples covering the most frequent real-world use cases.
# Each example shows a plain-English task → the correct ARGS (no tool name) → why.

[[examples]]
task = "describe the task in plain English"
args  = "--flag value input.file -o output.file"
explanation = "why these specific flags were chosen"

[[examples]]
task = ""
args  = ""
explanation = ""
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
    fn test_all_builtin_skills_parse() {
        for (name, toml_str) in BUILTIN_SKILLS {
            let skill: Skill = toml::from_str(toml_str)
                .unwrap_or_else(|e| panic!("built-in skill '{name}' failed to parse: {e}"));
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

        for (name, toml_str) in BUILTIN_SKILLS {
            if let Ok(skill) = toml::from_str::<Skill>(toml_str) {
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
}
