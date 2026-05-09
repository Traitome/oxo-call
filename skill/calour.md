---
name: calour
category: Bioinformatics / Microbiome Analysis
description: Calour is a Python library and command-line tool for interactive microbiome data analysis, featuring heatmap visualisation, clustering, and machine-learning-based biomarker discovery from 16S rRNA and shotgun metagenomics data.
tags:
  - microbiome
  - 16S rRNA
  - metagenomics
  - heatmap
  - biomarker-discovery
  - clustering
  - python
author: AI-generated
source_url: https://github.com/amnona/calour
---

## Concepts

- Calour represents microbiome abundance data as an `Experiment` object with sample rows by feature columns (e.g., OTUs or gene families), storing an accompanying data frame of sample metadata and feature annotations, making it straightforward to filter, transform, and annotate subsets in place.
- It reads directly from BIOM (JSON/HDF5), QIIME2 Artifact API (`.qza`), TSV, and CSV formats, and exports processed tables as BIOM or as tab-delimited text, preserving sample and feature metadata through all conversions.
- The core analysis pipeline follows the pattern: import data → add annotations → filter outliers → normalise (e.g., rarefaction or CPM) → group/cluster → machine-learning classifier (e.g., L1-regularised logistic regression or random forest) → export results or interactive heatmap.
- Interactive HTML heatmaps are generated with D3.js and expose click-to-drill-down on taxa, samples, or metadata groups; static PNG/SVG exports are also supported for reproducible reports.
- Calour's `--sortmerna` option performs in-silico rRNA removal and maps reads to a reference database before creating the Experiment object, which is a common preprocessing step for shotgun metagenomics.

## Pitfalls

- Applying normalisations such as TSS (Total Sum Scaling) or rarefaction after aggressive filtering can unexpectedly shift sample ranks, leading to misleading downstream beta-diversity statistics.
- When importing QIIME2 artifacts with `calour --qiime2`, any hidden metadata columns (e.g., those excluded by `--pct-inflate` or set to `discriminative`) are silently dropped, which may cause the classifier to miss key confound variables.
- Exporting an Experiment with mismatched sample metadata (e.g., after column subsetting) will write a table where row order no longer corresponds to the original sample IDs, breaking reproducibility unless `--sample-id-file` is used.
- Specifying an `--outdir` that does not yet exist will raise an `OSError` on POSIX systems unless the `-p` flag is used first; always create the directory proactively to avoid pipeline interruption.
- Calour's clustering methods assume compositional data properties; running Euclidean-distance-based clustering on raw counts without first log-transforming them can produce spurious groupings due to the mean-variance dependency.

## Examples

### Annotate a BIOM file with taxonomic labels from a reference database
**Args:** `annotate --experiment fishers_exact_test.jobs/fisch.dbm --database tax_asv.db --cutoffs 0.05,10 -o annotated_fisch.db`
**Explanation:** This aligns each ASV sequence against the reference database and attaches taxonomy strings (e.g., Genus, Phylum) to the feature axis, enabling downstream phylum-level grouping.

### Filter low-abundance samples and rarefy to an even depth, then save the processed table
**Args:** `filter --jobs rarefaction.jobs/rare.job --min_reads 500 --min_samples 3 --normalize total --subsample 10000 --outfmt biom -o rarefied_filt.db`
**Explanation:** The pipeline removes samples with fewer than 500 total reads, then rarefies the remaining samples to 10,000 reads each, creating a balanced dataset for beta-diversity comparisons.

### Identify discriminating taxa between control and treatment groups using L1-regularised logistic regression
**Args:** `classify --jobs class_lr.jobs/lr.job --method lr_l1 --top 20 --cv 5 --alpha 0.01 --out results.txt`
**Explanation:** The classifier uses five-fold cross-validation to select up to 20 taxa whose abundance profiles best separate the two groups, outputting importance scores for biomarker reporting.

### Generate an interactive HTML heatmap of genus-level abundances with sample metadata colors
**Args:** `heatmap --experiment genus_heat.db --annotation genus_tax.tsv --combine tax_level:genus --metadata_color sample_type:group --html out_heat.html`
**Explanation:** The resulting heatmap colours columns by sample type (e.g., control vs. disease) and rows by genus, allowing rapid visual inspection of differential abundance patterns.

### Remove rRNA reads from shotgun metagenomics data and map to a reference database
**Args:** `sortmerna --input shot_reads.fq.gz --ref SILVA_SSU Parc.fasta --aligned --other --paired_in --output shot_norrna.fq.gz`
**Explanation:** Sortmerna aligns each read against the SILVA rRNA database, discards the aligned rRNA fraction, and keeps the remaining non-rRNA reads in interleaved FASTQ format for taxonomic profiling.