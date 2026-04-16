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
- --barcode_threshold sets minimum identity (default 75%) for barcode assignment during demultiplexing.
- --barcode_diff requires difference between best and second-best barcode match (default 5%); prevents ambiguous assignments.
- --require_two_barcodes requires barcode match at both ends; stricter but reduces false binning.
- --adapter_threshold sets minimum adapter identity (default 90%); lower for more sensitive detection.
- --no_split disables chimera splitting; only trim end adapters.

## Pitfalls
- Porechop is deprecated — for modern ONT data use Dorado for demultiplexing and adapter trimming.
- Without -o, Porechop outputs to stdout — always specify -o for file output.
- Porechop barcode demultiplexing (-b) requires ONT native barcodes — it does not handle custom barcodes.
- The trimming may be too aggressive for some very short reads — check output read length distribution.
- --check_reads default (10000) is usually sufficient; increase only if adapter detection seems wrong.
- --barcode_threshold 75 (default) may miss low-quality barcode matches; decrease to 70 for more sensitivity.
- --barcode_diff 5 (default) filters ambiguous assignments; increase to 10 for stricter binning.
- --require_two_barcodes reduces false binning but may miss valid single-end barcoded reads.
- --adapter_threshold 90 (default) is conservative; decrease for detecting divergent adapter sequences.
- -b (barcode binning) and -o (output file) are mutually exclusive; use one or the other.

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

### strict demultiplexing with barcode threshold
**Args:** `-i barcoded.fastq.gz -b demux/ --barcode_threshold 80 --barcode_diff 10 --threads 8`
**Explanation:** --barcode_threshold 80 requires 80% identity; --barcode_diff 10 requires 10% difference from second-best

### require barcode match at both ends
**Args:** `-i barcoded.fastq.gz -b demux_strict/ --require_two_barcodes --threads 8`
**Explanation:** --require_two_barcodes only bins reads with barcode at both start and end; reduces false assignments

### more sensitive adapter detection
**Args:** `-i reads.fastq.gz -o trimmed.fastq.gz --adapter_threshold 85 --threads 8`
**Explanation:** --adapter_threshold 85 lowers identity requirement from 90%; more sensitive but may increase false positives

### trim only end adapters without splitting chimeras
**Args:** `-i reads.fastq.gz -o trimmed.fastq.gz --no_split --threads 8`
**Explanation:** --no_split disables chimera splitting; only removes adapters from read ends

### keep untrimmed reads in separate file
**Args:** `-i reads.fastq.gz -o trimmed.fastq.gz --untrimmed --threads 8`
**Explanation:** --untrimmed outputs untrimmed reads to separate file; useful for QC

### discard unassigned reads during demultiplexing
**Args:** `-i barcoded.fastq.gz -b demux/ --discard_unassigned --threads 8`
**Explanation:** --discard_unassigned removes reads that cannot be assigned to any barcode; cleaner output

### check more reads for adapter detection
**Args:** `-i reads.fastq.gz -o trimmed.fastq.gz --check_reads 50000 --threads 8`
**Explanation:** --check_reads 50000 checks more reads for adapter detection; use if adapter detection seems incomplete
