---
name: contrafold
category: RNA secondary structure prediction
description: Predicts RNA secondary structures from nucleotide sequences using weighted context-free grammars (WCGF) trained on known RNA structure databases. Supports single-sequence prediction, ensemble output, and base pair probability calculation.
tags:
  - RNA
  - secondary structure
  - bioinformatics
  - fold prediction
  - probabilistic models
author: AI-generated
source_url: https://contrafold.csail.mit.edu/
---

## Concepts

- **Input formats**: Contrafold accepts raw nucleotide sequences (one per line) or FASTA format with sequence names. Sequences must contain only A, C, G, U/T characters — any whitespace, numbers, or non-nucleotide characters cause parsing errors.
- **Output formats**: The tool produces predictions in dot-bracket notation (e.g., "(((...)))"), CT format (with nucleotide positions and base pair annotations), or Vienna format. The `--struct` flag outputs just the dot-bracket string; `--ct` outputs the CT format with base pair partner information.
- **Probabilistic output**: Using `--bp` outputs base pair probabilities for each position pair, enabling uncertainty visualization. The `--ensemble` flag outputs multiple structure predictions ranked by probability, useful for exploring alternative folds.
- **Model behavior**: Contrafold uses a pre-trained weighted context-free grammar model by default. The model applies the CYK algorithm for optimal structure finding and calculates partition functions for probabilistic outputs.

## Pitfalls

- **Missing sequence input**: Running contrafold without providing input sequences (either via stdin, file argument, or `-q` flag) causes the tool to wait for input interactively or error out, depending on flags.
- **Invalid nucleotide characters**: Sequences containing ambiguity codes (like N, R, Y), numbering, or FASTA headers without the `>` prefix trigger parse errors or produce incorrect predictions. Always clean sequences before running.
- **Ignoring probabilistic output for critical predictions**: Using only the single best structure (`--struct` alone) without checking base pair probabilities (`--bp`) can hide uncertain regions where the prediction may be unreliable.
- **Conflicting output flags**: Combining incompatible output flags (e.g., both `--score` and `--struct`) results in only one output type being produced, often silently overwriting the other.

## Examples

### Predict secondary structure from a single RNA sequence
**Args:** `--struct`
**Explanation:** The `--struct` flag tells Contrafold to output only the optimal secondary structure in dot-bracket notation, which is the most common use case for quick structure visualization.

### Output base pair probabilities for uncertainty analysis
**Args:** `--bp`
**Explanation:** Outputs a matrix of base pair probabilities between all position pairs, allowing you to identify regions with high prediction uncertainty that may require experimental validation.

### Generate ensemble of multiple plausible structures
**Args:** `--ensemble`
**Explanation:** Outputs multiple candidate structures with their associated probabilities, useful when the optimal structure is uncertain and you want to explore alternative folding possibilities.

### Get structure in CT format with annotation
**Args:** `--ct`
**Explanation:** Outputs the structure in CT (connectivity table) format, which includes nucleotide positions, base pair partners, and structural annotation — required for visualization in tools like VARNA.

### Output only the prediction score without structure
**Args:** `--score`
**Explanation:** Useful for scoring a given sequence against the Contrafold model to evaluate structural plausibility without generating a full prediction, often used in comparative analysis pipelines.

### Predict structure from FASTA file with sequence name
**Args:** `input.fasta --struct`
**Explanation:** Takes a FASTA-formatted file containing one or more sequences and predicts secondary structure for each, outputting structures with their corresponding sequence names.