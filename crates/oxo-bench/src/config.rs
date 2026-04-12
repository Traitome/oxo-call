//! Multi-model benchmark configuration.
//!
//! Parsed from a TOML file that lets users specify any number of LLM models
//! (with optional provider overrides), control parallelism, and set the
//! repeat count for consistency measurements.
//!
//! # Example `bench_config.toml`
//!
//! ```toml
//! [benchmark]
//! repeats = 3           # repeat each description N times per model
//! parallel = false       # run models in parallel (true) or serial (false)
//! skills_dir = "skills"  # path to the skills/ directory
//! output_dir = "bench_results"
//! scenarios = ["full"]   # ablation scenarios to evaluate
//!
//! [[models]]
//! name = "gpt-4o-mini"
//!
//! [[models]]
//! name = "qwen3.5:9b"
//! provider = "ollama"
//! api_base = "http://localhost:11434"
//! ```
//!
//! ## Ablation scenarios
//!
//! Each scenario controls which grounding sources are available to oxo-call:
//!
//! | Scenario | Skill | Doc (help index) | System prompt |
//! |----------|-------|-----------------|---------------|
//! | `bare`   | ✗     | ✗               | ✗             |
//! | `prompt` | ✗     | ✗               | ✓             |
//! | `skill`  | ✓     | ✗               | ✓             |
//! | `doc`    | ✗     | ✓               | ✓             |
//! | `full`   | ✓     | ✓               | ✓             |

use std::path::Path;

/// Top-level benchmark configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchConfig {
    pub benchmark: BenchmarkSettings,
    pub models: Vec<ModelEntry>,
}

/// Global benchmark settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkSettings {
    /// Number of repeat runs per description per model (default: 3).
    #[serde(default = "default_repeats")]
    pub repeats: usize,
    /// Whether to evaluate models in parallel using threads (default: false).
    #[serde(default)]
    pub parallel: bool,
    /// Number of parallel threads for description processing (default: 3).
    /// Only used when `parallel` is true.
    #[serde(default = "default_parallel_threads")]
    pub parallel_threads: usize,
    /// Path to the skills/ directory (default: "skills").
    #[serde(default = "default_skills_dir")]
    pub skills_dir: String,
    /// Output directory for result CSVs (default: "bench_results").
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    /// Ablation scenarios to evaluate (default: ["full"]).
    ///
    /// Valid values: "bare", "prompt", "skill", "doc", "full".
    #[serde(default = "default_scenarios")]
    pub scenarios: Vec<String>,
}

/// Configuration entry for a single LLM model.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelEntry {
    /// Model identifier passed to `oxo-call -m <name>`.
    pub name: String,
    /// Provider hint (e.g. "ollama", "openai", "anthropic").
    /// When set to "ollama", the benchmark uses the Ollama-compatible
    /// OpenAI endpoint at `api_base`.
    #[serde(default)]
    pub provider: Option<String>,
    /// Base URL for the model API (e.g. "http://localhost:11434" for Ollama).
    #[serde(default)]
    pub api_base: Option<String>,
    /// Optional API key for authenticated providers.
    #[serde(default)]
    pub api_key: Option<String>,
}

fn default_repeats() -> usize {
    3
}

fn default_parallel_threads() -> usize {
    3
}

fn default_skills_dir() -> String {
    "skills".to_string()
}

fn default_output_dir() -> String {
    "bench_results".to_string()
}

fn default_scenarios() -> Vec<String> {
    vec!["full".to_string()]
}

/// Ablation scenario — controls which grounding sources are available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AblationScenario {
    /// Bare LLM — no system prompt, no skill, no doc.
    Bare,
    /// LLM + oxo-call system prompt only.
    Prompt,
    /// LLM + system prompt + skill.
    Skill,
    /// LLM + system prompt + doc (help index).
    Doc,
    /// Full: LLM + system prompt + skill + doc.
    Full,
}

