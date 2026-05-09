---
name: ciriquant
category: RNA-seq Analysis / Circular RNA Quantification
description: A bioinformatic tool for quantification of circular RNAs (circRNAs) from RNA-seq data. Ciriquant identifies circular RNAs by detecting backspliced junction (BSJ) reads and provides expression values including counts and TPMs.
tags:
  - circular RNA
  - circRNA
  - RNA-seq
  - quantification
  - expression
  - bioinformatics
  - non-coding RNA
author: AI-generated
source_url: https://github.com/xr2016/CIRIquant
---

## Concepts

- **Back-spliced Junctions (BSJ)**: Ciriquant identifies circRNAs by detecting reads that span backspliced junctions — the unique junction where the 3' end of a circular RNA connects back to its own 5' end, which is absent in linear transcripts.
- **Input Requirements**: The tool requires (1) a reference genome annotation in GTF/GFF format containing circRNA annotations, (2) alignment files in BAM/SAM format from RNA-seq reads aligned to the genome, and (3) optionally a reference database built with the companion `ciriquant-build` tool for known circRNAs.
- **Output Format**: Results are provided in BED or CSV format with columns including circRNA ID, chromosome, start/end positions, junction reads count, and expression values ( TPM/FPKM).
- **Companion Binary**: Use `ciriquant-build` (the companion tool) to build a reference database from genome annotation files before running quantification.

## Pitfalls

- **Skipping Database Build**: Running `ciriquant` without first building a reference database with `ciriquant-build` will fail because the tool requires a pre-built index of known circRNA positions for efficient quantification.
- **Incorrect Alignment Files**: Using alignment files from DNA-seq or from non-spliced aligners (that don't report split reads) will miss most backspliced junction reads; you must use RNA-seq alignments from splice-aware aligners like STAR or BWA-MEM with appropriate settings.
- **Strand-Specific Library Confusion**: Failing to specify the correct strand-specific library type (`--strand`) leads to incorrectly assigned junctions when reads overlap both sense and antisense transcripts.

## Examples

### Quantify circRNAs from RNA-seq alignments using a reference annotation
**Args:** --input aligned.bam --gtf annotation.gtf --output circRNA_expression --thread 8
**Explanation:** This runs ciriquant on a sorted BAM file using genome annotation to identify and quantify circular RNAs, outputting results to the specified directory with parallel threading.

### Quantify circRNAs with a pre-built reference database
**Args:** --input aligned.bam --ref circRNA_ref.db --output results.txt
**Explanation:** This runs quantification using a pre-built reference database (created by ciriquant-build) for faster execution when analyzing known circRNAs.

### Quantify with strand-specific library type
**Args:** --input aligned.bam --gtf annotation.gtf --strand reverse --output circ_counts
**Explanation:** This specifies that the library was prepared with a reverse strand-specific protocol, ensuring correct assignment of sense versus antisense backspliced junctions.

### Output results in CSV format with TPM values
**Args:** --input aligned.bam --gtf annotation.gtf --output results.csv --format csv --expr TPM
**Explanation:** This produces output in CSV format with TPM (Transcripts Per Million) expression values instead of raw counts.

### Run with specified read length and filter low-confidence junctions
**Args:** --input aligned.bam --gtf annotation.gtf --output filtered_results --min_overlap 20 --min_reads 2
**Explanation:** This filters results to only include circRNAs with at least 20bp overlap with the reference and a minimum of 2 supporting reads, reducing false positives.

### Specify output directory and use multiple threads for parallel processing
**Args:** --input sample1.bam --gtf ref.gtf --output /path/to/output_dir --thread 16
**Explanation:** This processes the sample using 16 threads for faster computation and writes results to a specified output directory.

### Quantify using a FASTA genome file for read alignment verification
**Args:** --input aligned.bam --gtf annotation.gtf --genome genome.fa --output output_results
**Explanation:** This provides a reference genome sequence for verifying backspliced junction sequences against the genome, improving detection accuracy.

### Run in verbose mode to see detailed processing log
**Args:** --input aligned.bam --gtf annotation.gtf --output results --verbose
**Explanation:** This enables verbose logging to display detailed progress messages and debugging information during the quantification process.