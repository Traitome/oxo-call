//! Context presets for one command-generation invocation.

use clap::ValueEnum;

/// Sources of context to prioritize for a command-generation request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ContextScenario {
    /// Tool and task only.
    Bare,
    /// Tool, task, and the system prompt.
    Prompt,
    /// Tool documentation.
    Doc,
    /// A curated skill.
    Skill,
    /// Documentation and a curated skill.
    Full,
}

impl ContextScenario {
    /// All scenarios use single-call evidence-graded mode in 0.20.0.
    pub fn describe(&self) -> &'static str {
        match self {
            Self::Bare => "tool + task only",
            Self::Prompt => "tool + task + system prompt",
            Self::Skill => "tool + task + skill",
            Self::Doc => "tool + task + docs",
            Self::Full => "tool + task + docs + skill",
        }
    }
}
