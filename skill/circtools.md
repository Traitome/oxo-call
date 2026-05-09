---
name: circtools
category: circular RNA bioinformatics pipeline
description: A bioinformatics pipeline for circular RNA (circRNA) detection, annotation, and quantification from RNA-seq data. Integrates BWA-MEM alignment, genomic annotation, and differential expression analysis workflows.
tags:
  - circRNA
  - RNA-seq
  - alignment
  - genomics
  - differential expression
  - SAM/BAM
  - annotation
author: AI-generated
source_url: https://github.com/Guo-cc/circtools
---

## Concepts

- circtools operates on pre-detected circRNA candidates expressed as genomic coordinates in BED format, then performs read alignment against a reference genome to validate circular junction spanning reads using BWA-MEM.
- The pipeline requires reference genome files (FASTA), annotation files (GTF/GFF), and read data (FASTQ/FASTA) as primary inputs, producing validated circRNA entries with supporting read counts and junction sequences.
- Multiple subcommands handle distinct stages: circtools-annotate assigns genomic features using reference annotations, circtools-quantify calculates expression values, and circtools-deseq performs differential expression comparisons between sample groups.
- Output formats include BED files for genomic coordinates, tabular text files for expression matrices, and SAM/BAM formats for read alignments at validated junction sites.

## Pitfalls

- Providing incorrectly formatted BED files with non-integer chromosome coordinates causes silent failures or empty output files, resulting in zero validated circRNAs without error messages.
- Using an outdated or incompatible genome annotation version leads to systematic misannotation of circRNA origins, assigning circular transcripts to incorrect gene biotypes or genomic features.
- Specifying mismatched read encoding (Phred+33 vs Phred+64) during FASTQ input causes base quality parsing errors, resulting in low-quality alignments and artificially inflated discard rates.
- Omitting proper paired-end read mate information when processing PE data produces unreliable circular junction detection, as circtools requires both reads to establish valid insert size expectations.
- Insufficient disk space during BAM output generation leads to truncated alignment files that cannot be properly sorted or indexed downstream, corrupting subsequent analysis stages.

## Examples

### Build a BWA-MEM reference index for the human genome

**Args:** circtools-build /path/to/hg38.fa /path/to/hg38_index
**Explanation:** BWA-MEM requires pre-built FM-index files to perform efficient read alignment; without indexing, alignment operations will fail or timeout.

### Annotate circRNA candidates with gene feature information

**Args:** annotate -b /path/to/candidates.bed -a /path/to/annotations.gtf -g /path/to/hg38.fa -o /path/to/output
**Explanation:** The annotate subcommand cross-references genomic coordinates from BED input against GTF annotations to assign gene names, biotypes, and transcript IDs to each circRNA entry.

### Quantify circRNA expression from paired-end RNA-seq reads

**Args:** quantify -1 sample_R1.fastq -2 sample_R2.fastq -c /path/to/candidates.bed -g /path/to/hg38.fa -i /path/to/hg38_index -o /path/to/quant_output -t 16
**Explanation:** Processing paired-end reads requires both mate files specified separately, enabling proper insert size validation and双人 read alignment for circular junction spanning detection.

### Perform differential expression analysis between two conditions

**Args:** deseq -c /path/to/control_counts.txt -t /path/to/treatment_counts.txt -o /path/to/deseq_results.csv
**Explanation:** The deseq subcommand compares expression matrices from control and treatment groups to identify statistically significant circRNA expression changes using negative binomial distribution modeling.

### Filter validated circRNAs by minimum junction-spanning read count

**Args:** filter -i /path/to/annotated_circrnas.bed -m 5 -o /path/to/filtered_circrnas.bed
**Explanation:** Applying a minimum read count threshold removes low-confidence circRNA predictions with insufficient sequencing evidence, improving downstream analytical reliability.