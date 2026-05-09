---
name: atactk
category: Bioinformatics - Chromatin Accessibility Analysis
description: A comprehensive toolkit for ATAC-seq data processing, quality control, peak calling, and chromatin accessibility analysis. Provides utilities for fragment file manipulation, motif enrichment, and differential accessibility detection.
tags:
  - atac-seq
  - chromatin
  - ngs
  - genomics
  - epigenetics
  - peak-calling
  - accessibility
author: AI-generated
source_url: https://github.com/atactk/atactk
---

## Concepts

- **Fragment file manipulation**: The toolkit operates on BEDPE format fragment files from ATAC-seq alignments, where each record represents a transposition event with chromosome, start, end, and read name fields. Fragment lengths determine nucleosome occupancy patterns.

- **Multi-modal quality control**: Provides quality metrics including fragment length distribution, PCR duplication rates, mitochondrial read fraction, and accessibility peak enrichment scores. These metrics follow ENCODE ATAC-seq standards.

- **Peak calling and annotation**: Implements chromatin accessibility peak identification using shift-based models accounting for the characteristic 5-bp periodicity of ATAC-seq signals. Peaks can be annotated to genomic features and transcription factor binding sites.

- **Strand-specific signal generation**: Generates genome-wide accessibility tracks with strand-specific resolution, enabling analysis of open chromatin regions, transcription factor footprinting, and nucleosome positioning around binding sites.

- **Companion utilities for integration**: Works alongside standard tools like MACS3 for peak calling refinement, and produces outputs compatible with GREAT, ChIPseeker, and other epigenomics annotation platforms.

## Pitfalls

- **Using unfiltered fragment files**: Processing raw BEDPE files without removing duplicates and low-quality reads leads to inflated peak signals and false-positive accessibility sites. Always apply the built-in deduplication filter before peak calling.

- **Ignoring mitochondrial read contamination**: High mitochondrial read percentages (>20%) indicate poor nuclei preparation and significantly reduce effective sequencing depth. The tool provides filtering flags that should be used when mitochondrial fraction exceeds recommended thresholds.

- **Incompatible genome builds**: Attempting to map peaks or annotations to a genome build different from the alignment reference causes coordinate mismatches and incorrect genomic feature assignment. Verify genome build consistency throughout the analysis pipeline.

- **Neglecting blacklist regions**: Failing to filter known artifact-rich genomic regions (ENCODE blacklist) results in false-positive peaks at repetitive elements and copy number variation-prone areas. Include blacklist filtering in standard workflows.

- **Insufficient read depth for specific analyses**: Using default parameters for low-coverage datasets produces noisy signals. For transcription factor footprinting, aim for >30 million mapped reads; for broadpeak detection, >50 million mapped reads provides better resolution.

## Examples

### Generate fragment length distribution plot for QC
**Args:** `qc --input sample.bedpe --output qc_report/`
**Explanation:** Computes fragment length distribution metrics and generates QC plots showing nucleosome patterning, which validates library quality and nucleosome occupancy profiles.

### Filter and deduplicate ATAC-seq fragments
**Args:** `filter --input raw.fragments.bedpe --output clean.fragments.bedpe --min-length 38 --max-length 2000 --remove-duplicates --remove-mitochondria`
**Explanation:** Removes PCR duplicates, filters fragment sizes to biologically relevant ranges, and eliminates mitochondrial reads to produce clean input for downstream analysis.

### Call chromatin accessibility peaks
**Args:** `callpeaks --input clean.fragments.bedpe --genome hg38 --outdir peaks/ --qval 0.01`
**Explanation:** Identifies accessible chromatin regions using shift-based peak calling with significance threshold, generating narrowPeak format files for downstream annotation.

### Generate genome-wide accessibility bigWig track
**Args:** `bigwig --input clean.fragments.bedpe --genome hg38 --outfile accessibility.bigWig --strand-specific --bin-size 10`
**Explanation:** Creates normalized genome accessibility track in bigWig format for visualization in genome browsers, using strand-specific bins for footprinting analysis.

### Annotate peaks to genomic features
**Args:** `annotate --peaks narrowPeaks.bed --genome hg38 --annotation gtf --output annotations.tsv`
**Explanation:** Associates called peaks with genomic features including promoter, intron, exon, and intergenic regions using gene model annotations.

### Calculate transcription factor enrichment
**Args:** `motif-enrichment --peaks peaks.bed --genome hg38 --database JASPAR --output motif_results.txt`
**Explanation:** Tests accessible regions for known transcription factor motif enrichment, identifying candidate regulators of open chromatin.

### Extract nucleosome-free region fragments
**Args:** `extract-nfr --input fragments.bedpe --output nfr.fragments.bedpe --max-length 100`
**Explanation:** Isolates short fragments