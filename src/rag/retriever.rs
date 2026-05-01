//! RAG Retriever Module
//!
//! Provides context retrieval for command generation tasks.
//! Retrieves similar examples, relevant documentation sections, and best practices.

#![allow(dead_code)]

use crate::rag::embedding::EmbeddingModel;
use crate::rag::vector_store::{VectorEntry, VectorStore};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration for retrieval
#[derive(Debug, Clone)]
pub struct RetrievalConfig {
    /// Number of similar examples to retrieve
    pub num_examples: usize,
    /// Similarity threshold for examples
    pub example_threshold: f32,
    /// Number of documentation chunks
    pub num_doc_chunks: usize,
    /// Weight for task similarity
    pub task_weight: f32,
    /// Weight for command similarity
    pub command_weight: f32,
    /// Enable semantic search
    pub semantic_search: bool,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            num_examples: 3,
            example_threshold: 0.3,
            num_doc_chunks: 2,
            task_weight: 0.6,
            command_weight: 0.4,
            semantic_search: true,
        }
    }
}

/// Context retrieved for command generation
#[derive(Debug, Clone, Default)]
pub struct RetrievalContext {
    /// Tool name
    pub tool: String,
    /// Task description
    pub task: String,
    /// Selected subcommand (if any)
    pub selected_subcommand: Option<SubcommandContext>,
    /// Similar command examples
    pub similar_examples: Vec<ExampleContext>,
    /// Relevant documentation sections
    pub doc_sections: Vec<DocSection>,
    /// Best practices for this tool
    pub best_practices: Vec<String>,
    /// Common pitfalls
    pub pitfalls: Vec<String>,
}

/// Subcommand context
#[derive(Debug, Clone)]
pub struct SubcommandContext {
    /// Subcommand name
    pub name: String,
    /// Description
    pub description: String,
    /// Usage pattern
    pub usage: String,
    /// Relevant flags
    pub relevant_flags: Vec<String>,
}

/// Example command with metadata
#[derive(Debug, Clone)]
pub struct ExampleContext {
    /// Task description
    pub task: String,
    /// Generated command
    pub command: String,
    /// Similarity score
    pub similarity: f32,
    /// Tool name
    pub tool: String,
    /// Subcommand
    pub subcommand: Option<String>,
    /// Explanation
    pub explanation: Option<String>,
}

/// Documentation section
#[derive(Debug, Clone)]
pub struct DocSection {
    /// Section title
    pub title: String,
    /// Section content
    pub content: String,
    /// Relevance score
    pub relevance: f32,
}

/// RAG Retriever
#[derive(Clone)]
pub struct RagRetriever {
    /// Vector store for examples
    example_store: VectorStore,
    /// Vector store for documentation
    doc_store: VectorStore,
    /// Embedding model
    embedding_model: Arc<dyn EmbeddingModel>,
    /// Configuration
    config: RetrievalConfig,
    /// In-memory cache for frequent queries (uses RefCell for interior mutability)
    cache: RefCell<HashMap<String, RetrievalContext>>,
}

