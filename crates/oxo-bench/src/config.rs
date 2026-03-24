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
//!
//! [[models]]
//! name = "gpt-4o-mini"
//!
//! [[models]]
//! name = "gpt-4o"
//!
//! [[models]]
//! name = "claude-3-5-sonnet-20241022"
//! ```

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
    /// Path to the skills/ directory (default: "skills").
    #[serde(default = "default_skills_dir")]
    pub skills_dir: String,
    /// Output directory for result CSVs (default: "bench_results").
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

/// Configuration entry for a single LLM model.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelEntry {
    /// Model identifier passed to `oxo-call -m <name>`.
    pub name: String,
}

fn default_repeats() -> usize {
    3
}

fn default_skills_dir() -> String {
    "skills".to_string()
}

fn default_output_dir() -> String {
    "bench_results".to_string()
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            benchmark: BenchmarkSettings {
                repeats: 3,
                parallel: false,
                skills_dir: "skills".to_string(),
                output_dir: "bench_results".to_string(),
            },
            models: vec![
                ModelEntry {
                    name: "gpt-4o-mini".to_string(),
                },
                ModelEntry {
                    name: "gpt-4o".to_string(),
                },
                ModelEntry {
                    name: "claude-3-5-sonnet-20241022".to_string(),
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
    }
}
