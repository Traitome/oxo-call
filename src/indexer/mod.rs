//! Local document indexing for L2 evidence.
//!
//! Provides bioconda metadata lookup and a keyword-based document store
//! that retrieves relevant documentation chunks for any bioinformatics tool.
//! Zero ML dependencies — uses TF-IDF weighted keyword matching.

pub mod bioconda;
pub mod store;

pub use bioconda::BiocondaIndex;
