---
name: chromsize
category: genomics-utilities
description: Extracts and formats chromosome/scaffold names and their lengths from a FASTA genome assembly or chromosome size file. Outputs tab-separated, BED, or JSON formats suitable for downstream bioinformatics tools.
tags:
  - genome-assembly
  - chromosome-lengths
  - reference-genomics
  - fasta
  - bed
  - chrom-sizes
author: AI-generated
source_url: https://github.com/samtools/samtools
---

## Concepts

- **Input sources**: chromsize accepts either a FASTA genome assembly file (it scans sequence headers to compute lengths) or a pre-existing chrom.sizes text file where each line contains a chromosome name followed by its length in base pairs.
- **Output formats**: The tool can emit tab-separated values (TSV) with chromosome name and length, strict BED format requiring chr, start, end columns, or JSON for programmatic consumption. Format is controlled by flags such as `--bed` or `--json`.
- **Filtering and sorting**: Users can filter output to specific chromosomes using `--include` or `--exclude` patterns (regular expressions), and sort results by name (`--sort-name`) or by length descending (`--sort-length`).
- **Sequence vs. assembly awareness**: When reading a FASTA input, chromsize treats any line beginning with `>` as a sequence header and calculates the length of the concatenated bases that follow until the next header.

## Pitfalls

- **Feeding FASTA with line-wrap**: If the input FASTA uses sequence lines longer than typical 80-character wrapping, the tool correctly handles it. However, mixing DOS line endings (`\r\n`) with some FASTA files may cause silent length miscalculations on certain builds—use `dos2unix` to normalize first.
- **Sorting by length after filtering**: When both `--sort-length` and `--filter` are used, the sort operation is applied before filtering, which can cause unexpected output ordering if you expected filtered chromosomes to be sorted independently.
- **BED format coordinate system**: The BED output uses zero-based, half-open intervals (start = 0, end = length). Tools expecting one-based coordinates (like certain VCF utilities) will misalign features by one base if not adjusted.
- **Quiet failures on malformed input**: If a chrom.sizes file contains non-numeric values in the length column, chromsize prints the offending line to stderr and exits with code 1 but does not indicate which line number failed—manual inspection is required.
- **Omitting header in TSV mode**: In default TSV mode, no header row is emitted, so downstream tools expecting a `chrom` and `length` column header may incorrectly parse the first chromosome as a column name.

## Examples

### Extract chromosome sizes from a FASTA genome assembly
**Args:** `refgenome.fa`
**Explanation:** When given a FASTA file, chromsize reads all sequence headers, calculates the length of each contig by counting bases between headers, and prints a default tab-separated list of chromosome names and their lengths in base pairs.

### Output chromosome sizes in BED format sorted by chromosome name
**Args:** `refgenome.fa --bed --sort-name`
**Explanation:** The `--bed` flag switches output to BED three-column format (chr, 0, length), and `--sort-name` orders the chromosomes alphabetically, which is required for tools like UCSC Genome Browser that expect sorted BED files.

### Filter to include only chromosomes matching a pattern
**Args:** `chromsizes.txt --include "chr[0-9]+$" --format json`
**Explanation:** The `--include` flag filters to chromosomes matching the given regular expression, and `--format json` emits machine-readable JSON objects per chromosome, useful for integrating with pipelines that consume JSON configuration.

### Compute lengths from a pre-existing chrom.sizes file
**Args:** `chromsizes.txt --sort-length --desc`
**Explanation:** When the input is already a chrom.sizes file (name-length pairs), the tool passes through without recalculation but applies sorting—in this case descending by length so the largest chromosomes appear first for manual inspection or reporting.

### Exclude mitochondria and plasmid sequences from output
**Args:** `refgenome.fa --exclude "chrM|MT|plasmid" --tsv`
**Explanation:** Using `--exclude` with a regex removes unwanted sequences from output, and `--tsv` explicitly requests tab-separated format even though it is the default, making the command self-documenting in workflow scripts.