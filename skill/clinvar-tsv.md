---
name: clinvar-tsv
category: bioinformatics/variant-annotation
description: Command-line tool for extracting, filtering, and converting ClinVar variant data to tab-separated values (TSV) format for downstream bioinformatics analysis.
tags: [clinvar, variant-annotation, genomics, tsv, vcf-conversion, clinical-variants]
author: AI-generated
source_url: https://github.com/ncbi/clinvar-tsv
---

## Concepts

- **ClinVar Data Model**: ClinVar databases contain variant records with fields including SCV (submitted variant caller), RCV (reference clinical variant), clinical significance ( pathogenic, benign, etc.), review status, and associated disease names. Each variant is identified by a unique ClinVar ID (e.g., RCV000123456).

- **Input Formats**: The tool accepts ClinVar XML dumps (full archive releases), VCF files with ClinVar annotations, or pre-parsed TSV files. For batch processing, the recommended input is a tab-delimited file with one variant ID per line in the first column.

- **Output TSV Structure**: The output TSV contains standardized columns: chrom, pos, ref, alt, clinvar_id, clinical_significance, review_status, disease_name, genes, allele_frequency, and publication references. Empty fields are represented as "." to maintain consistent column counts.

- **Filtering Capabilities**: The tool supports filtering by clinical significance (pathogenic, likely_pathogenic, benign, likely_benign, uncertain_significance), review status (criteria_provided, reviewed_by_expert, practice_guideline), and gene symbol. Multiple filters can be combined using AND logic.

- **Companion Binary - clinvar-tsv-build**: This companion tool builds an indexed ClinVar database from the official XML archive for faster queries. Run `clinvar-tsv-build /path/to/clinvar.xml.gz /path/to/clinvar.db` to create the index before running queries.

---

## Pitfalls

- **Using Unindexed XML for Large Queries**: Querying the full ClinVar XML archive without first building an index using `clinvar-tsv-build` results in extremely slow performance (minutes to hours per query). Always build an indexed database for batch operations to reduce runtime from O(n) to O(log n).

- **Mismatched Column Headers**: Specifying custom output columns that don't match the exact ClinVar field names causes silent failures where those columns contain missing data. Use `--list-fields` to see available field names before specifying custom output formats.

- **Ignoring Review Status Thresholds**: Treating all pathogenic variants equally without filtering by review status includes variants with conflicting interpretations or single-submitter reports. Use `--min-review-status reviewed_by_expert` to ensure clinically actionable variants.

- **Incorrect Reference Allele Notation**: Failing to normalize variant notation (e.g., "A" vs "AAA") causes no matches when querying by variant coordinate. Ensure alleles use the same notation as the ClinVar reference database or use `--normalize-alleles` flag.

- **Missing Gene Symbol Mapping**: Not specifying `--include-gene-symbols` when extracting variant data omits gene names, making downstream functional enrichment analysis impossible without re-processing the data.

---

## Examples

### Extract pathogenic variants for a specific gene

**Args:** --gene BRCA1 --clinical-significance pathogenic --min-review-status reviewed_by_expert --output /path/to/brca1_pathogenic.tsv

**Explanation:** This extracts all pathogenic BRCA1 variants with expert review status, suitable for clinical variant interpretation pipelines.

### Convert a VCF file with ClinVar annotations to TSV

**Args:** --input-vcf /path/to/variants.vcf --output /path/output/variants_clinvar.tsv --include-all-fields

**Explanation:** Converts a VCF file containing ClinVar INFO field annotations into a structured TSV format for easier downstream analysis in R or Python.

### Build an indexed ClinVar database from the XML archive

**Args:** /path/to/clinvar.xml.gz /path/to/clinvar.db

**Explanation:** Creates a binary indexed database from the ClinVar XML release for fast subsequent queries. Run once per ClinVar release update.

### Query the indexed database for uncertain significance variants

**Args:** --db /path/to/clinvar.db --clinical-significance uncertain_significance --output /path/to/uncertain.tsv

**Explanation:** Queries the pre-built index for variants of uncertain significance, returning results in seconds rather than minutes.

### Generate a summary report of variant counts by clinical significance

**Args:** --db /path/to/clinvar.db --summary-by clinical_significance --output /path/to/summary.tsv

**Explanation:** Produces a summary TSV showing the count of variants in each clinical significance category, useful for quality control and reporting.