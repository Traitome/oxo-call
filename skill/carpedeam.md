---
name: carpedeam
category: Sequence Simulation / Data Generation
description: carpedeam is a command-line tool for generating synthetic DNA sequences and read datasets for benchmarking, testing, and simulation purposes. It produces FASTA/FASTQ output with configurable sequence complexity, GC bias, and coverage patterns. The tool supports paired-end read simulation with configurable insert size distributions and quality score modeling.
tags:
  - simulation
  - read-generation
  - synthetic-data
  - benchmarking
  - genomics
  - fastq
  - fasta
author: AI-generated
source_url: https://github.com/carpedeam/carpedeam
---

## Concepts

- **Output Formats**: carpedeam generates either FASTA (nucleotide sequences) or FASTQ (sequencing reads with quality scores) depending on the `--format` flag. FASTQ output includes Illumina-standard ASCII-encoded quality strings based on a configurable error model.
- **GC Content Control**: The `--gc` flag controls the overall guanine-cytosine fraction of generated sequences (0.0–1.0). This directly influences AT/GC bias in sequencing coverage and affects downstream analyses like alignment sensitivity.
- **Paired-End Simulation**: Using `--paired` with `--insert-size` and `--insert-std` generates two read files (R1 and R2) with a normal distribution of fragment lengths. Reads oriented in opposite directions require downstream tools to handle inward-facing or outward-facing orientation correctly.
- **Quality Score Modeling**: The `--quality-offset` flag (33 or 64) selects the ASCII encoding standard: 33 for modern Illumina (Phred+33), 64 for older platforms (Phred+64). Mismatching this with your analysis pipeline causes quality-score-based filters to fail silently.
- **Seed Reproducibility**: The `--seed` flag initializes the random number generator. Identical parameters with the same seed produce byte-identical output, which is essential for reproducible benchmarking across runs and environments.

## Pitfalls

- **Missing Quality Offset for FASTQ**: When generating FASTQ output, omitting `--quality-offset` defaults to Phred+33. If your downstream pipeline expects Phred+64 (older Illumina data), all quality-based thresholds will be evaluated incorrectly, leading to either over-filtering or under-filtering of reads.
- **Insert Size Distribution Mismatch**: Specifying `--insert-size` without `--insert-std` causes the tool to use a default standard deviation that may not match your experimental library. Overly narrow distributions produce reads with unnatural uniformity; overly wide distributions create implausibly overlapping or widely separated pairs.
- **Integer Overflow for Large Read Counts**: The `--num-reads` flag accepts large values but may overflow on 32-bit systems when combined with long `--read-length` values. Always verify output file sizes match expectations before using synthetic reads in production benchmarks.
- **Confusing GC Content with Base Composition**: The `--gc` flag controls overall GC fraction but does not guarantee uniform dinucleotide or trinucleotide composition. Regions with extreme GC content (e.g., extreme AT-rich or GC-rich genomes) will not be represented accurately unless combined with additional complexity flags.
- **Format Mismatch in Pipelines**: If carpedeam output is piped directly into alignment tools without explicit format flags, assumptions about file format may vary by tool version. Always specify `--format fasta` or `--format fastq` explicitly rather than relying on filename extensions.

## Examples

### Generate a single FASTA file with 1000 bp sequences
**Args:** `--num-sequences 50 --seq-length 1000 --gc 0.5 --format fasta --output simulated_genome.fa`
**Explanation:** This creates 50 synthetic sequences of 1000 nucleotides each with 50% GC content, output as uncompressed FASTA. The balanced GC content approximates average bacterial genome composition.

### Generate paired-end FASTQ reads with Illumina 1.8+ quality encoding
**Args:** `--num-reads 1000000 --read-length 150 --paired --insert-size 300 --insert-std 25 --quality-offset 33 --format fastq --output reads`
**Explanation:** This generates 1 million paired-end reads (500,000 pairs) of 150 bp length with a mean insert size of 300 bp and standard deviation of 25 bp, using Phred+33 quality encoding compatible with Illumina 1.8+.

### Generate synthetic reads with extreme GC bias for stress-testing
**Args:** `--num-reads 500000 --read-length 250 --gc 0.65 --quality-mean 30 --format fastq --output gc_rich_test`
**Explanation:** This produces 500,000 reads with 65% GC content (above typical 40–45% for many genomes), enabling alignment tool evaluation under biased base composition conditions that reveal GC-dependent bias in mappers.

### Generate single-end reads for testing without pairing
**Args:** `--num-reads 200000 --read-length 100 --quality-mean 35 --format fastq --output single_end_test`
**Explanation:** This generates single-end (unpaired) reads of 100 bp length with average quality score of Phred 35, suitable for testing workflows that do not involve paired-end resolution.

### Reproducible dataset generation using a fixed seed
**Args:** `--num-reads 10000 --read-length 125 --seed 42 --gc 0.5 --format fastq --output reproducible_test`
**Explanation:** The `--seed 42` parameter ensures this exact command produces byte-identical output on any platform, making it ideal for unit tests, method comparisons, and reproducible benchmark datasets across different compute environments.

### Generate a multi-sample dataset with varying GC profiles
**Args:** `--num-reads 300000 --read-length 200 --gc 0.35 --format fastq --output at_rich_sample`
**Explanation:** This generates reads with 35% GC content (AT-rich), simulating AT-rich genomes such as Plasmodium falciparum or other eukaryotic pathogens where standard aligners may exhibit reduced sensitivity.