---
name: fastp
category: qc
description: Ultra-fast FASTQ preprocessor with quality control, adapter trimming, and filtering
tags: [qc, trimming, adapter, fastq, quality-control, ngs, illumina, paired-end, dedup, umi]
author: oxo-call built-in
source_url: "https://github.com/OpenGene/fastp"
---

## Concepts

- fastp performs QC + adapter trimming in a single pass; it auto-detects Illumina adapters without specification.
- Use -i/-I for paired-end input (R1/R2) and -o/-O for paired-end output; -i/-o for single-end.
- fastp outputs an HTML report (-h) and JSON report (-j) by default; always specify meaningful output filenames.
- Use -w N for multi-threading (default: 3); -q for quality threshold (default: 15); -l for minimum length.
- fastp --dedup enables deduplication for PCR duplicate removal (slower but no need for samtools markdup).
- For UMI-based data, use --umi --umi_loc=read1 (or read2/index1/index2/per_index/per_read).
- --cut_front / --cut_tail / --cut_right perform per-read quality trimming from 5', 3', or both ends using sliding window.
- --merge merges overlapping paired-end reads into single reads; use --merged_out to specify output file.
- --correction corrects mismatched bases in overlapped regions using high-quality base to fix low-quality base.
- --trim_poly_g / --trim_poly_x remove polyG (NovaSeq/NextSeq) or polyX tails (polyA for RNA-seq).
- -z sets gzip compression level (1-9); default 4 balances speed and size.

## Pitfalls

- CRITICAL: fastp has NO subcommands. ARGS starts directly with flags (e.g., -i, -I, -o, -O). Do NOT put a subcommand like 'trim' or 'qc' before flags.
- For paired-end data, use -i R1.fq -I R2.fq and -o clean_R1.fq -O clean_R2.fq (uppercase for R2).
- The default fastp output filenames are output.fastq and output2.fastq — always specify -o and -O explicitly.
- fastp generates fastp.html and fastp.json by default — redirect with -h and -j for better organization.
- For polyA trimming (RNA-seq), add --trim_poly_x.
- --adapter_sequence only works for single-end; for paired-end use --adapter_sequence AND --adapter_sequence_r2.
- --cut_front / --cut_tail interfere with deduplication; dedup should run before quality trimming.
- --dedup requires significant memory; --dup_calc_accuracy controls memory usage (1-6 levels, 1G-24G).
- --merge only works for paired-end data with overlapping regions; non-overlapping pairs remain unmerged.
- Default thread count is 3, not auto-detected; explicitly set -w for faster processing on multi-core systems.

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

### merge overlapping paired-end reads
**Args:** `-i R1.fq.gz -I R2.fq.gz --merge --merged_out merged.fq.gz -o unmerged_R1.fq.gz -O unmerged_R2.fq.gz -w 8`
**Explanation:** --merge combines overlapping PE reads; merged reads go to --merged_out; unmerged reads to -o/-O

### enable base correction in overlapped regions
**Args:** `-i R1.fq.gz -I R2.fq.gz -o out_R1.fq.gz -O out_R2.fq.gz --correction -w 8`
**Explanation:** --correction fixes mismatched bases in PE overlap using high-quality base; improves data accuracy

### trim polyG tails (NovaSeq/NextSeq)
**Args:** `-i R1.fq.gz -I R2.fq.gz -o out_R1.fq.gz -O out_R2.fq.gz --trim_poly_g --poly_g_min_len 10`
**Explanation:** --trim_poly_g removes polyG tails common in NovaSeq/NextSeq; --poly_g_min_len sets detection threshold

### sliding window quality trimming from both ends
**Args:** `-i reads.fq.gz -o clean.fq.gz --cut_front --cut_tail -q 20 -w 8`
**Explanation:** --cut_front/--cut_tail enable sliding window trimming from 5' and 3' ends; -q 20 sets quality threshold

### deduplication with high accuracy
**Args:** `-i R1.fq.gz -I R2.fq.gz -o dedup_R1.fq.gz -O dedup_R2.fq.gz --dedup --dup_calc_accuracy 4 -w 8`
**Explanation:** --dedup removes PCR duplicates; --dup_calc_accuracy 4 uses 8GB RAM for more accurate duplicate detection

### process UMI data
**Args:** `-i R1.fq.gz -I R2.fq.gz -o out_R1.fq.gz -O out_R2.fq.gz --umi --umi_loc=read1 --umi_len 8 -w 8`
**Explanation:** --umi enables UMI processing; --umi_loc=read1 specifies UMI location; --umi_len 8 sets UMI length
