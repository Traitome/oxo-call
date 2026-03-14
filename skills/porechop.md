---
name: porechop
category: qc
description: Adapter finder and trimmer for Oxford Nanopore reads with chimera detection
tags: [nanopore, long-read, adapter-trimming, qc, ont, chimera]
author: oxo-call built-in
source_url: "https://github.com/rrwick/Porechop"
---

## Concepts

- Porechop finds and removes adapters from Oxford Nanopore reads; it also detects and splits chimeric reads.
- Use -i for input FASTQ (or directory); -o for output FASTQ (or -b for barcode binning directory).
- Porechop handles gzipped input/output when the file extension is .gz.
- Use --threads N for parallel adapter searching.
- Demultiplexing: use -b barcode_dir to split reads by native barcodes into separate files.
- Porechop is no longer actively maintained — Dorado (ONT's official tool) now handles adapter trimming.
- Use --check_reads N to set how many reads to check for adapter detection (default: 10000).
- The --discard_middle flag removes reads with middle adapters (chimeras) instead of splitting them.

## Pitfalls

- Porechop is deprecated — for modern ONT data use Dorado for demultiplexing and adapter trimming.
- Without -o, Porechop outputs to stdout — always specify -o for file output.
- Porechop barcode demultiplexing (-b) requires ONT native barcodes — it does not handle custom barcodes.
- The trimming may be too aggressive for some very short reads — check output read length distribution.
- --check_reads default (10000) is usually sufficient; increase only if adapter detection seems wrong.

## Examples

### trim adapters from Oxford Nanopore FASTQ reads
**Args:** `-i reads.fastq.gz -o trimmed_reads.fastq.gz --threads 8`
**Explanation:** -i input gzipped FASTQ; -o output trimmed gzipped FASTQ; --threads 8 parallel

### trim adapters and remove chimeric reads
**Args:** `-i reads.fastq.gz -o trimmed_no_chimeras.fastq.gz --discard_middle --threads 8`
**Explanation:** --discard_middle removes reads with internal adapters (chimeras) instead of splitting

### demultiplex barcoded ONT reads into separate files
**Args:** `-i barcoded_reads.fastq.gz -b demultiplexed_reads/ --threads 8`
**Explanation:** -b specifies output directory; creates one FASTQ per barcode bin

### trim adapters and set minimum length output
**Args:** `-i reads.fastq.gz -o trimmed.fastq.gz --min_split_read_size 1000 --threads 8`
**Explanation:** --min_split_read_size 1000 discards split reads shorter than 1000 bp
