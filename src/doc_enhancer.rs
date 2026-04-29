//! Document Enhancement Module
//!
//! Provides enhanced document processing with CLI pattern analysis,
//! subcommand detection, constraint validation, and RAG (Retrieval-Augmented Generation).

#![allow(dead_code)]

use crate::cli_pattern::{CliPattern, CliPatternClassifier};
use crate::constraint_graph::{ConstraintGraph, extract_from_docs};
use crate::doc_processor::StructuredDoc;
use crate::rag::vector_store::VectorStore;
use crate::rag::{
    EmbeddingConfig, ExampleStoreBuilder, LocalEmbeddingModel, RagEnhancer, RagRetriever,
    RetrievalContext,
};
use crate::subcommand_detector::{SubcommandDef, SubcommandDetectorV2};
use std::sync::Arc;

/// Enhanced document analysis result
#[derive(Debug, Clone)]
pub struct EnhancedDocAnalysis {
    /// Detected CLI pattern
    pub cli_pattern: CliPattern,
    /// All detected subcommands
    pub subcommands: Vec<SubcommandDef>,
    /// Selected subcommand for the task (if any)
    pub selected_subcommand: Option<SubcommandDef>,
    /// Constraint graph for validation
    pub constraint_graph: ConstraintGraph,
    /// Valid flags whitelist
    pub valid_flags: Vec<String>,
    /// RAG retrieval context (if enabled)
    pub rag_context: Option<RetrievalContext>,
}

/// Document Enhancer with optional RAG support
#[derive(Debug, Clone)]
pub struct DocEnhancer {
    /// RAG retriever (if enabled)
    rag_retriever: Option<RagRetriever>,
    /// RAG enhancer
    rag_enhancer: RagEnhancer,
    /// Whether RAG is enabled
    rag_enabled: bool,
}

impl Default for DocEnhancer {
    fn default() -> Self {
        Self::new()
    }
}

impl DocEnhancer {
    /// Create a new DocEnhancer (RAG disabled by default)
    pub fn new() -> Self {
        Self {
            rag_retriever: None,
            rag_enhancer: RagEnhancer::new(),
            rag_enabled: false,
        }
    }

    /// Create a DocEnhancer with RAG enabled
    pub fn with_rag() -> Self {
        let config = EmbeddingConfig::default();
        let model = Arc::new(LocalEmbeddingModel::new(config.clone()));

        // Build example store with common bioinformatics examples
        let example_store = ExampleStoreBuilder::build_common_examples(model.clone());

        // Create empty doc store (will be populated as docs are processed)
        let doc_store = VectorStore::new(config.dimension);

        let retriever = RagRetriever::new(example_store, doc_store, model);

        Self {
            rag_retriever: Some(retriever),
            rag_enhancer: RagEnhancer::new(),
            rag_enabled: true,
        }
    }

    /// Enable RAG with custom retriever
    pub fn with_custom_rag(mut self, retriever: RagRetriever) -> Self {
        self.rag_retriever = Some(retriever);
        self.rag_enabled = true;
        self
    }

    /// Check if RAG is enabled
    pub fn is_rag_enabled(&self) -> bool {
        self.rag_enabled
    }

    /// Toggle RAG
    pub fn set_rag_enabled(&mut self, enabled: bool) {
        self.rag_enabled = enabled;
    }

    /// Analyze CLI pattern and detect subcommands for enhanced processing
    ///
    /// This method integrates the new CLI Pattern Classifier and Subcommand
    /// Detector v2 to provide richer context for command generation.
    /// If RAG is enabled, also retrieves similar examples.
    pub fn analyze(&self, raw_doc: &str, tool_name: &str, task: &str) -> EnhancedDocAnalysis {
        // Step 1: Classify CLI pattern
        let pattern_classifier = CliPatternClassifier::new();
        let cli_pattern = pattern_classifier.classify(raw_doc, tool_name);

        // Step 2: Detect subcommands if applicable
        let subcommand_detector = SubcommandDetectorV2::new();
        let subcommands = if cli_pattern.requires_subcommand() {
            subcommand_detector.detect(raw_doc, tool_name)
        } else {
            Vec::new()
        };

        // Step 3: Select best subcommand for the task
        let selected_subcommand = if !subcommands.is_empty() {
            subcommand_detector
                .select_for_task(task, &subcommands)
                .cloned()
        } else {
            None
        };

        // Step 4: Build constraint graph
        let constraint_graph = extract_from_docs(raw_doc, tool_name);

        // Step 5: Get valid flags whitelist
        let valid_flags: Vec<String> = constraint_graph.valid_flags.iter().cloned().collect();

        // Step 6: RAG retrieval (if enabled)
        let rag_context = if self.rag_enabled {
            self.rag_retriever
                .as_ref()
                .map(|r| r.retrieve(tool_name, task))
        } else {
            None
        };

        EnhancedDocAnalysis {
            cli_pattern,
            subcommands,
            selected_subcommand,
            constraint_graph,
            valid_flags,
            rag_context,
        }
    }

