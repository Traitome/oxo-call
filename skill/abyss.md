---
name: abyss
category: assembly
description: A de novo genome assembler for short reads using the shoring algorithm. Assembles genomes from FASTA/FASTQ input and outputs contigs in FASTA format.
tags: assembly, genome-assembler, de-novo, short-reads, bioinformatics
author: AI-generated
source_url: https://github.com/bcgsc/abyss
---

## Concepts

- Abyss uses a "shoring" algorithm that builds a graph of overlapping k-mers and repeatedly " shores" up consensus sequences, making it effective for large genomes with high coverage short-read data.
- Input format accepts FASTA or FASTQ files (optionally compressed with gzip or bzip2) for raw reads, and the assembler outputs assembled contigs in FASTA format along with auxiliary files (like coverage statistics).
- Key parameters include the k-mer length (`-k`), which must be odd and typically ranges from 20-64 depending on coverage, and the coverage cutoff (`-c`) to filter low-complexity regions; smaller k-mers improve sensitivity but increase memory usage.
- Abyss is parallelized using OpenMP and MPI, with the `-t` flag controlling thread count and MPI processes specified via `mpirun` for distributed assembly across nodes.

## Pitfalls

- Using an even k-mer size will cause the assembler to crash or produce malformed output; only odd integers are accepted for `-k` because the algorithm relies on unambiguous node traversal.
- Insufficient coverage (below ~5x) leads to fragmented assemblies with many small contigs, as the assembler cannot reliably resolve repeats or bridge gaps in the de Bruijn-like graph.
- Specifying a k-mer size larger than the read length results in zero overlaps and complete failure to assemble, since no k-mer can span the full read length.
- Running without setting thread limits (`-t`) on multi-core systems wastes potential parallelism, especially when MPI is not available for distributed computing.

## Examples

### Assemble paired-end reads with a specified k-mer size
**Args:** -k 31 -t 8 -o output_prefix reads1.fq reads2.fq
**Explanation:** Uses k-mer length 31 with 8 threads to assemble paired-end reads into contigs saved as output_prefix-contigs.fa.

### Assemble single-end reads with coverage cutoff filtering
**Args:** -k 25 -c 10 -t 4 -o assembly input.fq
**Explanation:** Applies a coverage cutoff of 10 to filter low-depth k-mers, improving assembly quality for single-end data with 4 threads.

### Resume a previous assembly run from checkpoint
**Args:** -k 33 -t 12 -o continue_output --resume previous_output
**Explanation:** Resumes an interrupted assembly using previously saved checkpoint files, specifying a different k-mer or thread count for completion.

### Assemble long reads with reduced k-mer size
**Args:** -k 21 -t 16 -o long_output long_reads.fq
**Explanation:** Uses a smaller k-mer (21) to accommodate longer read lengths where larger k-mers would fail to find overlaps.

### Use custom bubble popping sensitivity
**Args:** -k 41 -b 100 -t 8 -o bubble_test reads.fq
**Explanation:** Increases the bubble population limit to 100 before popping, allowing more aggressive consensus resolution for repetitive genomes.