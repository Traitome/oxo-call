---
name: fastp
category: qc
description: Ultra-fast FASTQ preprocessor with quality control, adapter trimming, and filtering
tags: [qc, trimming, adapter, fastq, quality-control, ngs, illumina]
author: oxo-call built-in
source_url: "https://github.com/OpenGene/fastp"
---

## Concepts

- fastp performs QC + adapter trimming in a single pass; it auto-detects Illumina adapters without specification.
- Use -i/-I for paired-end input (R1/R2) and -o/-O for paired-end output; -i/-o for single-end.
- fastp outputs an HTML report (-h) and JSON report (-j) by default; always specify meaningful output filenames.
- Use -w N for multi-threading (default: 16 if available); -q for quality threshold (default: 15); -l for minimum length.
- fastp --dedup enables deduplication for PCR duplicate removal (slower but no need for samtools markdup).
- For UMI-based data, use --umi --umi_loc=read1 (or read2/index1/index2/per_index/per_read).

## Pitfalls

- For paired-end data, use -i R1.fq -I R2.fq and -o clean_R1.fq -O clean_R2.fq (uppercase for R2).
- The default fastp output filenames are output.fastq and output2.fastq — always specify -o and -O explicitly.
- fastp generates fastp.html and fastp.json by default — redirect with -h and -j for better organization.
- For polyA trimming (RNA-seq), add --trim_poly_a.
- --adapter_sequence only works for single-end; for paired-end use --adapter_sequence AND --adapter_sequence_r2.

## Examples

### quality trim and filter paired-end FASTQ reads with 8 threads
**Args:** `-i R1.fastq.gz -I R2.fastq.gz -o clean_R1.fastq.gz -O clean_R2.fastq.gz -h report.html -j report.json -w 8`
**Explanation:** -i/-I for R1/R2 input; -o/-O for R1/R2 output; -h/-j for HTML and JSON reports

### trim adapters from single-end reads and filter reads shorter than 50 bp
**Args:** `-i reads.fastq.gz -o clean_reads.fastq.gz -l 50 -h report.html -j report.json`
**Explanation:** -l 50 discards reads shorter than 50 bp after trimming; auto-detects adapters

### quality trim paired-end reads and set minimum quality to 20
**Args:** `-i R1.fq.gz -I R2.fq.gz -o out_R1.fq.gz -O out_R2.fq.gz -q 20 -l 36 -w 8 -h qc.html -j qc.json`
**Explanation:** -q 20 sets minimum base quality; -l 36 minimum read length after trimming

### run fastp on paired-end RNA-seq data with polyA trimming
**Args:** `-i R1.fq.gz -I R2.fq.gz -o out_R1.fq.gz -O out_R2.fq.gz --trim_poly_a -w 8 -h rna_qc.html -j rna_qc.json`
**Explanation:** --trim_poly_a removes polyA tails common in RNA-seq data

### quality control only (no trimming, just generate the QC report)
**Args:** `-i R1.fq.gz -I R2.fq.gz -o /dev/null -O /dev/null --disable_adapter_trimming --disable_quality_filtering -h qc_report.html -j qc_report.json`
**Explanation:** output to /dev/null and disable filters to get a pure QC report without modifying reads
