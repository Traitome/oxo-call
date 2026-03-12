//! Simulation utilities for generating realistic bioinformatics datasets.
//!
//! All generated data is synthetic and suitable for benchmarking without
//! requiring access to real experimental data or large reference genomes.

pub mod fastq;
pub mod genome;
pub mod omics;

pub use fastq::{FastqSimParams, simulate_paired_fastq};
pub use genome::{FastaSimParams, simulate_fasta};
pub use omics::{OmicsScenario, simulate_scenario};
