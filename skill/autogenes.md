---
name: autogenes
category: Genomics
description: Automated gene detection and annotation tool for genomic sequences. Identifies coding regions, predicts gene structures, and generates functional annotations using statistical models and sequence homology.
tags:
  - gene-detection
  - annotation
  - genomics
  - sequence-analysis
  - bioinformatics
author: AI-generated
source_url: https://github.com/bioinformatics-tools/autogenes
---

## Concepts

- **Input Formats**: autogenes accepts FASTA files for single-genome or multi-genome analysis, and supports GenBank format for sequences with existing annotations to use as training data.
- **Output Formats**: Produces GFF3 annotation files with gene models, predicted coding sequences (CDS), and translation start/stop positions. Can optionally output FASTA files with extracted gene sequences.
- **Detection Models**: Uses hidden Markov models (HMMs) combined with codon usage bias statistics to distinguish coding regions from non-coding genomic DNA with configurable sensitivity thresholds.
- **Threshold Parameters**: The `--min-length` and `--score-threshold` flags control minimum gene length (default 90bp) and detection confidence score (default 0.5) to filter false positives.

## Pitfalls

- **Setting score threshold too low**: Using `--score-threshold 0.2` will generate many false-positive gene predictions, bloating annotation files with non-coding fragments that complicate downstream analysis.
- **Forgetting to specify output format**: Without `--output-format gff3`, autogenes defaults to text summary format, which cannot be imported into genome browsers or downstream tools like BEDTools.
- **Ignoring frame shifts in prokaryotes**: Attempting gene prediction on bacterial genomes without enabling `--coding-table 11` (alternative genetic code) leads to incorrect protein translations for organisms with alternative codons.
- **Large input without memory allocation**: Processing whole-chromosome FASTA files (>100MB) without `--memory-limit 8g` causes out-of-memory errors; the tool buffers entire sequences in RAM by default.

## Examples

### Running gene detection on a bacterial genome FASTA file
**Args:** `--input bacteria_genome.fasta --output genes.gff3 --score-threshold 0.7`
**Explanation:** Sets a higher confidence threshold to reduce false positives in bacterial genomes where gene density is high and spurious predictions are common.

### Extracting predicted CDS sequences to FASTA
**Args:** `--input genome.fasta --extract-cds --output-format fasta --min-length 150`
**Explanation:** Exports the extracted coding sequences in FASTA format for downstream protein analysis, filtering out small ORFs below 150bp.

### Using a trained model file for eukaryotic gene prediction
**Args:** `--input yeast_chr1.fasta --model yeast_hmm.nhmm --output annotations.gff3`
**Explanation:** Applies a pre-trained hidden Markov model specific to yeast to improve gene structure prediction accuracy.

### Running with GenBank reference for training
**Args:** `--input new_genome.fasta --reference annotated.gb --train-model custom.hmm`
**Explanation:** Uses existing annotations in GenBank file to build a custom gene-finding model before running prediction on new genomic data.

### Batch processing multiple FASTA files with parallel threads
**Args:** `--batch genomes/*.fasta --output-dir results/ --threads 8 --min-length 300`
**Explanation:** Processes all FASTA files in a directory using 8 parallel threads, filtering for genes at least 300bp in length.

### Exporting both annotation and summary statistics
**Args:** `--input sequence.fasta --output genes.gff3 --stats-file summary.tsv --verbose`
**Explanation:** Generates a detailed statistics file with base composition, codon usage, and prediction confidence metrics alongside the standard GFF3 output.