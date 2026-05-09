---
name: breseq
category: Microbial Variant Calling / Whole-Genome Sequencing
description: A computational tool for identifying mutations (point mutations, indels, and structural variants) in bacterial whole-genome sequencing data by performing integrated read alignment and de novo assembly-based mutation detection against a reference genome.
tags:
  - variant-calling
  - microbial genomics
  - bacterial WGS
  - resequencing
  - mutation detection
  - bioinformatics
author: AI-generated
source_url: https://github.com/barricklab/breseq
---

## Concepts

- **Reference Genome Indexing**: Before running mutation detection, breseq requires a pre-built reference index using the companion binary `breseq-build`. This index stores pre-computed alignment scores and candidate polymorphism positions, enabling accurate comparison of reads to both the reference and consensus sequences during iterative rounds of base calling.

- **Dual Evidence Scoring**: breseq uses two types of evidence when evaluating candidate mutations: (1) **RA** (Read Alignment) evidence scores mutations purely from aligned reads against the reference, and (2) **BC** (BrowseCandidates) evidence scores mutations from both aligned reads and *de novo* assembled contigs generated from unmapped reads. Mutations must reach the minimum evidence threshold defined by the `--minimum-quality-score` cutoff in at least one evidence category to be reported.

- **Output Artifacts**: breseq produces several output files: `output.gdna` (annotated HTML report and underlying data), `output.vcf` (variant calls in Variant Call Format for downstream processing), `output Mutations`.txt (tab-delimited mutation summary), and `output evidence.html` (interactive read alignment visualizations). The `gdna` file can be re-opened with breseq to review evidence without re-running analysis.

- **Evidence Classes**: Called mutations are categorized into evidence classes: **RA** (read alignment only), **UN** (unmapped-read contig assembly only), **MC** (mixed evidence, read alignment + contig), and **JC** (junction evidence for structural rearrangements). Classifying evidence upfront helps interpret which mutations are high-confidence versus assembly artifacts.

- **Parallelization with Reference Files**: breseq can distribute work across multiple reference sequences using `--reference-files` with a comma-separated list or by specifying a directory containing multiple FASTA files. Each reference sequence is processed in parallel when the `-j` (jobs) option is set greater than 1, significantly reducing runtime for large pan-genome analyses.

## Pitfalls

- **Inconsistent Sample Name Matching**: If mate-pair read files are named inconsistently (e.g., `sample_1.fastq` and `sample2.fastq` without the underscore prefix on one file), breseq cannot pair them, resulting in a crash with a "Sample name mismatch" error and no mutation output generated. Always ensure both read files share the exact same base name prefix and differ only in `_1`/`_2` suffixes.

- **Missing Reference Index Creation**: Running `breseq` directly without first running `breseq-build` on the reference FASTA file produces a fatal error: "Error: Reference file not found. Index files should be generated using breseq-build." This is a hard dependency—always build the index before any analysis step.

- **Running Without Sufficient RAM**: breseq loads the entire reference genome index and all read data into memory per reference sequence being processed. Running with insufficient RAM causes OOM (Out of Memory) crashes, particularly when analyzing large genomes (>10 Mb) with deep sequencing coverage (>100x). Always allocate at least 4 GB RAM per reference sequence being analyzed in parallel, scaling with coverage depth.

- **Misconfiguring Evidence Score Cutoffs**: Setting `--minimum-quality-score` too low (below 2) causes false positive mutation calls from sequencing noise to be reported as valid mutations. Setting it too high (above 20) silently suppresses real low-frequency variants present at 5–20% frequency in mixed populations, defeating the purpose of breseq's sensitivity for heteroresistance and quasispecies analysis.

- **Relying on Default Output Paths**: If no explicit output directory is specified, breseq writes output files to the current working directory, overwriting any previous results without warning. Always use the `-o` flag to direct output to a dedicated directory to preserve analysis history and prevent accidental data loss during iterative re-runs.

## Examples

### Basic paired-end mutation detection against a reference
**Args:** `-r reference.gbk -j 4 -o breseq_output sample_1.fastq.gz sample_2.fastq.gz`
**Explanation:** This runs breseq in standard mode with read alignment and de novo assembly against the provided GenBank-formatted reference, using 4 parallel jobs for faster processing, and directing all output files into a dedicated `breseq_output/` directory.

### Indexing a reference genome for later use
**Args:** `reference.fasta`
**Explanation:** The `breseq-build` companion binary pre-computes and caches alignment lookup tables and candidate polymorphism positions from the input FASTA file, creating binary index files in the same directory that `breseq` will automatically locate at runtime.

### Mutation detection from a BAM file (pre-aligned reads)
**Args:** `-r reference.gbk -j 4 -o breseq_output -a alignments.bam`
**Explanation:** Specifying the BAM file input (`-a`) bypasses breseq's internal read alignment step and uses pre-aligned reads directly for variant calling, which is useful when reads have already been mapped with a different aligner or when working with archived alignment data.

### High-sensitivity detection of low-frequency variants in mixed populations
**Args:** `-r reference.gbk -j 4 -o breseq_output --minimum-quality-score 2 sample_1.fastq.gz sample_2.fastq.gz`
**Explanation:** Setting `--minimum-quality-score` to 2 (the lowest valid threshold) maximizes sensitivity for detecting rare variants present at frequencies as low as ~1% in heterogeneous microbial populations, such as during antibiotic heteroresistance studies or within viral quasispecies.

### Processing multiple reference sequences in parallel from a directory
**Args:** `-r references_dir/ -j 8 -o breseq_output sample_1.fastq.gz sample_2.fastq.gz`
**Explanation:** Pointing `-r` at a directory containing multiple FASTA files treats each file as an independent reference and distributes analysis across 8 parallel jobs, enabling efficient pan-genome screens where the sample is tested for mutations against all available reference genomes simultaneously.