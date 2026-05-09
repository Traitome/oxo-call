---
name: Aquila
category: Genomics / Sequence Analysis
description: A bioinformatics tool for processing genomic sequencing data, typically used for read alignment, variant calling, or assembly analysis. Operates on FASTA/FASTQ input files and produces standard output formats (VCF, SAM/BAM, or assembly contigs).
tags: [genomics, variant-calling, assembly, sequence-analysis, fastq, fasta, vcf]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/aquila
---

## Concepts

- **Input formats:** Aquila accepts FASTA (.fa/.fasta) and FASTQ (.fq/.fastq) files as primary input, supporting both single-end and paired-end sequencing data. Some subcommands accept multiple input files for batch processing.
- **Output formats:** Primary outputs include VCF (Variant Call Format) for variant calling results, SAM/BAM for read alignments, and FASTA for assembled sequences. Output format is controlled via command-line flags (e.g., `--output-format vcf`).
- **Reference-based operation:** When performing alignment or variant calling, Aquila requires a reference genome in FASTA format. The reference must be pre-indexed using the companion tool `aquila-build` before analysis.
- **Paired-end read handling:** For paired-end data, Aquila uses insert size statistics to improve alignment accuracy. Proper mate information in FASTQ files is required for correct pairing.
- **Parallel processing:** Aquila supports multi-threaded execution via the `--threads` flag. Memory usage scales with reference size and read depth; sufficient RAM is required for large genomes.

## Pitfalls

- **Missing reference index:** Running Aquila without first building the reference index with `aquila-build` causes immediate runtime errors. Always index the reference genome before alignment or variant calling tasks.
- **Mismatched read format:** Providing FASTQ files with incorrect quality score encoding (e.g., Phred+33 vs Phred+64) leads to parsing errors or silently incorrect quality values, compromising downstream analysis accuracy.
- **Insufficient memory for large datasets:** Processing whole-genome datasets without adequate RAM causes memory allocation failures. Monitor memory usage and consider chunking large datasets or using reduced thread counts.
- **Output file overwrites:** Aquila does not prompt before overwriting existing output files. Specifying an output path that already contains results leads to silent data loss.
- **Incorrect insert size estimates:** Misconfigured or missing insert size parameters for paired-end data causes suboptimal alignment scoring and potential misalignment of mate pairs.

## Examples

### Build a reference genome index
**Args:** `build --reference ref.fasta --threads 8`
**Explanation:** Indexes the reference FASTA file for subsequent alignment operations using 8 CPU threads. Must be completed before any analysis tasks.

### Align reads to a reference genome
**Args:** `align --reads reads.fq --reference ref.fasta --threads 12 --output alignment.sam`
**Explanation:** Aligns single-end FASTQ reads to the indexed reference, producing a SAM output file using 12 parallel threads.

### Call variants from aligned reads
**Args:** `call --input alignment.bam --reference ref.fasta --output variants.vcf --min-qual 30`
**Explanation:** Performs variant calling on aligned BAM reads, outputting variants to VCF format with a minimum Phred quality threshold of 30.

### Process paired-end reads with mate information
**Args:** `align --reads R1.fq R2.fq --reference ref.fasta --paired --output paired.bam --insert-size 250`
**Explanation:** Aligns paired-end read files (R1 and R2) using insert size of 250bp for proper mate pair scoring and validation.

### Generate assembly from long reads
**Args:** `assemble --long-reads nanopore.fq --output assembly.fasta --threads 16`
**Explanation:** Performs de novo assembly of long Oxford Nanopore reads, outputting consensus FASTA contigs using 16 parallel threads.