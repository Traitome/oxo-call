//! Context presets for one command-generation invocation.

use crate::task_complexity::GenerationMode;
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
    /// Select the generation strategy appropriate for this context preset.
    pub const fn default_generation_mode(self) -> GenerationMode {
        match self {
            Self::Bare | Self::Prompt | Self::Skill => GenerationMode::Fast,
            Self::Doc | Self::Full => GenerationMode::Quality,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_scenarios_select_expected_generation_modes() {
        assert_eq!(
            ContextScenario::Bare.default_generation_mode(),
            GenerationMode::Fast
        );
        assert_eq!(
            ContextScenario::Doc.default_generation_mode(),
            GenerationMode::Quality
        );
    }
}
