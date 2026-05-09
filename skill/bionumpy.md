---
name: bionumpy
category: Bioinformatics Library / Data Processing
description: BioNumPy is a Python bioinformatics library that provides numpy-like array operations for biological data, enabling efficient processing of genomic sequences, variants, and intervals with native support for FASTA, FASTQ, VCF, BED, and BAM formats.
tags:
  - genomics
  - sequences
  - variants
  - intervals
  - array-computing
  - ngs
  - bioinformatics
  - bcftools-comparison
author: AI-generated
source_url: https://github.com/bionumpy/bionumpy
---

## Concepts

- **Array-backed biological data model**: BioNumPy represents sequences, variants, and genomic intervals as array-backed data structures (DnaNumpyArray, VariantArray, IntervalArray) that enable vectorized numpy-style operations without loading entire files into memory, enabling efficient processing of large genomics datasets.

- **Unified data access layer for common bioinformatics formats**: BioNumPy provides a single high-level API to read and write FASTA, FASTQ, VCF, BED, SAM, and BAM files, abstracting format-specific details while preserving metadata like quality scores, sequence names, and genomic coordinates across operations.

- **Lazy evaluation and chunked processing**: Many BioNumPy operations use lazy evaluation with configurable chunk sizes, meaning data is processed incrementally rather than loaded entirely into RAM, which is critical for handling whole-genome VCF files or large BAM files on limited compute resources.

- **Interoperability with numpy and scipy**: BioNumPy data structures inherit from numpy ndarray, allowing direct use of numpy broadcasting, ufuncs, and scipy functions on biological data without explicit type conversion, enabling rapid prototyping of genomic analysis pipelines.

## Pitfalls

- **Mismatched chromosome naming conventions**: BioNumPy requires consistent chromosome naming between input files (e.g., "chr1" vs "1"), and silently produces empty results or alignment failures when one file uses UCSC-style names and another uses Ensembl/NCBI-style names without explicit remapping.

- **Incorrect chunk size leading to memory exhaustion**: Setting chunk_size too small causes excessive I/O overhead with slow performance, while setting it too large causes out-of-memory errors when processing large VCF files; the default chunk_size may be inappropriate for your available RAM, requiring explicit tuning.

- **Unsorted genomic data producing unexpected results**: BioNumPy's IntervalArray operations assume sorted genomic coordinates, and operations on unsorted BED or VCF files produce incorrect overlap counts or truncated results without raising an error, making it essential to pre-sort data with external tools before analysis.

- **Quality score encoding mismatch**: FASTQ files with different quality score encodings (Sanger vs Illumina 1.8+ vs Solexa) are not automatically detected or normalized, causing incorrect filtering of reads based on quality thresholds when the encoding is assumed but not verified.

- **Loss of phase information in VCF conversion**: Converting VCF to other formats or performing operations that reorder variants (like sorting by position) without preserving phased/unphased status can silently discard phase information, compromising downstream haplotyping analysis.

## Examples

### Count total variants in a VCF file
**Args:** `stats input.vcf`
**Explanation:** This counts variants across all chromosomes in a VCF file using chunked reading, providing basic statistics without loading the entire file into memory.

### Filter variants by genomic region
**Args:** `filter input.vcf --chromosome chr1 --start 1000000 --end 2000000`
**Explanation:** This extracts all variants overlapping the specified genomic interval on chromosome 1, leveraging indexed access for efficient region-based queries.

### Convert FASTA to FASTQ with dummy quality scores
**Args:** `convert sequences.fasta sequences.fastq --add-dummy-quality`
**Explanation:** This transforms a FASTA file to FASTQ format by adding placeholder quality scores (typically 'I' characters indicating high confidence), enabling downstream tools that require FASTQ input.

### Calculate per-sample variant counts from multi-sample VCF
**Args:** `stats multi_sample.vcf --per-sample`
**Explanation:** This aggregates variant counts broken down by sample, reporting how many variants each sample contributes, useful for quality control in cohort studies.

### Validate VCF format compliance
**Args:** `validate input.vcf`
**Explanation:** This checks VCF files for format correctness, including header presence, required INFO/FORMAT fields, and data type compliance, reporting warnings for non-fatal issues and errors for critical violations.

### Extract sequences from indexed FASTA by genomic coordinates
**Args:** `extract reference.fasta --regions intervals.bed`
**Explanation:** This retrieves DNA sequences from a reference genome corresponding to genomic intervals defined in a BED file, returning sequences in the same order as the input intervals.

### Convert BAM to BED for downstream interval analysis
**Args:** `convert alignments.bam alignments.bed --output-format bed`
**Explanation:** This transforms read alignments from BAM format to BED format, extracting genomic intervals covered by reads while discarding mapping quality and alignment details.

### Count sequencing depth across genomic windows
**Args:** `stats coverage.bedgraph --window-size 1000 --stat mean`
**Explanation:** This computes mean coverage in 1kb genomic windows from a bedGraph file, enabling quick assessment of coverage uniformity across the genome.