---
name: art_modern-openmpi
category: Read Simulation / WGS
description: A high-performance read simulator based on ART (Adaptive Read Training) that generates realistic single-end or paired-end NGS reads withOpenMPI parallelization support. Simulates sequencing errors, insertions, deletions, and quality variations based on empirical error profiles from real sequencing runs.
tags: [ngs, simulation, read-generation, wgs, variant-calling, openmpi, parallel]
author: AI-generated
source_url: https://github.com/ART-NG/art_modern-openmpi
---

## Concepts

- **Input Formats**: Accepts FASTA or FASTQ reference genomes as input. Can also use built-in error profiles from Illumina, Roche 454, or SOLiD platforms. Supports reading reference sequences directly or from compressed (`.gz`) files.
- **Output Formats**: Generates simulated reads in FASTQ format (single-end or paired-end). Produces separate output files for forward (`_1.fq`) and reverse (`_2.fq`) reads when simulating paired-end reads. Includes quality scores per base following Phred+33 encoding.
- **Error Modeling**: Uses platform-specific error models that account for base-calling errors, insertion-deletion (indel) errors, and systematic biases. The `--errfree` option can generate error-free reads for perfect reference comparisons, while default operation includes realistic quality degradation across read length.
- **Paired-End Simulation**: When simulating paired-end reads, uses the `--paired` flag with `--mateLen` (mean inner distance) and `--mateStd` (standard deviation) to control fragment size distribution. Reads are written to interleaved or separate files depending on `--listOut` settings.
- **OpenMPI Parallelization**: Distributes read generation across multiple processes using OpenMPI for high-throughput simulation. Use `--nproc` to specify the number of parallel processes, which significantly reduces runtime for large-scale simulations.

## Pitfalls

- **Incorrect fragment size specification**: Setting `--mateLen` (inner distance) smaller than twice the read length produces overlapping or invalid pairs, leading to errors in downstream variant calling that relies on proper read pairing.
- **Forgetting to specify read length**: Default read length may differ from intended simulation (often 100bp or 150bp depending on platform profile), causing inconsistent coverage calculations when comparing simulated to expected data.
- **Mismatched platform profiles**: Using an inappropriate error profile (e.g., simulating SOLiD errors with an Illumina profile) produces unrealistic read quality distributions, invalidating downstream method comparisons.
- **Output directory permissions**: If the output directory doesn't exist or lacks write permissions, the tool fails silently without generating output files, causing downstream pipeline failures with cryptic errors.
- **Reference sequence case sensitivity**: ART treats lowercase and uppercase bases differently (lowercase may be masked or converted), causing unexpected reference representation in output reads.

## Examples

### Simulate paired-end reads from a bacterial reference genome
**Args:** `--genome ref.fa --paired --len 150 --coverage 30 --mateLen 200 --mateStd 20 --output bacterial_sim`
**Explanation:** Generates ~30x coverage paired-end reads (2x150bp) with 200bp mean inner distance and 20bp standard deviation from the reference, creating realistic Illumina-style paired-end data for bacterial genomics.

### Generate single-end reads with high coverage for variant calling training
**Args:** `--genome human_chr21.fa --single --len 100 --coverage 50 --err prof --output vc_training`
**Explanation:** Produces 50x coverage single-end 100bp reads using the built-in error profile for training variant calling workflows, creating sufficient depth for realistic variant detection exercises.

### Use a specific platform error profile (Roche 454)
**Args:** `--genome ref.fa --platform 454 --paired --len 400 --coverage 10 --output r454_data`
**Explanation:** Simulates Roche 454 paired-end reads (400bp read length) with 10x coverage using the 454-specific error model, appropriate for simulating 454 GS FLX or Junior sequencing data.

### Generate error-free reads for benchmarking alignment tools
**Args:** --errfree --genome ref.fa --single --len 200 --coverage 10 --output perfect_reads
**Explanation:** Creates error-free reads with perfect quality scores for testing alignment algorithm accuracy without confounding sequencing errors, useful for-aligner-specific performance benchmarks.

### Run parallel simulation with 8 MPI processes
**Args:** --genome large_ref.fa --paired --len 150 --coverage 50 --nproc 8 --output parallel_sim
**Explanation:** Uses 8 OpenMPI processes to generate 50x paired-end reads in parallel, dramatically reducing simulation time for large reference genomes or high-coverage scenarios.

### Specify output read count instead of coverage
**Args:** --genome ref.fa --paired --len 150 --numReads 1000000 --output count_based
**Explanation:** Generates exactly one million read pairs rather than calculating coverage automatically, useful when exact read counts are required for specific pipeline testing.

### Simulate with custom error profile file
**Args:** --genome ref.fa --paired --len 150 --coverage 30 --err custom_err.prof --output custom_err_data
**Explanation:** Uses a custom error profile (created from real data) rather than built-in platform profiles, enabling simulation that matches specific sequencing run characteristics.