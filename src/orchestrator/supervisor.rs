#![allow(dead_code)]
//! Supervisor Agent — routes tasks and selects orchestration strategy (HDA).
//!
//! Uses confidence-driven decision making instead of task_complexity.

use crate::confidence::{ConfidenceLevel, ConfidenceResult, estimate_confidence};
use crate::knowledge::best_practices::BestPracticesDb;
use crate::knowledge::tool_knowledge::ToolKnowledgeBase;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrchestrationMode {
    SingleCall,
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
    pub fn to_workflow_mode(self) -> crate::workflow_graph::WorkflowMode {
        match self {
            Self::SingleCall => crate::workflow_graph::WorkflowMode::Fast,
            Self::MultiStage => crate::workflow_graph::WorkflowMode::Quality,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SupervisorDecision {
    pub mode: OrchestrationMode,
    pub confidence: ConfidenceResult,
    pub enrichment_hints: Vec<String>,
    pub domain: Option<String>,
    pub reasons: Vec<String>,
}

pub struct SupervisorAgent {
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
            knowledge_base: ToolKnowledgeBase::new(),
            best_practices: BestPracticesDb::new(),
        }
    }

    pub fn decide(
        &self,
        tool: &str,
        task: &str,
        has_skill: bool,
        doc_quality: f32,
        force_mode: Option<OrchestrationMode>,
    ) -> SupervisorDecision {
        if let Some(forced) = force_mode {
            return SupervisorDecision {
                mode: forced,
                confidence: estimate_confidence(0, doc_quality, has_skill, 0, None, has_skill),
                enrichment_hints: self.gather_hints(tool),
                domain: self.infer_domain(tool),
                reasons: vec![format!("mode forced to {forced}")],
            };
        }

        let confidence = estimate_confidence(
            0,
            doc_quality,
            false,
            if task.len() > 20 { 2 } else { 0 },
            None,
            has_skill,
        );

        let mut reasons = Vec::new();
        let mode = if has_skill {
            reasons.push("skill available — single call sufficient".to_string());
            OrchestrationMode::SingleCall
        } else if confidence.level == ConfidenceLevel::Low {
            reasons.push("confidence is low — multi-stage helps".to_string());
            OrchestrationMode::MultiStage
        } else {
            reasons.push("confidence is sufficient — single call".to_string());
            OrchestrationMode::SingleCall
        };

        let enrichment_hints = self.gather_hints(tool);
        let domain = self.infer_domain(tool);

        SupervisorDecision {
            mode,
            confidence,
            enrichment_hints,
            domain,
            reasons,
        }
    }

    fn gather_hints(&self, tool: &str) -> Vec<String> {
        let mut hints = Vec::new();
        let practices = self.best_practices.for_tool(tool);
        for p in practices.iter().take(3) {
            hints.push(format!("{}: {}", p.title, p.recommendation));
        }
        let related = self.knowledge_base.related_tools(tool, 3);
        if !related.is_empty() {
            let names: Vec<&str> = related.iter().map(|t| t.name.as_str()).collect();
            hints.push(format!("Related tools: {}", names.join(", ")));
        }
        hints
    }

    fn infer_domain(&self, tool: &str) -> Option<String> {
        self.knowledge_base
            .lookup(tool)
            .map(|entry| entry.category.clone())
    }

    #[allow(dead_code)]
    pub fn knowledge_base(&self) -> &ToolKnowledgeBase {
        &self.knowledge_base
    }

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
    }

    #[test]
    fn test_low_confidence_multi_stage() {
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
    }

    #[test]
    fn test_skill_overrides_to_single_call() {
        let supervisor = SupervisorAgent::new();
        let decision = supervisor.decide(
            "samtools",
            "build a complex pipeline with multiple parallel steps",
            true,
            0.8,
            None,
        );
        assert_eq!(decision.mode, OrchestrationMode::SingleCall);
    }
}