impl AblationScenario {
    /// Parse a scenario name string (case-insensitive).
    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bare" => Some(Self::Bare),
            "prompt" => Some(Self::Prompt),
            "skill" => Some(Self::Skill),
            "doc" => Some(Self::Doc),
            "full" => Some(Self::Full),
            _ => None,
        }
    }

    /// Canonical string name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Bare => "bare",
            Self::Prompt => "prompt",
            Self::Skill => "skill",
            Self::Doc => "doc",
            Self::Full => "full",
        }
    }

    /// Whether the skill file is loaded for this scenario.
    pub fn use_skill(&self) -> bool {
        matches!(self, Self::Skill | Self::Full)
    }

    /// Whether the doc/help index is loaded for this scenario.
    pub fn use_doc(&self) -> bool {
        matches!(self, Self::Doc | Self::Full)
    }

    /// Whether the oxo-call system prompt is used.
    pub fn use_prompt(&self) -> bool {
        !matches!(self, Self::Bare)
    }

    /// All valid scenario variants.
    pub fn all() -> &'static [Self] {
        &[Self::Bare, Self::Prompt, Self::Skill, Self::Doc, Self::Full]
    }
}

impl std::fmt::Display for AblationScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            benchmark: BenchmarkSettings {
                repeats: 3,
                parallel: false,
                parallel_threads: 3,
                skills_dir: "skills".to_string(),
                output_dir: "bench_results".to_string(),
                scenarios: vec!["full".to_string()],
            },
            models: vec![
                ModelEntry {
                    name: "gpt-4o-mini".to_string(),
                    provider: None,
                    api_base: None,
                    api_key: None,
                },
                ModelEntry {
                    name: "gpt-4o".to_string(),
                    provider: None,
                    api_base: None,
                    api_key: None,
                },
                ModelEntry {
                    name: "claude-3-5-sonnet-20241022".to_string(),
                    provider: None,
                    api_base: None,
                    api_key: None,
                },
            ],
        }
    }
}

