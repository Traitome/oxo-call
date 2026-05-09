---
name: capcruncher
category: bioinformatics/chromatin-interaction-analysis
description: A Python toolkit for analyzing Cap-C (Capture Carbon-Copy) chromatin interaction sequencing data. Provides tools for read extraction, alignment, interaction calling, and peak detection from targeted chromatin capture experiments.
tags: [chromatin-interaction, Hi-C, Cap-C, ChIA-Drop, genomic-interactions, 3D-genome, capture-C]
author: AI-generated
source_-url: https://github.com/aboulafia/capcruncher
---

## Concepts

- CapCRuncher processes Cap-C data, which enriches chromatin interactions through targeted capture probes before library preparation. Understanding this probe-based enrichment model is essential — interactions are only called when both anchors are captured by probes, unlike unrestricted Hi-C.
- Input files typically include FASTQ (raw reads), BAM/CRAM (aligned reads), and BEDPE (interactions). Output includes BED files for peaks, TSV/MATRIX formats for interaction matrices, and per-chromosome contact matrices at configurable resolutions.
- The pipeline consists of four core subcommands: `capcruncher-extract` for read preprocessing and barcode handling, `capcruncher-align` for mapping to a reference using a companion index built by `capcruncher-build`, `capcruncher-call` for identifying enriched interaction peaks, and `capcruncher-plot` for generating visualization files.
- CapCRuncher supports fragment-based and bin-based matrix resolution modes. Resolution choice affects downstream peak calling sensitivity — fragment resolution preserves restriction-fragment granularity, while bin resolution aggregates reads into fixed-size genomic windows.
- The tool uses a Python-based restriction enzyme fragment model for Cap-C analysis. You must provide the correct restriction enzyme sequence or select a built-in enzyme preset so that fragment boundaries are computed accurately for interaction calling.

## Pitfalls

- Specifying the wrong restriction enzyme or its recognition sequence in `--enzyme` causes fragment boundaries to be miscomputed, which leads to incorrect interaction calls and spurious peaks. Always verify the enzyme used during library preparation matches the value passed to the tool.
- Running `capcruncher-extract` without setting `--trim-adapters` or `--trim-UMI-length` correctly for your library preparation protocol can cause reads to be mis-trimmed or barcode information to be lost, degrading alignment rates in subsequent steps.
- Attempting to call interactions across pooled samples without a consistent sample sheet (via `--samplesheet`) or with mismatched probe definitions results in silent sample label confusion and incorrect aggregated peak reports.
- Using bin-based resolution that is smaller than the mean fragment length in your data produces matrices with excessive sparsity and artificially high FDR values in peak calls. Choose resolutions that make biological sense for your restriction enzyme.
- Forgetting to run `capcruncher-build` before `capcruncher-align`, or pointing to an outdated genome index, causes reads to map to the wrong genomic coordinates, producing invalid interaction calls with incorrect chromosomal coordinates.

## Examples

### Build a reference genome index for Cap-C alignment
**Args:** `build --genome hg38 --outdir ./ref/hg38_index --enzyme HindIII`
**Explanation:** This constructs a fragment-based genome index using the HindIII restriction enzyme, enabling `capcruncher-align` to correctly anchor reads at fragment boundaries.

### Extract and preprocess FASTQ reads from a Cap-C experiment
**Args:** `extract --fastq input_R1.fastq.gz input_R2.fastq.gz --outdir ./extract --enzyme HindIII --trim-adapters --min-length 30`
**Explanation:** This demultiplexes, trims adapters, and strips UMIs from raw FASTQ files, outputting processed reads ready for alignment with proper fragment assignment.

### Align preprocessed reads to a reference genome using a fragment index
**Args:** `align --fastq ./extract/processed_R1.fastq.gz ./extract/processed_R2.fastq.gz --index ./ref/hg38_index --outfile aligned.bam --enzyme HindIII --threads 16`
**Explanation:** This maps read pairs to the reference while respecting HindIII fragment boundaries, producing a BAM file where each read is tagged with its capture-fragment coordinates.

### Call enriched chromatin interaction peaks from aligned Cap-C data
**Args:** `call --input aligned.bam --outdir ./peaks --enzyme HindIII --resolution fragment --padj-cutoff 0.05 --min-interactions 5`
**Explanation:** This identifies statistically significant chromatin interaction peaks by testing for overrepresented read-pair counts above the background model, filtering by adjusted p-value and minimum interaction support.

### Generate a contact matrix for a specific chromosome at bin resolution
**Args:** `plot --input aligned.bam --chromosome chr12 --resolution 10000 --outdir ./matrices --enzyme HindIII --format matrix`
**Explanation:** This aggregates read pairs from chr12 into 10kb genomic bins and exports a contact matrix file for downstream visualization or comparison with other 3D-genome datasets.

### Pool and call interactions across multiple samples using a sample sheet
**Args:** `call --input sample_sheet.tsv --outdir ./pooled_peaks --enzyme HindIII --resolution fragment --pool-samples --padj-cutoff 0.01`
**Explanation:** This aggregates interaction reads from multiple samples listed in the TSV sample sheet, performs joint peak calling, and reports consensus peaks that are significant across the pooled dataset.

### Extract FASTQ reads with UMI handling for deduplication
**Args:** `extract --fastq input_R1.fastq.gz input_R2.fastq.gz --outdir ./umi_extract --enzyme HindIII --trim-adapters --trim-UMI-length 8 --umi-tag BB`
**Explanation:** This extracts reads while trimming an 8-base UMI from the barcode tag and demultiplexing by sample barcodes embedded in the read, ensuring reads are correctly grouped for deduplication during interaction calling.