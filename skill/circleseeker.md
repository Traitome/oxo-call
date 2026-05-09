---
name: circleseeker
category: Bioinformatics / RNA Analysis
description: Detects circular RNAs (circRNAs) from RNA-seq data by identifying back-spliced junction reads using a genomic index built by circleseeker-build.
tags:
  - circRNA
  - circular RNA
  - RNA-seq
  - back-spliced junction
  - splice junction
  - non-canonical splicing
  - genomics
  - transcriptomics
author: AI-generated
source_url: https://github.com/bioinfoCircular/circleseeker
---

## Concepts

- **Back-spliced junction detection**: circleseeker identifiescircular RNAs by finding reads that span non-canonical back-splice junctions, where the 3' splice site of an exon connects to the 5' splice site of an upstream exon (forming a circular structure).
- **Index-based search**: The tool requires a genomic index built with `circleseeker-build`, containing annotated splice site positions. Reads are aligned against this index to identify junction-spanning alignments.
- **Input formats**: Accepts FASTQ, FASTA, SAM, or BAM format files containing RNA-seq reads. Paired-end and single-end reads are both supported.
- **Output format**: Produces a BED or tabular file listing circRNA candidates with genomic coordinates (chromosome, start, end, strand), anchor lengths, read count support, and detection quality scores.

## Pitfalls

- **Using an unindexed or mismatched genome**: Running `circleseeker` without first building an index with `circleseeker-build`, or using an index built from a different genome version, results in no detections or meaningless coordinates.
- **Specifying wrong read orientation for paired-end data**: For paired-end RNA-seq, specifying the wrong library strandedness (`--fr` vs `--rf`) causes junction calls on the wrong strand, generating incorrect circRNA strand annotations.
- **Filtering too aggressively**: Setting high minimum read count or junction quality thresholds when input data has low sequencing depth removes legitimate circRNAs, leading to false negatives.
- **Ignoring anchor length requirements**: Not specifying appropriate minimum anchor lengths (`--min-anchor`) can allow spurious junction detections from lowly abundant reads or alignment artifacts.
- **Mixing sequencing protocols**: Using parameters optimized for polyA-selected RNA-seq on total RNA-seq data (or vice versa) produces inconsistent detection rates due to different background noise profiles.

## Examples

### Detect circular RNAs from a single-end FASTQ file using a pre-built index

**Args:** `--index reference_index --reads reads.fastq --output circRNA_candidates.bed --min-anchor 8 --min-reads 2`

**Explanation:** This runs circleseeker on a single-end FASTQ file, requiring at least 8bp anchor sequences on each side of the junction and at least 2 supporting reads for each circRNA call.

### Detect circular RNAs from paired-end FASTQ files with strand specificity

**Args:** `--index reference_index --mates1 reads1.fastq --mates2 reads2.fastq --output circRNA_candidates.bed --fr --min-anchor 10 --min-reads 3`

**Explanation:** Processes paired-end data assuming forward-reverse strandedness, appropriate for standard dUTP-based library prep, with stricter anchoring and read support requirements.

### Output results in tabular format instead of BED

**Args:** `--index reference_index --reads reads.fastq --output circRNA_results.txt --format tabular --min-reads 1`

**Explanation:** Outputs results in a custom tabular format including detailed alignment information rather than standard BED format.

### Run with relaxed thresholds to maximize detection sensitivity

**Args:** `--index reference_index --reads reads.fastq --output sensitive_calls.bed --min-anchor 4 --min-reads 1 --allow-fuzzy`

**Explanation:** Uses very relaxed parameters to detect circRNAs in low-coverage or degraded RNA-seq data, accepting more false positives for higher sensitivity.

### Run with stringent thresholds for high-confidence calls only

**Args:** `--index reference_index --reads reads.fastq --output strict_calls.bed --min-anchor 15 --min-reads 5 --min-qual 30 --filter-rRNA`

**Explanation:** Applies strict filtering requiring long anchors, multiple supporting reads, high alignment quality, and explicit removal of ribosomal RNA-derived junctions.

### Detect circRNAs and also save junction-spanning reads

**Args:** `--index reference_index --reads reads.fastq --output circRNA_calls.bed --junction-reads junction_reads.sam --min-anchor 8`

**Explanation:** In addition to calling circRNAs, also extracts and writes all junction-spanning read alignments to a separate SAM file for downstream validation.