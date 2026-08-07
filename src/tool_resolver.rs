//! Dynamic Tool Resolver — resolves any tool name to canonical ToolInfo.
//!
//! Evidence hierarchy (L0 → L4):
//!   L0: Live --help output (authority — ground truth)
//!   L1: Curated skills (human-written expertise)
//!   L2: Vector KB (indexed internet docs — bioconda, homepages)
//!   L3: LLM knowledge (model training data + live retrieval)
//!   L4: Knowledge graph (tool relationships, pipeline patterns)
//!
//! The resolver supports:
//! - Alias resolution: iqtree2 → iqtree, emapper → eggnog-mapper
//! - CLI type detection: flags vs subcommands vs positional
//! - Version detection from installed binary
//! - Help text caching with version-aware keys

use crate::error::{OxoError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

/// Evidence level for a piece of knowledge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceLevel {
    /// L0: Live --help output — highest authority, ground truth
    Authority = 0,
    /// L1: Curated skill — human-written, verified
    Curated = 1,
    /// L2: Indexed internet documentation (bioconda, homepages, etc.)
    Indexed = 2,
    /// L3: LLM training knowledge + real-time retrieval
    Model = 3,
    /// L4: Knowledge graph (tool relationships, patterns)
    Graph = 4,
}

/// Evidence grade for L2 (internet) sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceGrade {
    /// Official documentation (homepage, official docs)
    A,
    /// Structured metadata (bioconda recipe)
    AMinus,
    /// Good secondary source (man page)
    BPlus,
    /// General secondary source (ReadTheDocs, README)
    B,
    /// Low-authority source (blog, tutorial)
    C,
}

/// CLI invocation type detected from --help output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CliType {
    /// Plain flags: `samtools sort -@ 8 -o out.bam in.bam`
    Flags,
    /// Single subcommand: `truvari bench -b truth.vcf.gz -c calls.vcf.gz`
    Subcommand,
    /// Multiple subcommands: `gatk HaplotypeCaller`, `gatk MarkDuplicates`
    MultiSubcommand,
    /// Positional-only: `wget URL`, `bgzip file.vcf`
    Positional,
    /// Command-as-tool: `bbduk.sh`, `agat_sp_add_attribute.pl`
    ScriptOrAlias,
}

/// Resolved tool information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// The actual binary name on PATH (e.g., "iqtree", not "iqtree2")
    pub binary: String,
    /// Canonical toolset name (e.g., "iqtree2")
    pub toolset: String,
    /// All known aliases (e.g., ["iqtree", "iqtree3"])
    pub aliases: Vec<String>,
    /// Tool version string from --version
    pub version: Option<String>,
    /// CLI invocation type
    pub cli_type: CliType,
    /// Known subcommands (if applicable)
    pub subcommands: Vec<String>,
    /// Tool category (23 categories)
    pub category: String,
    /// Whether a skill file exists for this tool
    pub has_skill: bool,
    /// Whether --help was successfully cached
    pub help_cached: bool,
}

/// Evidence block for prompt construction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBlock {
    pub level: EvidenceLevel,
    pub grade: Option<EvidenceGrade>,
    pub title: String,
    pub content: String,
    pub instruction: String,
}

/// The core tool resolver.
pub struct ToolResolver {
    /// Canonical name → ToolInfo (loaded from manifest.toml)
    registry: HashMap<String, ToolInfo>,
    /// Alias → canonical name (binary names → toolset names)
    alias_map: HashMap<String, String>,
    /// Toolset → cached --help text (keyed by binary:version)
    help_cache: HashMap<String, String>,
}

impl ToolResolver {
    /// Create a new ToolResolver from the built-in manifest.
    pub fn new() -> Result<Self> {
        let manifest_str = include_str!("../tools/manifest.toml");
        let manifest: Manifest = toml::from_str(manifest_str)
            .map_err(|e| OxoError::ConfigError(format!("Failed to parse manifest: {}", e)))?;

        let mut registry = HashMap::new();
        let mut alias_map = HashMap::new();

        for (name, entry) in &manifest.tools {
            let cli_type = match entry.cli_type.as_deref() {
                Some("subcommands") => CliType::Subcommand,
                Some("multi-subcommand") => CliType::MultiSubcommand,
                Some("positional") => CliType::Positional,
                Some("script") => CliType::ScriptOrAlias,
                _ => CliType::Flags,
            };

            let info = ToolInfo {
                binary: entry.binary.clone().unwrap_or_else(|| name.clone()),
                toolset: name.clone(),
                aliases: entry.aliases.clone().unwrap_or_default(),
                version: None,
                cli_type,
                subcommands: entry.subcommands.clone().unwrap_or_default(),
                category: entry.category.clone().unwrap_or_default(),
                has_skill: false,
                help_cached: false,
            };

            // Map binary → toolset
            alias_map.insert(info.binary.clone(), name.clone());
            // Map all aliases → toolset
            for alias in &info.aliases {
                alias_map.insert(alias.clone(), name.clone());
            }
            // Map toolset name itself
            alias_map.insert(name.clone(), name.clone());

            registry.insert(name.clone(), info);
        }

        Ok(Self {
            registry,
            alias_map,
            help_cache: HashMap::new(),
        })
    }

