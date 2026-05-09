---
name: chira
category: ChIP-seq Analysis
description: A bioinformatics tool for integrative ChIP-seq and RNA-seq analysis, including peak calling, motif enrichment, and regulatory element annotation.
tags: [chip-seq, peak-calling, ngs, genomics, motif-analysis, differential-binding]
author: AI-generated
source_url: https://github.com/vfscalfani/chira
---

## Concepts

- **Input Data Format**: chira accepts aligned BAM/SAM files for ChIP-seq and RNA-seq, BED files for pre-called peaks, and FASTA sequences for motif analysis. Use `-i/--input` to specify aligned reads and `-b/--bed` for peak region files.
- **Peak Calling Mode**: The `peaks` subcommand performs peak detection using statistical models (e.g., SICER, MACS2 backends). Configure window size with `-w/--window-size` and gap size with `-g/--gap-size` for fragmented chromatin data.
- **Output Annotations**: Results are written in BED format with score columns, or CSV for gene-centric summaries. Use `-o/--output` to specify the output filename; default streams to stdout.
- **Reference Genome**: Requires a pre-indexed genome via `chira-build`. The index must match your alignment reference. Uses `-d/--database` to specify the genome directory.

## Pitfalls

- **Mismatched Genome Index**: Using a different genome build (e.g., hg19 vs hg38) between alignment and chira causes coordinate mismatches, leading to incorrect peak-to-gene mapping. Always verify the `--database` path matches your BAM alignment source.
- **Insufficient Read Depth**: Tools like chira require sufficient read depth for statistical significance. Low-quality or under-sequenced data produces false-positive peaks with low p-value confidence. Check read counts with `samtools idxstats` before analysis.
- **Duplicate Peak IDs**: When merging multiple BED files, duplicate region coordinates cause false enrichment scores. Use the `--merge` flag with appropriate distance thresholds to coalesce overlapping peaks.
- **Missing配偶配偶配偶配偶**: The `--paired-end` flag must be set for paired-end data; otherwise,Fragment length estimation fails and peak calling accuracy degrades significantly.

## Examples

### Call peaks from ChIP-seq BAM file
**Args:** peaks -i chipseq.bam -o peaks.bed --method macs2
**Explanation:** Uses the MACS2 algorithm to call narrow peaks from aligned ChIP-seq reads, outputting genomic intervals with enrichment scores.

### Annotate peaks with nearest genes
**Args:** annotate -b peaks.bed -g annotation.gtf -o annotated_peaks.bed
**Explanation:** Maps each peak region to the nearest transcribed gene using the provided GTF annotation file, useful for downstream regulatory analysis.

### Find motif enrichment in peak sequences
**Args:** motif -b peaks.bed -d genome_index/ -o motif_enrichment.txt --evalue 0.01
**Explanation:** Extracts sequences from peak regions and searches for statistically enriched motifs compared to background sequences, reporting significance via E-value.

### Perform differential binding analysis
**Args:** diffbind -i treated.bam,control.bam -o differential_peaks.csv
**Explanation:** Compares peak enrichment between treated and control samples using statistical testing to identify condition-specific binding sites.

### Build genome index for motif searching
**Args:** build -g hg38.fa -o hg38_index/
**Explanation:** Creates a pre-indexed FASTA database for fast motif scanning; required before running motif analysis with the `--database` parameter.

### Merge overlapping peak regions from replicates
**Args:** merge -i replicate1.bed,replicate2.bed -o merged_peaks.bed --distance 100
**Explanation:** Combines peaks from biological replicates using a 100bp distance threshold to create a consensus peak set for robust analysis.

### Filter peaks by minimum fold enrichment
**Args:** filter -i peaks.bed -o high_conf_peaks.bed --min-fold 5.0 --pval 0.01
**Explanation:** Retains only peaks exceeding 5-fold enrichment and p-value below 0.01, reducing false positives in downstream validation experiments.