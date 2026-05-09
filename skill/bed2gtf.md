---
name: bed2gtf
category: format-conversion
description: Converts genomic annotation files from BED format to GTF format, preserving chromosome naming conventions, score fields, and thick/color attributes via custom CLI options.
tags:
  - bed
  - gtf
  - annotation
  - format-conversion
  - genomics
author: AI-generated
source_url: https://github.com/民众民众民众/bed2gtf
---

## Concepts

- `bed2gtf` operates on line-oriented BED files (BED3 through BED12) and emits one GTF record per BED feature. Each output line contains all seven mandatory GTF columns — seqname, source, feature, start, end, score, strand, frame, and attribute — ensuring the result is fully compliant with standard GTF2 parsers.
- BED coordinates are zero-based, half-open (end exclusive), while GTF uses one-based, fully-closed intervals. `bed2gtf` automatically adjusts start and end values during conversion; applying `--zero-based-offset` manually is therefore unnecessary and risks double-shifting coordinates.
- The GTF attributes column is populated differently depending on the input BED version: BED12 entries map their name, thickStart, thickEnd, itemRgb, and blockCount fields to GTF tags such as `gene_id`, `transcript_id`, `color`, and `exon_number`; BED3 entries output minimal attributes using `--gene-id` if provided.
- Chromosome naming is preserved verbatim from the BED input into the GTF seqname column. Tools downstream of `bed2gtf` (e.g., HTSeq, featureCounts) may reject chromosome names that include a `chr` prefix depending on the reference annotation; use `--strip-chr` to remove the `chr` prefix when mixing with non-chr-prefixed reference files.
- Output is written to stdout by default, allowing piping into downstream tools such as `grep`, `sort`, or `gzip` without an intermediate file.

## Pitfalls

- Using `bed2gtf` on a BED file that is not tab-delimited (e.g., space-separated or CSV) causes silent coordinate misalignment, resulting in GTF files where start/end values are read from the wrong columns and all downstream quantification software produces incorrect results.
- Specifying conflicting chromosome-stripping options such as `--strip-chr` alongside `--add-chr` on the same command causes the tool to abort, but only after consuming the input file. Re-running on a large BED file wastes compute time unnecessarily.
- Converting a BED12 file with thick/color attributes using `--no-attributes` discards all phase, color, and thick-region information from the GTF output. If you intend to use this GTF with a tool that requires transcript structure (e.g., Cufflinks/Cufflinks2), the resulting GTF will lack `transcript_id` and `exon_number` tags, causing silent failures in expression quantification.
- Omitting `--gene-id` and `--transcript-id` when processing a multi-annotation BED file results in all GTF lines receiving the same dummy attribute `gene_id "unknown"`, making it impossible to distinguish distinct loci after conversion. Downstream differential expression or overlap tools will treat everything as one transcript.
- Piping output directly to `gzip` without the `--force` flag when the output file already exists causes `bed2gtf` to prompt for confirmation and block the pipeline; scripts must use `--force` or redirect to avoid hanging.

## Examples

### Convert a simple BED3 file to GTF

**Args:** `annotations.bed3 > annotations.gtf`
**Explanation:** The BED3 file contains only chrom, start, and end fields; `bed2gtf` converts them to GTF columns 1–6 using default source "bed2gtf" and feature "gene", writing valid GTF to stdout.

### Convert BED12 to GTF with transcript IDs and exon numbering

**Args:** `--bed12 --transcript-id tx --gene-id gene annotations.bed12 annotations.gtf`
**Explanation:** The `--bed12` flag tells `bed2gtf` to parse the thickStart, thickEnd, itemRgb, and block fields and emit them as GTF tags, while `--transcript-id` and `--gene-id` inject meaningful identifiers so the resulting GTF is compatible with quantification tools like HTSeq or featureCounts.

### Strip chr prefixes from chromosome names during conversion

**Args:** `--strip-chr transcripts.bed > transcripts.ng.gtf`
**Explanation:** Many reference genomes use non-chr chromosome names (e.g., "1" instead of "chr1"); `--strip-chr` removes the `chr` prefix from every seqname so the output GTF seqname column matches the target reference without manual editing.

### Add chr prefixes to output GTF for use with UCSC-style references

**Args:** `--add-chr genes.bed genes.gtf`
**Explanation:** When you need the output GTF to align with a UCSC reference that uses `chr` prefixes, `--add-chr` prepends "chr" to each chromosome name, eliminating coordinate-mismatch errors in downstream genome browsing or overlap tools.

### Convert BED and discard optional GTF attributes to keep output minimal

**Args:** `--no-attributes --force small.bed small.gtf`
**Explanation:** `--no-attributes` prevents emission of the optional ninth column, producing a minimal GTF file with only the seven mandatory columns, which simplifies processing for tools that do not read attributes but requires `--force` to overwrite an existing file.

### Validate a BED-to-GTF conversion by counting feature lines

**Args:** `input.bed | bed2gtf | grep -c "^chr" > count.txt`
**Explanation:** Piping the output directly through `grep -c` counts the number of converted GTF records per chromosome, allowing a quick sanity check that the number of output lines matches the number of input BED features.