impl BenchConfig {
    /// Load configuration from a TOML file.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: BenchConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Generate a default configuration file at the given path.
    pub fn write_default(path: &Path) -> anyhow::Result<()> {
        let config = Self::default();
        let toml_str = toml::to_string_pretty(&config)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, toml_str)?;
        Ok(())
    }

    /// Parse the configured ablation scenarios into typed values.
    ///
    /// Returns an error if any scenario name is unrecognised.
    pub fn ablation_scenarios(&self) -> anyhow::Result<Vec<AblationScenario>> {
        let mut scenarios = Vec::new();
        for s in &self.benchmark.scenarios {
            match AblationScenario::from_name(s) {
                Some(sc) => scenarios.push(sc),
                None => anyhow::bail!(
                    "Unknown ablation scenario '{}'. Valid values: bare, prompt, skill, doc, full",
                    s
                ),
            }
        }
        if scenarios.is_empty() {
            scenarios.push(AblationScenario::Full);
        }
        Ok(scenarios)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BenchConfig::default();
        assert_eq!(config.benchmark.repeats, 3);
        assert!(!config.benchmark.parallel);
        assert_eq!(config.benchmark.skills_dir, "skills");
        assert_eq!(config.models.len(), 3);
    }

    #[test]
    fn test_parse_minimal_toml() {
        let toml_str = r#"
[benchmark]
repeats = 5

[[models]]
name = "gpt-4o-mini"
"#;
        let config: BenchConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.benchmark.repeats, 5);
        assert_eq!(config.models.len(), 1);
        assert_eq!(config.models[0].name, "gpt-4o-mini");
    }

    #[test]
    fn test_parse_full_toml() {
        let toml_str = r#"
[benchmark]
repeats = 3
parallel = true
skills_dir = "/path/to/skills"
output_dir = "/path/to/output"

[[models]]
name = "gpt-4o-mini"

[[models]]
name = "gpt-4o"

[[models]]
name = "claude-3-5-sonnet-20241022"
"#;
        let config: BenchConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.benchmark.repeats, 3);
        assert!(config.benchmark.parallel);
        assert_eq!(config.models.len(), 3);
        assert_eq!(config.models[2].name, "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_roundtrip_serialization() {
        let config = BenchConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: BenchConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.benchmark.repeats, config.benchmark.repeats);
        assert_eq!(parsed.models.len(), config.models.len());
    }

    #[test]
    fn test_write_and_load_default() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("bench_config.toml");
        BenchConfig::write_default(&path).unwrap();
        assert!(path.exists());
        let loaded = BenchConfig::load(&path).unwrap();
        assert_eq!(loaded.benchmark.repeats, 3);
        assert_eq!(loaded.models.len(), 3);
    }

    #[test]
    fn test_defaults_when_fields_missing() {
        let toml_str = r#"
[benchmark]

[[models]]
name = "test-model"
"#;
        let config: BenchConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.benchmark.repeats, 3); // default
        assert!(!config.benchmark.parallel); // default
        assert_eq!(config.benchmark.skills_dir, "skills"); // default
        assert_eq!(config.benchmark.output_dir, "bench_results"); // default
        assert_eq!(config.benchmark.scenarios, vec!["full"]); // default
        assert!(config.models[0].provider.is_none()); // default
        assert!(config.models[0].api_base.is_none()); // default
    }

    #[test]
    fn test_parse_ollama_model() {
        let toml_str = r#"
[benchmark]
repeats = 1
scenarios = ["bare", "full"]

[[models]]
name = "qwen3.5:9b"
provider = "ollama"
api_base = "http://localhost:11434"
"#;
        let config: BenchConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.models[0].name, "qwen3.5:9b");
        assert_eq!(config.models[0].provider.as_deref(), Some("ollama"));
        assert_eq!(
            config.models[0].api_base.as_deref(),
            Some("http://localhost:11434")
        );
        assert_eq!(config.benchmark.scenarios, vec!["bare", "full"]);
    }

    #[test]
    fn test_ablation_scenarios_parsing() {
        let config = BenchConfig {
            benchmark: BenchmarkSettings {
                repeats: 1,
                parallel: false,
                parallel_threads: 3,
                skills_dir: "skills".to_string(),
                output_dir: "out".to_string(),
                scenarios: vec![
                    "bare".to_string(),
                    "prompt".to_string(),
                    "skill".to_string(),
                    "doc".to_string(),
                    "full".to_string(),
                ],
            },
            models: vec![],
        };
        let scenarios = config.ablation_scenarios().unwrap();
        assert_eq!(scenarios.len(), 5);
        assert_eq!(scenarios[0], AblationScenario::Bare);
        assert_eq!(scenarios[4], AblationScenario::Full);
    }

    #[test]
    fn test_ablation_scenario_properties() {
        assert!(!AblationScenario::Bare.use_skill());
        assert!(!AblationScenario::Bare.use_doc());
        assert!(!AblationScenario::Bare.use_prompt());

        assert!(!AblationScenario::Prompt.use_skill());
        assert!(!AblationScenario::Prompt.use_doc());
        assert!(AblationScenario::Prompt.use_prompt());

        assert!(AblationScenario::Skill.use_skill());
        assert!(!AblationScenario::Skill.use_doc());

        assert!(!AblationScenario::Doc.use_skill());
        assert!(AblationScenario::Doc.use_doc());

        assert!(AblationScenario::Full.use_skill());
        assert!(AblationScenario::Full.use_doc());
        assert!(AblationScenario::Full.use_prompt());
    }

    #[test]
    fn test_ablation_scenario_invalid() {
        let config = BenchConfig {
            benchmark: BenchmarkSettings {
                repeats: 1,
                parallel: false,
                parallel_threads: 3,
                skills_dir: "skills".to_string(),
                output_dir: "out".to_string(),
                scenarios: vec!["invalid_scenario".to_string()],
            },
            models: vec![],
        };
        assert!(config.ablation_scenarios().is_err());
    }
}
