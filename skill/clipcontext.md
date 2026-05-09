---
name: clipcontext
category: genomics
description: Extracts genomic context regions around CLIP-seq binding sites to identify nearby genes and regulatory elements.
tags: [clip-seq, rna-binding-proteins, peak-analysis, genomics, beds]
author: AI-generated
source_url: https://github.com/宿主Repository/clipcontext
---

## Concepts

- **Input Format**: Takes CLIP-seq peak files in standard BED or narrowPeak format containing chromosome, start, end, and score columns; uses a reference genome annotation file (GTF/GFF3) to identify overlapping or nearby genomic features.
- **Window-based Extraction**: Defines genomic context by specifying an upstream/downstream window size (e.g., `--window 5000`) around each peak center, extracting all genes or features that fall within or intersect this window.
- **Output Data Model**: Produces a table with one row per peak-feature pair, including peak coordinates, distance to feature start/end, feature ID (gene name/transcript ID), and optionally the overlapping genomic region type (exon, intron, UTR, promoter).
- **Strand Awareness**: By default considers both strands but can be configured with `--stranded` to report only features on the same strand as the CLIP peak, which is critical for determining sense-strand binding events.
- **Multi-feature Handling**: When multiple features overlap a single peak, reports all overlapping features separately rather than collapsing to a single result, preserving the complete genomic context.

## Pitfalls

- **Mismatched Genome Builds**: Using a GTF file from a different genome build (e.g., hg19 GTF with hg38 peak coordinates) will result in silent failures where no features are found at expected coordinates, producing empty or incorrect output without error messages.
- **Window Size Too Small**: Setting `--window 100` when analyzing broad RBP binding profiles may miss distal regulatory elements that influence mRNA stability or localization, leading to incomplete context maps.
- **Non-numeric Score Columns**: Providing narrowPeak files where the score column contains non-integer values or is missing will cause parsing errors that halt execution, as the tool expects properly formatted narrowPeak files.
- **Duplicate Peak Entries**: Having duplicate peak rows in the input file leads to duplicate feature annotations in the output, artificially inflating the count of genes associated with each CLIP peak.
- **Missing Strand Information**: Using genome annotation files without strand orientation (column 7 in standard GTF) prevents strand-specific analysis, producing results that cannot distinguish sense from antisense binding events.

## Examples

### Extract genes within 2kb upstream and downstream of each CLIP peak
**Args:** `--peaks peaks.narrowPeak --annotation genes.gtf --window 2000 --output context.tsv`
**Explanation:** This extracts all gene features from the GTF file that fall within 2kb of any CLIP peak center, producing a table useful for initial downstream functional enrichment analysis.

### Get promoter-proximal CLIP binding sites only
**Args:** `--peaks peaks.bed --annotation genes.gtf --window 2000 --feature-type promoter --output promoter_clips.tsv`
**Explanation:** Filters to only report peaks overlapping with promoter regions (typically -2000 to +100 of TSS), identifying RBPs that may regulate transcriptional initiation.

### Generate strand-specific peak-context associations
**Args:** `--peaks peaks.narrowPeak --annotation transcripts.gtf --window 5000 --stranded --output stranded_context.tsv`
**Explanation:** Reports only those genomic features on the same strand as each CLIP peak, which is essential for distinguishing sense mRNA binding from antisense lncRNA binding events.

### Use a larger window to capture distal regulatory elements
**Args:** `--peaks peaks.narrowPeak --annotation genes.gtf --window 50000 --output distal_context.tsv`
**Explanation:** Expands the context window to 50kb to capture distal enhancers, long-range chromatin interactions, and genes under potential indirect RBP regulation.

### Specify exon-only features for splicing factor analysis
**Args:** `--peaks peaks.bed --annotation genes.gtf --window 1000 --feature-type exon --output exon_clips.tsv`
**Explanation:** Restricts output to exon features only, which is ideal for analyzing splicing regulator binding patterns across coding sequences.

### Process multiple peak files in batch mode
**Args:** `--peaks "*.narrowPeak" --annotation genes.gtf --window 3000 --output batch_results/ --batch`
**Explanation:** Uses glob pattern to process all matching peak files in a directory simultaneously, producing individual output files for each input with a batch mode header.