---
name: aster
category: sequence_analysis
description: A bioinformatics tool for high-performance sequence alignment visualization, quality assessment, and statistical analysis of NGS data. Operates on SAM/BAM/CRAM alignment files to generate metrics, reports, and filtered outputs.
tags: [ngs, alignment, sam, bam, quality-control, visualization, statistics]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/aster
---

## Concepts

- **Input Format Support**: aster accepts SAM (text), BAM (binary), and CRAM (compressed) alignment formats. Use `-i/--input` to specify the alignment file; aster auto-detects the format from the file extension (.sam, .bam, .cram).
- **Quality Flag Metrics**: The tool calculates alignment quality metrics including mapping quality (MAPQ), flag statistics, insert size distributions, andread coverage across reference sequences. Results are output as tabular text or JSON when `--output-format json` is specified.
- **Filtering Capabilities**: aster supports complex filtering expressions using flag combinations, MAPQ thresholds, read length ranges, and custom tag values (e.g., `XS:A:+`). Filtered reads can be written to a new output file or used for subsequent metric calculations.
- **Reference Indexing**: When processing BAM/CRAM files, aster requires a corresponding index file (.bai, .crai) in the same directory. For paired-end data, proper pairing flags (0x1) and coordinate sorting are assumed for insert size calculations.

## Pitfalls

- **Missing Index Files**: Running aster on BAM or CRAM files without the corresponding .bai or .crai index file causes immediate failure with a "no index found" error. Always generate index files with `samtools index` prior to analysis.
- **Incorrect Flag Interpretation**: Misinterpreting SAM flags (e.g., treating 0x1 as single-end when it's actually paired-end) leads to incorrect filtering and bogus metrics. Reference the SAM specification: flag 0x1=paired, 0x2=mapped in proper pair, 0x4=unmapped.
- **Memory Exhaustion with Large Files**: For whole-genome BAM files (>50GB), insufficient memory causes crashes. Use the `--chunk-size` parameter to process alignments in genomic intervals rather than loading entire files.
- **Assuming Sorted Order**: Insert size calculations and proper-pair metrics require coordinate-sorted alignments. Running aster on queryname-sorted files produces incorrect insert size distributions without warning.

## Examples

### Generate alignment summary statistics
**Args:** `-i alignments.bam --summary`
**Explanation:** Generates a text report with total reads, mapped/unmapped counts, and flag distribution for the entire BAM file.

### Filter alignments by mapping quality
**Args:** `-i sample.bam -o highqual.bam --min-mapq 30`
**Explanation:** Writes all alignments with MAPQ >= 30 to a new BAM file, useful for downstream variant calling filters.

### Calculate insert size distribution
**Args:** `-i paired_end.bam --insert-size --output-json stats.json`
**Explanation:** Computes mean, median, and standard deviation of insert sizes from properly paired reads, outputting results as JSON.

### Visualize coverage across a specific region
**Args:** `-i experiment.bam --region chr1:1000000-2000000 --coverage --bedgraph output.bdg`
**Explanation:** Generates coverage values in BEDGRAPH format for the specified genomic interval, enabling visualization in genome browsers.

### Exclude duplicate reads from metrics
**Args:** `-i marked.bam --summary --remove-duplicates`
**Explanation:** Calculates summary statistics while ignoring reads marked with the 0x400 duplicate flag, giving accurate downstream analysis metrics.