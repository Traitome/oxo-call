---
name: ccsmeth
category: Epigenetics / Modified Base Detection
description: A bioinformatics tool for detecting DNA and RNA modifications (especially m6A, m4C, m5C) from Oxford Nanopore sequencing data using signal-based analysis and machine learning models.
tags:
  - nanopore
  - epigenetics
  - modified bases
  - m6a
  - basecalling
  - single-molecule
  - dna-methylation
  - rna-methylation
author: AI-generated
source_url: https://github.com/genomicinformatics/ccsmeth
---

## Concepts

- **Signal-based modification detection**: CCsmeth analyzes the raw nanopore current signals surroundingcandidate bases, extracting features from the signal window (typically ±5-10 bases from the modification site) to predict modification states using trained models.
- **Input requirements**: CCsmeth requires alignedOxford Nanopore sequencing data in coordinate-sorted and indexed BAM/CRAM format, along with the reference genome in FASTA format. The alignments must retain original signal information (typically FASTQ or POD5 input aligned with minimap2 or ngmlr).
- **Output formats**: Modification calls are output in modified BED format (or optionally VCF), with per-read modification probabilities. Each call includes genomic position, modification type, read-level分数, and strand-specific information.
- **Strand-specific detection**: Modified bases on the positive and negative DNA strands produce distinct signal patterns. CCsmeth processes each strand independently and reports modification frequencies per strand, which is critical for distinguishing true m6A from passive modification states.
- **Model dependency**: CCsmeth relies on trained machine learning models (random forest or neural network) specific to modification types, flow cells, and basecalling versions. Using outdated or mismatched models significantly reduces detection accuracy.

## Pitfalls

- **Mismatched training models**: Using a model trained on a different flow cell type (e.g., R9.4 versus R10) or basecalling version than the input data will produce high false positive or false negative rates. Always verify model compatibility before running detection.
- **Insufficient coverage per site**: Modification detection requires statistically significant read coverage at each genomic position (minimum 10-20 reads recommended). Sites with very low coverage will have unreliable modification frequency estimates.
- **Ignoring read filtering**: Poor-quality reads or reads with mapping quality below 20 can distort modification signals. Failing to filter low-quality alignments introduces noise and potentially invalidates results.
- **Misinterpreting modification probabilities**: CCsmeth outputs probability scores per read, not definitive modification calls. A threshold (typically 0.5-0.9) must be applied to convert probabilities to binary calls; using a threshold too low inflates false positives.
- **Ignoring sequence context**: Certain sequence motifs (e.g., DRACH for m6A) influence modification detection accuracy. Failing to filter or stratify results by sequence context can confound biological interpretation.

## Examples

### Detect m6A modifications in bacterial DNA

**Args:** call -b alignments.bam -f reference.fa -m r9.4_rna_m6a.model -o modification_calls.bed --type m6A --threads 16
**Explanation:** Runs modification detection on aligned nanopore reads using the R9.4 m6A model, outputting per-position modification calls in BED format.

### Train a custom modification detection model

**Args:** train --positive positive_reads.bam --negative negative_reads.bam -f reference.fa -o custom_model.model --epochs 50 --algorithm random_forest
**Explanation:** Trains a custom model using labeled positive and negative read sets from the provided alignments to detect a specific modification type.

### Extract per-read signals for manual inspection

**Args:** extract_signals -b alignments.bam -f reference.fa -o signals.tsv --window 10 --region chr1:100000-200000
**Explanation:** Extracts raw signal features from a specific genomic region for validation or retraining purposes, useful for model debugging.

### Detect multiple modification types simultaneously

**Args:** call -b alignments.bam -f reference.fa -m models/ --o all_mods.bed --min_prob 0.7 --threads 8
**Explanation:** Runs detection for all modification types covered by the model directory, requiring minimum 0.7 probability for each call.

### Generate genome-wide modification frequency table

**Args:** call -b alignments.bam -f reference.fa -m r9.4_rna_m6a.model -o freq.tsv --aggregate --min_cov 20
**Explanation:** Aggregates per-read calls into genome-wide frequencies, requiring at least 20x coverage per site for reliable frequency estimates.