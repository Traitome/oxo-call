---
name: collect-columns
category: data-manipulation
description: A bioinformatics utility for extracting, reordering, and combining specific columns from tabular data files in common formats such as BED, VCF, CSV, and TSV. Supports column selection by index or header name, header preservation, and output in multiple formats.
tags: [column-extraction, data-filtering, tabular-data, bioinformatics-formats, data-manipulation]
author: AI-generated
source_url: https://github.com/placeholder/collect-columns
---

## Concepts

- `collect-columns` operates on tabular data where rows represent features and columns represent attributes. Column indices begin at 0, so the first column is index 0, the second is index 1, and so forth. When specifying columns by name, exact header matches are required—case sensitivity depends on the input file format.
- Input file formats are auto-detected based on file extension: `.vcf` or `.vcf.gz` for VCF files, `.bed` for BED files, `.csv` for CSV files, and any other extension defaults to TSV (tab-separated values). Files may be gzip-compressed (`.gz` extension) and the tool will transparently handle decompression.
- The tool outputs to stdout by default, allowing piping to other Unix tools or redirection to a file. The output delimiter matches the input delimiter unless overridden with the `--delimiter` flag. Missing columns (empty values) are preserved as empty fields in the output.

## Pitfalls

- Specifying a column index that exceeds the total number of columns in the input file causes the tool to fail with a non-zero exit code and print an error to stderr, producing no output. Always verify column counts using `head -1 input.tsv | tr '\t' '\n' | wc -l` before running `collect-columns`.
- When using header-based column selection on files without headers (such as plain BED files), specifying column names will silently select zero columns and output only the header line (if `--keep-header` is set) or produce an empty file, leading to downstream analysis failures that are difficult to diagnose.
- Combining columns from multiple input files with `collect-columns` requires all files to have the same number of rows in the same order; if row counts differ, the tool completes but outputs fewer rows than expected, silently dropping trailing rows from longer files.

## Examples

### Extract the first three columns from a BED file
**Args:** `input.bed --indices 0,1,2`
**Explanation:** This selects chromosome, start, and end columns from a standard 12-column BED file by their zero-indexed positions, outputting a minimal BED-like file with only the coordinate information.

### Extract a specific column by header name from a VCF file
**Args:** `variants.vcf --columns CHROM,POS,INFO`
**Explanation:** This extracts the CHROM, POS, and INFO columns by their exact header names, preserving the VCF header lines in the output so the resulting file remains valid VCF format.

### Reorder columns in a CSV file with custom output delimiter
**Args:** `data.csv --columns b,c,a --delimiter "," --keep-header`
**Explanation:** This reorders columns so that column 'b' becomes first, 'c' second, and 'a' third, while keeping the header row intact and maintaining comma as the output separator rather than defaulting to tab.

### Combine columns from two TSV files into a single output
**Args:** `file1.tsv file2.tsv --columns 0,2 --columns 1,3 --merge`
**Explanation:** This takes column 0 and 2 from file1.tsv and column 1 and 3 from file2.tsv, concatenating them horizontally so each output row contains four columns derived from both input files.

### Extract columns 5 through 10 from a gzip-compressed expression matrix
**Args:** `expression_matrix.tsv.gz --indices 5-10`
**Explanation:** This uses range notation to select columns at indices 5, 6, 7, 8, 9, and 10 from a large gene expression matrix stored in compressed format, without requiring manual decompression beforehand.

### Remove a single column and output to a new file
**Args:** `input.tsv --exclude 3 --output filtered.tsv`
**Explanation:** This removes the fourth column (index 3) from the input file and writes the result to filtered.tsv, keeping all other columns in their original order. The `--output` flag prevents writing to stdout.

### Extract columns from multiple files and find their intersection
**Args:** `file1.tsv file2.tsv file3.tsv --columns ID,VALUE --intersect`
**Explanation:** This selects 'ID' and 'VALUE' columns from three files, then outputs only rows where the ID value exists in all three input files, performing a set intersection operation on the selected column data.