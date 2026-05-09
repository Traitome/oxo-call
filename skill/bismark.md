---
name: bismark
category: epigenetics
description: A bisulfite sequence mapper and methylation caller for next-generation sequencing data. Bismark aligns bisulfite-treated reads to a reference genome and identifies cytosine methylation in CpG, CHG, and CHH contexts.
tags: bisulfite, methylation, NGS, epigenetics, WGBS, DNA-methylation, bisulfite-sequencing
author: AI-generated
source_url: https://github.com/FelixKrueger/Bismark
---

## Concepts

- **Bisulfite conversion principle:** Unmethylated cytosines are converted to uracil (read as thymine after PCR), while methylated cytosines remain protected. Bismark uses a custom alignment algorithm that allows reads to align to both the original and converted reference to detect this difference.
- **Genome indexing requirement:** Before alignment, the reference genome must be indexed using the companion tool `bismark_genome_preparator` (which wraps bowtie-build or bowtie2-build), creating three bisulfite-converted versions of the genome for optimal alignment.
- **Methylation context detection:** Bismark reports methylation calls in three sequence contexts—CpG (guanine follows cytosine), CHG (any base except G precedes G), and CHH (no G follows)—each with distinct biological implications for gene regulation.
- **Output formats:** Alignment produces BAM/SAM files with methylation preserved in the sequence. The `bismark_methylation_extractor` generates cytosine reports in CGmap, bedGraph, or genome-wide coverage files for downstream analysis.

## Pitfalls

- **Skipping genome preparation:** Running bismark alignment without first creating the bisulfite genome index using `bismark_genome_preparator` causes the aligner to fail or produce incorrect mappings because the three converted reference versions do not exist.
- **Incorrect library strandedness:** Using the wrong `--lib` flag for single-end library direction (e.g., `--lib` set to wrong strand) leads to significant false positive or false negative methylation calls since the alignment algorithm expects specific read orientations.
- **Forgetting deduplication for paired-end data:** Not specifying `--deduplicate` when processing paired-end PCR-amplified libraries causes inflated methylation frequencies at duplicate positions, artificially skewing methylation level calculations.
- **Insufficient memory for large genomes:** Aligning whole-genome bisulfite sequencing data to large genomes (e.g., mammalian) without adequate memory allocation causes the bowtie/bowtie2 component to terminate with memory exhaustion errors.

## Examples

### Align single-end bisulfite-treated reads to a reference genome

**Args:** --genome /path/to/genome_folder --single_end -o output_folder input_reads.fq.gz

**Explanation:** This aligns single-end bisulfite-treated FASTQ reads using bowtie (default) to a pre-indexed reference genome, writing alignment results to the specified output directory.

### Align paired-end bisulfite-treated reads with Bowtie2

**Args:** --genome /path/to/genome_folder --bowtie2 --paired_end -1 read1.fq.gz -2 read2.fq.gz -o output_folder

**Explanation:** This aligns paired-end bisulfite-treated reads using bowtie2 as the underlying aligner, which offers improved alignment sensitivity compared to the original bowtie.

### Extract methylation calls from aligned BAM files

**Args:** --bedGraph --cytosine --genome_folder /path/to/genome_folder --output output_methylation_folder alignment_results.bam

**Explanation:** This extracts methylation information from aligned BAM files, generating a bedGraph file and a genome-wide cytosine report showing methylation percentages at each cytosine position.

### Deduplicate paired-end alignment files before methylation extraction

**Args:** --deduplicate_bam --bam sample_aligned.bam

**Explanation:** This removes PCR duplicates from paired-end aligned BAM files, which is essential for accurate methylation quantification when working with library-prepped samples.

### Generate alignment summary report

**Args:** --bam2nap --dir output_folder

**Explanation:** This generates a comprehensive HTML and text summary report of all bismark alignment results in the specified directory, including mapping efficiency statistics and per-chromosome coverage metrics.