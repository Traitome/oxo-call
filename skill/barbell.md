---
name: barbell
category: Sequencing QC / Read Trimming
description: Illumina paired-end and single-end barcode and adapter trimmer with quality-based window trimming. Removes barcode contaminations and low-quality bases from read ends using a sliding window approach.
tags: [trimming, adapter-removal, illumina, quality-filter, paired-end, barcodes]
author: AI-generated
source_url: https://github.com/ksahlin/barbell
---

## Concepts

- **Paired and single-end I/O flow:** Barbell accepts one or two input files (via `-i` / `-i2`) and writes to one or two output files (via `-o` / `-o2`). Input files may be FASTA, FASTQ, gzipped, or BAM. Correct pairing of reads is verified before any operation, so out-of-order or missing mates are flagged.
- **Barcode detection and trimming:** Barcode sequences are provided with `-bc` and auto-detected at the 5′ end of read 1. After alignment of the barcode, the matched prefix is removed from the read. Use `--bc-off` to skip barcode trimming when barcodes are already removed.
- **Window-based quality trimming:** The `-w` option sets the window size (bases scanned from each end inward), and `-Q` sets the minimum average quality threshold for a window to be retained. Bases falling below this threshold are hard-clipped from both ends, and reads shorter than `-l` after trimming are discarded.
- **Adapter sequence specification:** Adapter contaminations are given with `-a` (read 1 adapter) and optionally `-a2` (read 2 adapter for paired-end). The tool aligns the reverse complement of the adapter against the read tail and soft-clips matching bases.
- **Output modes:** Default output is gzipped FASTQ (`-o` / `-o2`). Use `-c 0` for uncompressed, `-c 1` for gzip, or `-c 2` for bzip2. The `-f` flag forces overwrite of existing output files.

## Pitfalls

- **Specifying only one adapter for paired-end reads:** If `-a2` is omitted in paired-end mode, read 2 adapter contamination may go undetected, leaving residual adapter sequence at the tail of read 2 and downstream analyses (e.g., assembly, alignment) can fail silently.
- **Setting `-w` larger than the read length:** Choosing a window size greater than the trimmed read length causes all bases to be dropped, resulting in zero-length output reads that downstream tools may reject or handle as errors.
- **Omitting `-q` in BAM input mode:** When input is BAM, quality scores are required for trimming. If quality strings are absent from the BAM records and `-q` is not set to specify a default phred score, barcode and adapter alignment may degrade, increasing false-positive or false-negative trimming.
- **Mismatching `-i2` without `-o2`:** Providing a second input file without a corresponding second output file causes the trimmed mates to be written to the same output file as read 1, corrupting the read pairing and breaking tools that rely on correct order.
- **Using `--bc-off` with barcode-containing libraries:** Disabling barcode trimming on libraries that genuinely carry barcode prefix sequences leaves these artificial bases in the output, which can cause spurious matches in reference alignment and inflate soft-clipping or mismatches.

## Examples

### Trim adapters and barcode from paired-end Illumina FASTQ files with default settings
**Args:** `-i sample_R1.fastq.gz -i2 sample_R2.fastq.gz -o sample_R1.trimmed.fastq.gz -o2 sample_R2.trimmed.fastq.gz -a AGATCGGAAGAGC -bc GATCGGAAG`
**Explanation:** Barbell reads both mate files, strips the barcode from the 5′ end of read 1, removes the Illumina standard adapter from read ends, and writes gzipped trimmed mates to the specified output files.

### Perform aggressive quality trimming with a small window and discard short reads
**Args:** `-i sample_R1.fastq.gz -i2 sample_R2.fastq.gz -o sample_R1.trimmed.fastq.gz -o2 sample_R2.trimmed.fastq.gz -w 4 -Q 20 -l 50`
**Explanation:** A sliding window of 4 bases is scanned from each end; any window with average quality below phred 20 is hard-clipped, and reads shorter than 50 bases after trimming are discarded from both output files.

### Trim single-end reads and output uncompressed FASTQ
**Args:** `-i sample_SE.fastq.gz -o sample_SE.trimmed.fastq -q 33 -w 5 -Q 25 -a AGATCGGAAGAGC -f`
**Explanation:** Barbell operates on a single input file, sets the quality offset to 33 (standard Illumina), trims low-quality tails with a window of 5, removes adapter sequence, and overwrites any existing output file.

### Process BAM input with default quality offset (33) without barcode trimming
**Args:** `-i sample.bam -o sample.trimmed.bam --bc-off -w 3 -Q 30 -l 40 -f`
**Explanation:** Input is read from a BAM file; since barcode trimming is disabled, only window-quality and adapter trimming are applied using a strict phred 30 threshold, and reads under 40 bases are discarded.

### Trim paired-end reads with custom mismatch tolerance and bzip2 compression
**Args:** `-i sample_R1.fastq.gz -i2 sample_R2.fastq.gz -o sample_R1.trimmed.bz2 -o2 sample_R2.trimmed.bz2 -a AGATCGGAAGAGC -a2 AGATCGGAAGAGC -m 2 -c 2`
**Explanation:** Both adapter sequences are explicitly provided, mismatch tolerance for barcode/adapter alignment is set to 2, output is compressed with bzip2, and barbell proceeds with trimming using the default quality window parameters.