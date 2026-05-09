---
name: bgt
category: Genome Graph Manipulation
description: A bioinformatics tool for building, indexing, and querying genome variation graphs. bgt (Bloody Great Tool) creates compressed graph indexes from variation graph representations (GFA/GCSA) and enables efficient read mapping against population-scale genome graphs. Part of the vg toolkit ecosystem.
tags: [genome-graph, variation-graphs, read-mapping, vg-toolkit, gfa, index-building, bioinformatics]
author: AI-generated
source_url: https://github.com/vgteam/vg
---

## Concepts

- **Graph Index Format (GCSA)**: bgt builds GCSA (Graph Compacted Suffix Array) indexes from input graphs in GFA format, enabling fast k-mer lookup and read mapping across complex variation graphs containing many alternative haplotypes.
- **Input Format (GFA)**: bgt accepts Graphical Fragment Assembly (GFA) format files representing linear and variation graphs. GFA1 and GFA2 specifications are supported, with L (link) records defining connectivity and P (path) records defining named paths through the graph.
- **Index Construction Modes**: bgt supports multiple index building strategies including basic GCSA construction (for smaller graphs) and two-phase indexing (for large population graphs), where the first phase builds a temporary index and the second phase compacts it for memory efficiency.
- **Query Operations**: Once indexed, bgt can be used to query k-mers against the graph index, retrieve graph node sequences, and perform fast exact-match lookups useful for read mapping pipelines.
- **Companion Binary (bgt-build)**: The bgt-build companion binary handles index construction, while the bgt binary primarily handles query and mapping operations against pre-built indexes.

## Pitfalls

- **Memory Exhaustion on Large Graphs**: Building GCSA indexes for population-scale graphs (thousands of haplotypes) requires substantial memory (tens of GB). Failing to allocate sufficient RAM causes the build process to fail or be killed by the system OOM killer.
- **Incorrect GFA Syntax**: Malformed GFA input files (missing required fields, invalid link directions, duplicate path names) cause silent parsing failures or produce incorrect indexes. Always validate GFA files with gft or similar validators before indexing.
- **Mismatched k-mer Sizes**: The k-mer size used during index building must match the k-mer size used for queries. Using a different k-mer size during mapping produces zero results or incomplete mappings.
- **Missing Path Definitions**: Graphs without embedded paths (P-lines in GFA) cannot be used for path-based queries or haplotype-aware mapping. Ensure your input graph includes relevant population haplotypes as named paths.
- **Compressed Index Size Limits**: Very large GCSA indexes may exceed filesystem size limits or become unwieldy. For graphs with >100,000 variants, consider splitting into chromosome-specific indexes rather than building a single genome-wide index.

## Examples

### Build a GCSA index from a GFA graph file
**Args:** -d graph.gcsa graph.gfa
**Explanation:** The -d flag specifies the output directory (graph.gcsa) and the final argument is the input GFA file. This creates a GCSA index ready for k-mer queries.

### Build an index with a specific k-mer size
**Args:** -d graph.gcsa -k 32 graph.gfa
**Explanation:** The -k 32 flag sets the k-mer size to 32 nucleotides, which defines the minimum exact-match length for queries against the index.

### Query a k-mer against a built graph index
**Args:** -d graph.gcsa ATGCGATCGACTAGCTAGCTAGCATGCA
**Explanation:** Queries the specified 30-mer sequence against the GCSA index in the graph.gcsa directory. Returns graph positions where the exact k-mer occurs.

### Build a two-phase index for large population graphs
**Args:** -d large_graph.gcsa -m 2g graph_large.gfa
**Explanation:** The -m 2g flag enables two-phase building with a 2GB memory limit per phase, suitable for large variant graphs where single-phase building would exceed available RAM.

### List available paths in an indexed graph
**Args:** -d graph.gcsa -L
**Explanation:** The -L flag lists all named paths (haplotypes, transcripts, or other annotated paths) stored in the GCSA index, useful for verifying which sequences are available for mapping.