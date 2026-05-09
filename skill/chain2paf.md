---
name: chain2paf
category: Genome Alignment / Format Conversion
description: Converts UCSC chain format alignment files to PAF (Pairwise Alignment Format), enabling interoperability with tools like minimap2, DAGcon, and other alignment processors that consume PAF format.
tags: [ucsc, chain, paf, alignment, format-conversion, genomics, genome-alignment]
author: AI-generated
source_url: https://github.com/ucscGenomeTools/chain2paf
---

## Concepts

- **Chain Format Input**: The tool reads UCSC chain format files, which represent transitive genome-to-genome alignments as ordered blocks (gaps) with coordinate mappings between reference and query sequences. Each chain contains a header line with sequence names and alignment score, followed by alignment blocks.

- **PAF Output Format**: Produces 12-column PAF format with mandatory fields: Query Name, Query Length, Query Start, Query End, Strand, Ref Name, Ref Length, Ref Start, Ref End, Matching Bases, Alignment Length, and Mapping Quality. Additional optional tags may follow for extended metadata.

- **Standard Input/Output Streams**: By default, chain2paf reads from stdin and writes to stdout, enabling shell pipelines. Use input file arguments or redirect output to handle files explicitly. The tool processes one chain file at a time.

- **Coordinate System Handling**: Alignments are output using 0-based coordinates for start positions and 1-based for end positions in PAF format. Strand information is preserved from the chain file (+ or - for reverse complement alignments).

## Pitfalls

- **Missing Sequence Dictionary**: If the chain file references sequences not found in the alignment database, the conversion may skip alignments or produce incomplete PAF entries. Always verify that sequence names in the chain file match your target genome assembly.

- **Incompatible Strand Orientation**: Chains with negative strand query alignments will have reversed coordinates in PAF output. Downstream tools expecting forwardstrand only alignments may fail or misalign. Check the strand column (field 5) in your PAF output before processing.

- **Large Chain Files Causing Memory Pressure**: Very large chain files (e.g., whole-genome alignments) loaded entirely into memory can cause the tool to crash or become unresponsive. Process large files in chunks or use streaming approaches if available.

- **Incorrect File Permissions**: Attempting to write output to a file without write permissions will silently fail or produce an error. Verify write permissions on the output directory before redirection.

## Examples

### Convert a chain file to PAF format
**Args:** input.chain > output.paf
**Explanation:** Reads the UCSC chain file and converts alignment blocks to PAF format, redirecting output to a file for downstream processing.

### Stream chain conversion through a pipeline
**Args:** cat align.chain | chain2paf | sort -k1,1 -k3,3n > sorted.paf
**Explanation:** Demonstrates chaining the conversion with sorting by query name and start position, useful for preparing alignments for tools requiring sorted input.

### Convert and filter alignments by minimum length
**Args:** chain2paf align.chain | awk '$10 >= 1000' > filtered.paf
**Explanation:** Converts the chain file then filters PAF output to retain only alignments with at least 1000 matching bases in field 10, removing small spurious alignments.

### Check chain file format before conversion
**Args:** head -n 20 align.chain
**Explanation:** Views the first 20 lines of a chain file to verify its format, alignment scores, and sequence names before running conversion.

### Count alignments in the PAF output
**Args:** chain2paf align.chain | wc -l
**Explanation:** Converts the chain and counts the number of resulting PAF alignment records, providing a quick metric of alignment density.