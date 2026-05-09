---
name: bs-seeker2
category: DNA Methylation Analysis / Bisulfite Sequencing Alignment
description: A tool for aligning bisulfite-converted sequencing reads to a reference genome, specifically designed for DNA methylation mapping. BS-Seeker2 uses an aligner-agnostic framework to handle bisulfite-treated DNA and supports both BS-Seq and PBAT protocols.
tags:
  - bisulfite-sequencing
  - methylation
  - dna-alignment
  - bs-seq
  - pbat
  - epigenetic
author: AI-generated
source_url: https://github.com/BSSeeker/BS-Seeker2
---

## Concepts

- BS-Seeker2 operates by converting both reads and reference genome into a unified bisulfite-aware representation before alignment, which allows it to work with any underlying aligner (Bowtie or Bowtie2) while correctly handling cytosine-to-thymine conversions that occur during bisulfite treatment of DNA.
- The tool requires two distinct index types: one for the forward strand alignment (C-to-T converted) and one for the reverse strand alignment (G-to-A converted on the reverse complement), meaning genome indexing must be performed using the companion `bs-seeker2-build` command before any alignment can occur.
- BS-Seeker2 supports three library types controlled by the `--lib-type` parameter: BS-Seq for standard bisulfite treatment, PBAT for post-bisulfite adaptor tagging, and WGBs for whole-genome bisulfite sequencing, each requiring different alignment strategies and reporting different read orientations.
- Output formats include SAM for standard alignment results and a JSON-based `cgmap` format for direct methylation call visualization, with the JSON output containing position-specific methylation percentages and coverage counts for each cytosine analyzed.

## Pitfalls

- Using a standard Bowtie or Bowtie2 index instead of a BS-Seeker2-generated index will cause alignment failures or silently produce incorrect results because standard indices do not account for the bisulfite conversion rules, leading to reads being mismapped or rejected at abnormally high rates.
- Specifying the wrong `--lib-type` (e.g., using BS-Seq when the library was prepared with PBAT protocol) results in reads being aligned to the wrong strand or orientation, fundamentally corrupting downstream methylation analysis by attributing signals to incorrect genomic positions.
- Failing to trim adapters before alignment with BS-Seeker2 degrades alignment quality because the tool does not perform internal adapter trimming; adapter sequences remaining in reads cause soft-clipping or complete alignment failure, artificially lowering mapping rates.
- Using an excessive `--insert-size` value when processing paired-end reads wastes memory and computational resources unnecessarily, while using a value too small causes valid read pairs with naturally larger inserts to be reported as discordant or unaligned.
- Attempting to process gzip-compressed input files without specifying the `--input-file` parameter correctly or without ensuring the file extension is properly recognized leads to the tool attempting to read the compressed file as raw text, producing corrupted or empty output.

## Examples

### Build a bisulfite-aware genome index for human reference

**Args:** `bs-seeker2-build -d /path/to/hg38.fa -p hg38_index`
**Explanation:** The `-d` flag specifies the input reference genome file and `-p` sets the prefix for the output index files, which BS-Seeker2 will use during alignment to reference the bisulfite-converted genome variant.

### Align single-end bisulfite sequencing reads with default BS-Seq settings

**Args:** `bs-seeker2-align -i hg38_index -1 reads_R1.fastq.gz -o aligned_output.sam -g hg38.fa`
**Explanation:** This aligns single-end reads from file `reads_R1.fastq.gz` using the pre-built `hg38_index` with standard BS-Seq library type assumptions, outputting to SAM format with `-o`.

### Align paired-end PBAT library reads with explicit library type specification

**Args:** `bs-seeker2-align -i hg38_index -1 pe_R1.fastq.gz -2 pe_R2.fastq.gz -o paired_output.sam -g hg38.fa --lib-type PBAT`
**Explanation:** The `--lib-type PBAT` flag tells BS-Seeker2 to use post-bisulfite adaptor tagging alignment rules, which aligns only the 3' ends of reads and expects different read orientation patterns than standard BS-Seq.

### Generate JSON-formatted methylation calls with 2nt upstream context

**Args:** `bs-seeker2-align -i hg38_index -1 bs_R1.fastq.gz -o aligned_output.sam -g hg38.fa -O json -- upstream-context 2`
**Explanation:** The `-O json` flag outputs methylation calls in JSON format rather than CGMap, and `--upstream-context 2` includes 2 nucleotides of sequence context upstream of each cytosine for downstream analysis.

### Align reads with non-default maximum edit distance and thread count

**Args:** `bs-seeker2-align -i hg38_index -1 reads.fastq -o aligned.sam -g hg38.fa -m 3 -t 8`
**Explanation:** The `-m 3` flag increases the maximum allowable edit distance to 3 mismatches per read (default is 2), and `-t 8` allocates 8 CPU threads for parallel processing to accelerate alignment on multi-core systems.