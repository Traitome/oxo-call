---
name: classpro
category: protein-classification
description: A tool for classifying protein sequences into functional families or structural classes using machine learning or alignment-based methods. Accepts FASTA input and produces classification results in text or JSON formats.
tags:
  - protein-classification
  - sequence-analysis
  - functional-annotation
  - machine-learning
  - bioinformatics
author: AI-generated
source_url: https://github.com/example/classpro
---

## Concepts

- **Input Format**: classpro accepts protein sequences in FASTA format. Multi-line sequences are supported, and input may be provided via stdin redirection. Headers should contain unique identifiers for result tracking.
- **Classification Models**: The tool uses pre-trained classification models stored in dedicated model directories. Models are referenced by name and must be compatible with the current classpro version for accurate predictions.
- **Output Modes**: classpro supports both human-readable text output (default) and machine-parseable JSON output via the `--json` flag. Text output includes sequence ID, predicted class, and confidence score for each sequence.
- **Batch Processing**: Large sequence sets are processed sequentially with configurable batch sizes. Memory usage scales with batch size; reducing `--batch-size` can prevent out-of-memory errors on resource-constrained systems.
- **Threshold Configuration**: The `--threshold` parameter controls the minimum confidence score required to assign a classification. Sequences below this threshold receive an "unclassified" label, preventing low-confidence assignments from propagating.

## Pitfalls

- **Incompatible Model Version**: Loading a model trained with a different classpro version produces unpredictable classifications or crashes with a version mismatch error. Always verify model compatibility before running production analyses.
- **Missing Sequence IDs**: When input FASTA files contain sequences without headers or with duplicate identifiers, results cannot be properly mapped back to source sequences, leading to ambiguous output and downstream analysis failures.
- **Threshold Too Strict**: Setting `--threshold` to a high value (e.g., 0.95) may cause most sequences to be labeled "unclassified" even when meaningful predictions exist, resulting in sparse output and wasted computational effort.
- **Memory Exhaustion with Large Batches**: Using a large `--batch-size` value on systems with limited RAM causes out-of-memory termination. Monitor memory usage and reduce batch size if the process is killed by the system.
- **Invalid FASTA Format**: Sequences containing non-standard amino acid characters (such as 'X' or 'B' in contexts where they are not permitted) are rejected with a parsing error, requiring pre-filtering of input sequences.

## Examples

### Classify a single protein sequence from a FASTA file

**Args:** `input.fasta --model random_forest --json`
**Explanation:** This command reads protein sequences from `input.fasta`, classifies them using the pre-trained `random_forest` model, and outputs results in JSON format for programmatic parsing.

### Classify sequences with a relaxed confidence threshold

**Args:** `proteins.fa --model ensemble_v2 --threshold 0.6`
**Explanation:** Using a threshold of 0.6 allows sequences with moderate confidence predictions to be classified rather than labeled "unclassified," producing more complete output for exploratory analysis.

### Reduce memory usage by processing in smaller batches

**Args:** `large_dataset.fa --model neural_net --batch-size 50`
**Explanation:** Setting `--batch-size` to 50 limits memory consumption, preventing out-of-memory errors when classifying large datasets on systems with constrained RAM.

### Output text results with detailed per-sequence information

**Args:** `sequences.fasta --model svm_classifier --output results.txt`
**Explanation:** This writes human-readable classification results including sequence IDs, predicted classes, and confidence scores to `results.txt` for manual review or integration into reports.

### Classify from stdin and pipe results to another tool

**Args:**