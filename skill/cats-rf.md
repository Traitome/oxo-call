---
name: cats-rf
category: Bioinformatics - Sequence Classification
description: A machine learning-based tool for classifying biological sequences using random forest classifiers. Cats-rf trains models on sequence features (k-mer composition, GC content, motif patterns) to predict taxonomy, function, or other labels. Works with FASTA, FASTQ, and tabular feature files, outputting classification predictions with confidence scores.
tags: [sequence-classification, machine-learning, random-forest, taxonomy-prediction, functional-annotation, feature-extraction]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/cats-rf
---

## Concepts

- **Input formats**: Cats-rf accepts FASTA/FASTQ files for sequence input, comma-separated feature matrices (CSV/TSV), and model definition files (JSON) for pre-trained classifiers. Sequences are automatically converted to numerical features using built-in k-mer counting (default k=3 to k=6) and compositional analysis.
- **Training workflow**: The tool requires a labeled training set (sequences with known class labels) and generates a random forest model stored in `.rf` binary format. Training parameters include number of trees (default 100), maximum depth, and feature selection thresholds. Cross-validation is built-in for hyperparameter optimization.
- **Prediction output**: Classification results include predicted label, confidence score (0-1 probability), and per-class probability distribution. Output formats mirror input (FASTA with annotations, CSV with predictions, or JSON for programmatic access). Batch processing supports thousands of sequences simultaneously.
- **Feature model**: Cats-rf uses a pluggable feature extraction architecture. Default extractors include: k-mer frequency vectors, GC content, dipeptide composition, and positional base preference. Custom feature extractors can be added via a plugin API for specialized analysis (e.g., protein physicochemical properties).

## Pitfalls

- **Insufficient training data**: Training with fewer than 50 sequences per class leads to overfitting and poor generalization. Consequences include inflated training accuracy but severe accuracy drops on new data (often below 50%). Always verify class balance and minimum sample thresholds before training.
- **Feature leakage**: Using sequence labels that directly encode the prediction target (e.g., including taxonomic names in training features) causes data leakage. This produces artificially high accuracy that collapses in real-world deployment. Always use independent validation sets and audit feature sources.
- **Mismatched feature extractors**: Using different k-mer sizes or extraction parameters between training and prediction leads to feature space misalignment. Predictions become essentially random because the model was trained on different feature dimensions than what the tool computes at prediction time.
- **Ignoring class imbalance**: Training on highly imbalanced datasets (e.g., 1000:1 ratio) without adjustment causes the model to always predict the majority class. Use the `--class-weight` option or apply sampling strategies to correct for imbalance before training.

## Examples

### Train a classifier on bacterial 16S rRNA sequences
**Args:** `--train --input training_seqs.fasta --labels training_labels.csv --output bacteria_taxonomy.rf --trees 200`
**Explanation:** Trains a random forest model using 200 trees on k-mer features extracted from FASTA sequences with corresponding class labels in CSV format for taxonomic classification.

### Predict gene function from unannotated sequences
**Args:** `--predict --model protein_function.rf --input unknown_genes.fasta --output predictions.tsv --confidence-threshold 0.7`
**Explanation:** Applies a pre-trained model to classify gene function while filtering out low-confidence predictions below 0.7 probability.

### Extract k-mer features from sequences for external analysis
**Args:** `--extract-features --input sequences.fasta --output feature_matrix.csv --kmer-sizes 3,4,5`
**Explanation:** Converts sequences to numerical feature vectors using k-mer sizes 3,4,5 and exports the matrix for use with external machine learning tools.

### Evaluate model performance using held-out validation
**Args:** `--evaluate --model bacteria_taxonomy.rf --validation validation_seqs.fasta --validation-labels validation_labels.csv`
**Explanation:** Tests model accuracy and generates a confusion matrix, precision/recall statistics, and F1 scores on an independent validation set.

### Optimize hyperparameters with built-in cross-validation
**Args:** `--train --input train.fasta --labels labels.csv --output optimized.rf --cv-folds 5 --grid-search`
**Explanation:** Performs 5-fold cross-validation across a predefined parameter grid to find optimal tree count and depth, then trains final model with best parameters.

### Handle class imbalance with weighted training
**Args:** `--train --input train.fasta --labels labels.csv --output balanced.rf --class-weight auto`
**Explanation:** Applies automatic class weighting to compensate for uneven class distribution, improving minority class recall without manual ratio specification.

### Export per-class probability distributions
**Args:** `--predict --model function.rf --input query.fasta --output probs.json --include-probabilities`
**Explanation:** Outputs detailed probability for each possible class rather than just the top prediction, useful for uncertainty analysis and multi-label scenarios.