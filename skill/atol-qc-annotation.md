---
name: atol-qc-annotation
category: Bioinformatics/Genomics
description: A quality control tool for genomic annotations that validates format, checks consistency, and generates summary statistics. Used to ensure annotation files (GFF3, BED, GTF) meet required standards before downstream analysis.
tags:
  - annotation
  - quality-control
  - genomics
  - gff3
  - bed
  - validation
author: AI-generated
source_url: https://github.com/atol-project/atol-qc-annotation
---

## Concepts

- **Input formats:** atol-qc-annotation accepts standard genomic annotation formats including GFF3, BED, and GTF. The tool automatically detects the format based on file extension or can be explicitly specified via the `--format` flag.
- **Data model:** Each annotation feature contains mandatory fields (chromosome, start, end, strand) and optional attribute columns depending on format. The tool validates both structural integrity (coordinate validity) and semantic correctness (feature type, gene IDs, transcript IDs).
- **Validation levels:** Three strictness tiers exist: `basic` (required columns and data types), `standard` (adds strand consistency and attribute completeness), and `strict` (enforces naming conventions and cross-feature reference validity).
- **Output reports:** The tool generates JSON, TSV, or HTML reports containing summary statistics (feature counts per chromosome, feature type distributions, attribute completeness rates) and lists of validation failures with line numbers.

## Pitfalls

- **Mismatched coordinate systems:** Failing to specify `--zero-based` when input uses BED format (zero-based start) while expecting one-based coordinates (GFF3/GTF) results in off-by-one errors that silently propagate to downstream analyses.
- **Ignoring strand information in filtering:** Using `--filter-on-strand` without understanding that "." represents "unknown/not stranded" in GTF/GFF3 leads to unintended exclusion of features that should be retained.
- **Memory limits with large files:** Processing annotation files larger than available RAM without specifying `--chunk-size` causes excessive memory consumption and potential system swapping or crashes.
- **Attribute parsing in strict mode:** Running in `--strict` mode with custom attribute naming conventions causes spurious failures because the tool expects standard keys like "gene_id", "transcript_id", "gene_name".

## Examples

### Validate a GFF3 annotation file in basic mode
**Args:** `--input annotations.gff3 --mode basic`
**Explanation:** Runs basic validation checking only that required columns are present and coordinates are valid integers, without checking attribute completeness or naming conventions.

### Generate summary statistics in JSON format
**Args:** `--input annotations.gff3 --output report.json --format json --stats-only`
**Explanation:** Outputs a JSON file containing counts of features by type, chromosome distribution, and attribute coverage rates without performing validation checks.

### Filter features by chromosome and type
**Args:** `--input annotations.gff3 --chromosome chr1 --feature-type gene --output filtered.gff3`
**Explanation:** Extracts only gene features located on chromosome chr1 and writes them to a new GFF3 file, useful for creating chromosome-specific annotation subsets.

### Check annotation in strict mode with verbose output
**Args:** `--input annotations.gff3 --mode strict --verbose --failures-only`
**Explanation:** Runs strict validation enforcing standard attribute names and outputs only the specific validation failures with line numbers for debugging.

### Convert BED to GFF3 with strand normalization
**Args:** `--input features.bed --output features.gff3 --convert --zero-based`
**Explanation:** Converts a BED format file to GFF3, explicitly noting that the input uses zero-based coordinates, ensuring accurate coordinate transformation during conversion.