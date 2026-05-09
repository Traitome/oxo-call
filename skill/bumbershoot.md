---
name: bumbershoot
category: Bioinformatics utilities
description: A suite of bioinformatics tools for sequence analysis, alignment, and variant calling. Provides modular companion binaries for different tasks including read alignment, sequence filtering, and statistical analysis.
tags:
  - bioinformatics
  - sequence-analysis
  - genomics
  - variant-calling
  - alignment
author: AI-generated
source_url: https://github.com/bumbershoot/bumbershoot
---

## Concepts

- **Modular Architecture**: bumbershoot consists of multiple companion binaries (e.g., `bumbershoot-align`, `bumbershoot-filter`, `bumbershoot-stats`) that can be piped together for complex workflows, each handling a specific stage of analysis.
- **Input Formats**: Accepts standard FASTQ, FASTA, BAM, and VCF files. Use `--input-format` to explicitly specify non-standard encodings. Supports gzipped input when suffixed with `.gz`.
- **Output Streams**: By default outputs to stdout; use `--output` to write to file. Streaming mode (`--stream`) enables processing of large files without loading entirely into memory.
- **Index Files**: For reference-based operations, bumbershoot creates `.bsi` index files via `bumbershoot-build` which must be generated before alignment or mapping operations.

## Pitfalls

- **Missing Index Files**: Running alignment commands without pre-built `.bsi` index causes immediate failure with a cryptic index-not-found error. Always run `bumbershoot-build` on reference sequences before alignment tasks.
- **Mismatched Input Format**: Specifying incorrect `--input-format` (e.g., passing FASTQ when the file is FASTA) results in parsing errors and corrupted output. Verify file format before execution.
- **Memory Overflow on Large Datasets**: Not using `--chunk-size` for datasets exceeding available RAM causes system slowdown or termination. Process in chunks for files larger than half the available memory.
- **Overwriting Outputs Silently**: bumbershoot silently overwrites existing output files without warning. Use `--no-clobber` to prevent accidental data loss.

## Examples

### Build index for a reference genome
**Args:** `--reference hg19.fa --output hg19.bsi`
**Explanation:** Creates a bumbershoot index file from a FASTA reference genome, required for subsequent alignment operations.

### Align FASTQ reads to indexed reference
**Args:** `reads.fq.gz --index hg19.bsi --output aligned.sam`
**Explanation:** Aligns gzipped FASTQ reads to the indexed reference genome, outputting in SAM format.

### Filter alignments by mapping quality
**Args:** `aligned.sam --min-mapq 30 --output filtered.bam`
**Explanation:** Removes alignments with mapping quality below 30, significantly reducing false positives in variant calling.

### Convert BAM to FASTQ
**Args:** `aligned.bam --output reads.fq --format fastq`
**Explanation:** Extracts read sequences from aligned BAM file back to FASTQ format for downstream re-analysis.

### Generate alignment statistics
**Args:** `aligned.bam --stats --output report.txt`
**Explanation:** Produces summary statistics including coverage depth, read counts, and per-chromosome alignment rates.

### Variant calling from alignments
**Args:** `filtered.bam --reference hg19.fa --vcf output.vcf --min-depth 10`
**Explanation:** Calls variants requiring minimum 10x coverage depth, outputting variants in VCF format for annotation.

### Parallel processing of multiple files
**Args:** `sample1.fq sample2.fq sample3.fq --index hg19.bsi --parallel 4 --output-dir results/`
**Explanation:** Processes three FASTQ files concurrently using 4 threads, writing outputs to a specified directory.

---