    /// Build enhanced prompt with constraint awareness
    ///
    /// This creates a prompt that explicitly includes:
    /// - CLI type instructions
    /// - Selected subcommand (if any)
    /// - Valid flags whitelist
    /// - Constraint rules
    pub fn build_enhanced_prompt(
        &self,
        tool: &str,
        task: &str,
        structured: &StructuredDoc,
        analysis: &EnhancedDocAnalysis,
    ) -> String {
        let mut prompt = String::new();

        // 1. System instruction based on CLI pattern
        prompt.push_str(&self.pattern_aware_instruction(&analysis.cli_pattern));
        prompt.push('\n');

        // 2. Tool type context
        prompt.push_str(&format!(
            "\nTool: {} ({})",
            tool,
            analysis.cli_pattern.description()
        ));

        // 3. Subcommand section (critical for subcommand-based tools)
        if let Some(ref subcmd) = analysis.selected_subcommand {
            prompt.push_str(&format!("\n\nSELECTED SUBCOMMAND: {}", subcmd.name));
            prompt.push_str(&format!("\nDescription: {}", subcmd.description));
            if !subcmd.usage_pattern.is_empty() {
                prompt.push_str(&format!(
                    "\nUsage: {} {} {}",
                    tool, subcmd.name, subcmd.usage_pattern
                ));
            }
            prompt.push_str("\n\nIMPORTANT: Include the subcommand name FIRST in your output.");
        } else if analysis.cli_pattern.requires_subcommand() {
            prompt.push_str(
                "\n\nWARNING: This tool requires a subcommand, but none could be detected.",
            );
            if !analysis.subcommands.is_empty() {
                prompt.push_str("\nAvailable subcommands:");
                for sc in &analysis.subcommands {
                    prompt.push_str(&format!("\n  - {}: {}", sc.name, sc.description));
                }
            }
        }

        // 4. Valid flags whitelist (anti-hallucination measure)
        if !analysis.valid_flags.is_empty() {
            prompt.push_str("\n\nVALID FLAGS (use ONLY these - NO hallucination):");
            let flags_to_show: Vec<&String> = if let Some(ref sc) = analysis.selected_subcommand {
                // Show subcommand-specific flags first
                analysis
                    .constraint_graph
                    .subcommand_flags
                    .get(&sc.name)
                    .map(|flags| flags.iter().take(20).collect())
                    .unwrap_or_else(|| analysis.valid_flags.iter().take(20).collect())
            } else {
                analysis.valid_flags.iter().take(30).collect()
            };

            for flag in &flags_to_show {
                // Try to find description from catalog
                let desc = structured
                    .flag_catalog
                    .iter()
                    .find(|e| &e.flag == *flag)
                    .map(|e| e.description.as_str())
                    .unwrap_or("");
                prompt.push_str(&format!("\n  {} {}", flag, desc));
            }
        }

        // 5. Constraint rules
        if !analysis.constraint_graph.required.is_empty() {
            prompt.push_str("\n\nREQUIRED PARAMETERS:");
            for constraint in &analysis.constraint_graph.required {
                prompt.push_str(&format!("\n  - {:?}", constraint));
            }
        }

        if !analysis.constraint_graph.mutually_exclusive.is_empty() {
            prompt.push_str("\n\nMUTUALLY EXCLUSIVE (don't use together):");
            for group in &analysis.constraint_graph.mutually_exclusive {
                let joined = group.join(" vs ");
                prompt.push_str(&format!("\n  - {}", joined));
            }
        }

        // 6. Examples from documentation
        if !structured.extracted_examples.is_empty() {
            prompt.push_str("\n\nUSAGE EXAMPLES:");
            for ex in structured.extracted_examples.iter().take(3) {
                prompt.push_str(&format!("\n  {}", ex));
            }
        }

        // 7. User task
        prompt.push_str(&format!("\n\nTASK: {}", task));
        prompt.push_str(
            "\n\nOUTPUT: Generate ONLY the arguments (including subcommand if needed):\n",
        );

        prompt
    }

