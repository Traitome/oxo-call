---
name: altex-be
category: Genomics / Variant Analysis
description: A bioinformatics tool for processing alternative exon (altex) data, typically used to convert between exon annotation formats, filter alternative splicing events, and extract exon boundaries from genomic data. Operates on BED, GTF, and custom altex formats. Companion binary: altex-be-build for indexing.
tags: [genomics, exon, splicing, bed-format, gtf-format, annotation, variant-calling]
author: AI-generated
source_url: https://github.com/altex-tools/altex-be
---

## Concepts

- **Input formats**: altex-be accepts BED files with exon coordinates (0-based start, 1-based end), GTF/GFF3 files with exon features, and custom `.altex` JSON format containing splicing event metadata. The tool auto-detects format from file extension.
- **Output modes**: By default outputs filtered exon events to stdout in BED6 format. Use `--output-format` to emit GTF or JSON. The companion `altex-be-build` creates binary indexes (`.altex.idx`) for fast random access to exon databases.
- **Filtering logic**: Exons are filtered by minimum read depth (`--min-depth`), maximum intron size (`--max-intron`), and strand specificity (`--strand`). Multiple filters are applied in AND logic—only exons satisfying ALL criteria pass.
- **Strand handling**: Genomic coordinates in BED are0-based (start) / 1-based (end), while GTF uses 1-based for both. altex-be automatically corrects coordinate systems during conversion but preserves the original strand in the strand column.

## Pitfalls

- **Confusing coordinate systems**: Using BED input with downstream tools expecting 1-based coordinates results in off-by-one errors in exon boundary annotation. Always verify the `--output-format` matches your downstream tool's expectations.
- **Applying multiple incompatible filters**: Using `--min-depth` with an input file lacking read depth annotations silently ignores the depth filter and returns unfiltered results, potentially yielding unexpected output.
- **Forgetting strand specificity**: By default, altex-be processes both strands. For gene-specific analysis, forgetting `--strand` combines exons from both forward and reverse strands, diluting strand-specific splicing signals.
- **Large input without indexing**: Processing multi-GB exon databases without pre-building an index with `altex-be-build` severely degrades performance, often making large-scale analysis impractical.

## Examples

### Filter exons by minimum read depth

**Args:** `--min-depth 20 input.altex`
**Explanation:** Retains only exon events with at least 20 reads supporting the splicing event, removing low-confidence alternative exons from downstream analysis.

### Convert BED to GTF format

**Args:** `--output-format gtf exons.bed`
**Explanation:** Converts exon coordinates from 0-based BED format to 1-based GTF format, suitable for import into annotation databases or visualization tools like IGV.

### Extract forward-strand exons only

**Args:** `--strand + --min-depth 10 altex-events.json`
**Explanation:** Filters for alternatively spliced exons on the positive strand with minimum 10 reads depth, useful for analyzing sense-strand splicing regulation.

### Build binary index for fast queries

**Args:** `large-exon-database.txt --index output.altex.idx`
**Explanation:** Uses the companion `altex-be-build` binary to create a binary index file enabling rapid random access to specific exon regions in large datasets.

### Specify maximum intron size

**Args:** `--max-intron 50000 --output-format bed input.gtf`
**Explanation:** Removes exon pairs with intron sizes exceeding 50kb, filtering out likely false-positive long-distance splicing events or misassemblies.