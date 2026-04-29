//! RAG (Retrieval-Augmented Generation) Module
//!
//! Provides vector-based retrieval of similar examples and context
//! to enhance command generation quality.

pub mod embedding;
pub mod rag_enhancer;
pub mod retriever;
pub mod vector_store;

pub use embedding::{EmbeddingConfig, LocalEmbeddingModel};
pub use rag_enhancer::RagEnhancer;
pub use retriever::{ExampleStoreBuilder, RagRetriever, RetrievalContext};
