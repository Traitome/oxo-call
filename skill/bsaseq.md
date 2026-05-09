---
name: bsaseq
category: Sequence Analysis
description: A command-line tool for processing and analyzing base-level sequencing data, including quality control, filtering, and consensus sequence generation from aligned reads.
tags: [sequencing, quality-control, bioinformatics, base-calling, ngs]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/bsaseq
---

## Concepts

- bsaseq operates on SAM/BAM alignment files and outputs filtered reads, consensus sequences, or quality reports in standard formats (FASTA, FASTQ, VCF).
- The tool uses a streaming architecture to process reads incrementally, which reduces memory footprint for large input files while maintaining processing order.
- Base quality score recalibration is performed using a built-in statistical model that adjusts Phred scores based on sequence context and machine learning-derived error profiles.
- Output formats are controlled by explicit flags: `-o` for output directory, `-f` for file format (fasta/fastq/vcf), and `-q` for minimum quality threshold.
- The tool supports parallel processing via `-t` for thread count, which scales linearly for independent read groups but shows diminishing returns beyond 8 threads on standard alignment data.

## Pitfalls

- Specifying conflicting output flags (e.g., both `-f vcf` and `-f fasta`) causes the tool to abort silently and produce no output, wasting computational time on large datasets.
- Using `--legacy-QUAL` with newer Illumina data inflates quality scores artificially, leading to reads being retained that should be filtered, downstream of which variant callers produce false positives.
- Omitting `-q` when input files contain mixed quality encodings (Sanger vs. Illumina 1.8+) results in inconsistent filtering, as the tool assumes Sanger encoding by default.
- Setting thread count (`-t`) higher than available CPU cores causes excessive context switching and slower processing, with some systems triggering memory exhaustion on 100GB+ BAM files.
- Forgetting to specify output directory (`-o`) writes results to the current working directory, potentially overwriting existing files without confirmation prompts.

## Examples

### Generate a filtered FASTQ file with minimum quality threshold
**Args:** `input.bam -o filtered_reads/ -f fastq -q 30`
**Explanation:** This extracts all reads with base qualities of 30 or higher and writes them as FASTQ to the specified directory for downstream variant analysis.

### Create a consensus FASTA sequence from aligned reads
**Args:** `alignment.sam -o consensus.fasta --consensus -d 0.5 --min-cov 10`
**Explanation:** This generates a consensus sequence requiring at least 50% agreement at each position and minimum 10x coverage depth across the entire contig.

### Recalibrate base quality scores and save to new BAM
**Args:** `raw_alignments.bam -o recalibrated.bam --recal-qual --known-sites variants.vcf`
**Explanation:** This adjusts Phred scores using the provided variant sites as ground truth, producing a recalibrated BAM suitable for sensitive variant calling.

### Generate quality control report in HTML format
**Args:** `sample1.bam sample2.bam -o qc_report.html --report html --threads 4`
**Explanation:** This produces an interactive HTML quality control report comparing two samples using 4 parallel threads for faster report generation.

### Filter reads by genomic region and output BED coordinates
**Args:** `alignment.bam -o roi_reads.bed -F bed --region chr1:1000000-2000000`
**Explanation:** This extracts only reads mapping to the specified chromosome 1 region and outputs their genomic coordinates in BED format for targeted analysis.

### Stream process large BAM without loading into memory
**Args:** `large_dataset.bam -o filtered_stream.fastq -f fastq -q 25 --stream`
**Explanation:** This enables memory-efficient streaming mode that processes the BAM file in chunks, preventing memory exhaustion on datasets exceeding available RAM.