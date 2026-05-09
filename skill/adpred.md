---
name: "adpred"
category: "Sequence Analysis"
description: "A bioinformatics tool for prediction and analysis of sequence features, often used for identifying functional elements in DNA or RNA sequences. Works with common bioinformatics formats including FASTA, FASTQ, and custom annotation files."
tags: ["sequence", "prediction", "functional-analysis", "genomics"]
author: "AI-generated"
source_url: "https://github.com/example/adpred"
---

## Concepts

- **Input Formats**: adpred accepts FASTA (`.fasta`, `.fa`) and FASTQ (`.fastq`, `.fq`) files as primary inputs, and can process both single-sequence and multi-sequence files containing DNA or RNA sequences.

- **Output Options**: Results are written to stdout in tab-separated format by default, with optional CSV or JSON output when `--out-format` is specified. An index file (`.adpred.idx`) can be generated for faster subsequent runs on the same input.

- **Score Thresholds**: The tool uses a configurable prediction score threshold (default: 0.5) to distinguish positive from negative predictions. Higher thresholds increase precision but reduce recall, and can be adjusted using `--threshold` or `-t`.

- **Model Parameters**: Prediction models are specified via `--model` or `-m`, with built-in models for different sequence types. Custom models require a pre-trained parameter file in JSON format.

## Pitfalls

- **Empty Input Files**: Running adpred on empty input files or files containing only newline characters produces no output and exits with error code 4, causing downstream pipeline failures if not handled explicitly.

- **Mismatched Alphabet**: Providing protein sequences when a DNA model is selected (or vice versa) produces meaningless scores and may silently complete without raising an error—always verify `--alphabet` matches your input data type.

- **Floating-Point Threshold Values**: Using threshold values outside the valid range [0.0, 1.0] causes the tool to fail with a parse error; use `--threshold 0.7` rather than `--threshold 70%` or `--threshold 0.5-0.9`.

- **File Permission Errors**: Attempting to write output to read-only directories or files without write permission fails with a generic I/O error—verify write permissions before specifying `--output` paths.

## Examples

### Predict functional regions in a DNA sequence file
**Args:** `input.fasta --model dna-functional --threshold 0.6`
**Explanation:** Runs prediction using the DNA functional element model with a higher threshold (0.6) to only report high-confidence predictions, reducing false positives.

### Process multiple sequences with verbose output
**Args:** `sequences.fq --model rna-struct --verbose`
**Explanation:** Enables verbose logging to stderr, useful for debugging when processing multiple sequences or diagnosing unexpected predictions.

### Output results in JSON format
**Args:** `input.fa --model peptide-binding --out-format json --output predictions.json`
**Explanation:** Writes prediction results in JSON format to the specified output file instead of default tab-separated stdout, enabling easier integration with automated pipelines.

### Adjust minimum prediction score for high precision
**Args:** ` enhancer_seqs.fasta --model enhancer --threshold 0.85 --min-length 200`
**Explanation:** Uses a high threshold (0.85) to Filter out low-confidence predictions and `--min-length` to only consider predictions spanning at least 200 base pairs, improving precision at the cost of recall.

### Run with custom pre-trained model parameters
**Args:** ` query.fasta --model custom --model-params custom_model.json`
**Explanation:** Loads a custom-trained model from the provided JSON parameter file rather than using built-in models, enabling prediction with user-trained models or updated parameters.