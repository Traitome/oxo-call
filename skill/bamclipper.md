---
name: bamclipper
category: sequence manipulation
description: Soft-clips adapter sequences and low-quality bases from the ends of reads in SAM/BAM files. Useful for removing primer/adapter contamination from sequencing data.
tags: [bam, sam, clipping, adapters, trimming, preprocessing]
author: AI-generated
source_url: https://github.com/ksahlin/bamclipper
---

## Concepts

- **Input/Output Format**: bamclipper accepts SAM or BAM files as input and outputs to the specified format (SAM or BAM). The tool performs soft-clipping by modifying the CIGAR string and SEQ fields without removing the bases entirely—soft-clipped bases remain in the sequence but are marked as clipped in the CIGAR.
- **Adapter Matching**: The tool scans read ends for exact or near-exact matches to user-provided adapter sequences using the specified minimum match length. Matching can use full adapter sequences or shorter k-mers defined via `--min-length`.
- **Quality Trimming Integration**: When combined with the `-q` flag, bamclipper performs simultaneous quality-based trimming at read ends, removing bases with quality scores below the threshold before applying adapter clipping.
- **Paired-End Support**: For paired-end data, bamclipper can process read pairs together using `--pe` mode, ensuring both reads in a pair are clipped consistently and maintaining proper pairing information in the output.

## Pitfalls

- **Mismatched Adapter Sequence**: Providing the wrong adapter sequence (e.g., using Illumina Universal Adapter instead of Nextera primers) results in no clipping occurring. The tool will process the file but leave adapter contamination in place, leading to false downstream analysis results.
- **Minimum Match Length Too Short**: Setting `--min-length` too low (e.g., 3 or 4 bp) can cause false-positive clipping where random sequence homology is mistaken for adapters, artificially shortening valid reads and reducing mapping quality.
- **Overwriting Input Files**: Specifying the same file for both input (`-i`) and output (`-o`) without using a temporary file will corrupt the input before the output is complete, resulting in data loss.
- **Incorrect File Format Extension**: Providing a `.sam` output filename when expecting BAM output (or vice versa) without specifying `-O` may cause format detection failures or unexpected behavior in downstream tools.

## Examples

### Basic adapter clipping from a BAM file
**Args:** `-i input.bam -o clipped.bam --adapter AGATCGGAAGAGCACACGTCTGAACTCCAGTCAC -l 10`
**Explanation:** Soft-clips the Illumina UniversalAdapter sequence from read ends where at least 10 bp match, writing the clipped output to a new BAM file.

### Using a FASTA file with multiple adapters
**Args:** `-i input.bam -o clipped.bam -f adapters.fa -l 12`
**Explanation:** Clips reads using multiple adapter sequences defined in a FASTA file, requiring a minimum 12 bp match for clipping to occur.

### Simultaneous adapter clipping and quality trimming
**Args:** `-i input.bam -o clipped.bam --adapter AGATCGGAAGAGCACACGTCTGAACTCCAGTCAC -l 10 -q 20`
**Explanation:** First trims bases with quality scores below 20 from read ends, then applies adapter clipping, producing cleaner sequences for downstream analysis.

### Force clipping and output as SAM format
**Args:** `-i input.bam -o clipped.sam --adapter AGATCGGAAGAGCACACGTCTGAACTCCAGTCAC -l 10 --force -O sam`
**Explanation:** Forces clipping to occur even when the full adapter match is not found (partial clipping) and outputs the result in SAM format for easier inspection.

### Processing paired-end data with both reads clipped
**Args:** `-i input.bam -o clipped.bam -f adapters.fa -l 10 --pe`
**Explanation:** Processes paired-end reads in tandem, ensuring both read 1 and read 2 in each pair have adapter sequences clipped consistently, maintaining proper pairing coordinates.

### Minimum adapter match of 15bp with verbose logging
**Args:** `-i input.bam -o clipped.bam --adapter AGATCGGAAGAGCACACGTCTGAACTCCAGTCAC -l 15 -v`
**Explanation:** Uses a stricter 15 bp minimum match length to reduce false-positive clipping and enables verbose logging to track how many reads were clipped during processing.