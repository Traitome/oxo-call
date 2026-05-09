---
name: bcalm
category: Genome Assembly / de Bruijn Graph Construction
description: A multithreaded tool for constructing compacted de Bruijn graphs (unitigs) from k-mer counts in FASTA/FASTQ input reads. Optimized for large-scale genome assembly and error correction using disk-based algorithms.
tags:
  - de-bruijn-graph
  - genome-assembly
  - k-mer
  - unitigs
  - compaction
  - bioinformatics
  - sequence-analysis
author: AI-generated
source_url: https://github.com/GATB/bcalm
---

## Concepts

- **Compacted de Bruijn Graphs**: bcalm constructs compacted de Bruijn graphs where unitigs represent maximal non-branching paths. Each unitig corresponds to a sequence where every internal k-mer has abundance ≥ the specified minimum threshold, and branching occurs only at low-abundance or variant positions.

- **Input/Output Formats**: Input accepts FASTA or FASTQ files (plain, gzipped, or bzip2 compressed) containing sequencing reads. Output produces a `*.unitigs.fa` FASTA file containing the assembled unitig sequences, with one entry per unitig, plus a `*.h5` HDF5 file storing graph topology for downstream analysis.

- **Minimizer-based Partitioning**: The tool uses a minimizerスキemme approach to partition k-mers across threads and disk files, enabling scalable processing of billions of k-mers with bounded memory usage. The `-nb-cores` flag controls parallelism, and `-kmer-size` defines the k-mer length used for graph construction.

- **Abundance Filtering**: The `-abundance-min` parameter is critical—it discards k-mers appearing fewer times than the threshold before graph construction begins. This step removes likely sequencing errors, reducing graph complexity and improving assembly quality. The `-abundance-max` parameter can additionally cap high-abundance k-mers for repetitive region handling.

- **Companion Script**: The `bcalm-2-to-json.py` Python script (distributed with bcalm) converts HDF5 output to JSON format for visualization tools, enabling inspection of graph branching points and unitig coverage values.

## Pitfalls

- **Incorrect k-mer Size Selection**: Choosing a k-mer size too small produces highly connected graphs with false branching from repetitive sequences; choosing too large reduces sensitivity to repeat variations and increases memory usage exponentially. The k-mer size must be compatible with read length (typically ≥ read length minus overlap).

- **Setting abundance-min Too Low**: Specifying an abundance threshold of 1 or 2 retains k-mers from sequencing errors, inflating the graph with spurious branching paths and unitigs that do not represent true genomic sequence, making downstream assembly intractable.

- **Assuming Output File Locations**: The tool writes output files in the same directory as input files with auto-generated suffixes (`.unitigs.fa`, `.h5`), not to the current working directory. If input paths are relative, output may appear in unexpected directories, requiring explicit path checking.

- **Forgetting Read Orientation**: bcalm processes k-mers from both read strands automatically (canonical k-mers), but downstream tools may expect specific orientation conventions. Mixing outputs from tools with different orientation handling leads to assembly inconsistencies.

- **Insufficient Threading for Large Datasets**: Using the default single thread on large datasets (millions of reads) causes extremely long runtimes. The `-nb-cores` parameter must be explicitly set to utilize available CPU cores; however, exceeding physical core count degrades performance through thread contention.

## Examples

### Build a compacted de Bruijn graph from a single FASTQ file with default settings

**Args:** `reads.fastq.gz -kmer-size 31`
**Explanation:** Sets the k-mer size to 31 (bcalm's typical default) and processes the compressed FASTQ file, producing unitigs and HDF5 output files alongside the input.

### Build a graph with aggressive error removal using abundance threshold of 5

**Args:** `input_reads.fa -abundance-min 5 -kmer-size 47`
**Explanation:** Discards any k-mer appearing fewer than 5 times before graph construction, removing most sequencing errors while retaining sufficiently covered genomic regions.

### Process multiple input files in parallel using 8 CPU cores

**Args:** `file1.fastq file2.fastq file3.fastq -nb-cores 8 -kmer-size 35`
**Explanation:** Specifies three input files and enables parallel processing across 8 threads to accelerate graph construction for large multi-file datasets.

### Generate a graph filtering both low and high abundance k-mers

**Args:** `sample_R1.fastq.gz sample_R2.fastq.gz -abundance-min 3 -abundance-max 500 -kmer-size 55`
**Explanation:** Removes k-mers from sequencing errors (min