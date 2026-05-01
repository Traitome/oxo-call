//! RAG Enhancer Module
//!
//! Builds enhanced prompts using retrieved context from vector store.
//! Integrates similar examples, documentation, and best practices into prompts.

#![allow(dead_code)]

use crate::rag::retriever::{ExampleContext, RetrievalContext};

/// Enhanced prompt with RAG context
#[derive(Debug, Clone)]
pub struct RagEnhancedPrompt {
    /// Base task description
    pub task: String,
    /// Tool name
    pub tool: String,
    /// The complete enhanced prompt text
    pub prompt_text: String,
    /// Retrieved examples count
    pub examples_count: usize,
    /// Retrieved doc sections count
    pub doc_sections_count: usize,
}

/// RAG Enhancer for building context-aware prompts
#[derive(Debug, Clone)]
pub struct RagEnhancer {
    /// Max examples to include
    max_examples: usize,
    /// Max doc section length
    max_doc_length: usize,
    /// Include best practices
    include_best_practices: bool,
    /// Include pitfalls
    include_pitfalls: bool,
}

impl Default for RagEnhancer {
    fn default() -> Self {
        Self {
            max_examples: 3,
            max_doc_length: 500,
            include_best_practices: true,
            include_pitfalls: true,
        }
    }
}

impl RagEnhancer {
    /// Create new enhancer
    pub fn new() -> Self {
        Self::default()
    }

    /// Set max examples
    pub fn with_max_examples(mut self, max: usize) -> Self {
        self.max_examples = max;
        self
    }

    /// Build enhanced prompt
    pub fn enhance(&self, tool: &str, task: &str, context: &RetrievalContext) -> RagEnhancedPrompt {
        let mut sections = Vec::new();

        // 1. System instruction
        sections.push(self.build_system_instruction(tool, context));

        // 2. Similar examples (RAG core)
        if !context.similar_examples.is_empty() {
            sections.push(self.build_examples_section(&context.similar_examples));
        }

        // 3. Subcommand context
        if let Some(ref sc) = context.selected_subcommand {
            sections.push(self.build_subcommand_section(sc));
        }

        // 4. Documentation sections
        if !context.doc_sections.is_empty() {
            sections.push(self.build_doc_section(&context.doc_sections));
        }

        // 5. Best practices
        if self.include_best_practices && !context.best_practices.is_empty() {
            sections.push(self.build_best_practices_section(&context.best_practices));
        }

        // 6. Pitfalls to avoid
        if self.include_pitfalls && !context.pitfalls.is_empty() {
            sections.push(self.build_pitfalls_section(&context.pitfalls));
        }

        // 7. User task
        sections.push(self.build_task_section(task));

        // 8. Output format instruction
        sections.push(self.build_output_instruction());

        let prompt_text = sections.join("\n\n");

        RagEnhancedPrompt {
            task: task.to_string(),
            tool: tool.to_string(),
            prompt_text,
            examples_count: context.similar_examples.len(),
            doc_sections_count: context.doc_sections.len(),
        }
    }

    /// Build system instruction
    fn build_system_instruction(&self, tool: &str, context: &RetrievalContext) -> String {
        let mut instruction = format!(
            "You are a CLI command generator for {}. \\\n\
            Generate precise commands based on the task description and retrieved examples.",
            tool
        );

        if let Some(ref sc) = context.selected_subcommand {
            instruction.push_str(&format!(
                "\\\nThis task likely requires the '{}' subcommand.",
                sc.name
            ));
        }

        instruction
    }

    /// Build examples section
    fn build_examples_section(&self, examples: &[ExampleContext]) -> String {
        let mut section = "SIMILAR EXAMPLES (follow these patterns):\\n".to_string();

        for (i, ex) in examples.iter().take(self.max_examples).enumerate() {
            section.push_str(&format!("\\n{}. Task: {}\\n", i + 1, ex.task));
            section.push_str(&format!("   Command: {} {}\\n", ex.tool, ex.command));

            if let Some(ref explanation) = ex.explanation {
                section.push_str(&format!("   Note: {}\\n", explanation));
            }

            if ex.similarity > 0.8 {
                section.push_str("   [Highly relevant]\\n");
            }
        }

        section
    }

    /// Build subcommand section
    fn build_subcommand_section(&self, sc: &crate::rag::retriever::SubcommandContext) -> String {
        let mut section = format!("SUBCOMMAND: {}\\n", sc.name);
        section.push_str(&format!("Description: {}\\n", sc.description));

        if !sc.usage.is_empty() {
            section.push_str(&format!("Usage: {}\\n", sc.usage));
        }

        if !sc.relevant_flags.is_empty() {
            section.push_str("Common flags:\\n");
            for flag in &sc.relevant_flags {
                section.push_str(&format!("  - {}\\n", flag));
            }
        }

        section
    }

    /// Build documentation section
    fn build_doc_section(&self, sections: &[crate::rag::retriever::DocSection]) -> String {
        let mut section = "RELEVANT DOCUMENTATION:\\n".to_string();

        for doc in sections {
            section.push_str(&format!("\\n### {}\\n", doc.title));

            // Truncate if too long
            let content = if doc.content.len() > self.max_doc_length {
                format!("{}...", &doc.content[..self.max_doc_length])
            } else {
                doc.content.clone()
            };

            section.push_str(&content);
            section.push('\n');
        }

        section
    }

