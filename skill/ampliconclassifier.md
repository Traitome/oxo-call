---
name: ampliconclassifier
category: Metagenomics / Taxonomic Classification
description: A bioinformatics tool for classifying amplicon sequencing reads (16S rRNA, ITS, functional genes) against curated reference databases to assign taxonomic identity and confidence scores.
tags: [amplicon, classification, taxonomy, 16S, ITS, metagenomics, microbial]
author: AI-generated
source_url: https://github.com/biobakery/ampliconclassifier
---

## Concepts

- **Input Format**: Accepts FASTA or FASTQ files containing amplicon sequences (e.g., 16S rRNA gene V4-V5 region, ITS1 region). Sequence headers must be unique; duplicate identifiers cause parsing errors in output files.
- **Reference Database**: Requires a pre-built classification database in the tool's native format (typically created with the companion `ampliconclassifier-build` binary). Database must match the target marker gene (e.g., Silva for 16S, UNITE for ITS).
- **Classification Output**: Produces a tab-delimited results file with columns: sequence_id, taxonomy层级 (Kingdom/Phylum/Class/Order/Family/Genus/Species), confidence_score. Optionally exports a BIOM-format matrix for downstream diversity analysis.
- **Confidence Thresholding**: By default assigns all classifications; use `--min-confidence` to filter low-confidence assignments (recommended ≥0.5 for genus-level, ≥0.7 for species-level).
- **Parallelization**: Supports multi-threading via `--threads`; scales linearly up to the number of CPU cores available, reducing runtime for large datasets (10,000+ sequences).

## Pitfalls

- **Mismatched Database and Marker Gene**: Using a 16S rRNA database to classify ITS sequences leads to pervasive misclassification (false positives at high taxonomy ranks). Always verify database marker gene matches input amplicons.
- **Excessive Memory Usage with Large Databases**: Default database loading loads the entire reference taxonomy into RAM. Large databases (500,000+ sequences) can exhaust available memory on systems with ≤8GB RAM, causing crashes. Use `--database-index` to enable on-disk indexing.
- **Low-Quality Input Sequences**: Unfiltered raw amplicon reads containing adapter contaminants, chimera, or ambiguous bases (N) produce spurious classifications. Always run quality control (trimming, chimera detection) before classification.
- **Ignoring Classification Confidence**: Treating all output classifications as equally reliable produces misleading diversity estimates. Low-confidence assignments incorrectly inflate species richness in downstream alpha diversity analysis.

## Examples

### Classify 16S rRNA amplicon sequences against a Silva database

**Args:** `--query sequences.fasta --database silva_db --output classification.tsv --format tsv`
**Explanation:** Directly classify FASTA input against a pre-built Silva database, exporting results as a tab-delimited file suitable for downstream statistical analysis in R or Python.

### Batch classify multiple FASTQ files with 8 threads

**Args:** --query-dir ./fastq_files --database gtdb_db --output-dir ./results --threads 8 --format tsv
**Explanation:** Process all FASTQ files in a directory using 8 CPU threads for parallel classification, writing individual result files to the specified output directory.

### Filter results to high-confidence genus-level classifications

**Args:** --query input.fasta --database unite_db --output filtered.tsv --min-confidence 0.5 --rank Genus
**Explanation:** Apply a minimum confidence threshold of 0.5 and output only genus-level taxonomy, reducing noise from spurious classifications in ITSfungi analysis.

### Export classification results in BIOM format for QIIME2

**Args:** --query amplicons.fasta --database silva_db --output qiime2_biom.biom --format biom
**Explanation:** Export classification results in BIOM (Biological Observation Matrix) format, enabling direct import into QIIME2 for diversity metrics and statistical testing.

### Build a custom reference database from user sequences

**Args:** --sequences custom_refs.fasta --taxonomy custom_tax.txt --output custom_db --build
**Explanation:** Use the built-in construction mode to create a custom classification database from user-provided reference sequences and tab-delimited taxonomy files.

### Run classification with verbose logging for debugging

**Args:** --query test.fasta --database test_db --output debug.tsv --verbose --log-file classifier.log
**Explanation:** Enable verbose output and write detailed logging to a file for troubleshooting classification failures or unexpected taxonomy assignments during pipeline development.

### Classify and generate a krona interactive visualization

**Args:** --query reads.fasta --database silva_db --output krona.html --visualize krona --rank Family
**Explanation:** Generate an interactive Krona chart showing taxonomic distribution at the Family level, useful for exploratory data visualization and reports.