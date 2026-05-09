---
name: bed2gff
category: format-conversion/genomics
description: Converts genomic annotation files from BED (Browser Extensible Data) format to GFF (General Feature Format). Handles the coordinate system difference (BED is 0-based, GFF is 1-based) and maps variable BED columns to standardized GFF columns.
tags:
  - bed
  - gff
  - format-conversion
  - genomics
  - annotation
  - coordinate-systems
  - ucsc
author: AI-generated
source_url: https://github.com/johejo/bed2gff
---

## Concepts

- **Coordinate system translation**: BED format uses 0-based half-open intervals (start inclusive, end exclusive), while GFF uses 1-based fully-closed intervals. The tool automatically increments BED start coordinates by 1 and leaves end coordinates unchanged when converting to GFF.
- **Flexible column mapping**: BED files can have 3 to 12+ columns, but only the first 12 are standardized. The tool maps BED columns to GFF columns as follows: chrom→seqid, start+1→start, end→end, name→attribute, score→score, strand→strand. Fields without direct GFF equivalents (itemRgb, blockSizes, blockStarts) are encoded as GFF attributes.
- **GFF attribute encoding**: BED optional fields that lack GFF column counterparts—such as thickStart, thickEnd, itemRgb, blockCount, blockSizes, and blockStarts—are appended as semicolon-delimited key=value pairs in the GFF attributes column (column 9).
- **Strand normalization**: BED allows strand values of '+', '-', and '.' (no strand). The tool preserves these values directly in the GFF strand column (column 7), since both formats use the same notation.

## Pitfalls

- **Off-by-one coordinate errors**: If you manually convert BED to GFF without adjusting coordinates, all features will be shifted one base upstream. This causes annotations to misalign with the reference sequence, potentially affecting downstream analyses like variant calling or motif finding.
- **Missing required fields in input BED**: BED files with fewer than 3 columns will cause conversion failures. If column 4 (name) is absent, the tool may leave the GFF attribute field empty or use a placeholder, making feature identification difficult in downstream processing.
- **Incorrect handling of itemRgb color values**: The itemRgb field uses comma-separated RGB values (e.g., "255,0,0" for red) in BED but has no direct GFF column. The tool encodes this in attributes, but some GFF parsers expect hexadecimal color codes, causing compatibility issues with visualization tools.
- **Block-based features losing structure**: For BED12 format entries (genes with exons), the blockSizes and blockStarts fields define exon boundaries within the feature. Converting without preserving these as structured attributes may collapse gene models to single-exon features in downstream tools.

## Examples

### Convert a basic 3-column BED file to GFF

**Args:** `input.bed > output.gff`

**Explanation:** This minimal conversion uses only the required BED columns (chrom, start, end) and maps them to the first three GFF columns plus a default attribute field, producing a valid but minimally annotated GFF file.

### Convert a BED6 file with name and score

**Args:** `annot.bed -o annotated.gff`

**Explanation:** The `-o` flag specifies the output filename while the tool maps the name (column 4) to the GFF attribute column and score (column 5) to the GFF score column, preserving feature identity and confidence information.

### Convert multiple BED files and merge output

**Args:** `*.bed > combined.gff`

**Explanation:** Shell globbing passes all matching BED files sequentially, and the tool appends converted records to stdout in GFF format, creating a merged annotation file suitable for genome-wide analyses.

### Strip comments and convert with strand information

**Args:** `transcripts.bed --strip-comment > clean.gff`

**Explanation:** The `--strip-comment` flag removes any header lines or comments from the BED input before conversion, ensuring the output contains only valid feature records with strand information preserved in column 7.

### Convert with verbose logging for debugging

**Args:** `features.bed -v > output.gff 2> debug.log`

**Explanation:** The `-v` flag enables verbose output that reports conversion statistics and any warnings about unmapped fields, which is useful for diagnosing data quality issues in large annotation sets.