//! AI Orchestration Layer (LangGraph-Inspired).
//!
//! Implements a multi-agent coordination system with four core agents:
//! - **Supervisor**: Routes tasks and decides orchestration strategy
//! - **Planner**: Decomposes complex tasks into steps
//! - **Executor**: Generates and runs commands
//! - **Validator**: Verifies results and provides feedback
//!
//! Supports adaptive single-call (Fast) and multi-agent (Quality) modes.

pub mod executor;
pub mod planner;
pub mod supervisor;
pub mod validator;

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::orchestrator::executor::ExecutorAgent;
    use crate::orchestrator::planner::PlannerAgent;
    use crate::orchestrator::supervisor::{OrchestrationMode, SupervisorAgent};
    use crate::orchestrator::validator::ValidatorAgent;

    #[test]
    fn test_orchestration_mode_variants() {
        let modes = vec![OrchestrationMode::SingleCall, OrchestrationMode::MultiStage];
        // Verify we can create all variants without panic.
        assert_eq!(modes.len(), 2);
    }

    #[test]
    fn test_supervisor_agent_new() {
        let _agent = SupervisorAgent::new();
    }

    #[test]
    fn test_planner_agent_new() {
        let _planner = PlannerAgent::new();
    }

    #[test]
    fn test_validator_agent_new() {
        let _validator = ValidatorAgent::new();
    }

    #[test]
    fn test_executor_agent_new_with_config() {
        let config = Config::default();
        let _executor = ExecutorAgent::new_with_config(config);
    }

    #[test]
    fn test_task_plan_single_step() {
        let plan = PlannerAgent::new().plan("samtools", "sort bam file");
        assert!(!plan.steps.is_empty());
    }

    #[test]
    fn test_supervisor_decide_simple_task() {
        let agent = SupervisorAgent::new();
        let decision = agent.decide("samtools", "sort bam file", false, 0.8, None);
        // Should return some mode
        let _ = decision.mode;
    }
}
