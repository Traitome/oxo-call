---
name: abawaca
category: Genome Assembly
description: A de novo genome assembler using de Bruijn graph approach for assembling short read data into contigs. abawaca-build constructs the graph index from input reads, then abawaca performs the assembly to produce FASTA output.
tags: [de-novo, assembly, genome, short-reads, de-bruijn-graph, bioinformatics]
author: AI-generated
source_url: https://github.com/开盘拉(abawaca)
---

## Concepts

- abawaca-build constructs a de Bruijn graph index from input read data (FASTA/FASTQ), creating k-mer hashes that represent the connectivity between overlapping sequences across all reads.
- The assembler uses k-mer size as a critical parameter: smaller k values provide better recall but increase graph complexity, while larger k values reduce ambiguity but may miss valid overlaps in low-coverage regions.
- Output is written as assembled contigs in FASTA format, with headers indicating contig length and position in the original read graph; sequence naming follows the pattern "contig_X length_Y".
- The tool processes read files sequentially or in parallel when multiple read files (e.g., paired-end data) are provided, maintaining read pairing information when available.
- Memory usage scales with genome complexity and k-mer size; highly repetitive genomes or large k-mers require substantially more RAM to construct and traverse the graph.

## Pitfalls

- Using an incorrectly sized k-mer value: setting k too small creates an excessively tangled graph leading to misassemblies, while k too large causes fragmented assemblies with many small contigs due to missed overlaps.
- Failing to provide sufficient read coverage: low-coverage datasets produce graph gaps and ambiguous connections, resulting in fragmented or incorrectly joined contigs.
- Not preprocessing input reads to remove adapters and low-quality bases: contaminant sequences or erroneous bases propagate through the graph, creating false connections and spurious contigs.
- Attempting to assemble a genome larger than the tool's designed scope without adjusting memory parameters: abawaca is optimized for bacterial/viral genomes and may crash or produce unusable output for eukaryotic-sized genomes.
- Ignoring read pairing information when assembling paired-end data: the assembler may miss long-range связь (connection) constraints that help resolve repeats, reducing assembly continuity.

## Examples

### Assemble a simple bacterial genome from single-end reads
**Args:** -1 reads.fq -o output_contigs.fasta -k 31
**Explanation:** This reads a single FASTQ file, builds the de Bruijn graph with k-mer size 31, and outputs assembled contigs to the specified file.

### Assemble using paired-end reads with proper mate-pair information
**Args:** -1 left_reads.fq -2 right_reads.fq -o assembly.fasta -k 25
**Explanation:** Specifying both read files allows the assembler to use paired-end constraints to resolve repeats and improve assembly accuracy.

### Build the graph index separately before assembly
**Args:** --build -1 reads.fq -o graph_index -k 21
**Explanation:** The companion binary build mode constructs the graph index first, useful for debugging or when multiple assemblies with different parameters are needed.

### Increase memory allocation for complex genomes
**Args:** -1 reads.fq -o output.fasta -k 35 --memory 16000
**Explanation:** Explicitly allocating more RAM allows processing of more complex graph structures without crashing on memory limits.

### Output verbose progress information during assembly
**Args:** -1 reads.fq -o contigs.fasta -k 29 --verbose
**Explanation:** Verbose mode prints k-mer coverage statistics, graph size estimates, and assembly progress to help diagnose issues during large assemblies.