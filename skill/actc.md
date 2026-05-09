---
name: actc
category: genomics
description: A bioinformatics tool for analyzing and processing assembled contigs from genome assembly pipelines, enabling quality assessment, filtering, and export of contig statistics.
tags: [genomics, assembly, contigs, bioinformatics, sequence-analysis]
author: AI-generated
source_url: https://github.com/example/actc
---

## Concepts

- **Input Format**: `actc` accepts FASTA files containing assembled contigs as primary input, typically output from assembly tools such as SPAdes, Canu, or Flye. The tool parses sequence headers to extract contig identifiers and lengths.

- **Output Formats**: The tool generates multiple output formats including plain text reports (summary statistics), CSV/TSV tables (detailed per-contig metrics), and JSON for programmatic integration. Each output mode is controlled via distinct command-line flags.

- **Filtering Logic**: Contigs can be filtered based on length thresholds, N50 calculations, and ambiguous base counts (N characters). Filtering is applied before statistics computation, and filtered sets can be exported to new FASTA files.

- **Quality Metrics**: The tool computes standard assembly metrics including total length, contig count, maximum contig length, N50/N80/N90 values, GC content percentage, and gap/ambiguity statistics. These are calculated in-memory without requiring external dependencies.

## Pitfalls

- **Invalid FASTA Input**: Providing files with non-standard headers or missing sequence data causes parsing failures. The tool expects well-formed FASTA format; malformed headers result in silent skipping of affected contigs without warning.

- **Memory Constraints with Large Genomes**: For large assemblies (e.g., plant genomes with thousands of large contigs), loading entire sequences into memory can exceed available RAM. Processing fails with memory allocation errors when input exceeds system limits.

- **Threshold Interpretation Errors**: Applying length filters using incompatible units (e.g., specifying base pairs when the tool expects kilobases) produces unexpected results. Always verify the unit expected by each filter flag in the help documentation.

- **Output File Overwrites**: By default, the tool overwrites existing output files without confirmation. Accidentally specifying an existing output path results in data loss of previous analyses.

## Examples

### Calculate basic assembly statistics from a FASTA file
**Args:** input.fasta --stats
**Explanation:** Reads the input FASTA file and computes summary statistics including total length, contig count, and N50 value, displaying them to standard output.

### Filter contigs shorter than 500bp and save to a new file
**Args:** input.fasta --min-length 500 --output filtered_contigs.fasta
**Explanation:** Applies a minimum length filter to exclude all contigs shorter than 500 base pairs, writing the retained contigs to the specified output FASTA file.

### Export per-contig metrics to a CSV file
**Args:** input.fasta --csv metrics.csv
**Explanation:** Generates a comma-separated values file containing detailed metrics for each contig including length, GC content, and ambiguous base count.

### Compute N80 value instead of default N50
**Args:** input.fasta --stats --n50-mode N80
**Explanation:** Calculates the N80 stat instead of the standard N50, representing the contig length at which 80% of the total assembly length is contained.

### Generate JSON output for programmatic parsing
**Args:** input.fasta --json assembly_stats.json
**Explanation:** Outputs statistics in JSON format, enabling integration with automated pipelines and scripts that require structured data instead of human-readable text.