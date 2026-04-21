//! Workflow Graph - DAG-based orchestration for command generation
//!
//! This module implements a LangGraph-inspired workflow engine that orchestrates
//! multiple stages of LLM processing based on input conditions.
//!
//! # Workflow Scenarios
//!
//! 1. **basic**: Tool + Task → Command (fastest)
//! 2. **prompt**: basic + Custom Prompt → Command
//! 3. **doc**: basic + Documentation + Mini-skill → Command
//! 4. **skill**: basic + Skill File → Command
//! 5. **full**: doc + skill (combined) → Command
//!
//! # Architecture
//!
//! ```text
//! Start → [Task Normalization] → [Complexity Estimation]
//!                                     ↓
//!                 ┌───────────────────┴───────────────────┐
//!                 │                                       │
//!            Fast Path                              Quality Path
//!                 │                                       │
//!                 ↓                                       ↓
//!         Basic Generator              ┌─────────────────┴─────────────────┐
//!                 │                     │                                   │
//!                 │                [Doc Processing]                  [Skill Loading]
//!                 │                     │                                   │
//!                 │                     ↓                                   ↓
//!                 │              [Mini-skill Gen]              [Skill Integration]
//!                 │                     │                                   │
//!                 │                     └───────────────┬───────────────────┘
//!                 │                                     │
//!                 │                              [Command Generation]
//!                 │                                     │
//!                 └─────────────────────────────────────┘
//!                                                       ↓
//!                                                [Validation]
//!                                                       ↓
//!                                                      End
//! ```

#![allow(dead_code)]

use crate::task_complexity::{ComplexityResult, TaskComplexityEstimator, WorkflowMode};
use crate::task_normalizer::{NormalizedTask, TaskNormalizer};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Workflow scenario type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WorkflowScenario {
    /// Bare: Tool + Task → Command (no additional context)
    #[default]
    Bare,
    /// Prompt: Bare + Custom Prompt
    Prompt,
    /// Doc: Bare + Documentation + Mini-skill
    Doc,
    /// Skill: Bare + Skill File
    Skill,
    /// Full: Doc + Skill (combined)
    Full,
}

impl std::fmt::Display for WorkflowScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowScenario::Bare => write!(f, "bare"),
            WorkflowScenario::Prompt => write!(f, "prompt"),
            WorkflowScenario::Doc => write!(f, "doc"),
            WorkflowScenario::Skill => write!(f, "skill"),
            WorkflowScenario::Full => write!(f, "full"),
        }
    }
}

impl WorkflowScenario {
    /// Get default workflow mode for this scenario
    ///
    /// # Scenario-Mode Mapping
    ///
    /// - **Bare**: Fast (simplest, no processing needed)
    /// - **Prompt**: Fast (user-defined, trust their prompt)
    /// - **Doc**: Quality (needs mini-skill generation)
    /// - **Skill**: Fast (existing skill is sufficient)
    /// - **Full**: Quality (complex combination)
    pub fn default_mode(&self) -> WorkflowMode {
        match self {
            Self::Bare => WorkflowMode::Fast,
            Self::Prompt => WorkflowMode::Fast,
            Self::Doc => WorkflowMode::Quality,
            Self::Skill => WorkflowMode::Fast,
            Self::Full => WorkflowMode::Quality,
        }
    }
}

/// Workflow input
#[derive(Debug, Clone, Default)]
pub struct WorkflowInput {
    /// Tool name
    pub tool: String,
    /// User task description
    pub task: String,
    /// Custom prompt (optional)
    pub custom_prompt: Option<String>,
    /// Documentation (optional)
    pub documentation: Option<String>,
    /// Skill file path (optional)
    pub skill_path: Option<String>,
    /// Force workflow mode (optional)
    pub force_mode: Option<WorkflowMode>,
    /// Force scenario (optional)
    pub force_scenario: Option<WorkflowScenario>,
}

