---
name: amira
category: read_simulator
description: A read simulator for Next-Generation Sequencing (NGS) data that generates synthetic paired-end reads from a reference genome, useful for testing bioinformatics pipelines and evaluating variant detection accuracy.
tags: [read-simulation, ngs, testing, variant-calling, quality-control]
author: AI-generated
source_url: https://github.com/amira-tools/amira
---

## Concepts

- **Input Format**: Accepts a reference genome in FASTA format. The tool parses the FASTA file to identify genomic coordinates where simulated reads will be generated, preserving sequence context andchromosome naming conventions.
- **Output Format**: Produces synthetic paired-end reads in FASTQ format, generating quality scores (Phred-scaled) that can be configured to mimic different sequencing platforms (Illumina, Ion Torrent, etc.).
- **Variant Simulation**: Supports injection of simulated variants (SNPs, insertions, deletions) at specified genomic positions, allowing users to create ground-truth datasets for benchmarking variant callers.
- **Read Parameters**: Controls read length, insert size, error rate, and coverage depth through command-line arguments, enabling realistic simulation scenarios for different sequencing chemistries.
- **Companion Binary**: The `amira-build` companion tool creates indices from the reference genome to accelerate simulation runtime, similar to how BWA and Bowtie2 use index files for efficient alignment.

## Pitfalls

- **Forgetting to Build Indices**: Running `amira` without pre-building indices with `amira-build` causes significantly slower simulation because the tool must dynamically parse the reference genome for each read, increasing runtime by 5-10x on large genomes.
- **Mismatched Read Length and Insert Size**: Specifying a read length larger than the insert size produces invalid paired-end reads where read pairs overlap incorrectly, leading to downstream analysis artifacts when used for pipeline testing.
- **Ignoring Quality Score Encoding**: Using Phred+64 encoding when the pipeline expects Phred+33 (standard Illumina 1.8+) causes base quality to be misinterpreted, resulting in false positives during variant calling validation.
- **Reference Genome Case Sensitivity**: The input FASTA must have consistent chromosome naming (e.g., "chr1" vs "1") matching the variant position file, otherwise injected variants will be placed at incorrect or null coordinates.
- **Coverage Overestimation**: Setting coverage targets too high for large genomes without accounting for memory constraints causes system resource exhaustion, resulting in simulation failure or truncation mid-process.

## Examples

### Simulate paired-end reads from a bacterial genome

**Args:** -i ref.fasta -o output -l 150 -p 300 -c 30

**Explanation:** Generates 150bp paired-end reads with 300bp insert size at 30x coverage from the reference genome, creating a realistic test dataset for bacterial variant calling pipelines.

### Include simulated SNPs in the output reads

**Args:** -i ref.fasta -v variants.vcf -o output -l 100 -p 200

**Explanation:** Injects SNPs from a VCF file into the simulated reads, creating a ground-truth dataset where true variant positions are known for validating variant callers.

### Simulate Illumina-style quality scores

**Args:** -i ref.fasta -o output -q illumina -l 250 -p 500 -c 50

**Explanation:** Uses Illumina-type quality score encoding (Phred+33) at 50x coverage with 250bp reads and 500bp insert size, suitable for testing modern Illumina pipelines.

### Build index for faster simulation runs

**Args:** -i ref.fasta -o amira_index

**Explanation:** Pre-builds index files from the reference genome to accelerate subsequent simulation runs, reducing computational overhead when generating multiple datasets.

### Simulate heterogeneous coverage across the genome

**Args:** -i ref.fasta -o output -l 150 -p 300 -c 20 --gc-bias

**Explanation:** Generates reads with systematic GC content bias affecting coverage distribution, mimicking real sequencing artifacts for testing pipeline robustness.