    /// Load additional aliases from bioconda alias_map.json.
    pub fn load_bioconda_aliases(&mut self, alias_path: &str) -> Result<usize> {
        let content = std::fs::read_to_string(alias_path)
            .map_err(|e| OxoError::ConfigError(format!("Failed to read alias map: {}", e)))?;
        let extra: HashMap<String, String> = serde_json::from_str(&content)
            .map_err(|e| OxoError::ConfigError(format!("Failed to parse alias map: {}", e)))?;

        let mut added = 0;
        for (binary, package) in extra {
            if !self.alias_map.contains_key(&binary) {
                self.alias_map.insert(binary.clone(), package.clone());
                added += 1;
            }
        }

        Ok(added)
    }

    /// Resolve any name (alias, binary, toolset) to canonical ToolInfo.
    pub fn resolve(&self, name: &str) -> Option<&ToolInfo> {
        let canonical = self.alias_map.get(name)?;
        self.registry.get(canonical)
    }

    /// Check if a tool binary exists on PATH.
    pub fn is_installed(&self, name: &str) -> bool {
        let binary = self
            .resolve(name)
            .map(|ti| ti.binary.as_str())
            .unwrap_or(name);
        which_cmd(binary)
    }

    /// Get the --help output for a tool (cached or live).
    pub fn help_text(&mut self, name: &str) -> Result<String> {
        let info = self
            .resolve(name)
            .ok_or_else(|| OxoError::ToolNotFound(name.to_string()))?;

        // Check cache first
        let cache_key = format!("{}:{}", info.binary, info.version.as_deref().unwrap_or("?"));
        if let Some(cached) = self.help_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Capture live --help
        match Command::new(&info.binary).arg("--help").output() {
            Ok(output) if output.status.success() => {
                let text = String::from_utf8_lossy(&output.stdout).to_string();
                self.help_cache.insert(cache_key, text.clone());
                Ok(text)
            }
            Ok(output) => {
                // Try -h as fallback
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if !stderr.is_empty() {
                    self.help_cache.insert(cache_key.clone(), stderr.clone());
                    return Ok(stderr);
                }
                return Err(OxoError::ConfigError(format!(
                    "Help capture failed for {}",
                    info.binary
                )));
            }
            Err(e) => Err(OxoError::ToolNotFound(format!(
                "{} ({})",
                info.binary, e
            ))),
        }
    }

    /// Detect tool version from --version output.
    pub fn detect_version(&mut self, name: &str) -> Result<String> {
        let info = self
            .resolve(name)
            .ok_or_else(|| OxoError::ToolNotFound(name.to_string()))?;

        match Command::new(&info.binary).arg("--version").output() {
            Ok(output) => {
                let text = String::from_utf8_lossy(
                    if output.status.success() {
                        &output.stdout
                    } else {
                        &output.stderr
                    },
                )
                .to_string();
                Ok(text.lines().next().unwrap_or("unknown").to_string())
            }
            Err(_) => Ok("unknown".to_string()),
        }
    }

    /// Get all known tool names.
    pub fn tool_names(&self) -> Vec<&str> {
        self.registry.keys().map(|s| s.as_str()).collect()
    }

    /// Get the total number of supported tools.
    pub fn tool_count(&self) -> usize {
        self.registry.len()
    }
}

/// Check if a command exists on PATH.
fn which_cmd(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// Manifest parsing structures

#[derive(Debug, Deserialize)]
struct Manifest {
    tools: HashMap<String, ToolEntry>,
}

#[derive(Debug, Deserialize)]
struct ToolEntry {
    binary: Option<String>,
    aliases: Option<Vec<String>>,
    category: Option<String>,
    cli_type: Option<String>,
    subcommands: Option<Vec<String>>,
    companions: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_aliases() {
        let resolver = ToolResolver::new().unwrap();

        // iqtree2 → iqtree (alias resolution)
        let info = resolver.resolve("iqtree2").unwrap();
        assert_eq!(info.binary, "iqtree");
        assert_eq!(info.toolset, "iqtree2");

        // repeatmasker → RepeatMasker
        let info = resolver.resolve("repeatmasker").unwrap();
        assert_eq!(info.binary, "RepeatMasker");

        // eggnog-mapper → emapper
        let info = resolver.resolve("eggnog-mapper").unwrap();
        assert_eq!(info.binary, "emapper");
    }

    #[test]
    fn test_tool_count() {
        let resolver = ToolResolver::new().unwrap();
        assert_eq!(resolver.tool_count(), 138);
    }

    #[test]
    fn test_samtools_cli_type() {
        let resolver = ToolResolver::new().unwrap();
        let info = resolver.resolve("samtools").unwrap();
        assert_eq!(info.cli_type, CliType::Subcommand);
    }
}
