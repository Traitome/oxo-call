---
name: cagecleaner
category: Transcriptomics / CAGE Data Processing
description: A bioinformatics tool for filtering, cleaning, and normalizing CAGE (Cap Analysis of Gene Expression) sequencing data. cagecleaner removes low-quality reads, handles multimapped tags, normalizes signal strength, and prepares CAGE data for downstream transcription start site (TSS) analysis.
tags:
  - CAGE
  - TSS
  - transcriptomics
  - RNA-seq
  - quality-control
  - normalization
author: AI-generated
source_url: https://github.com/bioinformatics-tools/cagecleaner
---

## Concepts

- **Input Format**: cagecleaner accepts aligned CAGE reads in BAM/SAM format or genomic coordinates in BED format. The tool expects standard CAGE protocol output where reads represent the 5' ends of transcripts corresponding to putative transcription start sites. Tags must contain mapping quality scores (MAPQ) in BAM files for quality-based filtering.

- **Filtering Pipeline**: The tool applies sequential filters for mapping quality (MAPQ ≥ threshold), edit distance to reference, read length validation, strand orientation verification, and duplicate removal. Each filter is independently configurable via command-line arguments, allowing precise control over which artifacts are removed from the dataset.

- **Normalization Modes**: cagecleaner supports multiple normalization strategies including TPM (transcripts per million), RPM (reads per million mapped), and raw counts with library size correction. Normalized output can be generated in bedGraph or bigWig format for visualization in genome browsers. The choice of normalization affects cross-sample comparability.

- **Multimapped Read Handling**: Tags that map equally well to multiple genomic locations are processed according to a configurable strategy: exclude all multimapped reads, assign to random location, or allocate proportionally based on prior probability. This decision significantly impacts CAGE signal quantification near repetitive regions.

## Pitfalls

- **Using Unfiltered Data**: Running downstream analyses (e.g., TSS peak calling) without applying cagecleaner's mapping quality filter can introduce false positive TSS calls. Low-MAPQ reads often derive from mispriming events or sequencing errors, artificially inflating signals at spurious genomic locations.

- **Inconsistent Normalization Across Samples**: Applying different normalization modes (e.g., TPM for one sample and RPM for another) within the same analysis invalidates quantitative comparisons. Cage signal intensity becomes incomparable, potentially reversing biological conclusions about differential transcription initiation.

- **Ignoring Strand Information**: CAGE data is inherently strand-specific—the 5' end of a nascent transcript is captured. Failing to verify strand orientation during cleaning can cause reads from antisense transcription to be erroneously combined with sense signals, obscuring promoter directionality which is biologically meaningful.

- **Incorrect Reference Genome Version**: Processing CAGE tags aligned to an older genome assembly version produces coordinate mismatches when compared to annotations from current assemblies. TSS coordinates become inconsistent with gene models, leading to incorrect promoter annotation assignments in downstream analyses.

- **Over-Aggressive Duplicate Removal**: CAGE data naturally exhibits biological duplication where multiple cDNA molecules derive from the same mRNA transcript due to high expression. Aggressive duplicate removal (e.g., treating all duplicates as artifacts) systematically underestimates expression for highly active promoters, distorting expression profiles.

## Examples

### Filter CAGE reads by mapping quality threshold

**Args:** `input.bam --mapq 30 --output filtered.bam`
**Explanation:** This filters out reads with mapping quality below 30, which removes alignment ambiguity and reduces false positive TSS assignments caused by multi-mapping or misaligned reads.

### Generate normalized bedGraph with TPM values

**Args:** `sample1.bam --normalize tpm --out-format bedgraph --output sample1_norm.bedGraph`
**Explanation:** Converting raw counts to TPM normalizes by the total number of uniquely mappable positions in the library, enabling fair comparison with other CAGE samples of similar complexity.

### Remove duplicate reads keeping highest MAPQ

**Args:** `input.bam --remove-duplicates --dedup-strategy best-mapq --output deduped.bam`
**Explanation:** When duplicate reads exist, retaining only the read with highest mapping quality preserves the most reliable alignment evidence while reducing PCR amplification bias artifacts.

### Handle multimapped reads by random assignment

**Args:** `input.bam --multi-method random --seed 42 --output multimap_handled.bam`
**Explanation:** Randomly assigning multimapped reads with a fixed seed ensures reproducibility across analyses while avoiding complete exclusion of these reads which may represent genuine signal from repetitive genomic regions.

### Generate bigWig for genome browser visualization

**Args:** `input.bam --normalize rpm --out-format bigwig --genome mm10 --output signal.bigWig`
**Explanation:** Converting to RPM-normalized bigWig format produces a genome-scale track suitable for UCSC or IGV visualization, where signal height directly reflects transcription initiation strength across the genome.