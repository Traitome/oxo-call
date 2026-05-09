---
name: bamtocov
category: bioinformatics/coverage-analysis
description: A tool for extracting coverage information from BAM files and generating coverage tracks in various formats including bigWig and bedGraph.
tags:
  - bam
  - coverage
  - genomics
  - visualization
  - bedgraph
  - bigwig
  - read-depth
author: AI-generated
source_url: https://github.com/brentp/bamtocov
---

## Concepts

- **BAM Input Handling**: bamtocov reads sorted and indexed BAM files to calculate per-position coverage. It supports both single-end and paired-end reads, treating each mate independently or as fragments based on configuration. The tool requires a BAM index (.bai) file to efficiently random-access regions.
- **Output Format Flexibility**: The tool produces coverage in multiple standard formats including bigWig (binary, compressed for genome browsers), bedGraph (text-based for UCSC), and raw coverage counts. Format selection depends on downstream use cases, with bigWig being preferred for visualization and bedGraph for text-based inspection.
- **Normalization Options**: bamtocov supports coverage normalization by reads per million mapped (RPM), reads per kilobase per million mapped (RPKM), or raw counts. This flexibility allows cross-sample comparison and accounts for library size differences in differential analysis.
- **Quality and Region Filtering**: Users can filter reads by mapping quality (MQ) threshold to exclude poorly aligned reads, and can restrict analysis to specific genomic regions via BED files or chromosomal coordinates. These filters reduce noise from ambiguous mappings and focus analysis on regions of interest.
- **Paired-End Fragment Mode**: When enabled, paired-end reads are treated as single fragments rather than independent reads, preventing double-counting of template molecules. This mode provides more accurate copy number and expression estimates for paired-end sequencing data.

## Pitfalls

- **Forgetting to Sort and Index BAM Files**: Running bamtocov on unsorted or unindexed BAM files produces errors or incorrect coverage estimates. The tool expects position-sorted BAM files with corresponding .bai indices; attempting to process a name-sorted or unindexed file will fail with an access error.
- **Inconsistent Normalization Across Comparisons**: Applying raw counts to one sample and RPM normalization to another creates non-comparable coverage values. Mixing normalization methods across samples or experiments leads to flawed downstream differential analysis and incorrect biological conclusions.
- **Ignoring Off-Target Coverage in Targeted Assays**: When analyzing targeted sequencing data (e.g., exomes, panels), bamtocov calculates genome-wide coverage including repetitive and off-target regions. Users may misinterpret low coverage as a problem when off-target regions are expected to have minimal reads, wasting analysis time investigating normal behavior.
- **Insufficient Memory for Large BAM Files**: Processing whole-genome BAM files without specifying memory constraints or using streaming mode can cause out-of-memory errors on systems with limited RAM. Users should either analyze by chromosome or use chunked processing for large files on constrained systems.
- **Incorrect Quality Score Interpretation**: Setting mapping quality threshold incorrectly (e.g., too high) may exclude valid alignments from repetitive regions that are inherently difficult to map uniquely. Over-filtering by MQ leads to coverage gaps in biologically relevant regions and underestimation of actual depth.

## Examples

### Generate a bigWig coverage track from a sorted BAM file
**Args:** `input.bam --outfile coverage.bw`
**Explanation:** This exports coverage from a BAM file directly to bigWig format, which is binary compressed and suitable for efficient visualization in genome browsers like UCSC or IGV.

### Create bedGraph output with RPM normalization for cross-sample comparison
**Args:** `sample1.bam --outfile sample1.coverage.bedGraph --normalize rpm`
**Explanation:** Normalizing by reads per million mapped reads allows fair comparison between samples with different sequencing depths, producing comparable coverage values across the genome.

### Calculate coverage for a specific chromosomal region with quality filtering
**Args:** `input.bam --outfile region.coverage.bw --chrom chr7 --start 117000000 --end 117500000 --min-mq 30`
**Explanation:** Restricting analysis to a specific genomic window and filtering by mapping quality ≥ 30 focuses on high-confidence alignments within the target region, reducing noise from ambiguous mappings.

### Generate fragment-based coverage for paired-end data
**Args:** `paired-end.bam --outfile fragments.bw --fragment --normalize rpkg --target-length 1000`
**Explanation:** Using fragment mode treats each read pair as a single molecule rather than counting each end separately, providing accurate fragment-level coverage appropriate for paired-end sequencing analysis.

### Process multiple BAM files to separate output files with parallel execution
**Args:** `bamtocov sample1.bam sample2.bam sample3.bam --outfile {sample}.bw`
**Explanation:** The tool can accept multiple input files and use placeholder syntax to generate corresponding output files, enabling batch processing of multiple samples in a single command invocation.