/// Workflow state (shared across nodes)
#[derive(Debug, Clone, Default)]
pub struct WorkflowState {
    /// Input data
    pub input: WorkflowInput,
    /// Normalized task
    pub normalized_task: Option<NormalizedTask>,
    /// Complexity estimation result
    pub complexity: Option<ComplexityResult>,
    /// Selected workflow mode
    pub mode: WorkflowMode,
    /// Selected scenario
    pub scenario: WorkflowScenario,
    /// Generated mini-skill (doc scenario)
    pub mini_skill: Option<MiniSkillData>,
    /// Loaded skill (skill scenario)
    pub skill: Option<SkillData>,
    /// Generated command
    pub command: Option<String>,
    /// Validation result
    pub validation_passed: bool,
    /// Processing metadata
    pub metadata: HashMap<String, String>,
}

/// Mini-skill data (generated from documentation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniSkillData {
    pub tool: String,
    pub concepts: Vec<String>,
    pub pitfalls: Vec<String>,
    pub examples: Vec<MiniSkillExample>,
}

/// Mini-skill example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniSkillExample {
    pub task: String,
    pub args: String,
    pub explanation: String,
}

/// Skill data (loaded from file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillData {
    pub name: String,
    pub category: String,
    pub concepts: Vec<String>,
    pub pitfalls: Vec<String>,
    pub examples: Vec<SkillExample>,
}

/// Skill example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExample {
    pub task: String,
    pub args: String,
    pub explanation: String,
}

/// Workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub command: String,
    pub scenario: WorkflowScenario,
    pub mode: WorkflowMode,
    pub confidence: f32,
    pub metadata: HashMap<String, String>,
}

/// Workflow Graph executor
pub struct WorkflowGraph {
    /// Task normalizer
    normalizer: Arc<TaskNormalizer>,
    /// Complexity estimator
    estimator: Arc<TaskComplexityEstimator>,
}

impl Default for WorkflowGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowGraph {
    pub fn new() -> Self {
        Self {
            normalizer: Arc::new(TaskNormalizer::new()),
            estimator: Arc::new(TaskComplexityEstimator::new()),
        }
    }

    /// Execute workflow
    pub async fn execute(&self, input: WorkflowInput) -> Result<WorkflowResult> {
        // Initialize state
        let mut state = WorkflowState {
            input: input.clone(),
            ..Default::default()
        };

        // Step 1: Determine scenario
        self.determine_scenario(&mut state)?;

        // Step 2: Task normalization (for quality mode)
        if state.mode == WorkflowMode::Quality {
            self.normalize_task(&mut state).await?;
        }

        // Step 3: Complexity estimation
        self.estimate_complexity(&mut state)?;

        // Step 4: Execute scenario-specific path
        match state.scenario {
            WorkflowScenario::Bare => self.execute_basic(&mut state).await?,
            WorkflowScenario::Prompt => self.execute_prompt(&mut state).await?,
            WorkflowScenario::Doc => self.execute_doc(&mut state).await?,
            WorkflowScenario::Skill => self.execute_skill(&mut state).await?,
            WorkflowScenario::Full => self.execute_full(&mut state).await?,
        }

        // Step 5: Validate result
        self.validate_result(&mut state)?;

        // Build result
        Ok(WorkflowResult {
            command: state.command.unwrap_or_default(),
            scenario: state.scenario,
            mode: state.mode,
            confidence: state.complexity.map(|c| c.confidence).unwrap_or(0.5),
            metadata: state.metadata,
        })
    }

    /// Determine workflow scenario based on input
    fn determine_scenario(&self, state: &mut WorkflowState) -> Result<()> {
        // Check if scenario is forced
        if let Some(scenario) = state.input.force_scenario {
            state.scenario = scenario;
            // Set default mode for this scenario
            if state.input.force_mode.is_none() {
                state.mode = scenario.default_mode();
            }
            return Ok(());
        }

        // Determine scenario based on available inputs
        let has_doc = state.input.documentation.is_some();
        let has_skill = state.input.skill_path.is_some();
        let has_prompt = state.input.custom_prompt.is_some();

        state.scenario = match (has_doc, has_skill, has_prompt) {
            (true, true, _) => WorkflowScenario::Full, // doc + skill = full
            (true, false, _) => WorkflowScenario::Doc, // doc only
            (false, true, _) => WorkflowScenario::Skill, // skill only
            (false, false, true) => WorkflowScenario::Prompt, // prompt only
            (false, false, false) => WorkflowScenario::Bare, // bare
        };

        // Set default mode for this scenario (unless forced)
        if state.input.force_mode.is_none() {
            state.mode = state.scenario.default_mode();
        }

        state.metadata.insert(
            "scenario".to_string(),
            format!("determined: {}", state.scenario),
        );

        Ok(())
    }

