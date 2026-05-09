---
name: contignet
category: Genomics
description: A tool for analyzing contig connectivity and constructing assembly networks from FASTA contig files, useful for genome assembly validation and metagenome analysis.
tags:
  - bioinformatics
  - genomics
  - assembly
  - contigs
  - network-analysis
author: AI-generated
source_url: https://github.com/contignet/contignet
---

## Concepts

- **Input Format**: contignet accepts FASTA or FASTQ files containing assembled contigs. Each sequence record must have a unique identifier; duplicate IDs cause parsing failures and will terminate the analysis.
- **Network Construction**: The tool builds a network graph where contigs are nodes and edges represent sequence similarity, overlap, or user-specified homology thresholds. Edge weights correspond to alignment scores or overlap lengths.
- **Connectivity Metrics**: Output includes node degree distributions, connected component statistics, and assembly quality metrics such as N50, L50, and network density values.
- **Filtering Parameters**: Minimum contig length (`-m`), minimum edge weight (`-w`), and identity threshold (`-i`) control which sequences and connections are included in the final network.
- **Output Formats**: Results are produced in multiple formats including GraphML (for network visualization in tools like Cytoscape), TSV summary statistics, and JSON for programmatic downstream processing.

## Pitfalls

- **Memory Overflow with Large Assemblies**: Setting `-m` (minimum length) too low results in processing millions of tiny contigs, causing excessive memory consumption and potential process termination. Always set a reasonable minimum length based on expected assembly quality.
- **Identity Threshold Too Low**: Using `-i` below 0.90 includes spurious connections between unrelated sequences, creating false edges that distort network topology and invalidate downstream analysis.
- **Missing Sequence Delimiters**: FASTA files without proper line breaks or with concatenated sequences cause parsing errors. Ensure sequences are properly formatted with `>` headers followed by newline-delimited bases.
- **Duplicate Contig Identifiers**: Duplicate sequence names in the input file lead to ambiguous node labeling and cause the network construction to fail silently or produce incorrect connectivity reports.
- **Incompatible Output Directory Permissions**: Attempting to write results to read-only directories or paths with special characters produces file I/O errors; verify write permissions before execution.

## Examples

### Build a basic contig network from an assembly file
**Args:** `-i assembly.fasta -o network_output`
**Explanation:** Loads the FASTA file and constructs a default network using built-in similarity thresholds, outputting results to the specified directory.

### Filter network by minimum contig length of 500bp
**Args:** `-i assembly.fasta -m 500 -o filtered_output`
**Explanation:** Excludes all contigs shorter than 500 base pairs before network construction, reducing noise from fragmented assemblies.

### Set minimum identity threshold to 95%
**Args:** `-i assembly.fasta -i 0.95 -o high_identity_network`
**Explanation:** Only includes edges between sequences with at least 95% identity, producing a high-confidence network of closely related contigs.

### Export network in GraphML format for Cytoscape visualization
**Args:** `-i assembly.fasta --format graphml -o viz_output/graph.xml`
**Explanation:** Generates GraphML output compatible with network visualization software, preserving node and edge attributes for downstream analysis.

### Generate summary statistics without full network construction
**Args:** `-i assembly.fasta --stats-only -o summary.tsv`
**Explanation:** Quickly computes basic assembly metrics (contig count, total length, N50, L50) without the computationally intensive network-building step.

### Adjust edge weight threshold to filter weak connections
**Args:** `-i assembly.fasta -w 100 -o strong_edges`
**Explanation:** Only includes edges with weights above 100 (alignment score or overlap length depending on mode), removing marginal connections from the network.

### Process gzipped FASTA input directly
**Args:** `-i assembly.fasta.gz -o output_dir`
**Explanation:** Accepts compressed input files without manual decompression, supporting both FASTA and FASTQ formats transparently.