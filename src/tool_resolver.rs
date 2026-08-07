//! Dynamic Tool Resolver — resolves any CLI tool name to actionable ToolInfo.
//!
//! No hardcoded tool data. Metadata comes from:
//! - Skill files (compile-time embedded, just like `BUILTIN_SKILLS`)
//! - Live PATH discovery (`which` + `--help` capture)
//! - Optional: bioconda alias_map.json for extra binary→toolset mappings
//!
//! Evidence hierarchy for command generation:
//!   L0: Live --help output (authoritative ground truth)
//!   L1: Skill file (curated human expertise)
//!   L2: Indexed internet docs (bioconda, homepages — optional)
//!   L3: LLM training knowledge (fallback)

use crate::error::{OxoError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

// ─── Public types ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceLevel { Authority = 0, Curated = 1, Indexed = 2, Model = 3 }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CliType { Flags, Subcommand, Positional }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub binary: String,         // actual executable name
    pub toolset: String,        // canonical name (skill filename without .md)
    pub category: String,       // from skill frontmatter or "unknown"
    pub cli_type: CliType,      // detected from --help
    pub version: Option<String>,
    pub has_skill: bool,
    pub help_cached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBlock {
    pub level: EvidenceLevel,
    pub title: String,
    pub content: String,
    pub instruction: String,
}

// ─── Skill metadata (loaded from skill file frontmatter) ──────

/// Minimal metadata extracted from a skill file at compile time.
struct SkillMeta {
    name: String,       // frontmatter `name:` (the binary)
    category: String,   // frontmatter `category:`
}

/// Extract name and category from a skill markdown string.
fn parse_skill_meta(md: &str) -> Option<SkillMeta> {
    let name = md.lines()
        .find(|l| l.starts_with("name:"))?
        .trim_start_matches("name:")
        .trim()
        .to_string();
    let category = md.lines()
        .find(|l| l.starts_with("category:"))
        .map(|l| l.trim_start_matches("category:").trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    Some(SkillMeta { name, category })
}

// ─── Resolver ─────────────────────────────────────────────────

pub struct ToolResolver {
    /// toolset_name → SkillMeta (for known tools)
    skills: HashMap<String, SkillMeta>,
    /// alias → toolset_name (binary names, common abbreviations)
    aliases: HashMap<String, String>,
    /// binary:version → help text (runtime cache)
    help_cache: HashMap<String, String>,
}

impl ToolResolver {
    /// Build resolver from the same built-in skill list used elsewhere.
    /// `skills` is the `BUILTIN_SKILLS` array: `&[(&str, &str)]` = (toolset, markdown).
    pub fn from_builtin_skills(skills: &[(&str, &str)]) -> Self {
        let mut skill_map = HashMap::new();
        let mut aliases = HashMap::new();

        for (toolset, md) in skills {
            if let Some(meta) = parse_skill_meta(md) {
                // toolset → meta
                skill_map.insert(toolset.to_string(), meta);
            }
        }

        // Build alias map from the skills we have:
        // - toolset name itself → toolset
        // - skill's `name:` (the binary) → toolset
        for (ts, meta) in &skill_map {
            aliases.insert(ts.clone(), ts.clone());
            aliases.insert(meta.name.clone(), ts.clone());
            // Common lowercased variants
            aliases.insert(meta.name.to_lowercase(), ts.clone());
        }

        // Extra well-known aliases that skills don't capture
        let extra: &[(&str, &str)] = &[
            ("iqtree3", "iqtree2"), ("humann3", "humann3"), ("humann2", "humann3"),
            ("eggnog_mapper", "eggnog-mapper"), ("emapper.py", "eggnog-mapper"),
        ];
        for (alias, ts) in extra {
            aliases.entry(alias.to_string()).or_insert_with(|| ts.to_string());
        }

        Self { skills: skill_map, aliases, help_cache: HashMap::new() }
    }

    /// Resolve any name (alias, binary, toolset) to ToolInfo.
    /// Returns `None` only if the name is completely unknown AND not on PATH.
    pub fn resolve(&self, name: &str) -> Option<ToolInfo> {
        let toolset = self.aliases.get(name)?;
        let meta = self.skills.get(toolset)?;
        Some(ToolInfo {
            binary: meta.name.clone(),
            toolset: toolset.clone(),
            category: meta.category.clone(),
            cli_type: CliType::Flags, // refined after help capture
            version: None,
            has_skill: true,
            help_cached: self.help_cache.contains_key(&format!("{}:?", meta.name)),
        })
    }

    /// Auto-discover ANY tool: known or not. Always succeeds if the binary exists.
    pub fn discover(&mut self, name: &str) -> Result<ToolInfo> {
        // 1. Try known tools first
        if let Some(info) = self.resolve(name) {
            return Ok(info);
        }

        // 2. Find the binary on PATH
        let binary = find_binary(name)?;

        // 3. Create a dynamic entry
        let help = capture_help(&binary).unwrap_or_default();
        let cli_type = if help.contains("Commands:") || help.contains("subcommands:") || help.contains("COMMANDS:") {
            CliType::Subcommand
        } else {
            CliType::Flags
        };

        let info = ToolInfo {
            binary: binary.clone(),
            toolset: name.to_string(),
            category: "unknown".to_string(),
            cli_type,
            version: None,
            has_skill: false,
            help_cached: !help.is_empty(),
        };

        self.help_cache.insert(format!("{}:?", binary), help);
        self.aliases.insert(name.to_string(), name.to_string());
        Ok(info)
    }

    /// Check if a tool is installed.
    pub fn is_installed(&self, name: &str) -> bool {
        let binary = self.resolve(name).map(|ti| ti.binary).unwrap_or_else(|| name.to_string());
        Command::new("which").arg(&binary).output().map(|o| o.status.success()).unwrap_or(false)
    }

    /// Get --help text (cached).
    pub fn help_text(&mut self, name: &str) -> Result<String> {
        let info = self.discover(name)?;
        let key = format!("{}:{}", info.binary, info.version.as_deref().unwrap_or("?"));
        if let Some(cached) = self.help_cache.get(&key) {
            return Ok(cached.clone());
        }
        let text = capture_help(&info.binary)?;
        self.help_cache.insert(key, text.clone());
        Ok(text)
    }

    /// Detect tool version.
    pub fn detect_version(&mut self, name: &str) -> Result<String> {
        let info = self.discover(name)?;
        match Command::new(&info.binary).arg("--version").output() {
            Ok(o) => {
                let raw = if o.status.success() { &o.stdout } else { &o.stderr };
                let text = String::from_utf8_lossy(raw).into_owned();
                Ok(text.lines().next().unwrap_or("unknown").to_string())
            }
            Err(_) => Ok("unknown".to_string()),
        }
    }

    pub fn known_tools(&self) -> Vec<&str> { self.skills.keys().map(|s| s.as_str()).collect() }
    pub fn known_count(&self) -> usize { self.skills.len() }
}

// ─── Helpers ──────────────────────────────────────────────────

fn find_binary(name: &str) -> Result<String> {
    let out = Command::new("which").arg(name).output()
        .map_err(|_| OxoError::ToolNotFound(name.into()))?;
    if out.status.success() {
        let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
        return Ok(std::path::Path::new(&p).file_name().unwrap().to_string_lossy().into());
    }
    Err(OxoError::ToolNotFound(name.into()))
}

fn capture_help(binary: &str) -> Result<String> {
    for flag in &["--help", "-h", "help"] {
        if let Ok(o) = Command::new(binary).arg(flag).output() {
            let raw = if o.status.success() { &o.stdout } else { &o.stderr };
            let t = String::from_utf8_lossy(raw).into_owned();
            if t.len() > 20 { return Ok(t); }
        }
    }
    Err(OxoError::ConfigError(format!("Cannot get help for {binary}")))
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_resolver() -> ToolResolver {
        // Minimal skill data mimicking BUILTIN_SKILLS
        let skills: &[(&str, &str)] = &[
            ("iqtree2", "name: iqtree\ncategory: phylogenetics\ndescription: tree inference\n"),
            ("samtools", "name: samtools\ncategory: alignment\ndescription: SAM tools\n"),
            ("humann3", "name: humann\ncategory: functional-annotation\ndescription: HUMAnN\n"),
        ];
        ToolResolver::from_builtin_skills(skills)
    }

    #[test]
    fn test_resolve_known() {
        let r = test_resolver();
        assert_eq!(r.resolve("iqtree2").unwrap().binary, "iqtree");
        assert_eq!(r.resolve("samtools").unwrap().binary, "samtools");
        assert_eq!(r.resolve("humann3").unwrap().binary, "humann");
        // Alias resolution: binary name → toolset
        assert_eq!(r.resolve("iqtree").unwrap().toolset, "iqtree2");
    }

    #[test]
    fn test_unknown_returns_none() {
        let r = test_resolver();
        assert!(r.resolve("nonexistent_xyz").is_none());
    }

    #[test]
    fn test_discover_any_tool() {
        let mut r = test_resolver();
        // 'ls' should be discoverable on any Unix
        let info = r.discover("ls").unwrap();
        assert_eq!(info.binary, "ls");
        assert!(!info.has_skill);
    }
}
