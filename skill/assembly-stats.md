---
name: assembly-stats
category: Genome Assembly Analysis
description: Calculates comprehensive statistics for genome assemblies from FASTA files, including N50, L50, contig counts, total length, and GC content metrics.
tags:
  - assembly
  - genomics
  - n50
  - l50
  - contigs
  - scaffolds
  - quality-control
author: AI-generated
source_url: https://github.com/satzhero/assembly-stats
---

## Concepts

- **N50/L50 Metrics**: N50 is the contig length at which 50% of the assembly is contained in contigs of that length or longer. L50 is the number of contigs required to reach 50% of the total assembly length. These metrics are primary indicators of assembly contiguity—higher N50 and lower L50 indicate better assembly quality.
- **FASTA Input Handling**: The tool accepts one or more FASTA files as input, processing each sequence as a separate contig or scaffold. Sequences are parsed by reading header lines (lines starting with `>`) and calculating statistics across all bases within each sequence record. Lines containing only `N` characters are typically excluded from GC content calculations.
- **Output Statistics**: The tool calculates total assembly length (sum of all valid bases), number of contigs, shortest/longest contig lengths, mean and median contig sizes, GC percentage, and N-series metrics (N10 through N100 in 10-unit increments).
- **Multi-file Comparison Mode**: When multiple FASTA files are provided as arguments, assembly-stats generates comparative statistics for each file side-by-side, enabling direct quality comparison between assembly versions or different samples.

## Pitfalls

- **Empty or Malformed FASTA Files**: Providing FASTA files with no valid sequence data (empty files, files containing only header lines, or files with corrupted headers) causes the tool to fail silently or produce undefined metric values. Always validate FASTA integrity with tools like `seqkit stat` before running assembly-stats.
- **Scaffolds with Large N-gaps**: The tool treats scaffolds containing `N` characters as single sequences, which can artificially inflate contig counts and distort N50 calculations when scaffolding is extensive. For accurate contig-level statistics, consider breaking scaffolds at N-gaps using `sed` or `fqextract` before analysis.
- **Memory Consumption with Very Large Assemblies**: Assemblies exceeding several gigabases in size may cause memory allocation errors during sorting operations. Process such assemblies in chunks or use the `--min-length` filter to exclude very short contigs before analysis.
- **Assuming Consistent Output Format Across Versions**: Output column names and ordering may change between tool versions. Scripts parsing specific output columns by index rather than by name will break when upgrading the tool. Always validate output parsing logic after tool updates.

## Examples

### Calculate basic statistics for a single assembly FASTA
**Args:** `assembly.fasta`
**Explanation:** Reads the FASTA file and outputs all available assembly metrics including total length, contig count, N50, L50, GC content, and length distributions for the single provided assembly.

### Generate statistics for multiple assemblies side-by-side
**Args:** `draft_assembly.fasta improved_assembly.fasta`
**Explanation:** Processes both FASTA files and outputs comparative statistics, allowing direct visual comparison of assembly quality metrics between two assembly versions.

### Export statistics in machine-readable format
**Args:** `-t assembly.fasta`
**Explanation:** Outputs statistics in tab-delimited format suitable for parsing by downstream scripts or importing into spreadsheets, rather than the default human-readable display format.

### Calculate statistics excluding very short contigs
**Args:** `-l 1000 my_assembly.fasta`
**Explanation:** Filters out contigs shorter than 1000 base pairs before calculating statistics, producing metrics representative of the usable assembly quality and preventing tiny fragments from skewing N50 and mean calculations.

### Display scaffold-level statistics without breaking at N-gaps
**Args:** `scaffolds.fasta`
**Explanation:** Treats each scaffold as a single contig regardless of internal N-gap characters, providing accurate scaffolding metrics when you need to assess scaffold-level rather than contig-level assembly quality.