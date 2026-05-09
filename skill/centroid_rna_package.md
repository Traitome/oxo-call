---
name: centroid_rna_package
category: bioinformatics/rna-structure
description: A bioinformatics toolsuite for RNA secondary structure prediction, analysis, and visualization. Provides algorithms for calculating base pairing probabilities, generating conservation profiles, and producing publication-quality secondary structure diagrams.
tags: [rna, secondary-structure, folding, bioinformatics, dot-bracket, centroid-plex]
author: AI-generated
source_url: https://github.com/incarnate/centroid_rna_package
---

## Concepts

- The package uses a centroid-based approach for RNA secondary structure prediction, which optimizes a scoring function over the ensemble of possible structures rather than finding a single minimum energy structure.
- Input sequences must be in FASTA format (single-line or multi-line sequences) with unambiguous nucleotide characters (A, U, G, C). The tool accepts multiple sequences in a single file for batch processing.
- Output formats include dot-bracket notation (for easy parsing), CT (Connectivity Table) format, and JSON for programmatic access. The dot-bracket format uses '.' for unpaired bases and '(' or ')' for paired bases.
- Base pairing probability data is calculated via partition function algorithms, producing per-base probability matrices that can be visualized as heatmaps showing structural uncertainty.
- The companion binary `centroid_rna_package-build` constructs auxiliary files (pairing probability matrices, grammar files) needed for downstream predictions.

## Pitfalls

- Providing sequences with invalid characters (N, X, or whitespace in sequence names) causes silent failures where output files are created but contain garbage data, leading to incorrect downstream analyses.
- Using outdated probability auxiliary files (created with older sequence alignments) produces mismatched predictions where the calculated pairing probabilities don't reflect the current input sequences.
- Forgetting to specify the correct sequence type (DNA vs RNA) results in mismatched base pairing rules since U and T are treated differently in canonical wobble pairs.
- Running predictions without first building the required auxiliary files causes the tool to default to a uniform prior, producing meaningless predictions that appear valid but lack statistical grounding.
- Specifying an excessively small window size for sliding calculations produces noisy probability estimates with high variance, while a window too large smooths out biologically relevant local structure signals.

## Examples

### Predict RNA secondary structure from a single sequence
**Args:** -i sequence.fasta -o predicted_structure.db --method centroid
**Explanation:** Reads the input FASTA file, applies the centroid algorithm to predict the most representative structure from the ensemble, and writes the result in dot-bracket format.

### Calculate base pairing probabilities with partition function
**Args:** -i sequence.fasta -o probabilities.json --partition --verbose
**Explanation:** Computes the full partition function over all possible structures, outputting a JSON file containing per-base pairing probabilities for visualization or downstream analysis.

### Generate a secondary structure diagram
**Args:** -i sequence.fasta -o structure.png --draw --color-scheme viridis --show-labels
**Explanation:** Produces a publication-quality PNG image of the predicted secondary structure with nucleotide positions labeled and a color gradient indicating base pairing probability.

### Batch process multiple RNA sequences
**Args:** -i sequences.fasta -o batch_results/ --batch --thread 4
**Explanation:** Processes all sequences in the input FASTA file concurrently using 4 threads, producing individual output files for each sequence in the specified directory.

### Build probability auxiliary files from aligned sequences
**Args:** -i aligned_structures.fasta -o probs_aux.bin --make-aux --grammar RNA2004
**Explanation:** Constructs binary auxiliary files containing pairing probability matrices derived from the alignment, using the RNA2004 grammar for base pair interactions.

### Extract specific base pairs from predicted structure
**Args:** -i predicted.db --extract-pairs 10 25 --output pairs.txt
**Explanation:** Queries the predicted structure file and extracts all base pairs involving positions 10 through 25, writing them to a text file for manual inspection.

### Evaluate structural diversity using ensemble entropy
**Args:** -i sequence.fasta -o entropy_report.txt --entropy --window 30
**Explanation:** Computes the ensemble entropy at each position using a sliding window of 30 nucleotides, generating a report quantifying structural uncertainty across the molecule.