    /// Build best practices section
    fn build_best_practices_section(&self, practices: &[String]) -> String {
        let mut section = "BEST PRACTICES:\\n".to_string();

        for (i, practice) in practices.iter().enumerate() {
            section.push_str(&format!("{}. {}\\n", i + 1, practice));
        }

        section
    }

    /// Build pitfalls section
    fn build_pitfalls_section(&self, pitfalls: &[String]) -> String {
        let mut section = "COMMON MISTAKES TO AVOID:\\n".to_string();

        for pitfall in pitfalls {
            section.push_str(&format!("- {}\\n", pitfall));
        }

        section
    }

    /// Build task section
    fn build_task_section(&self, task: &str) -> String {
        format!("YOUR TASK:\\n{}\\n", task)
    }

    /// Build output instruction
    fn build_output_instruction(&self) -> String {
        "GENERATE COMMAND:\\n".to_string()
            + "- Output ONLY the command arguments (without the tool name)\\n"
            + "- Use the patterns from similar examples above\\n"
            + "- Ensure all flags are valid for this tool\\n"
            + "- If a subcommand is needed, include it first\\n"
    }

    /// Build minimal prompt (without full RAG)
    pub fn build_minimal_prompt(&self, tool: &str, task: &str) -> String {
        format!(
            "Generate a {} command for: {}\\n\\n\
            Output only the arguments (not the tool name):",
            tool, task
        )
    }

    /// Calculate token estimate (rough)
    pub fn estimate_tokens(&self, prompt: &RagEnhancedPrompt) -> usize {
        // Rough estimate: 1 token ≈ 4 characters for English
        prompt.prompt_text.len() / 4
    }
}

/// RAG Enhancer with validation integration
pub struct ValidatedRagEnhancer {
    /// Base enhancer
    enhancer: RagEnhancer,
    /// Include validation hints
    include_validation_hints: bool,
}

impl ValidatedRagEnhancer {
    /// Create new validated enhancer
    pub fn new() -> Self {
        Self {
            enhancer: RagEnhancer::new(),
            include_validation_hints: true,
        }
    }

    /// Enhance with validation constraints
    pub fn enhance_with_constraints(
        &self,
        tool: &str,
        task: &str,
        context: &RetrievalContext,
        valid_flags: &[String],
    ) -> RagEnhancedPrompt {
        let mut prompt = self.enhancer.enhance(tool, task, context);

        if self.include_validation_hints && !valid_flags.is_empty() {
            let constraints = format!(
                "\\n\\nVALID FLAGS (use ONLY these):\\n{}\\n",
                valid_flags
                    .iter()
                    .take(20)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            prompt.prompt_text.push_str(&constraints);
        }

        prompt
    }
}

impl Default for ValidatedRagEnhancer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::retriever::{DocSection, ExampleContext, RetrievalContext, SubcommandContext};

    fn create_test_context() -> RetrievalContext {
        RetrievalContext {
            tool: "samtools".to_string(),
            task: "sort bam".to_string(),
            selected_subcommand: Some(SubcommandContext {
                name: "sort".to_string(),
                description: "Sort alignment file".to_string(),
                usage: "sort [options] <in.bam>".to_string(),
                relevant_flags: vec!["-o".to_string(), "-@".to_string()],
            }),
            similar_examples: vec![ExampleContext {
                task: "sort BAM".to_string(),
                command: "sort -o out.bam in.bam".to_string(),
                similarity: 0.95,
                tool: "samtools".to_string(),
                subcommand: Some("sort".to_string()),
                explanation: Some("Sort by coordinate".to_string()),
            }],
            doc_sections: vec![DocSection {
                title: "sort".to_string(),
                content: "Sort alignments by leftmost coordinates".to_string(),
                relevance: 0.9,
            }],
            best_practices: vec!["Always use -o".to_string()],
            pitfalls: vec!["Don't forget output".to_string()],
        }
    }

    #[test]
    fn test_enhance_prompt() {
        let enhancer = RagEnhancer::new();
        let context = create_test_context();

        let prompt = enhancer.enhance("samtools", "sort my file", &context);

        assert!(prompt.prompt_text.contains("SIMILAR EXAMPLES"));
        assert!(prompt.prompt_text.contains("sort -o out.bam"));
        assert!(prompt.prompt_text.contains("SUBCOMMAND: sort"));
        assert_eq!(prompt.examples_count, 1);
    }

    #[test]
    fn test_token_estimate() {
        let enhancer = RagEnhancer::new();
        let context = create_test_context();

        let prompt = enhancer.enhance("samtools", "sort", &context);
        let tokens = enhancer.estimate_tokens(&prompt);

        assert!(tokens > 0);
        assert!(tokens < 1000); // Should be reasonable
    }

    #[test]
    fn test_minimal_prompt() {
        let enhancer = RagEnhancer::new();
        let prompt = enhancer.build_minimal_prompt("samtools", "sort");

        assert!(prompt.contains("samtools"));
        assert!(prompt.contains("sort"));
    }
}
