---
name: convert_zero_one_based
category: coordinate-conversion
description: Converts genomic coordinates between 0-based (half-open) and 1-based (inclusive) coordinate systems. Essential for interoperability between BED/UCSC (0-based) and GFF/GTF/VCF (1-based) formats.
tags:
  - coordinate-systems
  - format-conversion
  - genomics
  - 0-based
  - 1-based
  - half-open
  - interval-conversion
  - bioinformatics-io
author: AI-generated
source_url: https://github.com/enameh1/convert_zero_one_based
---

## Concepts

- **Two coordinate systems**: 0-based (half-open) treats genomic positions as offsets from the chromosome start — position 0 is the first base. 1-based (inclusive) counts from 1 — the first base is position 1. Converting from 0→1 requires adding 1 to the start coordinate; converting from 1→0 requires subtracting 1 from the start coordinate.
- **Output formats**: The tool accepts `--bed` (0-based output) and `--gff` (1-based output) flags. If neither is specified, the tool may auto-detect the system from the input format or prompt for a mode selection. Always confirm the target system before downstream use.
- **Single-point vs. interval handling**: When converting a single genomic position (zero-length interval), the result is unambiguous. When converting a range, the start coordinate shifts but the end coordinate stays the same in half-open convention (e.g., range [5, 10) in 0-based = positions 6–9 in 1-based).
- **Input file formats**: BED files use 0-based coordinates; GFF/GTF files use 1-based coordinates. Mixing these conventions without explicit conversion is a major source of off-by-one errors in genome analysis pipelines.
- **Chromosome naming**: Some genomes use `chr1` (UCSC style) while others use `1` (Ensembl style). The tool preserves the chromosome prefix from the input; a mismatch with the target genome annotation can cause silent failures.

## Pitfalls

- **Omitting the direction flag**: Using `--bed` when you need 1-based output (or vice versa) silently inverts the meaning of your coordinates, shifting every annotation by one base pair — this can corrupt variant calls, gene models, and peak calls in downstream analyses.
- **Assuming the input system matches the desired output**: If you read coordinates from a VCF or GFF file without specifying `--bed`, the tool may treat them as 0-based and produce incorrectly shifted output, leading to misaligned reads or variants in downstream steps.
- **Off-by-one errors on range boundaries**: Subtracting 1 from the start position when converting 1→0 is correct, but subtracting 1 from the end position as well produces a range one base shorter than intended, which can cause truncated features or missing peak overlaps.
- **Converting single positions as ranges**: Passing a single genomic coordinate as a start–end pair where end equals start produces an empty interval in 0-based (valid for BED), but a zero-length or undefined interval in 1-based — the tool may reject or mishandle this edge case.
- **Chromosome prefix mismatches**: Converting `chr1` coordinates against an Ensembl-style genome annotation (without `chr` prefix) results in unmapped entries; the tool may silently drop rows or produce errors depending on strict-mode settings.

## Examples

### Convert a single genomic position from 1-based to 0-based for BED input
**Args:** `1-based_position --bed`
**Explanation:** Specifying `1-based_position` with the `--bed` flag ensures the tool outputs a 0-based coordinate, which is compatible with BED format requirements where the first base is position 0.

### Convert a genomic range from 1-based (GFF) to 0-based (BED) format
**Args:** `start_position end_position --bed`
**Explanation:** Passing both `start_position` and `end_position` along with `--bed` shifts the start by −1 while leaving the end unchanged, producing a correctly formatted BED interval.

### Batch-convert a BED file to 1-based GFF coordinates
**Args:** `input.bed --gff`
**Explanation:** Using `--gff` with an input file applies the 1-based convention to all intervals in the file, converting the entire BED dataset to GFF/GTF-compatible output in a single pass.

### Convert chromosome coordinates from 0-based to 1-based for variant annotation
**Args:** `chr1:500-600 --gff`
**Explanation:** The `--gff` flag tells the tool to output positions in 1-based inclusive format, which is required by VCF and most variant callers for correct variant representation.

### Convert an entire coordinate file and save the output
**Args:** `coordinates.tsv --bed --out converted_output.bed`
**Explanation:** Combining `--bed` with `--out` redirects the converted 0-based coordinates to a named output file, enabling pipeline integration without manual file redirection.

### Convert multiple coordinate pairs from 1-based to 0-based in one command
**Args:** `chr1:1200-1400 chr2:5000-5200 --bed`
**Explanation:** Passing multiple `chr:start-end` arguments with `--bed` processes all intervals sequentially, converting each from 1-based to 0-based while preserving chromosome labels and order.