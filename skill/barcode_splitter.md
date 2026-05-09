---
name: barcode_splitter
category: Sequencing / Demultiplexing
description: Part of the BBMap suite, barcode_splitter demultiplexes FASTA/FASTQ/QUAL/SAM/BAM reads into separate output files based on barcode/adapter detection. It reads barcode sequences from a file or inline, scans each read for the best matching barcode, and writes reads to barcode-specific output files. Supports perfect and imperfect (tolerated mismatches) matching, paired-end splitting, and molecular barcode (UMI) trimming.
tags:
  - demultiplexing
  - barcode-splitter
  - bbtools
  - bbmap
  - fastq
  - fasta
  - sequencing
  - adapter-trimming
  - umi
author: AI-generated
source_url: https://jgi.doe.gov/bbtools/bb-tools-user-guide/barcode-splitter-guide/
---

## Concepts

- **Input and output formats**: barcode_splitter auto-detects input format (FASTA, FASTQ, QUAL, SAM, BAM) and accepts the same types as output. Reads are assigned to output files named by the pattern `{barcode}.fq` or via the `--out` flag. When no barcode matches, reads land in the `unmatched` file.
- **Barcode file structure**: The barcode file (specified with `barcodes=` or `bc=`) is whitespace-delimited: each line holds one barcode name followed by one or more associated sequences. Blank lines and comment lines (`#`) are ignored. Example: `BC1 ATGCATGC\nBC2 GGCCGATC`. This allows multiple sequences per barcode to handle adapter variants.
- **Mismatch tolerance and mode selection**: The `mismatches=` parameter (default `1`) sets how many nucleotide mismatches are tolerated per barcode match. Setting `mismatches=0` enforces perfect-only matching, which is safer for short or low-complexity barcodes. The `mode=` parameter controls whether barcode sequences are expected at the **start**, **end**, or **anywhere** within a read.
- **Paired-end and UMI support**: With `paired=t`, reads are kept paired across output files only if both mates contain the same barcode. The `umi=` flag enables molecular-barcode-aware processing: a UMI regex is stripped from the read before barcode matching, and the UMI is prepended to the read name in the output.
- **Dual-index (combinatorial) demultiplexing**: When `dualindex=t`, barcode sequences on read 1 and read 2 are combined into a single composite key. This allows standard Illumina dual-index workflows where read 1 barcode and read 2 barcode together define the sample, without requiring pre-combined barcode sequences in the barcode file.

## Pitfalls

- **Mismatches set too high for short barcodes**: Using `mismatches=2` with a 6-bp barcode risks cross-assignment where two very different barcodes differ by only 2 mismatches, causing barcode bleed-through between samples. This leads to sample contamination in downstream analysis.
- **Assuming barcode must be at read start**: The default `mode=start` only scans from the 5-prime end. If your barcodes are embedded mid-read (e.g., internal splice sequences) or at the 3-prime end, all reads will be reported as unmatched. Always verify `mode=` matches your library structure.
- **Omitting `--barcodes` or using wrong path**: The barcode file path is mandatory. A missing or empty barcode file causes all reads to be written as unmatched with no error message. Always confirm the file exists and is readable before running on large datasets.
- **Ignoring the `unmatched` output**: When `outu=` is not specified, unmatched reads are silently dropped. If a large fraction of reads appear unmatched, it typically indicates a barcode file mismatch, wrong `mode=`, or read quality issues — not an absence of data loss.
- **Naming collisions with multiple barcodes sharing a prefix**: If barcodes `S1` and `S10` both exist, barcode_splitter may greedily match `S1` prefix-first depending on configuration, routing `S10` reads to `S1`. Use non-overlapping barcode names to avoid aliasing.

## Examples

### Demultiplex a FASTQ file using a 4-bp barcode at the start of each read with 1 allowed mismatch
**Args:** `in=reads.fq barcodes=bc_list.txt mismatches=1 outu=unmatched.fq`
**Explanation:** Reads are scanned from the 5-prime end for a matching barcode (default `mode=start`), and those with ≤1 mismatch are written to per-barcode output files; unmatched reads are saved separately.

### Require perfect barcode matching (zero mismatches) to prevent cross-sample contamination
**Args:** `in=sample.fq.gz barcodes=bc_list.txt mismatches=0 outu=unmatched.fq.gz`
**Explanation:** Only exact-match barcodes assign reads, which is critical for short barcodes where even a single mismatch could cause a read to be misassigned to the wrong sample.

### Scan for barcode sequences anywhere within each read (not just at the start)
**Args:** `in=reads.fq barcodes=bc_list.txt mode=any outu=unmatched.fq`
**Explanation:** Each read is searched from both ends inward; any barcode found anywhere in the read triggers assignment. Use this when barcodes are internal adapters rather than 5-prime anchors.

### Perform dual-index demultiplexing combining read 1 and read 2 barcodes into a composite key
**Args:** `in1=R1.fq in2=R2.fq barcodes=bc_list.txt dualindex=t outu=unmatched.fq`
**Explanation:** The barcode on read 1 and the barcode on read 2 are concatenated to form the lookup key, enabling standard Illumina P7/P5 dual-indexing without needing to pre-merge index sequences in the barcode file.

### Trim UMI (molecular barcode) from reads before barcode matching and prepend UMI to read names
**Args:** `in=reads.fq.gz barcodes=bc_list.txt umi=auto outu=unmatched.fq.gz`
**Explanation:** The UMI regex (`auto` detects common patterns) is stripped from the read before barcode matching, and the extracted UMI is prepended to the read name in the output files, preserving molecular barcode information for downstream deduplication.

### Split SAM input and write per-barcode BAM outputs while preserving read pairing
**Args:** `in=mapped.sam barcodes=bc_list.txt paired=t outu=unmatched.bam`
**Explanation:** SAM records are demultiplexed by barcode; paired reads are kept together in barcode-specific output files only if both mates match the same barcode, maintaining proper mate-pair information in BAM format.

### Generate a summary statistics file alongside output without altering read content
**Args:** `in=reads.fq.gz barcodes=bc_list.txt stats=t outu=unmatched.fq.gz`
**Explanation:** A tab-delimited statistics file is written (to `barcode_splitter_stats.txt` by default) summarizing the count of reads per barcode, making it easy to audit demultiplexing efficiency and detect sample swaps.