    /// Build RAG-enhanced prompt with retrieval context
    ///
    /// This method combines the enhanced prompt with RAG-retrieved similar examples
    /// and best practices for improved command generation.
    pub fn build_rag_enhanced_prompt(
        &self,
        tool: &str,
        task: &str,
        structured: &StructuredDoc,
        analysis: &EnhancedDocAnalysis,
    ) -> String {
        // If RAG is enabled and we have context, use RAG enhancer
        if let Some(context) = analysis.rag_context.as_ref()
            && self.rag_enabled
        {
            let rag_prompt = self.rag_enhancer.enhance(tool, task, context);

            // Combine with constraint information from analysis
            let mut combined = rag_prompt.prompt_text;

            // Add valid flags whitelist if not already included
            if !analysis.valid_flags.is_empty() && !combined.contains("VALID FLAGS") {
                combined.push_str("\n\nVALID FLAGS (use ONLY these - NO hallucination):");
                let flags_to_show: Vec<&String> = analysis.valid_flags.iter().take(20).collect();
                for flag in flags_to_show {
                    let desc = structured
                        .flag_catalog
                        .iter()
                        .find(|e| e.flag == **flag)
                        .map(|e| e.description.as_str())
                        .unwrap_or("");
                    combined.push_str(&format!("\n  {} {}", flag, desc));
                }
            }

            // Add constraint rules
            if !analysis.constraint_graph.required.is_empty() {
                combined.push_str("\n\nREQUIRED PARAMETERS:");
                for constraint in &analysis.constraint_graph.required {
                    combined.push_str(&format!("\n  - {:?}", constraint));
                }
            }

            if !analysis.constraint_graph.mutually_exclusive.is_empty() {
                combined.push_str("\n\nMUTUALLY EXCLUSIVE (don't use together):");
                for group in &analysis.constraint_graph.mutually_exclusive {
                    let joined = group.join(" vs ");
                    combined.push_str(&format!("\n  - {}", joined));
                }
            }

            return combined;
        }
        // Fall back to standard enhanced prompt
        self.build_enhanced_prompt(tool, task, structured, analysis)
    }

    /// Get pattern-aware instruction
    fn pattern_aware_instruction(&self, pattern: &CliPattern) -> String {
        match pattern {
            CliPattern::Simple => {
                "You are a CLI command generator. Generate arguments for the task. \
                 Use ONLY flags from the VALID FLAGS list. NEVER hallucinate flags."
                    .to_string()
            }
            CliPattern::Subcommand { .. } => {
                "You are a CLI command generator for a SUBCOMMAND-based tool. \
                 CRITICAL RULES:\n\
                 1. MUST include the SUBCOMMAND NAME FIRST (e.g., 'sort', 'filter', 'view')\n\
                 2. Then add flags from VALID FLAGS list\n\
                 3. Example CORRECT: 'sort -o out.bam in.bam'\n\
                 4. Example WRONG: '-o out.bam in.bam' (missing subcommand)\n\
                 5. NEVER make up flags not in the VALID FLAGS list."
                    .to_string()
            }
            CliPattern::MetaTool { requires_prefix } => {
                if *requires_prefix {
                    "You are a CLI command generator for a meta-tool. \
                     Include the module name as the first argument."
                        .to_string()
                } else {
                    "You are a CLI command generator. Use the specific tool module.".to_string()
                }
            }
            CliPattern::MultiEntry { .. } => {
                "You are a CLI command generator for a multi-entry tool suite. \
                 Use the specific sub-entry command name as shown."
                    .to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_enhancer_without_rag() {
        let enhancer = DocEnhancer::new();

        assert!(!enhancer.is_rag_enabled());

        // Simple doc for testing
        let doc = "Usage: test [options]\n\nOptions:\n  -o FILE  Output file\n  -v       Verbose";
        let analysis = enhancer.analyze(doc, "test", "set output file");

        assert!(analysis.rag_context.is_none());
        assert!(!analysis.valid_flags.is_empty());
    }

    #[test]
    fn test_doc_enhancer_with_rag() {
        let enhancer = DocEnhancer::with_rag();

        assert!(enhancer.is_rag_enabled());

        // Test with samtools-like task
        let doc = "Usage: samtools <command> [options]\n\nCommands:\n  sort      Sort alignment file\n  view      View alignment";
        let analysis = enhancer.analyze(doc, "samtools", "sort the BAM file");

        // RAG context should be populated
        assert!(analysis.rag_context.is_some());

        let context = analysis.rag_context.unwrap();
        // Should find similar examples
        assert!(!context.similar_examples.is_empty());
    }

    #[test]
    fn test_build_rag_enhanced_prompt() {
        let enhancer = DocEnhancer::with_rag();

        let doc = "Usage: samtools sort [options] <in.bam>\n\nOptions:\n  -o FILE  Output file\n  -@ INT   Threads";
        let structured = crate::doc_processor::DocProcessor::new().process(doc);
        let analysis = enhancer.analyze(doc, "samtools", "sort BAM file");

        let prompt =
            enhancer.build_rag_enhanced_prompt("samtools", "sort BAM file", &structured, &analysis);

        // Prompt should contain RAG elements
        assert!(prompt.contains("SIMILAR EXAMPLES") || prompt.contains("VALID FLAGS"));
        assert!(prompt.contains("samtools"));
    }
}
