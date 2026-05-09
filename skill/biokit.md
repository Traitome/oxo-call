---
name: biokit
category: bioinformatics-tools
description: A comprehensive bioinformatics toolkit for sequence analysis, alignment processing, and genomic data manipulation. Provides utilities for filtering, converting, and analyzing biological sequence data in common formats such as FASTA, FASTQ, SAM, and VCF.
tags:
  - sequence-analysis
  - alignment
  - genomics
  - data-conversion
  - quality-control
author: AI-Generated
source_url: https://github.com/biokit/biokit
---

## Concepts

- **Multi-format I/O**: biokit reads and writes common bioinformatics formats including FASTA, FASTQ, SAM, BAM, VCF, and BED. It automatically detects format from file extensions and uses stdin/stdout for pipeline integration.
- **Streaming architecture**: Most biokit commands process data line-by-line without loading entire files into memory, enabling efficient handling of large genomic files (multi-GB FASTQ files, whole-genome BAMs) on modest hardware.
- **Tabular data model**: Sequence and alignment data are represented as structured tables with named columns. Filtering, sorting, and aggregation operations use column-based expressions rather than regex matching on raw lines.
- **Modifier chaining**: Transformations can be chained using pipe operators (`--input-format | --output-format`), allowing complex pipelines like `biokit filter --quality 20 | biokit trim --length 50 | biokit convert --to-fasta`.

## Pitfalls

- **Format auto-detection fails on non-standard extensions**: If your file lacks a recognized extension (.fa, .fq, .sam, .vcf), biokit defaults to FASTA parsing, corrupting FASTQ or binary BAM data silently. Always specify `--input-format` explicitly for non-standard filenames.
- **Quality score encoding mismatch**: biokit assumes Sanger/Illumina 1.8+ quality encoding by default. Reading Solexa/Illumina 1.0- encoded files (Phred+64) without `--quality-encoding solexa` produces incorrect filtering results and downstream errors.
- **Chromosome naming inconsistencies**: Reference sequences with `chr1` vs `1` naming are treated as different keys. Mixing references from different sources (e.g., UCSC vs Ensembl) causes join/intersect operations to silently drop records.
- **Memory limits with unsorted inputs**: Operations like duplicate marking and variant calling require sorted inputs. Passing unsorted BAM or VCF files without `--sorted` flag triggers errors mid-process, wasting computation on already-processed records.

## Examples

### Filter FASTQ reads by minimum quality score
**Args:** `fastq filter --input reads.fastq --min-quality 30 --output high_quality.fastq`
**Explanation:** This keeps only reads where all bases exceed Phred score 30, useful for downstream analysis requiring high-confidence bases.

### Convert between FASTA and FASTQ formats
**Args:** `convert --input sequences.fa --input-format fasta --output sequences.fq --output-format fastq`
**Explanation:** Since FASTA lacks quality scores, biokit assigns the specified default quality or queries the `--default-quality` parameter.

### Extract specific genomic regions from a BAM file
**Args:** `bam extract --input alignment.bam --region chr22:30000000-35000000 --output region.bam`
**Explanation:** This subsamples the BAM to reads overlapping the specified chromosomal interval, reducing file size for targeted analysis.

### Remove duplicate reads from a BAM file
**Args:** `bam dedup --input alignment.bam --output deduped.bam --mark-duplicates`
**Explanation:** The `--mark-duplicates` flag tags duplicates with a flag rather than removing them, preserving information for metrics calculation.

### Intersect two BED files to find overlapping features
**Args:** `bed intersect --input peaks.bed --database genes.bed --output overlaps.bed --fraction 0.5`
**Explanation:** This reports regions where at least 50% of peaks.bed features overlap with genes.bed, suitable for enhancer-promoter interaction analysis.

### Count variants per chromosome from a VCF file
**Args:** `vcf count --input variants.vcf --group-by chromosome --output variant_counts.txt`
**Explanation:** This aggregates variant counts across chromosomes into a simple table, useful for quality control before variant effect prediction.

### Trim adapters from paired-end FASTQ files
**Args:** `fastq trim --input forward.fastq --reverse reverse.fastq --adapter AGATCGGAAGAGC --min-length 50 --output trimmed_forward.fastq --output-reverse trimmed_reverse.fastq`
**Explanation:** This removes adapter sequences and discards reads shorter than 50bp after trimming, preparing data for alignment.