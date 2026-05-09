---
name: align_it
category: Sequence Alignment
description: A DNA/RNA sequence alignment tool that maps query reads to a reference genome using a suffix array-based index. Supports SAM output, multi-threading, and paired-end reads.
tags: [alignment, genomics, ngs, read-mapping, sam]
author: AI-Generated
source_url: https://github.com/bioinformatics-tools/align-it
---

## Concepts

- **Index building with `align_it-build`**: The reference genome must be indexed before alignment using the companion binary. Input is a multi-FASTA file; output is a directory containing the suffix array index files (`.sa`, `.bisai`, `.amb`).
- **Alignment data flow**: `align_it` takes a FASTQ query file and an index directory, performs read-by-read alignment against the suffix array, and emits alignments to stdout in SAM format by default.
- **Paired-end read handling**: When two FASTQ files are provided (via `-1` and `-2`), the tool attempts concordant alignment and may apply an inner distance constraint via `-I`/`--inner-dist-mean` and `--inner-dist-std`.
- **Quality-aware scoring**: The default scoring scheme assigns +1 for matches, -1 for mismatches, and -2 for gap opening with -1 for gap extension. These can be tuned via `--match`, `--mismatch`, `--gap-open`, and `--gap-extend`.
- **Output formats**: The tool writes SAM by default; use `-b`/`--bam` to compress directly to BAM, or `-u`/`--unaligned` to emit unaligned reads to a separate FASTQ file.

## Pitfalls

- **Forgetting to build an index**: Running `align_it` without a pre-built index directory causes an immediate "index not found" error. The index must be created separately with `align_it-build` before any alignment.
- **Specifying the wrong library type for paired-end reads**: Passing `-1` and `-2` without `--fr`/`--rf`/`--ff` causes the tool to assume forward-oriented inserts by default, leading to incorrectly filtered concordant pairs and reduced mapping rates.
- **Mismatching inner distance parameters for mate pairs**: If the mean and standard deviation of the insert size do not roughly match the actual library, the tool will filter out valid pairs, inflating the discordant-read count and reducing overall concordance metrics.
- **Omitting `-p`/`--threads` on large BAM outputs**: By default, single-threaded output writing becomes the bottleneck when aligning with multiple threads, causing a performance plateau. Always pair `--threads` with output compression.
- **Using a reference file instead of an index directory in the query position**: Supplying the raw FASTA file as the query target (instead of the index directory path) produces a silent crash with zero output and exit code 1.

## Examples

### Build an index from a FASTA reference file
**Args:** `ref.fa --threads 8`
**Explanation:** The `align_it-build` companion binary reads `ref.fa`, builds a suffix array index in the current directory, and uses 8 parallel threads to accelerate the sorting step.

### Align single-end FASTQ reads to an existing index
**Args:** `-u unaligned.fq index_dir sample_R1.fq.gz`
**Explanation:** The `-u` flag writes any reads that fail to align to `unaligned.fq`, the first positional argument is the index directory, and the gzip-compressed FASTQ is accepted directly without pre-decompression.

### Align paired-end reads with forward-reverse orientation
**Args:** `-1 left.fq -2 right.fq --fr -I 250 --inner-dist-std 50 index_dir`
**Explanation:** The `-1`/`-2` flags specify the mate pair files, `--fr` declares the fragment orientation, and `-I`/`--inner-dist-std` set the expected insert size distribution so concordant pairs are identified correctly.

### Align and write BAM output with 4 threads
**Args:** `-b -p 4 index_dir reads.fq`
**Explanation:** The `-b` flag pipes output through htslib to produce a sorted BAM file, and `-p 4` allocates four threads for both the alignment engine and output compression.

### Align with custom scoring and report all hits
**Args:** `--match 2 --mismatch -3 --gap-open -4 --gap-extend -2 -a index_dir noisy.fq`
**Explanation:** Custom `--match`/`--mismatch`/`--gap-open`/`--gap-extend` override the default scoring matrix, and the `-a` flag enables the align-all mode to report every equally-scoring alignment rather than stopping at the first hit.

### Output only discordant read pairs to a separate file
**Args:** `--discordant-file disc.txt index_dir left.fq right.fq`
**Explanation:** After attempting concordant alignment, reads whose mates do not satisfy the insert size or orientation constraints are written to `disc.txt` in a tab-separated format for downstream structural variant analysis.