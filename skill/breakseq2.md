---
name: breakseq2
category: Structural Variant Detection
description: Detect sequence breakpoints and structural variants from BAM sequencing data using split-read and read-pair analysis, outputting annotated variants in VCF format.
tags:
  - sv-calling
  - breakpoint-detection
  - split-read
  - bam
  - vcf
  - genomic-structural-variants
author: AI-generated
source_url: https://github.com/zstutzman/breakseq2
---

## Concepts

- BreakSeq2 identifies structural variants by analyzing split-read alignments and anomalous read pairs in sorted, indexed BAM files, requiring the BAM to be coordinate-sorted and indexed with BAM index (.bai) present alongside the reference genome prepared via `breakseq2-build`.
- The workflow consists of two phases: `breakseq2` (analysis) which scans reads for breakpoint signatures, and `breakseq2-merge` which consolidates candidate calls across samples to produce a unified VCF with FILTER flags (e.g., LowScore, NoSupport) indicating call confidence.
- Output VCF files encode breakpoints as POS and END coordinates with BREAKSEQ2-specific INFO tags including SU (split reads supporting), RU (reference/uninterrupted sequence), and BU (breakpoint cluster unit size) to annotate variant evidence.
- BreakSeq2 supports gapped alignment modes and can process multiple BAMs simultaneously for cohort-based merging, enabling joint variant discovery across study samples with consistent filtering thresholds.

## Pitfalls

- Providing an unsorted BAM or one lacking a .bai index file causes the tool to fail silently or produce empty VCFs, because BreakSeq2 relies on coordinate-ordering and index access for efficient read traversal.
- Using a mismatched reference genome between the index built by `breakseq2-build` and the BAM alignments results in incorrect breakpoint positioning, as k-mer matching occurs against the built reference and not the raw genome sequence.
- Specifying excessively stringent merge thresholds (e.g., abnormally high SU values) removes true positive breakpoints, leading to undercalled structural variants and false negatives in downstream analyses.
- Forgetting to run `breakseq2-merge` when processing multiple samples produces fragmented candidate lists per-BAM rather than a consolidated callset, complicating joint interpretation of shared events.
- Overwriting output VCFs from separate runs without renaming causes loss of intermediate calls; the merge step reads all candidate VCFs from a specified directory and cannot selectively exclude earlier results.

## Examples

### Build a reference genome index for BreakSeq2
**Args:** build --reference Homo_sapiens_assembly38.fa --kmer-size 15
**Explanation:** This builds a k-mer index from the FASTA reference using a k-mer size of 15, which BreakSeq2 uses to match split-read anchors against the genome during breakpoint detection.

### Detect breakpoints in a single tumor BAM
**Args:** run --bam tumor_sample.bam --reference Homo_sapiens_assembly38.fa --output tumor_variants.vcf --threads 8
**Explanation:** This analyzes the tumor BAM for split-read and read-pair anomalies, spawning 8 threads for parallel processing, and writes raw candidate breakpoints to the specified VCF file.

### Analyze a matched tumor-normal pair
**Args:** run --bam tumor.bam --normal normal.bam --reference GRCh38.fa --output paired_variants.vcf
**Explanation:** By providing both tumor and matched normal BAMs, BreakSeq2 subtracts germline signal present in the normal sample, enriching the output VCF for somatic structural variants.

### Merge candidates across multiple sample VCFs
**Args:** merge --input-dir ./candidates/ --reference GRCh38.fa --output cohort_merged.vcf
**Explanation:** This consolidates breakpoint candidates from all VCF files in the specified directory into a single cohort-level callset, applying cross-sample clustering and统一的 FILTER annotations.

### Run with custom evidence thresholds
**Args:** run --bam sample.bam --reference ref.fa --min-split-reads 3 --min-anchor-length 12 --output filtered_calls.vcf
**Explanation:** Setting --min-split-reads to 3 and --min-anchor-length to 12 relaxes stringency, retaining breakpoints supported by fewer but longer split-read alignments, which improves recall in low-coverage regions.

### Generate a BED file of breakpoint regions
**Args:** run --bam exome.bam --reference ref.fa --bed-output breakpoints.bed
**Explanation:** Using the --bed-output flag instructs BreakSeq2 to also emit a BED-format file of breakpoint intervals alongside the VCF, which is useful for direct visualization in genome browsers like IGV or UCSC.

### Process multiple BAMs in a cohort batch
**Args:** run --bam sample1.bam sample2.bam sample3.bam --reference ref.fa --output-dir ./cohort_candidates/
**Explanation:** Providing multiple comma-separated BAM paths triggers simultaneous analysis of each sample, with individual VCFs written to the output directory for subsequent merge-step consolidation across the cohort.