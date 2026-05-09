---
name: ale-core
category: Genome Assembly Evaluation
description: A tool for evaluating the quality and completeness of genome assemblies by detecting misassemblies, structural variants, and assembly gaps. Provides likelihood-based scoring and detailed error reporting.
tags: [genome-assembly, assembly-evaluation, misassembly-detection, structural-variants, bioinformatics, quality-control]
author: AI-generated
source_url: https://github.com/ale-core/ale-core
---

## Concepts

- **Assembly Input Formats**: ale-core accepts FASTA or FASTQ assembly files as primary input, optionally paired with reference sequences for comparative analysis. The tool parses sequences internally and builds likelihood models for each contig.
- **Likelihood Scoring**: The core algorithm computes assembly likelihood scores (ALE scores) based on k-mer coverage, read depth consistency, and structural integrity. Lower scores indicate higher confidence in assembly correctness.
- **Output Reports**: Results are generated as plain-text reports containing per-contig scores, error locations (with genomic coordinates), and summary statistics. JSON output is available via the `--json` flag for programmatic parsing.
- **Reference-Free Mode**: When no reference genome is provided, ale-core performs reference-free evaluation using only coverage and k-mer statistics to estimate assembly quality.

## Pitfalls

- **Ignoring Input Format Validation**: Providing compressed input files without proper decompression causes parsing failures. Ensure FASTA/FASTQ files are uncompressed or use appropriate decompression tools before piping to ale-core.
- **Forgetting to Build Index First**: Running ale-core without pre-building the assembly index when using large genomes leads to excessive runtime. Always run `ale-core-index` for genomes over 100 Mb.
- **Mismatched Reference and Assembly**: Using a reference genome from a different species or strain produces meaningless ALE scores and false error calls. Ensure reference and assembly are from the same biological source.
- **Insufficient Memory for Large Genomes**: Processing mammalian-sized assemblies (>2 Gb) without increasing memory allocation causes OOM crashes. Use the `--memory` flag to allocate sufficient RAM.

## Examples

### Evaluate a small bacterial assembly against a reference
**Args:** `assembly.fasta --reference ref.fasta --output report.txt`
**Explanation:** Computes ALE scores for the bacterial assembly contigs by comparing against the reference genome and writes detailed results to a text report.

### Generate JSON output for automated pipelines
**Args:** `draft_assembly.fa --reference ref_genome.fa --json --output results.json`
**Explanation:** Produces machine-parseable JSON output containing ALE scores, error coordinates, and summary statistics for integration into automated analysis workflows.

### Run reference-free quality assessment
**Args:** `final_assembly.fasta --no-reference --output qc_report.txt`
**Explanation:** Evaluates assembly quality using only coverage metrics and k-mer statistics when no reference sequence is available, useful for de novo assembly validation.

### Process multi-contig assembly with verbose logging
**Args:** `genome_build.fa --reference ref.fa --verbose --log ale_log.txt`
**Explanation:** Enables detailed logging for troubleshooting, capturing all intermediate calculations and warnings during the evaluation process.

### Adjust k-mer size for specific organism
**Args:** `my_assembly.fasta --reference ref.fasta --kmer 31 --output ale_results.txt`
**Explanation:** Overrides the default k-mer size (25) with 31-mers, which is appropriate for larger eukaryotic genomes with higher repeat content.

### Build index for large genome prior to evaluation
**Args:** `large_genome.fa --build-index`
**Explanation:** Pre-processes and indexes a large genome assembly file to accelerate subsequent evaluation runs, essential for mammalian-sized genomes.