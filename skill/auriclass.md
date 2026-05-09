---
name: auriclass
category: Classification / Machine Learning
description: A bioinformatics tool for sequence classification and taxonomic assignment using machine learning models. Auriclass processes nucleotide or amino acid sequences and assigns them to predefined categories based on learned patterns from training data.
tags: [bioinformatics, classification, machine-learning, sequence-analysis, taxonomic-assignment]
author: AI-generated
source_url: https://github.com/bioinformatics/auriclass
---

## Concepts

- Auriclass accepts FASTA, FASTQ, or plain text sequence files as input, with support for both nucleotide and protein sequences. The tool reads sequences from standard input or a specified input file, processing one record at a time through the classification pipeline.
- Classification results are output as tab-delimited text with columns for sequence identifier, predicted category, confidence score, and optional alignment details. The confidence score ranges from 0.0 (uncertain) to 1.0 (high certainty), and categories correspond to labels defined in the training model.
- Models must be pre-trained using the companion binary `auriclass-build` before running classification. Models are stored as binary files with extension `.acmodel` and contain the learned feature vectors, category labels, and hyperparameters needed for prediction.

## Pitfalls

- Running `auriclass` without an existing model file produces no results and exits with an error message. Users must first create a model with `auriclass-build` using a labeled training dataset before classification can begin, otherwise the tool fails to locate the required `.acmodel` file.
- Specifying the wrong sequence type (e.g., providing protein sequences when the model was built for nucleotide data) leads to meaningless or inverted category assignments. The tool does not automatically detect sequence type; users must explicitly indicate the correct alphabet via flags, or results will be unreliable.
- Using a model trained on one organism group to classify sequences from a distantly related organism group produces low-confidence predictions. Classification confidence scores drop below 0.3 when input sequences fall outside the taxonomic scope of the training data, yet the tool still outputs results without warning.

## Examples

### Classify sequences from a FASTA file using a trained model
**Args:** -i sequences.fasta -m mymodel.acmodel -o results.tsv
**Explanation:** This reads input sequences from `sequences.fasta`, applies the classification model `mymodel.acmodel`, and writes prediction results to `results.tsv` in tab-delimited format.

### Classify sequences from standard input with CPU thread limit
**Args:** -m mymodel.acmodel --threads 4
**Explanation:** This reads sequences from standard input, utilizing up to 4 CPU threads for parallel processing to speed up classification of large datasets.

### Set minimum confidence threshold for reported predictions
**Args:** -i input.fasta -m mymodel.acmodel --min-confidence 0.7 -o filtered.tsv
**Explanation:** This filters out predictions with confidence scores below 0.7, ensuring only high-confidence assignments are reported in the output file.

### Output classification probabilities for all categories
**Args:** -i input.fasta -m mymodel.acmodel --probabilities -o probs.tsv
**Explanation:** This outputs the probability distribution across all possible categories for each input sequence instead of only the top prediction, useful for downstream uncertainty analysis.

### Enable verbose logging for debugging classification issues
**Args:** -i input.fasta -m mymodel.acmodel -v -o results.tsv
**Explanation:** This enables verbose output mode, printing detailed diagnostic information about each sequence processing step to stderr for troubleshooting unexpected classification behavior.