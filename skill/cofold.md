---
name: cofold
category: Bioinformatics / RNA Secondary Structure Prediction
description: Predicts RNA secondary structures using co-transcriptional folding kinetics, modeling the order in which nucleotides become available during transcription to determine biologically relevant folding pathways.
tags:
  - RNA
  - secondary structure
  - co-transcriptional folding
  - kinetic folding
  - ViennaRNA
  - energy minimization
author: AI-generated
source_url: https://www.tbi.univie.ac.at/RNA/cofold.html
---

## Concepts

- The transcription start position must be specified with the `-t` or `--transcript-start` flag; cofold uses this to determine the order in which nucleotides become available for base pairing, enabling prediction of folding pathways rather than just the final structure.
- Input sequences must use RNA nucleotide codes (A, C, G, U); using DNA sequences with T bases will either cause errors or produce incorrect structure predictions because the energy parameters are calibrated for RNA.
- Output provides three structures: the minimum free energy (MFE) structure, the centroid structure, and the ensemble defect, along with their respective free energy values in kcal/mol.
- The `-d` or `--dimer` flag enables prediction of intermolecular RNA-RNA interaction structures between two input sequences, useful for studying RNA-RNA complexes; input must then be provided as two sequences (either separate files or in multi-FASTA format).
- Co-fold evaluates multiple folding scenarios including the "kissing hairpin" intermediate states that form during transcription, outputting the conditional ensemble defect which measures the average base-pair mismatch between the predicted pathway and the final MFE structure.

## Pitfalls

- Failing to specify the correct transcription start position (`-t`) produces results equivalent to standard RNAfold (ignoring co-transcriptional timing), which defeats the purpose of using cofold and may yield biologically incorrect folding pathways for very long transcripts.
- Neglecting temperature and ionic conditions leads to inaccurate energy calculations; the default 37°C and 1M Na+ may not match experimental conditions, causing predicted structures to deviate from in vitro or in vivo observations.
- Providing sequences in lowercase instead of uppercase results in cofold treating lowercase bases as DNA instead of RNA (converting them to T), leading to fundamentally wrong structure predictions with thymine instead of uracil base-pairing rules.
- Using the dimer prediction mode (`--dimer`) without specifying both sequences produces only intramolecular folding results rather than the intended intermolecular prediction, wasting computational time on the wrong analysis.
- Interpreting the dot-bracket notation incorrectly: unpaired nucleotides are shown as "." while base pairs use matching opening and closing brackets, but multi-loops require specific counting rules that differ from simple pair counting.

## Examples

### Predict co-transcriptional folding for a single RNA starting at position 1
**Args:** `-t 1 sequences.fa`
**Explanation:** The `-t 1` flag specifies that transcription begins at nucleotide position 1, enabling cofold to model folding as each nucleotide becomes available during synthesis.

### Run cofold at physiological temperature (37°C)
**Args:** `-t 1 --temp=37 sequences.fa`
**Explanation:** The `--temp=37` flag adjusts the free energy calculations to 37°C, which is the standard physiological temperature for human and warm-blooded organisms.

### Predict RNA-RNA dimer interaction between two sequences
**Args:** `-t 1 --dimer seq1.fa seq2.fa`
**Explanation:** The `--dimer` flag enables intermolecular base-pairing prediction between the two input sequences, producing structures where bases from both RNAs can form pairs.

### Output all ensemble structures in verbose mode
**Args:** `-t 1 -v sequences.fa`
**Explanation:** The `-v` flag produces detailed output including the ensemble free energy, base pair probabilities, and the positional entropy for each nucleotide position.

### Use DNA input mode (T instead of U)
**Args:** `-t 1 -- DNAseqs.fa`
**Explanation:** The literal `--` flag tells cofold to interpret the input as DNA sequences (using T bases), which then get converted internally to RNA for energy calculation; useful when working with genomic coordinates.

### Predict with salt correction for near-physiological conditions
**Args:** `-t 1 --salt=0.1 sequences.fa`
**Explanation:** The `--salt=0.1` flag applies a salt correction for 100mM Na+ concentration rather than the default 1M, improving accuracy for typical laboratory conditions.

### Calculate ensemble defect to assess folding heterogeneity
**Args:** `-t 1 -- ensemble-only sequences.fa`
**Explanation:** The `--ensemble-only` flag outputs only the ensemble defect score without generating full structure predictions, useful for quickly assessing how heterogeneous the folding landscape is for a given sequence.