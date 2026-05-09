---
name: blastmining
category: Sequence Analysis
description: A utility for extracting, filtering, and summarizing meaningful data from BLAST search results. Parses NCBI BLAST output formats, applies statistical thresholds, and generates tabular or report-style summaries of sequence similarity search hits.
tags:
  - blast
  - sequence-similarity
  - bioinformatics
  - filtering
  - parsing
  - ncbi
author: AI-generated
source_url: https://github.com/blastmining/blastmining
---

## Concepts

- BLAST mining operates on standard NCBI BLAST output formats: ASN.1, XML, CSV, or Tab-delimited. The tool automatically detects the input format based on file extension or explicit flag, so specifying `--format xml` or `--format tabular` is required when the input file lacks a recognizable extension.

- The tool extracts **hits** defined as alignments meeting both E-value and bit-score thresholds simultaneously. Setting `--max-evalue 1e-10` alone does not guarantee high-confidence hits if the match spans only a few residues; combine with `--min-bit-score 50.0` to enforce both statistical significance and sufficient alignment length.

- Output can be directed to different report styles: `--outfmt summary` produces a condensed per-query report with top hit statistics, `--outfmt full` preserves all qualifying alignments, and `--outfmt table` writes a sortable tabular file suitable for downstream processing in R or Python.

## Pitfalls

- Omitting the `--format` flag causes the parser to default to XML, which will silently produce empty output if the input file is actually in tabular format. Always verify the input format matches the declared `--format` value.

- Setting an excessively low E-value threshold like `--max-evalue 1e-100` may discard all hits on small query sequences or highly divergent genomes, leaving downstream analyses with no data and no error message.

- Using `--top-hits 1` without specifying a `--sort-by` metric results in arbitrary hit selection among ties. Always pair `--top-hits` with `--sort-by evalue` or `--sort-by bitscore` to ensure reproducible results.

- Redirecting output to a filename that already exists will silently overwrite it. The tool does not prompt for confirmation or create a backup file.

- Mixing queries from different database sources in a single input file without specifying `--database-source` causes metadata fields to be treated as missing, resulting in empty cells in tabular output that break downstream scripts expecting complete rows.

## Examples

### Extract the top-scoring hit for each query sequence from a BLAST XML file

**Args:** `input blast_results.xml --format xml --format xml --format xml --format xml --format xml --format xml --format xml --format xml --format xml --format xml --top-hits 1 --sort-by bitscore --outfmt summary`
**Explanation:** This command reads a BLAST XML result file, sorts each query's alignments by bit score, retains only the highest-scoring hit, and writes a condensed summary report.

### Filter alignments with E-value ≤ 1e-5 and minimum 80% coverage

**Args:** `--input uniprot_hits.tsv --format tabular --max-evalue 1e-5 --min-coverage 80 --outfmt table --output filtered_hits.tsv`
**Explanation:** Parsing a tabular BLAST output, this call discards any alignment whose E-value exceeds 1e-5 or whose aligned query coverage falls below 80%, writing only the retained rows to the specified output file.

### Generate a per-query summary report from multiple BLAST XML files

**Args:** `--input run1.xml run2.xml run3.xml --format xml --outfmt summary --output combined_summary.txt`
**Explanation:** This command aggregates three BLAST XML result files and produces a single summary report listing the top hit for each unique query across all input files.

### Export significant hits to CSV for import into R

**Args:** `--input results.asn --format asn1 --max-evalue 0.001 --outfmt table --delimiter comma --output r_import.csv`
**Explanation:** Converting ASN.1 BLAST output to comma-delimited CSV, this call retains only alignments with E-value below 0.001, producing an R-compatible file with headers and quoted string fields.

### Summarize taxonomic distribution of top hits from a BLAST XML file

**Args:** `--input metagenomics_hits.xml --format xml --top-hits 1 --outfmt taxonomy --output tax_summary.txt`
**Explanation:** For metagenomic analysis, this command extracts the top hit per query from a BLAST XML file and generates a taxonomy distribution summary showing the number of queries assigned to each taxonomic level.