    /// Normalize task (Quality mode only)
    async fn normalize_task(&self, state: &mut WorkflowState) -> Result<()> {
        let normalized = self
            .normalizer
            .normalize(&state.input.task, &state.input.tool)
            .await?;
        state.normalized_task = Some(normalized);
        Ok(())
    }

    /// Estimate task complexity (for metadata and adaptive adjustment)
    fn estimate_complexity(&self, state: &mut WorkflowState) -> Result<()> {
        // Mode is already set by scenario.default_mode() or force_mode
        // This method only estimates complexity for metadata

        let has_skill = state.input.skill_path.is_some();
        let doc_quality = if state.input.documentation.is_some() {
            0.8
        } else {
            0.3
        };

        let complexity =
            self.estimator
                .estimate(&state.input.task, &state.input.tool, has_skill, doc_quality);

        state.complexity = Some(complexity.clone());

        // Optional: Override mode if complexity suggests a different mode
        // and mode is not forced
        if state.input.force_mode.is_none() {
            // Only override if there's a strong signal
            if complexity.score.0 > 0.8 && state.mode == WorkflowMode::Fast {
                // Very complex task, upgrade to Quality mode
                state.mode = WorkflowMode::Quality;
                state.metadata.insert(
                    "mode_override".to_string(),
                    "complexity-based upgrade to Quality".to_string(),
                );
            } else if complexity.score.0 < 0.2 && state.mode == WorkflowMode::Quality {
                // Very simple task, downgrade to Fast mode
                state.mode = WorkflowMode::Fast;
                state.metadata.insert(
                    "mode_override".to_string(),
                    "complexity-based downgrade to Fast".to_string(),
                );
            }
        }

        state.metadata.insert(
            "complexity_score".to_string(),
            format!("{:.2}", complexity.score.0),
        );

        Ok(())
    }

    /// Execute basic scenario
    async fn execute_basic(&self, state: &mut WorkflowState) -> Result<()> {
        // Direct command generation without any enhancement
        state.command = Some(format!("{} {}", state.input.tool, state.input.task));
        state
            .metadata
            .insert("path".to_string(), "basic".to_string());
        Ok(())
    }

    /// Execute prompt scenario
    async fn execute_prompt(&self, state: &mut WorkflowState) -> Result<()> {
        // Use custom prompt for generation
        let prompt =
            state.input.custom_prompt.as_ref().ok_or_else(|| {
                color_eyre::eyre::eyre!("Custom prompt required for prompt scenario")
            })?;
        state.command = Some(format!(
            "{} {} # prompt: {}",
            state.input.tool, state.input.task, prompt
        ));
        state
            .metadata
            .insert("path".to_string(), "prompt".to_string());
        Ok(())
    }

    /// Execute doc scenario (includes mini-skill generation)
    async fn execute_doc(&self, state: &mut WorkflowState) -> Result<()> {
        // Step 1: Process documentation
        let doc =
            state.input.documentation.as_ref().ok_or_else(|| {
                color_eyre::eyre::eyre!("Documentation required for doc scenario")
            })?;

        // Step 2: Generate mini-skill from documentation
        let mini_skill = self.generate_mini_skill(&state.input.tool, doc).await?;
        state.mini_skill = Some(mini_skill.clone());

        // Step 3: Use mini-skill for command generation
        state.command = Some(format!(
            "{} {} # mini-skill: {} examples",
            state.input.tool,
            state.input.task,
            mini_skill.examples.len()
        ));

        state.metadata.insert("path".to_string(), "doc".to_string());
        state
            .metadata
            .insert("mini_skill_generated".to_string(), "true".to_string());

        Ok(())
    }

    /// Execute skill scenario
    async fn execute_skill(&self, state: &mut WorkflowState) -> Result<()> {
        // Load skill from file
        let skill_path = state
            .input
            .skill_path
            .as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("Skill path required for skill scenario"))?;
        let skill = self.load_skill(skill_path).await?;
        state.skill = Some(skill.clone());

