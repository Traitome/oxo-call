//! Local document indexing for L2 evidence.
//!
//! Provides bioconda metadata lookup, web document fetching,
//! and a keyword-based document store for L2 evidence retrieval.

pub mod bioconda;
pub mod fetcher;
pub mod store;

pub use bioconda::BiocondaIndex;
