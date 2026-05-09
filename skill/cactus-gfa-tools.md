---
name: cactus-gfa-tools
category: Pangenomics / Graph Format Manipulation
description: Suite of tools for working with GFA (Graphical Fragment Assembly) format files used in pangenome construction. Provides functionality for building indexes, converting formats, and manipulating pangenome graph representations.
tags: [pangenomics, gfa-format, graph-manipulation, cactus, bioinformatics, sequence-graph]
author: AI-generated
source_url: https://github.com/ComparativeGenomicsToolkit/cactus
---

## Concepts

- **GFA Format Support**: cactus-gfa-tools operates on GFA 1.0 and GFA 2.0 formats, which represent sequence graphs as segments (sequences), links (adjacencies between segments), and paths (visits through the graph). The tools can parse, validate, and transform these graph representations.
- **Graph Indexing**: The build subcommand creates coordinate indexes mapping sequence positions to graph locations, enabling efficient queries and alignments against pangenome graphs. Indexes are stored in auxiliary files separate from the primary GFA.
- **Companion Binaries**: The package includes multiple specialized binaries accessed via the main wrapper, with `cactus-gfa-tools-build` handling index construction and other commands handling format conversion and validation.
- **Input/Output Patterns**: Tools accept GFA files via stdin or file arguments, output results to stdout or specified output paths, and support streaming operations for pipeline integration.

## Pitfalls

- **Conflicting GFA Versions**: Attempting to process GFA 1.0 files with GFA 2.0-specific operations or vice versa produces malformed output without clear error messages. Always verify the GFA format version before processing.
- **Missing Index Files**: Commands requiring pre-built indexes will fail silently or produce incorrect results if the index file is not generated first. Always run the build command before operations that require coordinate lookups.
- **Large Graph Memory Usage**: Processing entire pangenome graphs without chunking or streaming can exhaust available RAM on systems with limited memory. Use chunked processing or memory-mapping options when working with graphs containing billions of base pairs.
- **Incorrect Link Orientation**: Malformed GFA files with incorrect link orientation flags cause downstream alignment tools to misalign sequences. Validate graph connectivity before using the graph for alignments.

## Examples

### Build a GFA index for coordinate lookups
**Args:** cactus-gfa-tools-build input.gfa output.idx
**Explanation:** Creates a binary index file from the input GFA that enables fast coordinate-to-graph-position mapping in subsequent queries.

### Validate GFA format integrity
**Args:** cactus-gfa-tools-validate input.gfa
**Explanation:** Checks the input GFA file for structural integrity, reporting broken links, missing segments, and format violations.

### Convert GFA 1.0 to GFA 2.0 format
**Args:** cactus-gfa-tools-convert --to-gfa2 input.gfa output.gfa2
**Explanation:** Transforms a GFA 1.0 format file to GFA 2.0, enabling access to features like variable-length links and containment edges.

### Extract specific paths from a pangenome graph
**Args:** cactus-gfa-tools-extract-paths input.gfa --sample sample1,sample2
**Explanation:** Filters the input GFA to retain only paths corresponding to specified sample identifiers, producing a reduced graph.

### Stream a chunk of a large GFA file
**Args:** cactus-gfa-tools-stream input.gfa --chunk 1000000 --offset 5000000
**Explanation:** Outputs a 1-million-base-pair segment starting at position 5 million, enabling memory-efficient processing of very large graphs.

### Check for cycles in the graph topology
**Args:** cactus-gfa-tools-check-cycles input.gfa
**Explanation:** Analyzes the graph topology and reports whether circular paths exist, which may indicate biological rearrangements or data corruption.