        // Use skill for command generation
        state.command = Some(format!(
            "{} {} # skill: {} examples",
            state.input.tool,
            state.input.task,
            skill.examples.len()
        ));

        state
            .metadata
            .insert("path".to_string(), "skill".to_string());
        state
            .metadata
            .insert("skill_loaded".to_string(), "true".to_string());
        Ok(())
    }

    /// Execute full scenario (doc + skill combined)
    async fn execute_full(&self, state: &mut WorkflowState) -> Result<()> {
        // Step 1: Process documentation and generate mini-skill
        let doc =
            state.input.documentation.as_ref().ok_or_else(|| {
                color_eyre::eyre::eyre!("Documentation required for full scenario")
            })?;
        let mini_skill = self.generate_mini_skill(&state.input.tool, doc).await?;
        state.mini_skill = Some(mini_skill.clone());

        // Step 2: Load skill from file (best-effort; gracefully degrade if unavailable)
        let mut skill_examples = 0usize;
        if let Some(skill_path) = state.input.skill_path.as_ref() {
            match self.load_skill(skill_path).await {
                Ok(skill) => {
                    skill_examples = skill.examples.len();
                    state.skill = Some(skill);
                    state
                        .metadata
                        .insert("skill_loaded".to_string(), "true".to_string());
                }
                Err(_) => {
                    state
                        .metadata
                        .insert("skill_loaded".to_string(), "fallback".to_string());
                }
            }
        }

        // Step 3: Combine mini-skill and skill for command generation
        let combined_examples = mini_skill.examples.len() + skill_examples;
        state.command = Some(format!(
            "{} {} # combined: {} examples (mini-skill + skill)",
            state.input.tool, state.input.task, combined_examples
        ));

        state
            .metadata
            .insert("path".to_string(), "full".to_string());
        state
            .metadata
            .insert("mini_skill_generated".to_string(), "true".to_string());

        Ok(())
    }

    /// Generate mini-skill from documentation using rule-based extraction.
    ///
    /// Uses `DocProcessor::clean_and_structure` to extract concepts, pitfalls,
    /// and examples from tool documentation.
    async fn generate_mini_skill(&self, tool: &str, doc: &str) -> Result<MiniSkillData> {
        let processor = crate::doc_processor::DocProcessor::new();
        let structured = processor.clean_and_structure(doc);

        let concepts: Vec<String> = structured.quick_flags.iter().take(5).cloned().collect();

        let pitfalls: Vec<String> = if !structured.options.is_empty() {
            vec![format!("Check required options before running {tool}")]
        } else {
            vec![]
        };

        let examples: Vec<MiniSkillExample> = structured
            .extracted_examples
            .iter()
            .take(5)
            .map(|ex| MiniSkillExample {
                task: format!("run {tool}"),
                args: ex.clone(),
                explanation: format!("Example usage of {tool}"),
            })
            .collect();

        Ok(MiniSkillData {
            tool: tool.to_string(),
            concepts: if concepts.is_empty() {
                vec![format!("{tool} documentation processed")]
            } else {
                concepts
            },
            pitfalls: if pitfalls.is_empty() {
                vec![format!("Verify {tool} is installed and on PATH")]
            } else {
                pitfalls
            },
            examples: if examples.is_empty() {
                vec![MiniSkillExample {
                    task: format!("run {tool}"),
                    args: structured.usage.clone(),
                    explanation: format!("Basic usage of {tool}"),
                }]
            } else {
                examples
            },
        })
    }

    /// Load skill from file using the SkillManager.
    async fn load_skill(&self, path: &str) -> Result<SkillData> {
        let skill_path = std::path::Path::new(path);
        if skill_path.exists()
            && let Ok(content) = std::fs::read_to_string(skill_path)
            && let Some(skill) = crate::skill::parse_skill_md(&content)
        {
            return Ok(SkillData {
                name: skill.meta.name,
                category: skill.meta.category,
                concepts: skill.context.concepts,
                pitfalls: skill.context.pitfalls,
                examples: skill
                    .examples
                    .into_iter()
                    .map(|ex| SkillExample {
                        task: ex.task,
                        args: ex.args,
                        explanation: ex.explanation,
                    })
                    .collect(),
            });
        }

        // Fallback: try to load as a built-in skill by treating path as a tool name.
        let tool_name = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(path);
        let config = crate::config::Config::load().unwrap_or_default();
        let mgr = crate::skill::SkillManager::new(config);
        if let Some(skill) = mgr.load(tool_name) {
            return Ok(SkillData {
                name: skill.meta.name,
                category: skill.meta.category,
                concepts: skill.context.concepts,
                pitfalls: skill.context.pitfalls,
                examples: skill
                    .examples
                    .into_iter()
                    .map(|ex| SkillExample {
                        task: ex.task,
                        args: ex.args,
                        explanation: ex.explanation,
                    })
                    .collect(),
            });
        }

        Err(color_eyre::eyre::eyre!(
            "Could not load skill from '{path}'"
        ))
    }

    /// Validate generated command
    fn validate_result(&self, state: &mut WorkflowState) -> Result<()> {
        // Basic validation: command should not be empty
        if let Some(ref cmd) = state.command {
            state.validation_passed = !cmd.is_empty();
        } else {
            state.validation_passed = false;
        }

        state.metadata.insert(
            "validation".to_string(),
            if state.validation_passed {
                "passed"
            } else {
                "failed"
            }
            .to_string(),
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_scenario() {
        let graph = WorkflowGraph::new();
        let input = WorkflowInput {
            tool: "samtools".to_string(),
            task: "sort input.bam".to_string(),
            ..Default::default()
        };

        let result = graph.execute(input).await.unwrap();
        assert_eq!(result.scenario, WorkflowScenario::Bare);
        assert!(result.command.contains("samtools"));
    }

    #[tokio::test]
    async fn test_doc_scenario() {
        let graph = WorkflowGraph::new();
        let input = WorkflowInput {
            tool: "mytool".to_string(),
            task: "process input.txt".to_string(),
            documentation: Some("tool documentation".to_string()),
            ..Default::default()
        };

        let result = graph.execute(input).await.unwrap();
        assert_eq!(result.scenario, WorkflowScenario::Doc);
        assert!(result.metadata.contains_key("mini_skill_generated"));
    }

    #[tokio::test]
    async fn test_skill_scenario() {
        let graph = WorkflowGraph::new();
        let input = WorkflowInput {
            tool: "samtools".to_string(),
            task: "sort input.bam".to_string(),
            skill_path: Some("skills/samtools.md".to_string()),
            ..Default::default()
        };

        let result = graph.execute(input).await.unwrap();
        assert_eq!(result.scenario, WorkflowScenario::Skill);
        assert!(result.metadata.contains_key("skill_loaded"));
    }

    #[tokio::test]
    async fn test_full_scenario() {
        let graph = WorkflowGraph::new();
        let input = WorkflowInput {
            tool: "mytool".to_string(),
            task: "process input.txt".to_string(),
            documentation: Some("tool documentation".to_string()),
            skill_path: Some("skills/mytool.md".to_string()),
            ..Default::default()
        };

        let result = graph.execute(input).await.unwrap();
        assert_eq!(result.scenario, WorkflowScenario::Full);
        assert!(result.metadata.contains_key("mini_skill_generated"));
        // skill_loaded may be "true" or "fallback" depending on availability
        assert!(result.metadata.contains_key("skill_loaded"));
    }

    #[tokio::test]
    async fn test_forced_scenario() {
        let graph = WorkflowGraph::new();
        let input = WorkflowInput {
            tool: "samtools".to_string(),
            task: "sort input.bam".to_string(),
            documentation: Some("tool documentation".to_string()),
            force_scenario: Some(WorkflowScenario::Doc),
            ..Default::default()
        };

        let result = graph.execute(input).await.unwrap();
        assert_eq!(result.scenario, WorkflowScenario::Doc);
    }

    #[test]
    fn test_scenario_display() {
        assert_eq!(format!("{}", WorkflowScenario::Bare), "bare");
        assert_eq!(format!("{}", WorkflowScenario::Doc), "doc");
        assert_eq!(format!("{}", WorkflowScenario::Full), "full");
    }
}
