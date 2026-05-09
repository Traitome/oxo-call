---
name: contammix
category: Quality Control / Contamination Detection
description: A likelihood-based tool for detecting and quantifying cross-sample contamination in sequencing datasets by modeling sequence read proportions across multiple reference databases.
tags:
  - contamination
  - quality-control
  - read-filtering
  - mixture-model
  - fastq
  - sequencing
author: AI-Generated
source_url: https://github.com/bioinformatics-tools/contammix
---

## Concepts

- Contammix models read assignments as a mixture distribution across user-supplied reference databases, where the relative proportion of reads assigned to each database reveals the contamination profile. Reads are aligned (or queried) against all provided databases simultaneously, and the tool infers contamination fractions using maximum likelihood estimation (MLE).
- Input files are standard FASTQ/FASTA files for query reads, and FASTA files for each reference database. The tool accepts both single-end and paired-end libraries; for paired-end data, reads are evaluated as mate pairs to improve assignment confidence.
- Output consists of a contamination report (per-database fraction estimates with standard errors and confidence intervals), a summary table in TSV/CSV format, and optionally filtered FASTQ files where suspected contaminated reads are removed or tagged.

## Pitfalls

- Specifying the wrong database order or omitting required references causes the mixture model to assign reads to the wrong source, producing systematically biased contamination estimates that are difficult to detect without manual validation.
- For paired-end data, providing mismatched or out-of-order FASTQ files leads to read pair mismatches and the tool discarding or misclassifying a large fraction of reads, inflating the estimated discard rate.
- Using an excessively large number of reference databases (>20) causes memory exhaustion and dramatically slows convergence of the MLE optimizer, resulting in failed runs or unreliable estimates; partition databases logically or run sequentially.
- Failing to set the `--min-mapping-quality` threshold appropriately allows ambiguous or multi-mapping reads to skew the mixture proportions, particularly when references share homologous sequences.
- Omitting the `--library-type` flag for Illumina vs. Nanopore data results in suboptimal parameter initialization, as the tool applies different error models and read-length priors for each sequencing platform.

## Examples

### Running basic contamination detection with two reference databases
**Args:** `query_reads.fastq.gz --reference healthy_ref.fa --reference contaminants.fa --output-dir results/`
**Explanation:** This runs contammix in default single-end mode, aligning reads against both databases and estimating the fraction belonging to each, outputting results into a subdirectory.

### Detecting contamination in paired-end Illumina data with quality filtering
**Args:** `--pe R1.fastq.gz R2.fastq.gz --ref-db human_db.fa --ref-db bacterial_db.fa --library-type illumina --min-read-length 50 --min-mean-quality 25`
**Explanation:** The paired-end flag ensures read pairs are evaluated jointly, and the quality thresholds discard low-quality reads before mixture modeling to improve estimate accuracy.

### Generating filtered FASTQ with contaminated reads removed
**Args:** `sample.fastq --ref-db pure_culture.fa --ref-db human_contamination.fa --output-filtered clean_filtered.fastq --remove-contaminated`
**Explanation:** The filtered output removes reads assigned to the human_contamination database above the likelihood threshold, producing a decontaminated FASTQ for downstream analysis.

### Adjusting mapping stringency to reduce multi-mapping ambiguity
**Args:** `reads.fastq --ref-db database1.fa --ref-db database2.fa --min-mapping-quality 30 --max-multimaps 1`
**Explanation:** Raising the mapping quality cutoff and limiting multimapping reads to unique best hits forces more conservative assignments, reducing bias from reads matching multiple databases.

### Exporting contamination estimates as a TSV table for downstream reporting
**Args:** `--input all_samples/*.fastq --ref-db pooled_refs.fa --output-table contamination_estimates.tsv --format csv`
**Explanation:** Batch processing multiple FASTQ files against a single combined reference produces a tabular report of contamination fractions across all samples for quality control dashboards.

### Handling Nanopore long-read data with platform-specific error model
**Args:** `nanopore_reads.fastq --ref-db assembly_refs.fa --library-type nanopore --min-read-length 500 --output contamination_report.txt`
**Explanation:** Specifying the Nanopore library type activates the appropriate error model and longer read-length prior, ensuring accurate mixture estimation for high-error-rate long-read sequencing data.