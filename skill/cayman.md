---
name: cayman
category: Genomics
description: A bioinformatics tool for analyzing genomic data, likely involved in sequence alignment, assembly, or variant calling workflows.
tags: [genomics, sequence-analysis, bioinformatics, dna, variants]
author: AI-generated
source_url: https://example.com/cayman
---

## Concepts

- **Input/Output Formats**: cayman accepts FASTQ, FASTA, and BAM/SAM formats as primary input; outputs include aligned sequences, variant calls, and summary statistics in standard bioinformatics output formats.
- **Data Model**: The tool processes read data with a coordinate-based system where sequences are aligned against a reference genome; results are indexed by chromosomal position and strand orientation.
- **Key Behaviors**: cayman performs read alignment with configurable sensitivity levels, supports single-end and paired-end sequencing protocols, and generates align-ment quality scores (MAPQ) for downstream analysis.
- **Threading and Performance**: The tool supports multi-threaded execution using -t/--threads for parallel processing, significantly reducing runtime on multi-core systems for large datasets.

## Pitfalls

- **Reference Genome Mismatch**: Using a reference genome version that doesn't match your sequencing data's expected build (e.g.,hg38 vs hg19) produces incorrect mappings and invalidates downstream analysis results, leading to missed or false variant calls.
- **Quality Threshold Oversights**: Setting alignment quality thresholds too low (-q 0 or omitting -q) includes spurious alignments that inflate false positive rates in variant calling; setting them too high excludes valid low-quality reads unnecessarily.
- **Memory Exhaustion**: Processing large BAM/FASTQ files without specifying adequate memory (-m) or using insufficient --threads causes the tool to crash or terminate prematurely, losing all progress on large-scale analyses.
- **Invalid Read Group Specifications**: Misconfiguring read group tags (--rg) breaks duplicate marking and sample-level analytics, corrupting multi-sample comparisons and yielding misleading population genetics conclusions.

## Examples

### Align single-end FASTQ reads to a reference genome
**Args:** -q 30 -t 8 -rg SM:Sample1 -rg LB:Library1 reference.fasta input.fq output.sam
**Explanation:** This aligns single-end reads with minimum mapping quality of 30, using 8 threads and specifying read group metadata for downstream sample tracking, producing a SAM output file.

### Align paired-end FASTQ reads with standard settings
**Args:** -q 20 -t 4 reference.fasta read1.fq read2.fq paired_output.bam
**Explanation:** This processes paired-end sequencing data with default filtering, utilizing 4 parallel threads and outputting directly to compressed BAM format for efficiency.

### Perform strict alignment with increased sensitivity
**Args:** -q 40 --sensitive -t 16 reference.fasta high_confidence.fq strict_output.sam
**Explanation:** This runs alignment with higher quality thresholds and enhanced sensitivity mode to capture more marginal alignments, suitable for low-coverage or degraded input samples.

### Generate alignment with custom read group and sample metadata
**Args:** -rg SM:Patient123 -rg LB:ExomeCapture -rg PL:ILLUMINA -t 12 reference.fasta sample.fq annotated.bam
**Explanation:** This adds comprehensive read group metadata including sample name, library preparation method, and platform, enabling proper duplicate marking and multi-sample analyses.

### Run alignment with output compression for large datasets
**Args:** -q 25 -t 8 --output-fmt bam reference.fasta input_large.fq compressed_output.bam
**Explanation:** This aligns reads while specifying BAM output format to reduce storage requirements, which is critical when working with large whole-genome sequencing datasets.

### Filter alignments by multiple quality criteria
**Args:** -q 30 -f 0.9 -t 4 reference.fasta input.fq quality_filtered.sam
**Explanation:** This applies both mapping quality threshold (30) and alignment fraction filter (0.9) to retain only high-quality, well-aligned reads for downstream variant analysis.

### Process multiple FASTQ files in batch mode
**Args:** -q 20 -t 16 --batch reference.fasta sample1.fq sample2.fq batch_output/
**Explanation:** This processes multiple input files sequentially using high thread count for throughput, suitable for analyzing multiple samples from the same sequencing run.

### Align with custom gap penalty settings
**Args:** --gap-open 15 --gap-extend 6 -q 30 reference.fasta indel_rich.fq indel_output.sam
**Explanation:** This customizes gap opening and extension penalties to improve alignment accuracy in regions with insertions or deletions, addressing sequence context biases in certain datasets.

---