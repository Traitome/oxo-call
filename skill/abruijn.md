---
name: abruijn
category: Genome Assembly
description: A de novo genome assembler based on the A-Bruijn graph algorithm, designed for efficient assembly of large eukaryotic genomes from high-throughput sequencing reads.
tags:
  - genome-assembly
  - de-novo
  - graph-based
  - sequence-assembly
  - bruijn-graphs
author: AI-generated
source_url: https://github.com/bioinf/abruijn
---

## Concepts

- **Graph-Based Architecture**: abruijn constructs an A-Bruijn graph where k-mers form nodes and read connections form edges. The assembler traverses these graphs to generate contigs, making it memory-efficient for repeat-dense genomes compared to OLC (Overlap-Layout-Consensus) methods.

- **Configuration-Driven Input**: Unlike many assemblers that rely purely on CLI flags, abruijn uses an INI-style configuration file (`asm.cfg`) that specifies read libraries, k-mer sizes, cutoff thresholds, and output paths. Multiple read libraries (paired-end, mate-pair) are declared as separate sections with orientation, insert size, and file paths.

- **Two-Stage Assembly Process**: The assembly proceeds in distinct phases—first constructing the de Bruijn graph from k-mers, then simplifying the graph through bubble popping and tip removal before generating sequences. The `-k` parameter sets the k-mer length used during graph construction, directly impacting repeat resolution ability.

- **Output Formats**: The assembler produces FASTA files for contigs (final_asm.ctg.fasta) and scaffolds (final_asm.scaf.fasta), along with an assembly graph file (final_asm.graph) that can be used for manual inspection or downstream processing with companion tools.

- **Memory-Complexity Tradeoff**: Using a smaller k-mer size reduces graph complexity and memory usage but sacrifices repeat resolution; larger k-mers resolve more repeats but generate denser graphs requiring more RAM. The optimal k-mer is typically 1/6 to 1/3 of the average read length.

## Pitfalls

- **Mismatched Configuration Section Names**: Specifying read libraries with incorrect section headers (e.g., `[LIB1]` instead of `[LIB]` or using zero-padded names like `[LIB01]`) causes the configuration parser to silently ignore those libraries, resulting in an assembly using only the first declared library or failing entirely.

- **K-mer Size Too Large for Coverage**: Setting a k-mer size where expected coverage falls below ~5x can cause graph fragmentation, producing thousands of tiny contigs instead of complete chromosomes because insufficient k-mer connections exist to traverse repeat regions.

- **Confusing Insert Size Units**: The `avg_ins` parameter in the configuration expects insert size in base pairs, but users often mistakenly enter fragment size in kilobases (e.g., writing `200` instead of `200000` for a 200 kb fosmid library), causing wildly incorrect scaffolding and potential misassemblies.

- **Missing Reverse-Compliment Reads**: abruijn requires explicit declaration of read orientation using `rd_len_cutoff` and `reverse_seq` flags; omitting these for libraries where reads are provided in one orientation only results in half the usable data being ignored, dramatically reducing effective coverage.

- **Insufficient Memory for Large Genomes**: The graph simplification phase during bubble popping consumes RAM proportional to genome size and k-mer density. Attempting to assemble a human-size genome without reserving at least 1 GB RAM per 100 Mb of genome sequence causes out-of-memory failures mid-assembly with no intermediate output recovery.

## Examples

### Basic genome assembly with default k-mer
**Args:** `ASM_CFG=/path/to/config.ini K=63 GENOME_SIZE=3000000000 OUTPUT_DIR=/results/asm1`
**Explanation:** Setting K=63 and specifying the genome size helps abruijn optimize graph traversal parameters and memory allocation before starting the de Bruijn graph construction phase.

### Paired-end library with specified orientation
**Args:** `RD_LEN_CUTOFF=100 MIN_K=17 MAX_K=63 AVG_INS=350 reverse_seq=0 rank=0`
**Explanation:** Declaring reverse_seq=0 indicates the paired-end reads are in FR orientation (forward on first read, reverse on second), ensuring proper pairing during scaffold construction.

### Mate-pair library for long-range scaffolding
**Args:** `RD_LEN_CUTOFF=100 MIN_K=17 MAX_K=63 AVG_INS=3000 reverse_seq=1 rank=1`
**Explanation:** Specifying reverse_seq=1 for mate-pair data instructs abruijn to treat the library as RF-oriented reads, correct for most mate-pair protocols where the second read sequenced is upstream.

### Optimizing k-mer for repeat-dense genome regions
**Args:** `K=45 MIN_K=35 MAX_K=65 GENOME_SIZE=2500000000 bubble_popping=1`
**Explanation:** For AT-rich genomes with many repeat junctions, using a k-mer smaller than the default helps reduce graph complexity while still enabling reliable k-mer counting without excessive fragmentation.

### Scaffolding-only mode using pre-assembled contigs
**Args:** `CONTIG_FILE=/path/to/contigs.fasta USE_EXISTING_CONTIG=1 OUTPUT_DIR=/results/scaffold2`
**Explanation:** Setting USE_EXISTING_CONTIG=1 bypasses graph construction and directly uses provided contigs for scaffolding only, useful when improving an existing assembly with additional libraries.