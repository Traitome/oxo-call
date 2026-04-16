---
name: trim_galore
category: qc
description: Wrapper around Cutadapt and FastQC for quality and adapter trimming with automatic quality control
tags: [trimming, adapter, quality-control, illumina, bisulfite, rrbs, fastqc]
author: oxo-call built-in
source_url: "https://github.com/FelixKrueger/TrimGalore"
---

## Concepts
- Trim Galore wraps Cutadapt for adapter trimming and FastQC for QC; good for routine Illumina and RRBS data.
- Use --paired for paired-end data; input files are positional arguments (both files for PE).
- Trim Galore auto-detects adapter sequences by default; use --adapter to specify manually.
- Use --quality N (default: 20) for quality trimming threshold; --length N for minimum length.
- For RRBS data, use --rrbs flag (trims 2 extra bp to compensate for MspI filling reaction).
- For bisulfite sequencing: use --bisulfite for non-RRBS WGBS data.
- Use --cores N for multi-threading; --gzip for gzipped output.
- Output goes to current directory by default; use -o for custom output directory.
- --clip_R1/--clip_R2 trim fixed bases from 5' end; --three_prime_clip_R1/--three_prime_clip_R2 trim from 3' end.
- --hardtrim5/--hardtrim3 hard-clip reads to leave only N bp from 5'/3' end.
- --polyA trims poly-A tails from reads (useful for RNA-seq).
- --nextseq enables NextSeq-specific quality trimming (two-color chemistry).
- --max_n removes reads with more than N ambiguous bases; use fraction for percentage.

## Pitfalls
- trim_galore has NO subcommands. ARGS starts directly with flags (e.g., --paired, --quality, --length, --cores) or input files. Do NOT put a subcommand like 'trim' or 'run' before flags.
- Trim Galore requires both Cutadapt and FastQC to be installed and in PATH.
- For paired-end, both files must be specified and --paired flag must be present.
- Without --paired, each file in a PE pair is trimmed independently, losing pairing information.
- The --rrbs flag should ONLY be used for MspI-digested RRBS data, not for WGBS.
- Trim Galore's auto-adapter detection reads the first 1M reads — it may miss adapters in low-complexity data.
- Default minimum length is 20 bp — increase with --length for tools with higher minimum requirements.
- --clip_R1/--clip_R2 remove bases from 5' end BEFORE adapter/quality trimming; different from --three_prime_clip.
- --hardtrim5/--hardtrim3 output only N bp from respective end; useful for epigenetic clock analysis.
- --polyA removes poly-A tails but may also trim legitimate A-rich sequences; validate results.
- --nextseq is needed for NextSeq/NovaSeq two-color chemistry (G-only dark cycles).
- --max_n 0.1 removes reads where >10% of bases are N; use 0 for no ambiguous bases allowed.

## Examples

### trim adapters and quality-filter paired-end Illumina reads
**Args:** `--paired --quality 20 --length 36 --cores 4 --gzip -o trimmed_output/ R1.fastq.gz R2.fastq.gz`
**Explanation:** --paired PE mode; --quality 20 Q20 threshold; --length 36 minimum; --gzip compressed output; -o output dir

### trim RRBS bisulfite sequencing data
**Args:** `--paired --rrbs --quality 20 --length 20 --cores 4 --gzip -o rrbs_trimmed/ R1.fastq.gz R2.fastq.gz`
**Explanation:** --rrbs for MspI-digested RRBS; trims 2 bp from 3' end after adapter removal to remove MspI bias

### trim single-end reads with automatic adapter detection
**Args:** `--quality 20 --length 36 --cores 4 --gzip -o se_trimmed/ reads.fastq.gz`
**Explanation:** single-end mode (no --paired); auto-detects Illumina adapters; quality trim at Q20

### trim with specific adapter sequence for non-standard libraries
**Args:** `--paired --adapter AGATCGGAAGAGCACACGTCT --adapter2 AGATCGGAAGAGCGTCGTGTA --quality 20 --cores 4 --gzip -o custom_trimmed/ R1.fastq.gz R2.fastq.gz`
**Explanation:** --adapter and --adapter2 for explicit adapter specification for R1 and R2 respectively

### trim 5' end of reads (e.g., for UMI removal)
**Args:** `--paired --clip_R1 10 --clip_R2 10 --quality 20 --cores 4 --gzip -o clipped_5prime/ R1.fastq.gz R2.fastq.gz`
**Explanation:** --clip_R1/--clip_R2 remove 10 bp from 5' end; useful for UMI or barcode removal

### trim 3' end after adapter removal (e.g., for RRBS)
**Args:** `--paired --three_prime_clip_R1 2 --three_prime_clip_R2 2 --quality 20 --cores 4 --gzip -o clipped_3prime/ R1.fastq.gz R2.fastq.gz`
**Explanation:** --three_prime_clip removes bases from 3' end after quality/adapter trimming

### hard-trim to keep only first 50 bp from 5' end
**Args:** `--hardtrim5 50 --gzip -o hardtrimmed/ reads.fastq.gz`
**Explanation:** --hardtrim5 keeps only first 50 bp from 5' end; useful for epigenetic clock analysis

### hard-trim to keep only last 75 bp from 3' end
**Args:** `--hardtrim3 75 --gzip -o hardtrimmed_3prime/ reads.fastq.gz`
**Explanation:** --hardtrim3 keeps only last 75 bp from 3' end; alternative to --hardtrim5

### trim poly-A tails from RNA-seq reads
**Args:** `--paired --polyA --quality 20 --cores 4 --gzip -o polyA_trimmed/ R1.fastq.gz R2.fastq.gz`
**Explanation:** --polyA removes poly-A tails; useful for RNA-seq data with oligo-dT priming

### trim NextSeq/NovaSeq data (two-color chemistry)
**Args:** `--paired --nextseq 20 --quality 20 --cores 4 --gzip -o nextseq_trimmed/ R1.fastq.gz R2.fastq.gz`
**Explanation:** --nextseq 20 uses NextSeq-specific quality trimming for two-color chemistry

### remove reads with too many ambiguous bases
**Args:** `--paired --max_n 0.1 --quality 20 --cores 4 --gzip -o clean_n/ R1.fastq.gz R2.fastq.gz`
**Explanation:** --max_n 0.1 removes reads where >10% of bases are N (ambiguous)

### trim WGBS (whole genome bisulfite) data
**Args:** `--paired --bisulfite --quality 20 --cores 4 --gzip -o wgbs_trimmed/ R1.fastq.gz R2.fastq.gz`
**Explanation:** --bisulfite for WGBS data (different from --rrbs which is for RRBS only)
