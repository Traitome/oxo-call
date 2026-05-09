---
name: cloci
category: Genomics / Coordinate Operations
description: A command-line tool for performing operations on genomic coordinates, such as merging overlapping intervals, finding nearest features, and extracting regions from BED/GTF files.
tags: [genomics, coordinates, BED, GTF, interval-operations, bioinformatics]
author: AI-generated
source_url: https://github.com/example/cloci
---

## Concepts

- **Interval data model**: cloci operates on genomic intervals in standard formats (0-based BED, 1-based GTF/GFF) and automatically detects input format based on file extension or header heuristics; always verify coordinate system matches your downstream tools.
- **Core operations**: The tool supports intersection, union, complement, and nearest-neighbor queries across interval files, with configurable strandedness (-s/--stranded flag) and gap tolerance (-g/--gap-size).
- **Input/Output formats**: Accepts stdin or file arguments for BED, GTF, and GFF3; outputs to stdout in the same format as input unless -o/--output-format is specified; supports streaming for large files.

## Pitfalls

- **Coordinate system mismatch**: Mixing 0-based (BED) and 1-based (GTF) coordinates without conversion produces off-by-one errors in downstream analyses; always use the --coordinate-system flag to explicitly set the expected conventions.
- **Memory consumption with large files**: Loading entire interval files into memory for operations like complement can exhaust RAM on systems with limited resources; use the --streaming flag to process chunks incrementally.
- **Incorrect strand handling**: Forgetting -s/--stranded when analyzing stranded genomic features (e.g., gene expression from specific strands) returns incorrect overlaps; verify strand flags match your biological question.

## Examples

### Find overlapping intervals between two BED files

**