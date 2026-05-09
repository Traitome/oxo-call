---
name: capc-map
category: Read Mapping / Genomics
description: A bioinformatics tool for mapping sequencing reads to a reference genome, commonly used in cancer genomics pipelines for read alignment and variant analysis.
tags:
- genomics
- read-mapping
- alignment
- cancer-genomics
- sequencing
- bioinformatics
author: AI-generated
source_url: https://github.com/yourrepo/capc-map
---

## Concepts

- **Input format:** capc-map accepts FASTQ or FASTQ.gz files as primary input (single-end or paired-end reads), along with a reference genome index built with the companion tool `capc-map-build`.
- **Output format:** The primary output is a SAM (Sequence Alignment/Map) or BAM (binary SAM) file containing aligned reads with positional information, mapping qualities, and CIGAR strings.
- **Index requirement:** A reference genome must be indexed using `capc-map-build` before running `capc-map`; attempting to map without a pre-built index will cause the tool to fail.
- **Alignment modes:** capc-map supports both end-to-end (full-read) alignment and local (soft-clipping) alignment modes, controlled by the `--local`/`--end-to-end` flags.

## Pitfalls

- **Missing index file:** If the reference index files (*.bwt, *.sa, *.pac) are not in the same directory as the reference FASTA, capc-map will exit with an error about being unable to locate the index.
- **Wrong read orientation for paired-end:** Specifying `--fr` (forward-reverse) when the library preparation used reverse-forward orientation will produce incorrectly paired alignments.
- **Excessive memory usage with large files:** Not specifying the `--bmax` or `--maxram` flags when processing very large FASTQ files can cause the system to run out of memory and crash.
- **Incorrect output format suffix:** Outputting to a file named with `.bam` extension without using `--bam` flag results in a SAM file with the wrong extension, which may confuse downstream tools.

## Examples

### Map single-end reads to a reference genome using default settings
**Args:** `-x hg38 -U reads.fq -S mapped.sam`
**Explanation:** This uses the pre-built index `hg38` to align single-end reads from `reads.fq` and outputs results to `mapped.sam` in SAM format.

### Map paired-end reads with specific fragment size constraints
**Args:** `-x hg38 -1 read1.fq -2 read2.fq -I 200 400 -S paired.sam`
**Explanation:** Maps paired-end reads requiring the insert size (fragment length) to be between 200bp and 400bp, outputting aligned pairs to `paired.sam`.

### Output alignments in compressed BAM format
**Args:** `-x hg38 -U reads.fq --bam -o mapped.bam`
**Explanation:** Produces binary BAM output instead of text SAM format, which is smaller and faster for downstream processing.

### Use local alignment mode to allow soft-clipping at read ends
**Args:** `-x hg38 -U reads.fq --local -S local_aligned.sam`
**Explanation:** Enables local alignment mode which can clip unmatched bases at the ends of reads, useful for reads with adapters or low-quality bases.

### Limit memory usage to 4GB and use 4 threads
**Args:** `-x hg38 -U reads.fq -S mapped.sam --maxram 4G -p 4`
**Explanation:** Restricts capc-map to use no more than 4GB of RAM and parallelizes the alignment across 4 CPU threads for faster processing.