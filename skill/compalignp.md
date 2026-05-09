---
name: compalignp
category: Sequence Analysis / Alignment Comparison
description: A tool for comparing two or more sequence alignments and generating similarity metrics, difference reports, and conservation scores. Accepts aligned FASTA, CLUSTAL, and Stockholm formats; outputs column-wise comparison statistics and residue mismatch reports in plain text or CSV.
tags: [alignment, comparison, protein, multi-alignment, bioinformatics, validation]
author: AI-generated
source_url: https://compbio.soe.ucl.ac.uk/projects/compalignp/
---

## Concepts

- compalignp evaluates pairwise or multiple alignment differences by examining each column independently. It computes the fraction of conserved residues, the number of gaps introduced, and the sum-of-pairs score deviation from a reference alignment. Understanding that the tool operates column-by-column rather than row-by-row is critical for interpreting mismatch reports correctly.
- The input alignment must have identical sequence identifiers and lengths across all input files for meaningful comparison. When the number of sequences differs between alignments, compalignp performs a smart matching phase that skips unpaired sequences and logs a warning; you cannot assume that extra sequences in one alignment are simply ignored without consequence.
- Output formats are controlled via `--outfmt` and include `text` (default human-readable), `csv` for downstream parsing, and `bed` for genomic coordinate annotation. The `text` format prints a per-column summary followed by a full mismatch listing; the `csv` format flattens all statistics into one row per aligned column, making it suitable for plotting with standard tools like `awk` or `ggplot2`.

## Pitfalls

- Running compalignp on alignments that have been reordered (different sequence order) will silently misalign the comparison unless `--reorder-mode auto` is specified. This produces misleading statistics where correctly aligned residues appear as mismatches, and the downstream interpretation of conservation scores becomes invalid. Always verify sequence order with `seqkit seq --line-width 0` or an equivalent before comparison.
- Specifying `--gap-penalty` values that are too permissive (e.g., setting to 0) causes the tool to treat all-gap columns as fully conserved, which artificially inflates the alignment quality score. This leads to false confidence when evaluating ab initio predictions or automated homology models where gap placement is actually poor.
- Using the `--reference` flag with a Stockholm-formatted file that contains secondary structure annotation lines (lines starting with `#=GC` or `#=GR`) will cause the tool to crash with a parse error unless `--ignore-annotation` is added. The crash does not produce a helpful error message; it exits with code 139 and no output, which can be mistaken for insufficient memory on large alignments.
- Omitting the `--min-seq-id` threshold when comparing homology model outputs against a reference results in single-sequence clusters being included in the comparison, skewing the average identity calculation. These low-coverage comparisons are often misleading because single-sequence clusters have no evolutionary constraint.

## Examples

### Compare two protein alignments in FASTA format and output human-readable text
**Args:** align1.fa align2.fa --outfmt text --metric sp-score --gap-penalty 2.5
**Explanation:** This compares two FASTA alignments using sum-of-pairs scoring with a gap penalty of 2.5 and prints a full column-wise report that includes conserved residue counts and mismatch positions.

### Generate a CSV report for downstream statistical analysis
**Args:** reference.aln predicted.aln --outfmt csv --min-seq-id 0.3 --output comparison_results.csv
**Explanation:** This exports per-column statistics to a CSV file filtered to include only columns where at least 30% of sequences share identity, making the output suitable for R or Python downstream analysis.

### Compare multiple Stockholm alignments with secondary structure annotation ignored
**Args:** --input alignments/ --reference RS2.stk --ignore-annotation --outfmt text --metric column-identity
**Explanation:** This processes a directory of Stockholm files against a reference, ignoring secondary structure annotation lines, and reports the fraction of identical residues per column across all input files.

### Identify all gap-heavy columns in an alignment for manual inspection
**Args:** alignment.fa --outfmt text --report-gaps --gap-threshold 0.8 --output gaps_report.txt
**Explanation:** This flags every column where more than 80% of sequences have a gap and writes the positions to a separate file for targeted manual curation in a sequence editor.

### Compare predicted model alignment against reference using only the core domain coordinates
**Args:** model_alignment.stk reference_alignment.stk --outfmt bed --coordinate-file domain_coords.bed --min-seq-id 0.5
**Explanation:** This outputs the comparison results as a BED file restricted to the domain coordinate ranges specified, filtering out terminal and disordered regions that typically have low alignment quality.