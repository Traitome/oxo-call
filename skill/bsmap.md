---
name: bsmap
category: sequence-alignment/bisulfite-sequencing
description: A bisulfite sequencing read mapper that aligns methylated, bisulfite-converted reads to a reference genome using wildcard alignment with support for both directional and non-directional BS-Seq libraries.
tags: [bisulfite-seq, alignment, BS-Seq, DNA-methylation, SAM,WGBS]
author: AI-generated
source_url: https://code.google.com/p/bsmap/

## Concepts

- bsmap uses a wildcard alignment strategy where converted Ts in bisulfite reads are treated as ambiguous positions that can match either C or T in the reference genome, enabling accurate mapping of methylated cytosines that have been chemically converted to uracil.
- The tool requires a pre-built reference index created with `bsmap-build` (producing .2bit binary files), accepts FASTQ or FASTA input with optional quality scores, and outputs standard SAM format alignments with mapping quality scores in column 5.
- bsmap supports both directional (default) and non-directional bisulfite sequencing modes, configurable seed lengths, user-defined mismatch tolerances, and parallel execution via the `-p` flag for multi-threaded processing.

## Pitfalls

- Specifying mismatch limits that are too restrictive causes reads with excessive C→T conversions to fail mapping, leading to systematic loss of highly methylated regions and biased methylation estimates in downstream analyses.
- Attempting to run `bsmap` without first creating an index with `bsmap-build` results in an immediate error requiring re-indexing, which wastes time on large reference genomes.
- Using the default directional mode for non-directional bisulfite libraries (lacking `-S` flag) misaligns complementary strand reads, producing inverted methylation calls especially in promoter CpG islands.
- Failing to specify sufficient threads with `-p` dramatically slows processing for whole-genome bisulfite sequencing datasets with billions of reads.

## Examples

### Building a reference genome index
**Args:** bsmap-build -o hs37d5.2bit -a human_ref.fa
**Explanation:** Creates the required 2-bit binary index file that bsmap reads during alignment, which must be regenerated if the reference sequence changes.

### Mapping single-end reads with default settings
**Args:** bsmap -a hs37d5.2bit -d WGBS_reads.fastq -o mappings.sam
**Explanation:** The most basic workflow that aligns all reads to the indexed reference genome and outputs SAM format results with mapping quality scores.

### Mapping with increased mismatch tolerance for BS-Seq
**Args:** bsmap -a hs37d5.2bit -d WGBS_reads.fastq -o mappings.sam -m 6 -n 40
**Explanation:** Allows up to 6 mismatches and 40bp seed length, accommodating the high C→T conversion rate inherent to bisulfite-converted libraries.

### Mapping paired-end reads with parallel processing
**Args:** bsmap -a hs37d5.2bit -d left.fastq right.fastq -o paired.sam -p 16
**Explanation:** Uses 16 threads for parallel alignment of paired-end reads, significantly accelerating throughput for large whole-genome bisulfite datasets.

### Mapping non-directional bisulfite sequencing data
**Args:** bsmap -a hs37d5.2bit -d fastq -o output.sam -S -v 2
**Explanation:** Enables non-directional mode (`-S`) for libraries prepared without strand preservation, ensuring correct assignment of reads to both Watson and Crick strands.

### Mapping with custom output and filtering
**Args:** bsmap -a hs37d5.2bit -d reads.fastq -o filtered.sam -q 20 -r
**Explanation:** Outputs only uniquely mapped reads with mapping quality ≥20 (`-q 20`) and reports best mapping location when multiple hits exist (`-r`).