impl RagRetriever {
    /// Create a new retriever
    pub fn new(
        example_store: VectorStore,
        doc_store: VectorStore,
        embedding_model: Arc<dyn EmbeddingModel>,
    ) -> Self {
        Self {
            example_store,
            doc_store,
            embedding_model,
            config: RetrievalConfig::default(),
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: RetrievalConfig) -> Self {
        self.config = config;
        self
    }

    /// Retrieve context for a task
    pub fn retrieve(&self, tool: &str, task: &str) -> RetrievalContext {
        // Check cache
        let cache_key = format!("{}:{}", tool, task);
        {
            let cache = self.cache.borrow();
            if let Some(cached) = cache.get(&cache_key) {
                return cached.clone();
            }
        }

        // Encode task
        let task_embedding = self.embedding_model.encode(task);

        // Retrieve similar examples
        let similar_examples = self.retrieve_examples(&task_embedding, tool);

        // Retrieve relevant documentation
        let doc_sections = self.retrieve_docs(&task_embedding, tool);

        // Infer subcommand from examples
        let selected_subcommand = self.infer_subcommand(&similar_examples, task);

        // Get best practices
        let best_practices = self.get_best_practices(tool);

        // Get common pitfalls
        let pitfalls = self.get_pitfalls(tool);

        let context = RetrievalContext {
            tool: tool.to_string(),
            task: task.to_string(),
            selected_subcommand,
            similar_examples,
            doc_sections,
            best_practices,
            pitfalls,
        };

        // Cache result
        self.cache.borrow_mut().insert(cache_key, context.clone());

        context
    }

    /// Retrieve similar examples
    fn retrieve_examples(&self, task_embedding: &[f32], tool: &str) -> Vec<ExampleContext> {
        // Filter by tool
        let results = self.example_store.search_with_filter(
            task_embedding,
            self.config.num_examples * 2, // Get more for reranking
            |entry| {
                entry
                    .get_metadata("tool")
                    .map(|t| t == tool)
                    .unwrap_or(false)
            },
        );

        results
            .into_iter()
            .filter(|r| r.score >= self.config.example_threshold)
            .take(self.config.num_examples)
            .map(|r| ExampleContext {
                task: r
                    .entry
                    .get_metadata("task")
                    .cloned()
                    .unwrap_or_else(|| r.entry.text.clone()),
                command: r.entry.get_metadata("command").cloned().unwrap_or_default(),
                similarity: r.score,
                tool: tool.to_string(),
                subcommand: r.entry.get_metadata("subcommand").cloned(),
                explanation: r.entry.get_metadata("explanation").cloned(),
            })
            .collect()
    }

    /// Retrieve relevant documentation sections
    fn retrieve_docs(&self, task_embedding: &[f32], tool: &str) -> Vec<DocSection> {
        let results = self.doc_store.search_with_filter(
            task_embedding,
            self.config.num_doc_chunks,
            |entry| {
                entry
                    .get_metadata("tool")
                    .map(|t| t == tool)
                    .unwrap_or(false)
            },
        );

        results
            .into_iter()
            .map(|r| DocSection {
                title: r.entry.get_metadata("title").unwrap_or(&r.entry.id).clone(),
                content: r.entry.text.clone(),
                relevance: r.score,
            })
            .collect()
    }

    /// Infer subcommand from similar examples
    fn infer_subcommand(
        &self,
        examples: &[ExampleContext],
        _task: &str,
    ) -> Option<SubcommandContext> {
        // Count subcommand frequencies
        let mut subcmd_counts: HashMap<String, usize> = HashMap::new();
        for ex in examples {
            if let Some(ref sc) = ex.subcommand {
                *subcmd_counts.entry(sc.clone()).or_insert(0) += 1;
            }
        }

        // Find most common subcommand
        let most_common = subcmd_counts.into_iter().max_by_key(|(_, count)| *count)?;

        // Only return if at least 2 examples agree
        if most_common.1 < 2 {
            return None;
        }

        Some(SubcommandContext {
            name: most_common.0.clone(),
            description: format!("Inferred from {} similar examples", most_common.1),
            usage: String::new(),
            relevant_flags: Vec::new(),
        })
    }

    /// Get best practices for a tool
    fn get_best_practices(&self, tool: &str) -> Vec<String> {
        // This would come from a knowledge base
        // For now, return generic practices
        let practices: HashMap<&str, Vec<&str>> = [
            (
                "samtools",
                vec![
                    "Always use -o for output files",
                    "Use -@ for multi-threading",
                    "Sort before indexing",
                ],
            ),
            (
                "bedtools",
                vec![
                    "Use -sorted for large files",
                    "Consider -header to preserve headers",
                ],
            ),
            (
                "bcftools",
                vec![
                    "Use -Oz for compressed VCF output",
                    "Filter before processing large files",
                ],
            ),
        ]
        .into_iter()
        .collect();

        practices
            .get(tool)
            .map(|p| p.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    /// Get common pitfalls for a tool
    fn get_pitfalls(&self, tool: &str) -> Vec<String> {
        let pitfalls: HashMap<&str, Vec<&str>> = [
            (
                "samtools",
                vec![
                    "Forgetting to specify output with -o",
                    "Not checking if BAM is sorted before indexing",
                ],
            ),
            (
                "bedtools",
                vec![
                    "Forgetting -wa or -wb flags in intersect",
                    "Not sorting BED files before operations",
                ],
            ),
        ]
        .into_iter()
        .collect();

        pitfalls
            .get(tool)
            .map(|p| p.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    /// Add a new example to the store
    pub fn add_example(
        &mut self,
        tool: &str,
        task: &str,
        command: &str,
        subcommand: Option<&str>,
        explanation: Option<&str>,
    ) -> Result<(), String> {
        let id = format!("{}:{}:{}", tool, task, chrono::Utc::now().timestamp());
        let text = format!("{} {}", tool, task);
        let embedding = self.embedding_model.encode(&text);

        let entry = VectorEntry::new(&id, &text, embedding)
            .with_metadata("tool", tool)
            .with_metadata("task", task)
            .with_metadata("command", command);

        if let Some(sc) = subcommand {
            let _ = entry.clone().with_metadata("subcommand", sc);
        }

        if let Some(exp) = explanation {
            let _ = entry.clone().with_metadata("explanation", exp);
        }

        self.example_store.add(entry)
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.borrow_mut().clear();
    }

    /// Get store statistics
    pub fn stats(&self) -> RetrieverStats {
        RetrieverStats {
            example_count: self.example_store.count(),
            doc_count: self.doc_store.count(),
            cache_size: self.cache.borrow().len(),
        }
    }
}

impl std::fmt::Debug for RagRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RagRetriever")
            .field("example_store", &self.example_store)
            .field("doc_store", &self.doc_store)
            .field("config", &self.config)
            .field("cache_size", &self.cache.borrow().len())
            .finish()
    }
}

/// Statistics for the retriever
#[derive(Debug, Clone)]
pub struct RetrieverStats {
    /// Number of examples
    pub example_count: usize,
    /// Number of doc sections
    pub doc_count: usize,
    /// Cache size
    pub cache_size: usize,
}

/// Simple in-memory example store builder
pub struct ExampleStoreBuilder;

impl ExampleStoreBuilder {
    /// Build example store from predefined examples
    pub fn build_common_examples(embedding_model: Arc<dyn EmbeddingModel>) -> VectorStore {
        let mut store = VectorStore::new(embedding_model.dimension());

        // Common bioinformatics examples
        let examples = vec![
            (
                "samtools",
                "sort BAM file",
                "sort -o sorted.bam input.bam",
                Some("sort"),
            ),
            (
                "samtools",
                "index BAM file",
                "index input.bam",
                Some("index"),
            ),
            (
                "samtools",
                "view BAM header",
                "view -H input.bam",
                Some("view"),
            ),
            (
                "samtools",
                "filter by quality",
                "view -q 30 -b input.bam",
                Some("view"),
            ),
            (
                "bedtools",
                "intersect two BED files",
                "intersect -a a.bed -b b.bed",
                Some("intersect"),
            ),
            (
                "bedtools",
                "merge overlapping regions",
                "merge -i input.bed",
                Some("merge"),
            ),
            (
                "bcftools",
                "filter VCF variants",
                "filter -i 'QUAL>30' input.vcf",
                Some("filter"),
            ),
            (
                "bcftools",
                "view VCF header",
                "view -h input.vcf",
                Some("view"),
            ),
        ];

        for (tool, task, command, subcmd) in examples {
            let text = format!("{} {}", tool, task);
            let embedding = embedding_model.encode(&text);
            let id = format!("{}_{}", tool, task.replace(' ', "_"));

            let mut entry = VectorEntry::new(&id, &text, embedding)
                .with_metadata("tool", tool)
                .with_metadata("task", task)
                .with_metadata("command", command);

            if let Some(sc) = subcmd {
                entry = entry.with_metadata("subcommand", sc);
            }

            let _ = store.add(entry);
        }

        store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::embedding::{EmbeddingConfig, LocalEmbeddingModel};

    fn create_test_retriever() -> RagRetriever {
        let config = EmbeddingConfig::default();
        let model = Arc::new(LocalEmbeddingModel::new(config));

        let example_store = ExampleStoreBuilder::build_common_examples(model.clone());
        let doc_store = VectorStore::new(model.dimension());

        RagRetriever::new(example_store, doc_store, model)
    }

    #[test]
    fn test_retrieve_examples() {
        let retriever = create_test_retriever();

        let context = retriever.retrieve("samtools", "sort my bam file");

        assert!(!context.similar_examples.is_empty());
        assert_eq!(context.tool, "samtools");

        // Should find sort example
        let sort_example = context
            .similar_examples
            .iter()
            .any(|ex| ex.command.contains("sort"));
        assert!(sort_example);
    }

    #[test]
    fn test_infer_subcommand() {
        let retriever = create_test_retriever();

        let context = retriever.retrieve("samtools", "sort the alignment");

        if let Some(ref sc) = context.selected_subcommand {
            assert_eq!(sc.name, "sort");
        }
    }

    #[test]
    fn test_caching() {
        let retriever = create_test_retriever();

        let context1 = retriever.retrieve("samtools", "sort");
        let context2 = retriever.retrieve("samtools", "sort");

        assert_eq!(retriever.stats().cache_size, 1);
        assert_eq!(context1.tool, context2.tool);
    }
}
