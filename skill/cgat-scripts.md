---
name: cgat-scripts
category: Bioinformatics pipelines
description: Collection of Python-based scripts for computational genomics analysis including RNA-seq differential expression, ChIP-seq peak calling, window-based analysis, and NGS quality control
tags:
  - ngs
  - rna-seq
  - chip-seq
  - differential-expression
  - peak-calling
  - genomics
  - python
author: AI-generated
source_url: https://github.com/CGATOxford/cgat
```
## Concepts

- CGAT-scripts is a modular toolkit where subcommands perform specific analyses (e.g., `expression`, `windows`, `peaks`, `variants`, `qc`), each with its own flags and arguments
- Input files are typically SAM/BAM for alignment data, BED for genomic intervals, and CSV/TSV for expression matrices; outputs are written to stdout or specified output directories
- Pipeline execution uses configuration files (YAML) to define workflow parameters, enabling reproducible batch processing across samples
- The toolkit integrates with UCSC genome browsers for visualization and uses genome builds (e.g., hg19, mm10) specified via `--genome` flags
- Expression analysis supports common formats including Cufflinks/Cuffdiff output, HTSeq-count tables, and raw read count matrices for downstream differential expression

## Pitfalls

- Using an incompatible genome build between input alignment files and annotation references causes coordinate mismatches, leading to zero counts or incorrect feature assignment
- Failing to specify proper strand information (`--strand`) for RNA-seq libraries (strand-specific vs. unstranded) produces inverted or null expression values for anti-sense genes
- Omitting required configuration files for pipeline mode causes immediate failures with cryptic YAML parsing errors rather than descriptive messages
- Mixing file formats (e.g., providing FASTQ when BAM is expected) results in silent errors where tools complete without warning but produce empty output files
- Running on insufficient memory when processing whole-genome windows or large BAM files causes OOM termination, producing partial output that appears successful but is truncated

## Examples

### Generate expression count matrix from BAM alignment
**Args:** `expression --input-format=bam --output-file=counts.csv --genome=hg19 alignments/*.bam`
**Explanation:** This runs the expression module to count reads overlapping gene features using the hg19 annotation, outputting a CSV matrix for downstream differential expression.

### Perform window-based analysis on genomic intervals
**Args:** `windows --windows=10kb --genome=mm10 --input-file=peaks.bed --output-file=window_counts.tsv`
**Explanation:** This divides the mm10 genome into 10kb windows and counts reads from the input BED file, useful for covering analysis.

### Run quality control on FASTQ files
**Args:** `qc --output-dir=qc_reports --force --reads=*.fastq.gz`
**Explanation:** This generates quality control reports including per-base sequence quality, duplication rates, and adapter content for all specified FASTQ files.

### Generate HTML report from analysis results
**Args:** `report --template=standard --outputdir=html_report analysis_results/`
**Explanation:** This creates a formatted HTML report from analysis output files using the standard CGAT template for results visualization.

### Peak calling from ChIP-seq alignments
**Args:** `peaks --algorithm=macs --genome=hg38 --treatment=chip.bam --control=input.bam --output=peaks.bed`
**Explanation:** This runs MACS peak calling algorithm on ChIP-seq alignment against input control, producing BED file of significant peaks.

### Generate VariantCall Format (VCF) from variants
**Args:** `variants --input-file=snps.tsv --genome=hg19 --output-file=variants.vcf --filter=DP>10`
**Explanation:** This converts tab-separated variant calls to VCF format while applying minimum depth filter of 10 reads per variant.

### Create genome annotation tracks
**Args:** `annotation --genome=hg38 --annotations=gene,exon,intron --output-dir=annotation/`
**Explanation:** This extracts genome annotations for genes, exons, and introns from the hg38 build into BED format for downstream analysis.

### Compute read coverage normalized by library size
**Args:** `expression normalize --method=RPKM --output-file=normalized.tsv raw_counts.tsv`
**Explanation:** This normalizes read counts using RPKM method (reads per kilobase per million mapped) to account for both gene length and sequencing depth.

### Export analysis results to multiple formats
**Args:** `export --formats=bed,bigwig --genome=mm10 --outputdir=export/ counts.tsv`
**Explanation:** This exports numeric values to both BED and BigWig track formats compatible with genome browsers like UCSC and IGV.

### Run differential expression with statistical testing
**Args:** `expression diff --method=edger --design=condition --output-file=DE_results.csv counts.csv`
**Explanation:** This performs differential expression analysis using edgeR method with condition as the experimental factor, producing statistically ranked results.

### Extract sub-sequences from genome regions
**Args:** `fasta extract --genome=hg19 --regions=target_regions.bed --output-file=sequences.fa`
**Explanation:** This extracts DNA sequences from specified genomic regions in BED format using the hg19 genome build.

### Merge multiple sample count matrices
**Args:** `expression merge --output-file=combined_counts.csv sample1.csv sample2.csv sample3.csv`
**Explanation:** This combines expression count matrices from multiple samples, aligning by gene IDs and handling missing values as zeros.

### Validate input file format and integrity
**Args:** `qc validate --format=bam --strict *.bam`
**Explanation:** This validates BAM files for correct format, required headers, and coordinate sorting before running downstream analyses.

### Create sample sheet from directory
**Args:** `qc samplesheet --output-file=samples.csv --pattern=*.bam --separator=_`
**Explanation:** This generates a sample sheet CSV by parsing filenames using underscore as delimiter, useful for batch pipeline configuration.