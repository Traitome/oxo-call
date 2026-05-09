---
name: bloocoo
category: Genome Assembly / Error Correction
description: A k-mer based error corrector for Illumina sequencing data that uses coverage statistics to distinguish true k-mers from sequencing errors. Operates in color-space when possible and accepts paired-end or single-end FASTQ input, outputting corrected reads to stdout.
tags:
  - error-correction
  - k-mer
  - illumina
  - read-correction
  - color-space
  - genome-assembly
  - bloocoo-build
author: AI-generated
source_url: https://github.com/gt1/bloocoo
---

## Concepts

- **k-mer coverage filtering via binomial test**: bloocoo determines solid (trustworthy) k-mers by evaluating coverage across reads using a statistical binomial test controlled by the `-c` (coverage) and `-x` (extension) parameters. Reads containing low-coverage k-mers are flagged as potentially erroneous and candidates for correction.
- **FASTQ I/O via stdin/stdout**: The corrector reads raw sequencing data from stdin and writes corrected reads to stdout in FASTQ format. Users must redirect output to a file (e.g., `> corrected.fq`). Pipe-compatible for use in assembly pipelines.
- **bloocoo-build pre-indexing step**: Before correction, `bloocoo-build` must be run to construct a k-mer database (hash table) from the input reads. This companion binary accepts the same `-k` and `-c` parameters to ensure consistency between the index and the correction step.
- **Read pairing inferred from FASTQ headers**: When processing paired-end data, bloocoo detects mate pairs by parsing read name headers in the input FASTQ files. Headers must be consistent (e.g., `/1` and `/2` suffixes or equivalent) for proper pairing; malformed headers cause incorrect pairing.
- **Color-space and base-space modes**: bloocoo supports both color-space and base-space encoded reads. The encoding is auto-detected from the input unless explicitly overridden, affecting how k-mers are hashed and compared during the correction process.

## Pitfalls

- **Inconsistent k-mer size between bloocoo-build and bloocoo**: If the `-k` value passed to `bloocoo-build` differs from the one passed to `bloocoo`, the k-mer database will be incompatible with the correction algorithm, producing silently incorrect or empty output. Always use identical `-k` values for both steps.
- **Writing output without redirection**: Since corrected reads are written to stdout, forgetting to redirect (`>` or `>>`) causes the output to be lost or intermingled with terminal messages. There is no native output file flag.
- **Under-powered hardware causing OOM kills**: Memory consumption scales with input size and k-mer count. Using a k-mer size that is too large with deep coverage data (e.g., high-coverage WGS) without sufficient RAM triggers out-of-memory termination. Monitor memory usage with `-t` threads and adjust input chunk size.
- **FASTQ quality score encoding mismatch**: If the input FASTQ uses a different quality score encoding (e.g., Sanger vs. Illumina 1.8+), bloocoo may misinterpret base qualities during the correction ranking phase, leading to suboptimal correction decisions.
- **Unclean read names causing mate pair misalignment**: Read names that lack consistent mate identifiers or contain special characters confuse the pairing logic. This results in reads being treated as single-end when they should be paired, reducing correction accuracy for the second read of each pair.

## Examples

### Correct errors in a single-end FASTQ file using default settings
**Args:** `-k 21 input.fq.gz > corrected.fq.gz`
**Explanation:** Specifying a k-mer size of 21 (a commonly used odd length) and gzipped input while redirecting stdout captures the full error-corrected dataset in a single pipeline step.

### Correct paired-end FASTQ reads with a specific k-mer size
**Args:** `-k 25 -c 4 -x 2 file_R1.fq.gz file_R2.fq.gz > corrected_paired.fq.gz`
**Explanation:** With a larger k-mer size of 25 and a coverage threshold of 4, bloocoo requires at least 4 observations of a k-mer before it is considered solid, providing stricter correction.

### Build the k-mer index first for later correction
**Args:** `-k 21 -c 3 -t 8 reads_R1.fq.gz reads_R2.fq.gz`
**Explanation:** Running bloocoo-build with 8 threads creates the k-mer hash table from the paired reads using coverage cutoff 3, preparing the database for the subsequent bloocoo correction pass.

### Correct a large dataset with increased thread count to speed up processing
**Args:** `-k 23 -c 5 -t 16 large_R1.fq.gz large_R2.fq.gz > large_corrected.fq.gz`
**Explanation:** Raising thread count to 16 and coverage threshold to 5 enables parallel processing and stricter solid-k-mer determination for a high-coverage whole-genome sequencing dataset.

### Correct reads using bloocoo-build with explicit memory limit warning
**Args:** `-k 19 -c 2 -t 4 -M 32Gb sample.fq.gz`
**Explanation:** With a smaller k-mer size of 19 and lower coverage threshold of 2, bloocoo-build builds a smaller, less memory-intensive index suitable for memory-constrained environments, at the cost of potentially less accurate solid k-mer identification.