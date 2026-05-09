---
name: bygul
category: sequence-alignment
description: A fast and accurate DNA read aligner based on a bidirectional de Bruijn graph approach that maps single-end or paired-end sequencing reads to a reference genome with high sensitivity and speed.
tags: [alignment, dna-seq, de-bruijn-graph, read-mapping, sam-output]
author: AI-Generated
source_url: https://github.com/bgul-org/bygul
---

## Concepts

- Bygul builds a **bidirectional genome-length de Bruijn graph (DBG)** from the reference sequence, which enables sensitive detection of overlapping k-mers during read mapping while handling structural variants and large indels more robustly than seed-and-extend algorithms.
- The aligner outputs results in **SAM (Sequence Alignment/Map) format**, which stores query name, flags, reference coordinates, mapping quality (MAPQ), CIGAR strings, and optional tags such as MD (for mismatch visualization) and NM (edit distance).
- Bygul supports **paired-end read alignment** using the `-p` flag; when enabled, it uses the fragment length distribution to rescue reads with poor individual mapping quality, improving accuracy near repetitive regions.
- The tool uses **k-mer hashing with FM-index fallback** for seeds: primary alignment seeds are identified via DBG traversal, and unaligned regions are rescued via banded dynamic programming to produce CIGAR strings.
- Memory usage during index building scales with **O(genome size × k-mer size)** and is reported at completion; large genomes with small k values can consume tens of gigabytes of RAM.

## Pitfalls

- Specifying insufficient RAM for index construction (e.g., with `-M` or default limits) causes the `bygul-index` step to abort with a "k-mer hash table allocation failed" error, producing no output.
- Using a **k-mer size (`-k`)** that is too small (below 10) results in enormous RAM consumption and many spurious seed matches, inflating runtime and producing incorrectly tiled CIGAR strings; sizes above 31 are silently capped at the default maximum.
- Mixing **paired-end (`-p`)** mode with single-end input files or providing uneven numbers of reads in the left and right files causes bygul to report inconsistent fragment lengths and write malformed SAM records with `FLAG 0` for properly paired-looking lines.
- Providing a **reference genome in lowercase** (e.g., from a UCSC `*.genome.fa` download with soft-masked repeats) causes bygul to treat lowercase bases as part of the sequence rather than mask them, biasing k-mer counting and silently changing the output MAPQ values.
- Omitting the **mapping quality threshold filter (`-q`)** when downstream tools (e.g., GATK) expect MAPQ ≥ 20 results in low-quality alignments being passed through, which can inflate false-positive variant calls in downstream variant calling pipelines.

## Examples

### Align single-end reads to a reference genome
**Args:** ref.fasta reads.fastq -o output.sam
**Explanation:** Bygul loads the pre-built index (or builds it implicitly), maps each read by DBG traversal, and writes SAM records to output.sam.

### Align paired-end reads and enable fragment rescue
**Args:** ref.fasta left.fastq right.fastq -o output.sam -p
**Explanation:** The `-p` flag activates paired-end mode, preserving insert-size and orientation constraints so that discordant read pairs are flagged correctly in the SAM FLAG field.

### Tune alignment sensitivity for a divergent genome
**Args:** ref.fasta reads.fastq -o output.sam -k 12 -w 0.80
**Explanation:** Reducing k to 12 and lowering the CIGAR score threshold to 0.80 allows more mismatches per read while avoiding premature read failure, which is useful when mapping reads from a strain with >5% sequence divergence.

### Parallelize alignment across 16 threads for large datasets
**Args:** ref.fasta reads.fastq -o output.sam -t 16
**Explanation:** The `-t 16` flag spawns 16 worker threads, each handling a separate chromosomal region batch, dramatically reducing wall-clock time at the cost of proportional RAM overhead.

### Filter output to high-confidence mappings only
**Args:** ref.fasta reads.fastq -o output.sam -q 20
**Explanation:** Reads with MAPQ below 20 are discarded from the output SAM file, ensuring downstream tools receive only reliably mapped reads and reducing noise in variant calling or expression quantification.