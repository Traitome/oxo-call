//! Executor Agent — generates and runs commands.
//!
//! The executor is responsible for the actual command generation step.
//! It enriches the LLM prompt with knowledge-layer hints (best practices,
//! tool info) before calling the LLM client.

use crate::error::Result;
use crate::knowledge::best_practices::BestPracticesDb;
use crate::task_normalizer::{NormalizedTask, TaskNormalizer};

/// Result from the executor agent's preparation step.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExecutorContext {
    /// Normalized task description.
    pub normalized_task: NormalizedTask,
    /// Best practice hints injected into the prompt.
    pub practice_hints: String,
    /// Whether the task was normalized (changed from original).
    pub was_normalized: bool,
}

/// The Executor Agent.
pub struct ExecutorAgent {
    normalizer: TaskNormalizer,
    best_practices: BestPracticesDb,
}

impl Default for ExecutorAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutorAgent {
    pub fn new() -> Self {
        Self {
            normalizer: TaskNormalizer::new(),
            best_practices: BestPracticesDb::new(),
        }
    }

    /// Prepare execution context: normalize the task and gather enrichment.
    pub async fn prepare(&self, tool: &str, task: &str) -> Result<ExecutorContext> {
        // Step 1: Normalize the task.
        let normalized = self.normalizer.normalize(task, tool).await.map_err(|e| {
            crate::error::OxoError::LlmError(format!("task normalization failed: {e}"))
        })?;
        let was_normalized = normalized.description != task;

        // Step 2: Gather best practice hints.
        let practice_hints = self.best_practices.to_prompt_hint(tool);

        Ok(ExecutorContext {
            normalized_task: normalized,
            practice_hints,
            was_normalized,
        })
    }

    /// Build an enriched task string for the LLM prompt.
    ///
    /// Combines the normalized task with best practice hints and intent info.
    pub fn enrich_task(&self, ctx: &ExecutorContext) -> String {
        let mut parts = vec![ctx.normalized_task.description.clone()];

        // Add intent context.
        let intent = &ctx.normalized_task.intent;
        parts.push(format!("[Intent: {intent}]"));

        // Add extracted parameters.
        if !ctx.normalized_task.parameters.is_empty() {
            let params: Vec<String> = ctx
                .normalized_task
                .parameters
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            parts.push(format!("[Params: {}]", params.join(", ")));
        }

        // Add constraints.
        if !ctx.normalized_task.constraints.is_empty() {
            parts.push(format!(
                "[Constraints: {}]",
                ctx.normalized_task.constraints.join(", ")
            ));
        }

        // Add best practices (truncated).
        if !ctx.practice_hints.is_empty() {
            parts.push(ctx.practice_hints.clone());
        }

        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prepare_simple_task() {
        let executor = ExecutorAgent::new();
        let ctx = executor
            .prepare("samtools", "sort input.bam by coordinate")
            .await
            .unwrap();
        assert!(!ctx.normalized_task.description.is_empty());
    }

    #[tokio::test]
    async fn test_enrich_task_adds_intent() {
        let executor = ExecutorAgent::new();
        let ctx = executor
            .prepare("samtools", "sort input.bam by coordinate")
            .await
            .unwrap();
        let enriched = executor.enrich_task(&ctx);
        assert!(enriched.contains("[Intent:"));
    }

    #[tokio::test]
    async fn test_enrich_task_adds_practices() {
        let executor = ExecutorAgent::new();
        let ctx = executor
            .prepare("samtools", "sort input.bam by coordinate")
            .await
            .unwrap();
        let enriched = executor.enrich_task(&ctx);
        assert!(
            enriched.contains("[Best Practices]"),
            "Known tool should have best practice hints"
        );
    }

    #[tokio::test]
    async fn test_prepare_with_threads() {
        let executor = ExecutorAgent::new();
        let ctx = executor
            .prepare("bwa", "align reads.fq to ref.fa with 8 threads")
            .await
            .unwrap();
        assert_eq!(
            ctx.normalized_task.parameters.get("threads"),
            Some(&"8".to_string())
        );
    }
}
