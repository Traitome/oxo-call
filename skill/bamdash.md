---
name: bamdash
category: Alignment Visualization / Quality Control
description: Generates an interactive HTML dashboard from BAM alignment files displaying coverage depth, insert size distributions, GC content, flagstat summaries, and other alignment quality metrics for rapid QC assessment.
tags: [bam, html, visualization, qc, dashboard, alignment, coverage, insert-size]
author: AI-generated
source_url: https://github.com/pezmaster31/bamtools
---

## Concepts

- **Binary BAM Input Required**: bamdash accepts only BAM format files (not SAM). The BAM file must be sorted by coordinate and accompanied by a BGZIP-compressed .bai index file in the same directory with the extension `.bam.bai` or `.bai` for the tool to function properly.
- **HTML Dashboard Output**: The tool produces a self-contained HTML file with embedded JavaScript and CSS that renders interactive charts showing multiple alignment metrics including coverage histograms, insert size distributions, GC content per position, mapping quality profiles, and flagstat summaries using the Flot charting library.
- **Reference-Based Metric Calculation**: All metrics are computed relative to the reference genome specified via the `-ref` flag or detected from the BAM header. Coverage depth, GC content, and mapping quality plots require accurate reference sequence information.
- **Chromosome-Specific Filtering**: The `-region` flag restricts analysis to a specific genomic interval (e.g., `chr1:1000000-2000000`), enabling focused QC on particular genomic regions rather than processing the entire BAM file.

## Pitfalls

- **Using SAM Instead of BAM**: Specifying a SAM file as input will cause bamdash to fail with a parsing error since it expects binary BAM format. Always convert SAM to BAM using `samtools view -b` before running bamdash.
- **Missing or Improper Index**: Running bamdash without a corresponding BAI index file results in failure to random-access the BAM file. Ensure the index exists in the same directory as the BAM with proper naming (e.g., `sample.bam.bai`).
- **Insufficient Memory for Large Genomes**: Analyzing entire chromosomes without specifying a region can consume excessive memory for large genomes like wheat or repetitive plants. Use the `-region` flag to limit scope when working with huge genomes or limited computational resources.
- **Mismatched Reference Sequence**: Providing a reference via `-ref` that does not match the @SQ header in the BAM file will produce incorrect GC content and coverage calculations. Always use the exact reference FASTA used for alignment.

## Examples

### Generate a full BAM dashboard for a whole-genome analysis
**Args:** -bam sample.bam -out sample_qc.html
**Explanation:** Creates a comprehensive HTML dashboard containing all quality metrics for the entire BAM file, outputting to sample_qc.html for downstream review.

### Focus dashboard on a specific genomic region
**Args:** -bam sample.bam -region chr1:5000000-10000000 -out region_qc.html
**Explanation:** Restricts analysis to positions 5-10 million on chromosome 1, dramatically reducing runtime and memory usage for large BAM files while providing detailed QC on a specific locus.

### Specify a reference genome for accurate metrics
**Args:** -bam sample.bam -ref hg38.fa -out align_qc.html
**Explanation:** Provides the reference FASTA file to ensure correct GC content calculations and coverage depth normalization, critical when BAM header lacks complete reference information.

### Set minimum mapping quality threshold
**Args:** -bam sample.bam -minMapQ 30 -out filtered_qc.html
**Explanation:** Filters reads with mapping quality below 30 before computing metrics, effectively removing multimapping and low-confidence alignments from the dashboard statistics.

### Adjust coverage calculation window size
**Args:** -bam sample.bam -window 100 -out window_qc.html
**Explanation:** Sets the coverage histogram bin size to 100bp, smoothing the coverage distribution plot and reducing noise from very granular windowing in high-coverage datasets.