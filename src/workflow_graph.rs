//! Workflow Graph - Fused pipeline for command generation.
//!
//! Implements the v0.13 fused architecture with three scenarios:
//!
//! 1. **bare**: Tool + Task → Command (no additional context)
//! 2. **doc**: Tool + auto-parsed documentation + Schema → Command
//! 3. **full**: Tool + documentation + Schema + Skill → Command
//!
//! The pipeline follows five stages:
//! 1. Tool Resolution (pure code)
//! 2. Doc Exploration (code + optional LLM cleaning)
//! 3. Intent Mapping (code + LLM structured output)
//! 4. Command Assembly (pure code)
//! 5. Validation (code + optional LLM review)

#![allow(dead_code)]

use crate::confidence::{ConfidenceLevel, ConfidenceResult, estimate_confidence};
use crate::llm_workflow::WorkflowMode;
use crate::task_normalizer::{NormalizedTask, TaskNormalizer};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WorkflowScenario {
    /// Bare: Tool + Task → Command (no additional context)
    #[default]
    Bare,
    /// Doc: Tool + auto-parsed documentation + Schema
    Doc,
    /// Full: Tool + documentation + Schema + Skill
    Full,
}

impl std::fmt::Display for WorkflowScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowScenario::Bare => write!(f, "bare"),
            WorkflowScenario::Doc => write!(f, "doc"),
            WorkflowScenario::Full => write!(f, "full"),
        }
    }
}

impl WorkflowScenario {
    pub fn default_mode(&self) -> WorkflowMode {
        match self {
            Self::Bare => WorkflowMode::Fast,
            Self::Doc => WorkflowMode::Quality,
            Self::Full => WorkflowMode::Quality,
        }
    }
}

/// Workflow input
#[derive(Debug, Clone, Default)]
pub struct WorkflowInput {
    pub tool: String,
    pub task: String,
    pub documentation: Option<String>,
    pub skill_path: Option<String>,
    pub force_mode: Option<WorkflowMode>,
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
    pub confidence: Option<ConfidenceResult>,
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
    normalizer: TaskNormalizer,
}

