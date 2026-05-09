---
name: cage
category: bioinformatics/transcription-analysis
description: A computational tool for CAGE (Cap Analysis of Gene Expression) data processing, enabling high-throughput mapping of transcription start sites (TSSs) and quantification of promoter usage from short-read sequencing data.
tags:
  - RNA-Seq
  - Transcription
  - TSS-mapping
  - CAGE
  - promoter-analysis
  - cap-trapping
author: AI-generated
source_url: https://github.com/
---

## Concepts

- **CAGE data input**: cage accepts raw sequencing reads in FASTQ or BAM format. Raw CAGE reads contain a biotinylated cap structure that is captured during library preparation, making the first read position directly correspond to the transcription start site (TSS). The tool distinguishes between plus and minus strand information through the protocol design.
- **Output formats**: Primary outputs include BAM files with aligned reads, BED files for TSS coordinates, and CSV/TSV files with expression counts per promoter cluster. Expression values are reported as Tags Per Million (TPM) mapped reads, enabling cross-sample comparison.
- **Clustering model**: cage groups nearby CAGE tags into promoter clusters using iterative clustering with a dynamic threshold based on local tag density. The resulting clusters represent distinct promoter regions and are assigned unique cluster identifiers linked to gene annotations when a reference is provided.
- **Strand specificity**: CAGE data is strand-specific by design. cage preserves strand information throughout the pipeline, separating sense and antisense transcription initiation events. Failing to account for strand when combining data sources leads to incorrect TSS assignment.

## Pitfalls

- **Incorrect genome build**: Building a reference with the wrong genome assembly version causes systematic mis-mapping of reads. All downstream TSS coordinates will be positioned incorrectly relative to current gene annotations, rendering expression quantification meaningless without re-mapping.
- **Ignoring paired-end read orientation**: In paired-end CAGE protocols, cage requires correct specification of which read contains the CAGE tag (the 5' end). Using the wrong read as the source of TSS coordinates inverts the apparent promoter architecture and eliminates detectability of divergently transcribed genes.
- **Threshold misconfiguration**: Setting the clustering threshold too low (e.g., below 0.1) creates excessive singleton clusters that fragment legitimate promoters into dozens of non-functional units. Setting it too high merges distinct promoters sharing bidirectional regulatory elements, obscuring core-promoter functional divergence.
- **Memory allocation failure**: Large CAGE datasets with hundreds of millions of tags require sufficient RAM allocation. Under-allocating memory causes cage to crash mid-processing or produce corrupted cluster files that fail validation during downstream merging steps.

## Examples

### Build a reference index for hg38 from a FASTA file
**Args:** `build -r hg38.fa -o hg38_cage_idx --threads 16`
**Explanation:** This creates a indexed reference genome in hg38_cage_idx/ for rapid alignment of CAGE reads. The multi-threaded construction with 16 threads accelerates the process for large reference files.

### Align single-end CAGE reads to the reference index
**Args:** `align -q sample1.fastq.gz -i hg38_cage_idx -o sample1_align.bam --format BAM`
**Explanation:** Aligns compressed FASTQ reads directly to the pre-built reference using default mapping parameters. Output in BAM format enables efficient downstream sorting and filtering while preserving alignment quality scores.

### Process aligned BAM file and identify transcription start sites
**Args:** `cluster -b sample1_align.bam -o sample1_clusters.bed --threshold 0.25 --strand both`
**Explanation:** Clusters aligned reads into promoter clusters using a threshold of 0.25 (tags per million) while preserving strand orientation. The resulting BED file contains TSS cluster coordinates ready for annotation or cross-sample comparison.

### Quantify expression for each promoter cluster
**Args:** `quantify -c sample1_clusters.bed -o sample1_expression.tsv --metric TPM --normalize`
**Explanation:** Calculates Tags Per Million expression values for each cluster identified in the clustering step. Normalization corrects for library depth differences, enabling fair comparison across multiple CAGE samples.

### Merge clusters from multiple biological replicates
**Args:** `merge -i rep1_clusters.bed rep2_clusters.bed rep3_clusters.bed -o merged_clusters.bed --method union --dist 20`
**Explanation:** Combines promoter clusters from three replicates using union-based merging with a 20 base pair proximity threshold. This consolidates consistent TSS signals while preserving strand-specific information across the replicate panel.