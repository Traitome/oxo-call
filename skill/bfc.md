---
name: bfc
category: error correction
description: Memory-efficient Bloom filter-based k-mer counting tool for Illumina sequencing error correction using k-mer spectrum analysis.
tags: [error-correction, k-mer, sequencing, bloom-filter, quality-control]
author: AI-generated
source_url: https://github.com/lh3/bfc
---
## Concepts

- bfc performs error correction by building a k-mer frequency Bloom filter from input reads and then correcting low-frequency k-mers that are likely sequencing errors based on the k-mer spectrum.
- The tool operates in two phases: first it counts k-mers using a Bloom filter (or reads a pre-existing hash table with `-s`), then it votes on each base call using high-frequency k-mer matches across a sliding window.
- Input is FASTQ (gzipped or plain) and output is also FASTQ; for SAM/BAM output use `-t` to sort by read name rather than sequence coordinates.
- k-mer size is controlled by `-k` (default 23); larger k-mers increase specificity but require more sequence depth and increase memory usage.
- The Bloom filter size `-w` is specified in megabits; insufficient size increases false negatives (legitimate k-mers missed) and reduces correction accuracy.

## Pitfalls

- Using an undersized Bloom filter with `-w` causes high false-negative rates, leading to under-correction where real errors pass uncorrected. Use at least 2× the unique k-mer estimate for reliable correction.
- Specifying a k-mer size with `-k` that is too large relative to read length results in zero valid k-mers per read; k-mers must be ≤ read length minus the error-corrected base position.
- Running bfc without `-t` outputs reads sorted by coordinate, which is incompatible with paired-end processing pipelines that expect name-sorted BAM; reads may appear in wrong order when piped to downstream aligners.
- Not providing a pre-built k-mer hash table with `-s` forces on-the-fly counting, which is slower for repeated runs on the same dataset; use `-s` to load a cached hash table for iterative error correction workflows.
- Confusing `-1`/`-2` (input file flags) with read group specification causes bfc to treat all reads as single-end, potentially duplicating output when correcting paired-end data.

## Examples

### Correct single-end Illumina reads in place
**Args:** `-t -k 23 -w 300000000 seqs.fq.gz | gzip -1 > corrected.fq.gz`
**Explanation:** Counts k-mers, corrects reads by k-mer voting, outputs name-sorted FASTQ for downstream paired-end alignment compatibility.

### Build and cache a k-mer hash table for iterative correction
**Args:** `bfc -s ref.ctb -t -k 23 -w 300000000 -1 reads_1.fq.gz -2 reads_2.fq.gz > corrected.fq`
**Explanation:** Creates a Bloom filter hash table at `ref.ctb` for reuse; reads are name-sorted after correction for BAM compatibility.

### Correct reads using a pre-existing hash table
**Args:** `-s existing.ctb -t reads.fq.gz | gzip -1 > output.fq.gz`
**Explanation:** Loads the cached k-mer Bloom filter from `existing.ctb` to correct reads without rebuilding the hash table.

### Adjust k-mer size for short reads (50 bp)
**Args:** `-t -k 19 -w 100000000 short_reads.fq`
**Explanation:** Uses a smaller k-mer size to ensure sufficient coverage within short reads while reducing memory footprint.

### Tune Bloom filter size for deep sequencing data
**Args:** `-t -k 23 -w 800000000 highcov.fq | gzip -1 > corrected.fq.gz`
**Explanation:** Increases Bloom filter size to 800 megabits to handle the larger k-mer space from high-coverage sequencing runs.