---
name: bcov
category: Sequencing Coverage Analysis
description: A tool for calculating and analyzing sequencing coverage depth from aligned reads, generating per-base or per-window coverage statistics for genomic regions.
tags:
  - coverage
  - depth
  - sequencing
  - alignment
  - genomics
  - bioinformatics
  - read-depth
author: AI-generated
source_url: https://github.com/bxrd/bcov
---

## Concepts

- **Coverage Depth Model**: `bcov` calculates depth by counting reads that map to each genomic position, ignoring gaps and duplicates unless explicitly included. Depth values are reported as integer counts per base pair.
- **Input Formats**: Accepts sorted BAM files as primary input, with optional BED or interval files to restrict analysis to specific genomic regions. SAM input is not supported.
- **Output Modes**: Produces three output formats: per-base WIG for genome-wide tracks, interval summary BED for region-specific statistics, and CSV for downstream statistical analysis.
- **Strand Specificity**: By default, coverage is calculated for both strands combined. Use `--strand +` or `--strand -` to isolate reads from the positive or negative strand separately.
- **Memory Efficiency**: Processes BAM files in streaming mode to avoid loading entire files into memory, making it suitable for large genomes like wheat (>15 GB).

## Pitfalls

- **Forgetting to Sort**: Unsorted BAM input causes incorrect position-based coverage calculation and may produce silent errors where adjacent reads overwrite each other instead of accumulating depth.
- **Missing Index Files**: Running `bcov` on a BAM file without a corresponding .bai index file causes the tool to terminate with a generic "file not found" error instead of prompting for index creation.
- **Ignoring Mapping Quality Thresholds**: Using default MAPQ=0 includes multimapping reads that may inflate coverage estimates by 2-10x in repetitive regions, leading to artificially high apparent depth.
- **Inconsistent Interval Notation**: Specifying genomic intervals as `chr1:1000-2000` instead of tab-separated chromosome name and coordinates causes silent failures where no bases are selected for analysis.
- **Confusing Read Count with Depth**: Coverage reports depth (total reads at each base), not read count (unique reads). A single read spanning 100 bases contributes depth of 1 at each base it covers, not 1 total.

## Examples

### Calculate whole-genome coverage from a BAM file
**Args:** `input reads.bam output coverage.wig`
**Explanation:** This computes per-base coverage across the entire genome and writes a WIG format file for visualization in genome browsers like IGV.

### Restrict coverage to target gene regions only
**Args:** `input reads.bam intervals targets.bed output target_coverage.csv --format csv`
**Explanation:** This limits depth calculation to genomic intervals defined in the BED file, producing a CSV summary with per-region average and maximum depth.

### Exclude low-quality and duplicate reads from analysis
**Args:** `input reads.bam output clean_coverage.wig --mapq 20 --dedup`
**Explanation:** This removes reads with mapping quality below 20 and duplicate reads before calculating coverage, providing an accurate representation of unique read support.

### Calculate strand-specific coverage for RNA-seq data
**Args:** `input rnaseq.bam output antisense.wig --strand - --format wig`
**Explanation:** This isolates coverage from the antisense strand, which is essential for detecting antisense transcription or antisense RNA-seq validation.

### Generate summary statistics for a list of genes
**Args:** `input alignments.bam genes.bed output gene_stats.tsv --format tsv --stats mean,max,min`
**Explanation:** This produces a tab-separated file with mean, maximum, and minimum coverage depth for each gene, suitable for downstream differential expression comparison.

### Process multiple BAM files in batch mode
**Args:** `input *.bam output batch_results/ --format csv --stats mean`
**Explanation:** This applies coverage calculation to all BAM files matching the wildcard pattern, writing individual CSV summaries into the specified output directory.

---