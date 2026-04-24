//! Planner Agent — decomposes complex tasks into executable steps.
//!
//! When the supervisor selects multi-stage mode, the planner analyzes the task
//! and produces a structured execution plan.  For single-call mode, it
//! produces a trivial one-step plan.

use serde::{Deserialize, Serialize};

/// A single step in an execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    /// Step number (1-based).
    pub step: usize,
    /// Tool to use for this step.
    pub tool: String,
    /// What this step should accomplish.
    pub description: String,
    /// Dependencies: step numbers that must complete first.
    pub depends_on: Vec<usize>,
    /// Whether this step requires validation.
    pub needs_validation: bool,
}

/// A complete task execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    /// Original task.
    pub original_task: String,
    /// Ordered steps.
    pub steps: Vec<PlanStep>,
    /// Overall strategy description.
    pub strategy: String,
    /// Estimated complexity (steps × validation needs).
    pub estimated_llm_calls: usize,
}

impl TaskPlan {
    /// Create a trivial single-step plan (for SingleCall mode).
    pub fn single_step(tool: &str, task: &str) -> Self {
        Self {
            original_task: task.to_string(),
            steps: vec![PlanStep {
                step: 1,
                tool: tool.to_string(),
                description: task.to_string(),
                depends_on: vec![],
                needs_validation: false,
            }],
            strategy: "Direct single-call execution".to_string(),
            estimated_llm_calls: 1,
        }
    }

    /// Whether this is a multi-step plan.
    pub fn is_multi_step(&self) -> bool {
        self.steps.len() > 1
    }
}

/// The Planner Agent — task decomposition.
pub struct PlannerAgent;

impl Default for PlannerAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl PlannerAgent {
    pub fn new() -> Self {
        Self
    }

    /// Plan the execution of a task.
    ///
    /// For simple tasks, returns a single-step plan.
    /// For complex tasks (multi-tool pipelines), decomposes into steps.
    pub fn plan(&self, tool: &str, task: &str) -> TaskPlan {
        // Detect multi-step patterns without lowercase allocation.
        let is_pipeline = self.detect_pipeline_ci(task);

        if is_pipeline {
            self.plan_pipeline(tool, task)
        } else {
            TaskPlan::single_step(tool, task)
        }
    }

    /// Detect whether the task describes a multi-step pipeline (case-insensitive).
    /// Uses byte-level matching for ASCII indicators, exact match for Chinese.
    fn detect_pipeline_ci(&self, task: &str) -> bool {
        // ASCII pipeline indicators - check case-insensitively
        let ascii_indicators = [
            "then",
            "after that",
            "followed by",
            "pipeline",
            "workflow",
            "step 1",
            "step 2",
            "first",
            "second",
            "finally",
        ];

        // Case-insensitive check for ASCII indicators
        for ind in &ascii_indicators {
            if task.len() >= ind.len()
                && task.as_bytes().windows(ind.len()).any(|window| {
                    window
                        .iter()
                        .zip(ind.as_bytes())
                        .all(|(h, n)| h.eq_ignore_ascii_case(n))
                })
            {
                return true;
            }
        }

        // Chinese pipeline indicators - exact match (no case variation)
        let chinese_indicators = ["然后", "接着", "之后", "流程", "管道"];
        chinese_indicators.iter().any(|ind| task.contains(ind))
            || task.matches("&&").count() > 0
            || task.matches(';').count() > 1
    }

    /// Decompose a pipeline task into ordered steps.
    /// Uses single-pass split without creating intermediate Vecs per delimiter.
    fn plan_pipeline(&self, tool: &str, task: &str) -> TaskPlan {
        // Single-pass split: find earliest delimiter and split once
        // Pre-allocate result Vec with estimated capacity
        let mut parts: Vec<&str> = Vec::with_capacity(8);
        let mut remainder = task;

        // Process all delimiters in single pass
        loop {
            // Find earliest delimiter match
            let mut earliest_pos: Option<(usize, usize)> = None; // (pos, delim_len)

            for delim in [
                " then ",
                " after that ",
                " followed by ",
                ", then ",
                " 然后 ",
                " 接着 ",
                " 之后 ",
                "&&",
            ] {
                if let Some(pos) = remainder.find(delim) {
                    let candidate = (pos, delim.len());
                    earliest_pos = Some(
                        earliest_pos
                            .map_or(candidate, |e| if candidate.0 < e.0 { candidate } else { e }),
                    );
                }
            }

            match earliest_pos {
                Some((pos, delim_len)) => {
                    let segment = remainder[..pos].trim();
                    if !segment.is_empty() {
                        parts.push(segment);
                    }
                    remainder = &remainder[pos + delim_len..];
                }
                None => {
                    // No more delimiters, add remaining segment
                    let segment = remainder.trim();
                    if !segment.is_empty() {
                        parts.push(segment);
                    }
                    break;
                }
            }
        }

        if parts.len() <= 1 {
            return TaskPlan::single_step(tool, task);
        }

        let steps: Vec<PlanStep> = parts
            .iter()
            .enumerate()
            .map(|(i, desc)| {
                let step_num = i + 1;
                PlanStep {
                    step: step_num,
                    tool: tool.to_string(),
                    description: (*desc).to_string(),
                    depends_on: if i > 0 { vec![step_num - 1] } else { vec![] },
                    needs_validation: i == parts.len() - 1, // validate last step
                }
            })
            .collect();

        let estimated_calls = steps.len() + steps.iter().filter(|s| s.needs_validation).count();

        TaskPlan {
            original_task: task.to_string(),
            steps,
            strategy: "Sequential pipeline execution".to_string(),
            estimated_llm_calls: estimated_calls,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_task_single_step() {
        let planner = PlannerAgent::new();
        let plan = planner.plan("samtools", "sort input.bam by coordinate");
        assert_eq!(plan.steps.len(), 1);
        assert!(!plan.is_multi_step());
        assert_eq!(plan.estimated_llm_calls, 1);
    }

    #[test]
    fn test_pipeline_detection() {
        let planner = PlannerAgent::new();

        let plan = planner.plan(
            "samtools",
            "sort input.bam by coordinate then index the sorted file",
        );
        assert!(plan.is_multi_step(), "Should detect 'then' as pipeline");
        assert_eq!(plan.steps.len(), 2);
    }

    #[test]
    fn test_pipeline_dependencies() {
        let planner = PlannerAgent::new();
        let plan = planner.plan("samtools", "sort input.bam then index then flagstat");
        assert_eq!(plan.steps.len(), 3);
        assert!(plan.steps[0].depends_on.is_empty());
        assert_eq!(plan.steps[1].depends_on, vec![1]);
        assert_eq!(plan.steps[2].depends_on, vec![2]);
    }

    #[test]
    fn test_pipeline_with_ampersand() {
        let planner = PlannerAgent::new();
        let plan = planner.plan(
            "samtools",
            "sort input.bam && index sorted.bam && flagstat sorted.bam",
        );
        assert!(plan.is_multi_step());
    }

    #[test]
    fn test_chinese_pipeline() {
        let planner = PlannerAgent::new();
        let plan = planner.plan("samtools", "排序 input.bam 然后 建立索引");
        assert!(plan.is_multi_step(), "Should detect Chinese pipeline");
    }

    #[test]
    fn test_single_step_plan_fields() {
        let plan = TaskPlan::single_step("bwa", "align reads to reference");
        assert_eq!(plan.steps[0].tool, "bwa");
        assert_eq!(plan.strategy, "Direct single-call execution");
    }
}
