---
name: ataqv
category: Quality Control
description: A tool for assessing the quality of ATAC-seq (Assay for Transposase-Accessible Chromatin sequencing) experiments by analyzing fragment length distributions, mitochondrial read content, TSS enrichment, and other accessibility-related metrics.
tags:
  - ATAC-seq
  - quality control
  - nucleosome occupancy
  - chromatin accessibility
  - bioinformatics
author: AI-generated
source_url: https://github.com/ncbi/ataqv
---

## Concepts

- **Fragment Length Distribution**: ATAC-seq data exhibits characteristic nucleosome periodicity patterns, where fragments clustering around ~150bp (mononucleosomal), ~300bp (dinucleosomal), and ~450bp (trinucleosomal) indicate successful chromatin accessibility. ataqv analyzes these peaks to assess assay quality.
- **Mitochondrial Read Proportion**: Due to the high copy number of mitochondrial DNA in cells, ATAC-seq experiments oftencapture disproportionate mitochondrial reads. High mitochondrial content (>20-30% of reads) typically indicates poor nuclei isolation or cellular debris contamination, severely reducing usable data.
- **TSS Enrichment**: Active chromatin accessibility localizes near Transcription Start Sites. ataqv computes enrichment scores by comparing read density at annotated TSS regions versus flanking background regions; higher TSS enrichment indicates better signal-to-noise.
- **Input Format Requirements**: ataqv accepts aligned BAM or CRAM files with proper read group annotations. The input must be coordinate-sorted and indexed, with each read group containing sample identity (@RG SM tag) for proper sample disambiguation in multiplexed data.

## Pitfalls

- **Using Unfiltered BAM Files**: Passing raw BAM files without removing duplicate reads causes inflated fragment count estimates and skewed quality metrics, as duplicate fragments represent sequencing bias rather than biological signal. Always mark and remove duplicates beforehand.
- **Omitting Reference Genome Annotation**: Without a TSS annotation file (--tss-file), ataqv cannot compute TSS enrichment scores, one of the most important quality indicators. Ensure the annotation matches your build (e.g., hg38, mm10).
- **Ignoring Mitochondrial Read Metrics**: Failing to monitor mitochondrial read percentage allows poor-quality datasets to pass QC thresholds. Datasets with >30% mitochondrial reads typically require experimental optimization rather than downstream analysis.
- **Incorrect Read Group Handling**: Multiplexed samples lacking proper @RG SM tags are treated as a single sample, preventing per-sample quality assessment and potentially skewing aggregate metrics across the run.

## Examples

### Basic ATAC-seq quality assessment
**Args:** --name my_sample --bam sample.bam --outdir qc_reports/
**Explanation:** This runs ataqv with standard settings to generate a quality report for the ATAC-seq experiment, writing HTML and metric files to the specified output directory.

### Specifying TSS annotation for enrichment calculation
**Args:** --name hesc_atac --bam HESC.bam --tss-file annotations/hg38_tss.gtf.gz --outdir hesc_qc/
**Explanation:** Providing a TSS annotation file enables computation of TSS enrichment scores, the gold-standard metric for ATAC-seq signal quality.

### Reducing mitochondrial read bias with mitochondrial exclusion
**Args:** --name tcell_atac --bam Tcell.bam --omit-mitochondria --outdir tcell_qc/
**Explanation:** The omit-mitochondria flag excludes mitochondrial chromosome reads from all downstream analyses, preventing inflated fragment counts from contamination.

### Using a specific organism and genome build
**Args:** --name mouse_brain --bam mouse_brain.bam --genome mm10 --tss-file annotations/mm10_tss.gtf.gz --outdir mouse_qc/
**Explanation:** Specifying the correct genome build ensures chromosome naming conventions match the annotation file and enables accurate metric computation.

### Analyzing with relaxed fragment length bounds
**Args:** --name weak_signal --bam low_complexity.bam --min-frag-length 20 --max-frag-length 1000 --outdir lowcomplex_qc/
**Explanation:** Adjusting fragment length bounds accommodates low-complexity or degraded samples where nucleosome periodicity may span wider ranges.

### Ignoring duplicate reads without removal
**Args:** --name test_run --bam test.bam --keep-duplicates --outdir test_qc/
**Explanation:** The keep-duplicates flag reports raw metrics without duplicate removal, useful for debugging but should not be used for final quality reports.

### Customizing fragment length periodicity peaks
**Args:** --name custom_atac --bam custom.bam --nucleosome-lengths 140 --nucleosome-lengths 300 --nucleosome-lengths 460 --outdir custom_qc/
**Explanation:** When working with non-standard protocols or organisms with different nucleosome spacing, custom nucleosome-lengths flags override default periodicity assumptions.

### Disabling periodicity scoring for ultra-low input samples
**Args:** --name sc_atac --bam scATAC.bam --no-periodicity --outdir sc_qc/
**Explanation:** Single-cell or ultra-low input ATAC-seq may lack sufficient nucleosome periodicity for reliable scoring; disabling prevents misleading quality indicators.