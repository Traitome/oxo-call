---
name: coils
category: Protein Structure Prediction
description: A weight matrix-based method for predicting coiled-coil regions and leucine zipper motifs in protein sequences. Takes protein sequences in FASTA format and outputs positional scores and probability predictions for coiled-coil formation.
tags:
  - coiled-coil
  - leucine-zipper
  - protein-structure
  - motif-prediction
  - sequence-analysis
author: AI-generated
source_url: https://github.com/wrf/coils
---

## Concepts

- COILS predicts coiled-coil regions by scoring a sliding window of amino acids against position-specific weight matrices (mtidk or mtfile). Windows of 14, 21, or 28 residues are evaluated, and each position receives a score reflecting its propensity for coiled-coil formation.
- Input sequences must contain only valid single-letter amino acid codes. Ambiguous characters (B, J, O, U, X, Z) or nucleotide bases (A, T, G, C when unambiguous) cause silent failure or zero-score output, so pre-filtering with tools like `EMBOSS norf` is recommended.
- Output can be directed to three formats via `-format`: plain text (default), HTML (`html`), or a machine-readable matrix format (`matrix`). The HTML format embeds scores and color-coded residue annotations suitable for browser-based review.
- COILS uses the MTiDk weight matrix by default, which was derived from known coiled-coil training data. A custom matrix can be supplied via `-weights`, enabling domain-specific or updated scoring schemes.
- Scoring is sensitive to window size (`-window`). Smaller windows (14) are better for short isolated coils; larger windows (28) are better for long continuous coiled-coils. Specifying multiple windows (`-window 14 21 28`) produces parallel scans for comparative analysis.

## Pitfalls

- Using multi-sequence FASTA files without specifying `-sequence 1` (or the appropriate 1-based index) causes COILS to process only the first sequence, silently discarding all others. Always verify the index matches the entry you intend to analyze.
- Specifying an output filename with a path that does not exist (e.g., `/results/output.txt` for an unwritten directory) produces no error message; the file is simply not created and output defaults to stdout.
- Relying solely on the highest score without examining the positional profile leads to missed marginal coiled-coils. Regions near the score threshold (e.g., 0.5 probability) may still be biologically relevant and should be validated experimentally.
- Mixing up window size units causes unrealistic expectations. A window of 100 produces very smoothed scores that may miss short functional motifs; COILS is designed for windows in the 14–28 range.
- Not specifying `-format html` when preparing results for non-computational collaborators. Plain text output is hard to interpret without scripting, whereas HTML output presents color-coded scores that are immediately readable.

## Examples

### Predict coiled-coil regions in a protein sequence file using the default 21-residue window
**Args:** `-sequence protein.fasta -format text -window 21`
**Explanation:** This scans the input sequence with a 21-residue sliding window against the mtidk weight matrix and prints positional scores in plain text, which is the default output format suitable for piping into analysis scripts.

### Generate an HTML report with predictions for a specific sequence entry
**Args:** `-sequence protein.fasta -sequence 3 -format html -window 21 -output myprediction.html`
**Explanation:** Processing the third entry in the multi-sequence FASTA file and producing HTML output allows direct visual inspection of scored residues with color-coded confidence levels for coiled-coil regions.

### Compare coiled-coil propensity across three different window sizes simultaneously
**Args:** `-sequence protein.fasta -format matrix -window 14 21 28`
**Explanation:** Running three independent scans with windows of 14, 21, and 28 residues and outputting the matrix format enables side-by-side comparison of how window size affects score magnitude and positional peak identification.

### Save plain text predictions for the second sequence entry to a named output file
**Args:** `-sequence protein.fasta -sequence 2 -format text -window 21 -output entry2_predictions.txt`
**Explanation:** Directing output to a named file rather than relying on stdout redirection keeps results organized when processing multiple sequences in batch workflows.

### Predict coiled-coil regions using a custom weight matrix for a specific protein family
**Args:** `-sequence protein.fasta -sequence 1 -weights custom_matrix.txt -window 28 -format html`
**Explanation:** Supplying a custom weight matrix extends the default mtidk matrix with family-specific residue propensities, improving prediction accuracy for non-standard coiled-coil families when the default matrix may be poorly calibrated.