---
name: consent
category: Conservation Analysis
description: Extracts conservation scores and annotations from UCSC multiple alignment format (MAF) data for genomic regions. Part of the UCSC Genome Browser utilities, consent processes alignment data to generate quantitative conservation metrics useful for evolutionary analysis and functional annotation.
tags: [conservation, multiple-alignment, maf, genomic-annotation, evolutionary-analysis, ucsc-tools]
author: AI-generated
source_url: http://hgdownload.soe.ucsc.edu/admin/exe/
---

## Concepts

- consent reads Multiple Alignment Format (MAF) data from stdin or file arguments and computes conservation scores for each position or region based on evolutionary conservation across species alignments.
- The tool operates on genomic coordinates that must match the specified assembly; using mismatched assembly builds produces silently incorrect coordinate mappings in the output.
- Output formats are controlled via command-line options and include WIG (for genome browsers), BED (for genomic intervals), and text-based score reports; the default output streams to stdout in the chosen format.
- consent processes sequences in chunks and respects strand orientation, meaning the same genomic interval on opposite strands can yield different conservation values depending on the alignment orientation.
- The tool filters alignments by species subset and minimum score thresholds before aggregation, allowing focused conservation analysis on user-defined evolutionary lineages.

## Pitfalls

- Providing MAF input with missing or malformed block structure causes consent to skip entire regions without warning, resulting in sparse or empty output files.
- Forgetting to specify the correct `-assembly` (e.g., using `hg38` when the input is `hg19`) silently misaligns all output coordinates against external datasets.
- Specifying an incompatible output format option produces malformed output that downstream tools (like `bedGraphToBigWig` or genome browsers) cannot parse correctly.
- Not accounting for strand orientation when extracting conservation for genes on the negative strand leads to inverted or incorrect score orientation relative to gene annotations.
- Running consent on very short input sequences (fewer than 10 bp) produces unreliable conservation estimates due to insufficient alignment depth for statistical aggregation.

## Examples

### Extract basic conservation scores from a MAF file

**Args:** `-maf=input.maf -out=wifi stdout`
**Explanation:** Reads the MAF alignments from `input.maf`, computes per-position conservation scores, and writes WIG format output to standard output for direct visualization in genome browsers.

### Filter conservation output by minimum score threshold

**Args:** `-maf= alignments.maf -minScore=0.8 -out=wig stdout`
**Explanation:** Only outputs positions where the conservation score meets or exceeds 0.8, producing a high-confidence conservation track suitable for identifying highly conserved elements.

### Generate BED format output for genomic interval analysis

**Args:** `-maf= conservation_data.maf -out=bed stdout`
**Explanation:** Converts conservation scores into BED format, enabling direct import into genome viewers like IGV or UCSC Browser for integrated visualization with other genomic annotations.

### Specify a non-default reference assembly

**Args:** `-maf= alignments.maf -assembly=hg19 -out=wig stdout`
**Explanation:** Forces consent to interpret genomic coordinates as hg19 rather than the default assembly, ensuring correct coordinate system alignment when working with legacy datasets.

### Process conservation for a specific genomic region by chromosome and range

**Args:** `-maf= full_alignment.maf -chrom=chr7 -start=117000000 -end=117500000 -out=wig stdout`
**Explanation:** Restricts conservation analysis to a 500 kb window on chromosome 7, reducing processing time and output size when only a specific gene region is needed for downstream analysis.

### Combine multiple filtering options for focused conservation analysis

**Args:** `-maf= broad_maf.maf -minScore=0.5 -out=bed stdout`
**Explanation:** Applies a lower score threshold while outputting BED format, useful for capturing moderately conserved elements that may represent regulatory regions with intermediate evolutionary constraint.