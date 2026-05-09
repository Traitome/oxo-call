---
name: aquamis
category: metagenomics
description: A bioinformatics pipeline for analyzing aquatic microbiome sequencing data (FASTQ) to generate taxonomic profiles and abundance estimates.
tags: [metagenomics, microbiome, sequencing, fastq, taxonomy, aquatic]
author: AI-generated
source_url: https://github.com/aquamis/aquamis
---

## Concepts

- **Input Formats**: Aquamis accepts raw FASTQ files (single-end or paired-end), supporting gzipped (.gz) compression. Paired-end data must be provided as two files with explicit `_1` and `_2` naming conventions.
- **Data Model**: The pipeline processes reads through QC filtering, host depletion (optional), taxonomic classification using specified databases, and abundance aggregation at user-defined taxonomic ranks.
- **Output Formats**: Primary outputs include a taxonomic profile (CSV/TSV) with read counts and relative abundances, a JSON summary report, and a biome report with diversity metrics.
- **Pipeline Stages**: Four main stages—`qc` (quality control), `filter` (read filtering), `classify` (taxonomic assignment), and `profile` (abundance calculation)—can be run individually or as a full workflow.

## Pitfalls

- **Mismatched paired-end files**: Providing files that are not properly named or ordered causes the pipeline to treat data as single-end, resulting in missing read pairs and biased abundance estimates.
- **Insufficient disk space for intermediate files**: The classify stage writes large temporary databases and alignment files; running on a full disk causes partial output corruption without clear error messages.
- **Database version mismatch**: Using an older taxonomic database with newer classification software causes inconsistent taxonomy IDs, leading to silent failures in the profile aggregation step.
- **Memory limits during classification**: Large read sets with insufficient RAM cause the classification stage to crash mid-process; always specify `--memory` to match available resources.

## Examples

### Run complete pipeline on single-end FASTQ
**Args:** `run --input sample.fastq.gz --output results/ --db database/`
**Explanation:** Runs all four pipeline stages in one command on single-end reads, writing outputs to the specified directory using the provided taxonomy database.

### Run only quality control on paired-end data
**Args:** `run --input sample_1.fastq.gz --input sample_2.fastq.gz --output qc_results/ --stage qc --db database/`
**Explanation:** Performs quality control on both read files, skipping filtering and classification to quickly assess read quality before full analysis.

### Generate taxonomic profile at genus rank
**Args:** `profile --input classification_results.json --output genus_profile.tsv --rank genus --db database/`
**Explanation:** Aggregates classification results to genus level, producing a tab-separated table with read counts and relative abundances per genus.

### Filter reads by minimum quality score
**Args:** `filter --input raw_reads.fastq.gz --output filtered_reads.fastq.gz --min-quality 20 --min-length 50 --db database/`
**Explanation:** Removes reads with average quality below 20 and discards sequences shorter than 50 bases, improving downstream classification accuracy.

### Specify memory limit for classification stage
**Args:** `classify --input filtered_reads.fastq.gz --output classification.json --db database/ --memory 32GB`
**Explanation:** Allocates 32GB of RAM for the classification stage, preventing out-of-memory errors when processing large datasets.