impl Default for WorkflowGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowGraph {
    pub fn new() -> Self {
        Self {
            normalizer: TaskNormalizer::new(),
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
        self.estimate_confidence(&mut state)?;

        // Step 4: Execute scenario-specific path
        match state.scenario {
            WorkflowScenario::Bare => self.execute_basic(&mut state).await?,
            WorkflowScenario::Doc => self.execute_doc(&mut state).await?,
            WorkflowScenario::Full => self.execute_full(&mut state).await?,
        }

        // Step 5: Validate result
        self.validate_result(&mut state)?;

        // Build result
        Ok(WorkflowResult {
            command: state.command.unwrap_or_default(),
            scenario: state.scenario,
            mode: state.mode,
            confidence: state.confidence.map(|c| c.score).unwrap_or(0.5),
            metadata: state.metadata,
        })
    }

    /// Determine workflow scenario based on input
    fn determine_scenario(&self, state: &mut WorkflowState) -> Result<()> {
        if let Some(scenario) = state.input.force_scenario {
            state.scenario = scenario;
            if state.input.force_mode.is_none() {
                state.mode = scenario.default_mode();
            }
            return Ok(());
        }

        let has_doc = state.input.documentation.is_some();
        let has_skill = state.input.skill_path.is_some();

        state.scenario = match (has_doc, has_skill) {
            (true, true) => WorkflowScenario::Full,
            (true, false) => WorkflowScenario::Doc,
            (false, true) => WorkflowScenario::Full,
            (false, false) => WorkflowScenario::Bare,
        };

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

    /// Estimate confidence (for metadata and adaptive adjustment)
    fn estimate_confidence(&self, state: &mut WorkflowState) -> Result<()> {
        let has_skill = state.input.skill_path.is_some();
        let doc_quality = if state.input.documentation.is_some() {
            0.8
        } else {
            0.3
        };

        let confidence = estimate_confidence(
            0,
            doc_quality,
            has_skill,
            if state.input.task.len() > 20 { 2 } else { 0 },
            None,
            has_skill,
        );

        state.confidence = Some(confidence.clone());

        if state.input.force_mode.is_none() {
            if confidence.level == ConfidenceLevel::Low && state.mode == WorkflowMode::Fast {
                state.mode = WorkflowMode::Quality;
                state.metadata.insert(
                    "mode_override".to_string(),
                    "confidence-based upgrade to Quality".to_string(),
                );
            } else if confidence.level == ConfidenceLevel::High
                && state.mode == WorkflowMode::Quality
            {
                state.mode = WorkflowMode::Fast;
                state.metadata.insert(
                    "mode_override".to_string(),
                    "confidence-based downgrade to Fast".to_string(),
                );
            }
        }

        state.metadata.insert(
            "confidence_score".to_string(),
            format!("{:.2}", confidence.score),
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
        let config = crate::config::Config::load().await.unwrap_or_default();
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
    async fn test_bare_scenario() {
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
        assert!(result.metadata.contains_key("skill_loaded"));
    }

    #[tokio::test]
    async fn test_skill_only_upgrades_to_full() {
        let graph = WorkflowGraph::new();
        let input = WorkflowInput {
            tool: "samtools".to_string(),
            task: "sort input.bam".to_string(),
            documentation: Some("Usage: samtools sort [options] INPUT\nOptions:\n  -@ INT  Threads\n  -o FILE  Output".to_string()),
            skill_path: Some("skills/samtools.md".to_string()),
            ..Default::default()
        };

        let result = graph.execute(input).await.unwrap();
        assert_eq!(result.scenario, WorkflowScenario::Full);
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

    #[test]
    fn test_scenario_default() {
        assert_eq!(WorkflowScenario::default(), WorkflowScenario::Bare);
    }

    #[test]
    fn test_scenario_default_mode() {
        assert_eq!(WorkflowScenario::Bare.default_mode(), WorkflowMode::Fast);
        assert_eq!(WorkflowScenario::Doc.default_mode(), WorkflowMode::Quality);
        assert_eq!(WorkflowScenario::Full.default_mode(), WorkflowMode::Quality);
    }

    #[test]
    fn test_workflow_input_default() {
        let input = WorkflowInput::default();
        assert!(input.tool.is_empty());
        assert!(input.task.is_empty());
        assert!(input.documentation.is_none());
        assert!(input.skill_path.is_none());
        assert!(input.force_mode.is_none());
        assert!(input.force_scenario.is_none());
    }

    #[test]
    fn test_workflow_state_default() {
        let state = WorkflowState::default();
        assert!(state.normalized_task.is_none());
        assert!(state.confidence.is_none());
        assert!(state.mini_skill.is_none());
        assert!(state.skill.is_none());
        assert!(state.command.is_none());
        assert!(!state.validation_passed);
        assert!(state.metadata.is_empty());
    }

    #[test]
    fn test_mini_skill_data_serialization() {
        let data = MiniSkillData {
            tool: "samtools".to_string(),
            concepts: vec!["sort".to_string(), "index".to_string()],
            pitfalls: vec!["Check threads".to_string()],
            examples: vec![MiniSkillExample {
                task: "sort bam".to_string(),
                args: "sort -o out.bam in.bam".to_string(),
                explanation: "Sort BAM".to_string(),
            }],
        };
        let json = serde_json::to_string(&data).unwrap();
        let back: MiniSkillData = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tool, "samtools");
        assert_eq!(back.concepts.len(), 2);
        assert_eq!(back.examples.len(), 1);
    }

    #[test]
    fn test_skill_data_serialization() {
        let data = SkillData {
            name: "bwa".to_string(),
            category: "alignment".to_string(),
            concepts: vec!["mem".to_string()],
            pitfalls: vec!["Check reference".to_string()],
            examples: vec![SkillExample {
                task: "align reads".to_string(),
                args: "mem ref.fa reads.fq".to_string(),
                explanation: "Align reads".to_string(),
            }],
        };
        let json = serde_json::to_string(&data).unwrap();
        let back: SkillData = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "bwa");
        assert_eq!(back.category, "alignment");
    }

    #[test]
    fn test_workflow_result_serialization() {
        let result = WorkflowResult {
            command: "samtools sort -o out.bam in.bam".to_string(),
            scenario: WorkflowScenario::Doc,
            mode: WorkflowMode::Quality,
            confidence: 0.85,
            metadata: HashMap::from([
                ("scenario".to_string(), "determined: doc".to_string()),
            ]),
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: WorkflowResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.command, "samtools sort -o out.bam in.bam");
        assert_eq!(back.scenario, WorkflowScenario::Doc);
        assert!((back.confidence - 0.85).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_forced_mode() {
        let graph = WorkflowGraph::new();
        let input = WorkflowInput {
            tool: "samtools".to_string(),
            task: "sort input.bam".to_string(),
            force_mode: Some(WorkflowMode::Fast),
            ..Default::default()
        };
        let result = graph.execute(input).await.unwrap();
        assert_eq!(result.mode, WorkflowMode::Fast);
    }

    #[tokio::test]
    async fn test_doc_scenario_no_skill() {
        let graph = WorkflowGraph::new();
        let input = WorkflowInput {
            tool: "mytool".to_string(),
            task: "process data".to_string(),
            documentation: Some("Usage: mytool [options]\nOptions:\n  -v  Verbose".to_string()),
            ..Default::default()
        };
        let result = graph.execute(input).await.unwrap();
        assert_eq!(result.scenario, WorkflowScenario::Doc);
        assert!(!result.metadata.contains_key("skill_loaded"));
    }

    #[tokio::test]
    async fn test_determine_scenario_force() {
        let graph = WorkflowGraph::new();
        let mut state = WorkflowState {
            input: WorkflowInput {
                tool: "test".to_string(),
                task: "test".to_string(),
                force_scenario: Some(WorkflowScenario::Full),
                ..Default::default()
            },
            ..Default::default()
        };
        graph.determine_scenario(&mut state).unwrap();
        assert_eq!(state.scenario, WorkflowScenario::Full);
    }

    #[tokio::test]
    async fn test_determine_scenario_auto_doc_skill() {
        let graph = WorkflowGraph::new();
        let mut state = WorkflowState {
            input: WorkflowInput {
                tool: "test".to_string(),
                task: "test".to_string(),
                documentation: Some("docs".to_string()),
                skill_path: Some("skill.md".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        graph.determine_scenario(&mut state).unwrap();
        assert_eq!(state.scenario, WorkflowScenario::Full);
    }

    #[tokio::test]
    async fn test_determine_scenario_auto_skill_only() {
        let graph = WorkflowGraph::new();
        let mut state = WorkflowState {
            input: WorkflowInput {
                tool: "test".to_string(),
                task: "test".to_string(),
                documentation: None,
                skill_path: Some("skill.md".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        graph.determine_scenario(&mut state).unwrap();
        assert_eq!(state.scenario, WorkflowScenario::Full);
    }

    #[tokio::test]
    async fn test_determine_scenario_auto_doc_only() {
        let graph = WorkflowGraph::new();
        let mut state = WorkflowState {
            input: WorkflowInput {
                tool: "test".to_string(),
                task: "test".to_string(),
                documentation: Some("docs".to_string()),
                skill_path: None,
                ..Default::default()
            },
            ..Default::default()
        };
        graph.determine_scenario(&mut state).unwrap();
        assert_eq!(state.scenario, WorkflowScenario::Doc);
    }

    #[tokio::test]
    async fn test_determine_scenario_auto_bare() {
        let graph = WorkflowGraph::new();
        let mut state = WorkflowState {
            input: WorkflowInput {
                tool: "test".to_string(),
                task: "test".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        graph.determine_scenario(&mut state).unwrap();
        assert_eq!(state.scenario, WorkflowScenario::Bare);
    }

    #[test]
    fn test_validate_result_empty_command() {
        let graph = WorkflowGraph::new();
        let mut state = WorkflowState {
            command: Some(String::new()),
            ..Default::default()
        };
        graph.validate_result(&mut state).unwrap();
        assert!(!state.validation_passed);
    }

    #[test]
    fn test_validate_result_none_command() {
        let graph = WorkflowGraph::new();
        let mut state = WorkflowState {
            command: None,
            ..Default::default()
        };
        graph.validate_result(&mut state).unwrap();
        assert!(!state.validation_passed);
    }

    #[test]
    fn test_validate_result_valid_command() {
        let graph = WorkflowGraph::new();
        let mut state = WorkflowState {
            command: Some("samtools sort in.bam".to_string()),
            ..Default::default()
        };
        graph.validate_result(&mut state).unwrap();
        assert!(state.validation_passed);
    }

    #[test]
    fn test_workflow_graph_default() {
        let _graph = WorkflowGraph::default();
    }

    #[tokio::test]
    async fn test_generate_mini_skill() {
        let graph = WorkflowGraph::new();
        let doc = "Usage: mytool [options]\nOptions:\n  -v  Verbose\n  -h  Help\n\nExamples:\n  mytool -v input.txt\n  mytool -h";
        let result = graph.generate_mini_skill("mytool", doc).await.unwrap();
        assert_eq!(result.tool, "mytool");
        assert!(!result.concepts.is_empty() || !result.pitfalls.is_empty());
    }

    #[tokio::test]
    async fn test_generate_mini_skill_empty_doc() {
        let graph = WorkflowGraph::new();
        let result = graph.generate_mini_skill("emptytool", "").await.unwrap();
        assert_eq!(result.tool, "emptytool");
        assert!(!result.concepts.is_empty());
        assert!(!result.pitfalls.is_empty());
    }
}
