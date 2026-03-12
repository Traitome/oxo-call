//! Simulation utilities for generating realistic bioinformatics datasets.
//!
//! All generated data is synthetic and suitable for benchmarking without
//! requiring access to real experimental data or large reference genomes.

pub mod fastq;
pub mod genome;
pub mod omics;

pub use fastq::{simulate_paired_fastq, FastqSimParams};
pub use genome::{simulate_fasta, FastaSimParams};
pub use omics::{OmicsScenario, simulate_scenario};
