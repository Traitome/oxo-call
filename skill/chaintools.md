---
name: chaintools
category: Genomics Alignment Processing
description: A utility for manipulating UCSC chain files that represent pairwise genomic alignments with gaps and insertions. Supports filtering, merging, extracting, and converting chain alignments between reference genomes.
tags: [chain-file, genomics, alignment, ucsc, pairwise-alignment, gap-alignment]
author: AI-generated
source_url: https://github.com/Cmdirgame/chaintools
---

## Concepts

- Chain files store pairwise alignments as hierarchical blocks: each chain has a **score**, **target** (reference) coordinates, **query** (mapped) coordinates, and a series of **alignment blocks** separated by gaps on either strand. Blocks are expressed as runs of matching sequence, with intervening gaps tracked independently for target and query.
- The **data model** treats each chain as an independent alignment record with a unique ID. Multiple chains can align the same query to different target regions, and chains may overlap on either strand depending on the source data.
- **I/O formats**: input and output use the standard UCSC chain format (plain text, tab-separated header lines followed by space-separated block descriptors). chaintools does not accept BAM, SAM, or MAF—only `.chain` or `.chain.gz` files.
- Operations are **idempotent per invocation**: most commands read stdin/one file and write to stdout unless `--output` is specified. Chained pipeline usage requires explicit file redirection or shell piping.
- **Filtering criteria** operate on chain metadata fields (score, chromosome name, size, strand) and block-level metrics (block count, alignment length). Numeric thresholds are evaluated with numeric comparison operators.

## Pitfalls

- Feeding a chain file with **zero chains** (empty file or header-only) silently produces an empty output, which downstream tools may misinterpret as a successful empty result rather than a missing-data error.
- Using `--score-below` with a **non-integer threshold** causes silent truncation of decimal values, resulting in unintentionally lenient filtering since the comparison truncates to integer first.
- Specifying the **same file as input and output** (`-i in.chain -o in.chain`) without `--overwrite` can result in partial overwrite races on some filesystems, producing a corrupted output file.
- Chain files generated from **self-chains** (aligning a genome to itself) contain numerous tiny blocks that inflate file size and slow downstream processing; failing to filter these with `--min-block-size` yields unnecessarily large outputs.
- Using `--strand` without realizing that **strand information is per-chain**, not per-block, causes the tool to keep or discard entire chains regardless of individual block orientation on the target or query strand.

## Examples

### Filter chains by alignment score above a threshold
**Args:** `filter -i alignments.chain --score-above 1000 -o high_score.chain`
**Explanation:** Retains only those chains whose alignment score exceeds 1000, discarding lower-scoring alignments that may represent poor-quality or fragmented mappings.

### Extract chains aligning to a specific target chromosome
**Args:** `extract -i mappings.chain --tname chr17 -o chr17_mappings.chain`
**Explanation:** Pulls out all chains where the target (reference) chromosome is exactly chr17, enabling targeted downstream analysis for a single genomic region.

### Merge multiple chain files into a single sorted output
**Args:** `merge -i left.chain right.chain -o combined.chain`
**Explanation:** Concatenates two chain files and re-sorts them by target chromosome and start coordinate, producing a valid chain file ready for genome browser ingestion.

### Remove chains with fewer than a minimum number of alignment blocks
**Args:** `filter -i all_chains.chain --min-blocks 3 -o filtered_chains.chain`
**Explanation:** Eliminates chains consisting of only one or two alignment blocks, which often represent noise or spurious alignments with limited biological relevance.

### Convert a chain file to reciprocal best-hits (RBH) format
**Args:** `to-rbh -i forward.chain --reciprocal reverse.chain -o rbh_pairs.chain`
**Explanation:** Identifies pairs of chains that represent mutual best matches between two genomes, a common approach for orthology inference in comparative genomics.

### Extract chains within a specific genomic interval on the target
**Args:** `extract -i ref_alignments.chain --tname chr12 --tstart 50000000 --tend 75000000 -o region_chr12.chain`
**Explanation:** Returns only chains overlapping the target interval from position 50 Mb to 75 Mb on chr12, allowing focused analysis of a chromosomal region of interest.