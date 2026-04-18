//! Supervisor Agent — routes tasks and selects orchestration strategy.
//!
//! The supervisor is the entry point for the orchestration layer.  It
//! examines the user's task, the available context (skill, docs, history),
//! and decides whether to use a fast single-call or a multi-agent pipeline.

use crate::knowledge::best_practices::BestPracticesDb;
use crate::knowledge::tool_knowledge::ToolKnowledgeBase;
use crate::task_complexity::{ComplexityResult, TaskComplexityEstimator};
use serde::{Deserialize, Serialize};

// ─── Orchestration mode ──────────────────────────────────────────────────────

/// How the system should orchestrate the task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrchestrationMode {
    /// Single LLM call — fastest, best for simple tasks with good skill/docs.
    SingleCall,
    /// Multi-stage pipeline — plan → execute → validate.
    MultiStage,
}

impl std::fmt::Display for OrchestrationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SingleCall => write!(f, "single-call"),
            Self::MultiStage => write!(f, "multi-stage"),
        }
    }
}

impl OrchestrationMode {
    /// Convert to the corresponding `WorkflowMode` used by the LLM pipeline.
    pub fn to_workflow_mode(self) -> crate::llm_workflow::WorkflowMode {
        match self {
            Self::SingleCall => crate::llm_workflow::WorkflowMode::Fast,
            Self::MultiStage => crate::llm_workflow::WorkflowMode::Quality,
        }
    }
}

// ─── Supervisor Agent ────────────────────────────────────────────────────────

/// Decision output from the supervisor.
#[derive(Debug, Clone)]
pub struct SupervisorDecision {
    /// Selected orchestration mode.
    pub mode: OrchestrationMode,
    /// Task complexity analysis.
    #[allow(dead_code)]
    pub complexity: ComplexityResult,
    /// Enrichment hints from knowledge layer (best practices, tool info).
    pub enrichment_hints: Vec<String>,
    /// Domain category inferred from the tool/task.
    pub domain: Option<String>,
    /// Reasons for the decision.
    pub reasons: Vec<String>,
}

/// The Supervisor Agent — central decision maker.
pub struct SupervisorAgent {
    complexity_estimator: TaskComplexityEstimator,
    knowledge_base: ToolKnowledgeBase,
    best_practices: BestPracticesDb,
}

impl Default for SupervisorAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl SupervisorAgent {
    pub fn new() -> Self {
        Self {
            complexity_estimator: TaskComplexityEstimator::new(),
            knowledge_base: ToolKnowledgeBase::new(),
            best_practices: BestPracticesDb::new(),
        }
    }

    /// Analyze the task and decide on orchestration strategy.
    pub fn decide(
        &self,
        tool: &str,
        task: &str,
        has_skill: bool,
        doc_quality: f32,
        force_mode: Option<OrchestrationMode>,
    ) -> SupervisorDecision {
        // Honour explicit override.
        if let Some(forced) = force_mode {
            return SupervisorDecision {
                mode: forced,
                complexity: ComplexityResult::default(),
                enrichment_hints: self.gather_hints(tool),
                domain: self.infer_domain(tool),
                reasons: vec![format!("mode forced to {forced}")],
            };
        }

        // Step 1: Estimate complexity.
        let complexity = self
            .complexity_estimator
            .estimate(task, tool, has_skill, doc_quality);

        // Step 2: Decide mode based on complexity + available context.
        let mut reasons = Vec::new();
        let mode = if complexity.score.is_complex() {
            reasons.push("task complexity is high".to_string());
            if !has_skill {
                reasons.push("no skill available — multi-stage helps".to_string());
            }
            OrchestrationMode::MultiStage
        } else {
            reasons.push("task complexity is low".to_string());
            if has_skill {
                reasons.push("skill available — single call sufficient".to_string());
            }
            OrchestrationMode::SingleCall
        };

        // Step 3: Gather knowledge enrichment hints.
        let enrichment_hints = self.gather_hints(tool);
        let domain = self.infer_domain(tool);

        SupervisorDecision {
            mode,
            complexity,
            enrichment_hints,
            domain,
            reasons,
        }
    }

    /// Gather enrichment hints from the knowledge layer.
    fn gather_hints(&self, tool: &str) -> Vec<String> {
        let mut hints = Vec::new();

        // Tool-specific best practices.
        let practices = self.best_practices.for_tool(tool);
        for p in practices.iter().take(3) {
            hints.push(format!("{}: {}", p.title, p.recommendation));
        }

        // Related tools for context.
        let related = self.knowledge_base.related_tools(tool, 3);
        if !related.is_empty() {
            let names: Vec<&str> = related.iter().map(|t| t.name.as_str()).collect();
            hints.push(format!("Related tools: {}", names.join(", ")));
        }

        hints
    }

    /// Infer the bioinformatics domain from the tool name.
    fn infer_domain(&self, tool: &str) -> Option<String> {
        self.knowledge_base
            .lookup(tool)
            .map(|entry| entry.category.clone())
    }

    /// Access the knowledge base for external queries.
    #[allow(dead_code)]
    pub fn knowledge_base(&self) -> &ToolKnowledgeBase {
        &self.knowledge_base
    }

    /// Access the best practices DB.
    #[allow(dead_code)]
    pub fn best_practices(&self) -> &BestPracticesDb {
        &self.best_practices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_task_single_call() {
        let supervisor = SupervisorAgent::new();
        let decision = supervisor.decide("samtools", "sort input.bam", true, 0.8, None);
        assert_eq!(decision.mode, OrchestrationMode::SingleCall);
        assert!(!decision.reasons.is_empty());
    }

    #[test]
    fn test_complex_task_multi_stage() {
        let supervisor = SupervisorAgent::new();
        let decision = supervisor.decide(
            "unknown_tool",
            "build a complex pipeline with multiple parallel steps for variant calling optimization",
            false,
            0.2,
            None,
        );
        assert_eq!(decision.mode, OrchestrationMode::MultiStage);
    }

    #[test]
    fn test_forced_mode() {
        let supervisor = SupervisorAgent::new();
        let decision = supervisor.decide(
            "samtools",
            "sort input.bam",
            true,
            0.9,
            Some(OrchestrationMode::MultiStage),
        );
        assert_eq!(decision.mode, OrchestrationMode::MultiStage);
        assert!(decision.reasons.iter().any(|r| r.contains("forced")));
    }

    #[test]
    fn test_enrichment_hints_for_known_tool() {
        let supervisor = SupervisorAgent::new();
        let decision = supervisor.decide("samtools", "sort input.bam", true, 0.8, None);
        assert!(
            !decision.enrichment_hints.is_empty(),
            "Known tool should have enrichment hints"
        );
    }

    #[test]
    fn test_domain_inference() {
        let supervisor = SupervisorAgent::new();
        let decision = supervisor.decide("gatk4", "call variants", true, 0.8, None);
        assert_eq!(decision.domain, Some("variant-calling".to_string()));
    }

    #[test]
    fn test_unknown_tool_domain() {
        let supervisor = SupervisorAgent::new();
        let decision = supervisor.decide("my_custom_tool", "do stuff", false, 0.5, None);
        assert_eq!(decision.domain, None);
    }
}
