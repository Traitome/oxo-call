---
name: alfa
category: Genome Assembly Evaluation
description: Computes assembly metrics (N50, L50, contig statistics) from FASTA files to evaluate and compare genome assemblies.
tags: [assembly-quality, contig-stats, n50, fasta]
author: AI-Generated
source_url: https://github.com/mahui-liu/ALFA
---

## Concepts

- Alfa processes one or more FASTA files containing assembled contigs/scaffolds and calculates summary statistics including N50, L50, NG50, total assembly length, number of contigs, and longest/shortest contig lengths.
- Input FASTA files are specified directly as positional arguments; the tool reads all sequences from each file and computes metrics across the combined assembly.
- Output is printed to standard output in a tab-separated or formatted text table, with one row per input assembly file enabling side-by-side comparison.
- The tool automatically sorts contigs by length internally (descending) before computing cumulative metrics, which is essential for correct N50 and L50 calculations.
- Alfa is typically used post-assembly to assess quality, compare different assembler outputs, or track assembly improvements across versions.

## Pitfalls

- Providing empty or malformed FASTA files produces no metrics and exits silently with a zero exit code, making failures easy to miss in automated pipelines.
- Using the same output filename for multiple runs without clearing previous results will concatenate outputs, corrupting downstream analysis tables.
- Specifying non-existent or unreadable input files does not produce a clear error message but instead prints partial output or nothing, requiring manual verification of file accessibility.
- Mixing FASTA and FASTQ input files causes parsing errors and partial metric output since Alfa expects strictly FASTA-formatted sequences with '>' headers.
- On very large assemblies (>10^7 contigs), the runtime increases significantly; running without redirecting stdout to a file may cause output truncation in terminal buffers.

## Examples

### Compute metrics for a single assembly
**Args:** ` scaffolds.fasta`
**Explanation:** Passing a single FASTA file as a positional argument computes and outputs all assembly statistics for that one assembly.

### Compare two assembly versions
**Args:** ` assembly_v1.fasta assembly_v2.fasta`
**Explanation:** Providing two FASTA files produces a side-by-side comparison table, making it easy to identify which version improved N50 or reduced contig count.

### Save metrics to a file for tracking
**Args:** ` assembly.fasta > metrics.tsv`
**Explanation:** Redirecting standard output to a file captures the tab-separated metrics for logging, version control, or automated threshold checks.

### Compute metrics for all assemblies in a directory
**Args:** ` *.fasta`
**Explanation:** Using shell glob expansion feeds all matching FASTA files, useful when processing a batch of assembler runs from a single project directory.

### Use with a named output file to avoid overwrite
**Args:** ` assemblies/*.fasta > run_2024_metrics.txt`
**Explanation:** Redirecting to a uniquely named file prevents data loss and enables reproducible record-keeping across multiple evaluation runs.