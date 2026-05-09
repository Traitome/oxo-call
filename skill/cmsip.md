---
name: cmsip
category: Genomics / ChIP-seq Analysis
description: Tool for extracting and analyzing reads from ChIP-seq or similar immunoprecipitation sequencing experiments. Accepts indexed BAM/CRAM files and BED files to extract reads or coverage in specified genomic regions, generating outputs in various formats including BAM, BEDGraph, and Wiggle.
tags:
  - chip-seq
  - genomics
  - read-extraction
  - coverage
  - peak-analysis
  - bam
  - bed
author: AI-generated
source_url: https://github.com/placeholder/cmsip
---

## Concepts

- **Input formats**: `cmsip` works with aligned sequencing data in BAM or CRAM format (must be indexed with samtools index), along with genomic region definitions in BED format. The tool can also accept Wiggle or BEDGraph files for certain operations.
- **Output generation**: Depending on flags, `cmsip` outputs extracted reads as BAM, normalized coverage as BEDGraph or Wiggle, or summary statistics in tabular format. Output format is determined by the `--outfmt` flag.
- **Region matching behavior**: By default, `cmsip` extracts reads that overlap the specified regions by at least one base pair. Use `--overlap` to require minimum overlap length, or `--exact` for exact interval matching only.
- **Strand specificity**: The tool can extract reads from particular strands using `--strand` (values: `+`, `-`, or `.` for both). This is critical for motif analysis or directional enrichment studies.

## Pitfalls

- **Unindexed BAM files**: Running `cmsip` on BAM files that haven't been indexed will fail with a cryptic error. Always index BAM files with `samtools index` before using `cmsip`.
- **Mismatched chromosome names**: If your BED file uses chromosome names like "chr1" but your BAM file uses "1", no reads will be extracted. Verify chromosome nomenclature consistency between files.
- **Memory exhaustion with large regions**: Extracting very large genomic regions (whole chromosomes) without specifying `--chunk` can consume excessive memory. Use `--chunk` to process in manageable pieces for large requests.
- **Incorrect strand flags**: Using `--strand` without understanding read orientation conventions can silently return empty results. Remember that RNA-seq data often has opposite strand orientation compared to DNA-based ChIP-seq.

## Examples

### Extract reads from a specific gene promoter region

**Args:** `--bed input/genes.bed --bam align/sample.bam --out output/promoter_reads.bam --region-type promoter`
**Explanation:** This extracts all reads overlapping promoter regions defined in the BED file from the aligned BAM, output as BAM format for downstream analysis.

### Calculate coverage in BEDGraph format across all input intervals

**Args:** `--bed input/peaks.bed --bam align/chip.bam --out output/peaks Coverage.bdg --outfmt bedgraph --normalize`
**Explanation:** Generates normalized coverage values in BEDGraph format for each peak interval, suitable for visualization in genome browsers like IGV or UCSC.

### Extract reads from multiple chromosomes in parallel chunks

**Args:** `--bed input/regions.bed --bam align/experiment.bam --out output/extracted --split-by chr --chunk 10000000`
**Explanation:** Processes large regions in 10 Mb chunks to manage memory usage, splitting output files by chromosome for efficient parallel downstream processing.

### Filter for reads on the positive strand only

**Args:** `--bed analysis/tf_sites.bed --bam align/tf_chip.bam --out output/tf_pos.bam --strand +`
**Explanation:** Extracts only reads mapping to the positive strand that overlap the transcription factor binding sites, useful for motif directionality analysis.

### Generate summary statistics for each genomic interval

**Args:** `--bed input/enhancers.bed --bam align/h3k27ac.bam --out output/enhancer_stats.txt --outfmt stats --read-count`
**Explanation:** Produces a table with read counts, coverage depth, and basic statistics for each enhancer interval, enabling quick enrichment comparison across regulatory elements.

### Extract sequences in FASTA format for motif finding

**Args:** `--bed input/dmr_positions.bed --bam align/methyl.bam --out output/dmr_seqs.fa --outfmt fasta --flank 50`
**Explanation:** Extracts DNA sequences from DMR positions plus 50bp flanking regions on each side in FASTA format, ready for de novo motif discovery with tools like MEME.

### Require exact interval overlap for precise analysis

**Args:** `--bed input/cpg_islands.bed --bam align/bisulfite.bam --out output/cpg_exact.bam --exact`
**Explanation:** Only extracts reads that fall entirely within the specified CpG island boundaries, preventing boundary artifacts in methylation proportion calculations.