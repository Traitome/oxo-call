//! RAG (Retrieval-Augmented Generation) Module
//!
//! Provides vector-based retrieval of similar examples and context
//! to enhance command generation quality.

pub mod embedding;
pub mod rag_enhancer;
pub mod retriever;
pub mod vector_store;

#[allow(unused_imports)]
pub use embedding::{EmbeddingConfig, LocalEmbeddingModel};
#[allow(unused_imports)]
pub use rag_enhancer::RagEnhancer;
#[allow(unused_imports)]
pub use retriever::{ExampleStoreBuilder, RagRetriever, RetrievalContext};
