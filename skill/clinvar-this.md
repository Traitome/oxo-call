---
name: clinvar-This
category: variant-annotation
description: Query the ClinVar database for clinical significance and annotation data using genomic coordinates, rsIDs, or variant identifiers. Supports multiple genome builds and filtering by review status and pathogenicity classification.
tags: [clinvar, variant-annotation, clinical-significance, genomics, vcf-post-processing]
author: AI-generated
source_url: https://github.com/ClinVar/ClinVar-Tools
---

## Concepts

- **Input Formats**: clinvar-This accepts genomic positions (chr:start-end), rsIDs prefixed with "rs", or HGVS expressions. Positions must use chromosome names matching the declared genome build (e.g., "chr3" for GRCh38, "3" for GRCh37). Mixing builds silently produces wrong annotations.
- **Output Formats**: The tool returns clinical variant data as JSON (default), TSV, or VCF INFO-field annotations. JSON output includes fields: clinical_significance, review_status, conditions, allele_ids, and origin. TSV is preferable for cross-referencing with bedtools or awk pipelines.
- **Genome Build Handling**: clinvar-This tracks ClinVar's GRCh37 and GRCh38 release snapshots independently. Using the wrong build for a given coordinate set shifts positions by ~5 Mb in subtelomeric regions, producing empty results with no warning. Always verify with --genome-build before querying.
- **Filtering Modes**: Pre-filtering with --pathogenic, --likely-pathogenic, --benign, or --likely-benign restricts results to submissions matching the requested interpretation. Without a filter, all submission statuses are returned, including "uncertain" entries that may be unwanted in clinical pipelines.

## Pitfalls

- **Mismatched genome build**: Supplying GRCh37 coordinates without --genome-build hg19 causes silent coordinate failures. The tool defaults to GRCh38, so 37-based queries yield no match even for well-characterized variants.
- **Assuming variant presence**: Not all genomic positions have ClinVar submissions. Queries for intronic or non-coding positions return empty results with exit code 0 — this is not an error and gets missed in batch pipelines without explicit empty-result checking.
- **Using outdated ClinVar release**: clinvar-This bundles a specific ClinVar release date. Querying stale annotations means the clinical significance ratings reflect superseded ACMG criteria. Re-run with --update to fetch the latest release before clinical reporting.
- **Inconsistent chromosome notation**: Using "chr3" in one query and "3" in another within the same session causes inconsistent joins. Pick one notation style and enforce it across all input BED files.
- **Ignoring review stars**: ClinVar's review_status field uses a star system (0–4 stars) indicating evidence strength. Filtering only by pathogenicity without considering review status selects variants with conflicting interpretations from submitters with no track record.

## Examples

### Query a single genomic position for ClinVar annotations
**Args:** chr3:98533341-98533341 --genome-build hg38 --format json
**Explanation:** Queries a precise genomic position on chromosome 3 using GRCh38 coordinates and returns full ClinVar annotations as JSON, including clinical significance and review status.

### Batch query variants from a BED file using rsID input
**Args:** -i variants.bed --query-type rsid --format tsv --output clinvar-results.tsv
**Explanation:** Reads a list of rsIDs from an input BED file and outputs all matched ClinVar entries as a tab-separated file suitable for joining with other genomic datasets using standard UNIX tools.

### Filter to pathogenic variants only and export JSON
**Args:** -i my_variants.tsv --query-type coordinate --pathogenic --format json --out pathogenic_calls.json
**Explanation:** Applies a pathogenicity filter during query so that only variants classified as pathogenic or likely pathogenic are returned, reducing downstream noise in clinical interpretation pipelines.

### Query with a specific ClinVar release date for reproducibility
**Args:** --query "rs12345" --release 2024-01-01 --genome-build hg38 --format json
**Explanation:** Freezes the query to a specific ClinVar release snapshot, ensuring that re-runs of the same pipeline produce identical clinical significance results across different execution dates.

### Update cached ClinVar data before querying
**Args:** --update --query "rs67890" --genome-build hg19 --format tsv
**Explanation:** First downloads the latest ClinVar release and then queries the updated database using GRCh37 coordinates, ensuring annotations reflect the most recent ACMG criteria interpretations.