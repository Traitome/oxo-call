---
name: chexalign
category: cheminformatics/structure-alignment
description: Aligns chemical structures (SMILES, SDF) to a reference molecule using various alignment algorithms and scoring methods. Supports subgraph matching, maximum common substructure (MCS), and pharmacophore-based alignment.
tags: [chemistry, molecular-alignment, cheminformatics, SMILES, SDF, MCS, structure-matching]
author: AI-generated
source_url: https://github.com/chexalign/chexalign
---

## Concepts

- **Input Formats**: Accepts SMILES strings, SDF/mol files, and SMARTS patterns. Multiple query molecules can be supplied in a single file for batch alignment against one or more reference structures.
- **Alignment Modes**: Supports three primary modes: `exact` (full structure match), `substruct` (subgraph/partial match), and `mcs` (maximum common substructure) alignment. The MCS mode finds the largest shared substructure between query and reference.
- **Scoring and Output**: Generates alignment scores based on Tanimoto or Dice similarity coefficients. Output can be written as aligned SMILES, enriched SDF with alignment metadata, or CSV with per-molecule scores and atom mappings.
- **Reference-Based Operation**: One reference molecule is required (specified via `--reference` or `-r`). All queries are aligned independently against this single reference structure.
- **Atom Mapping**: When enabled via `--map` or `-m`, outputs atom-to-atom correspondences between query and reference, useful for reaction mapping or scaffold hopping analysis.

## Pitfalls

- **Mismatched Format Assumptions**: Providing SMILES to an input expecting SDF (or vice versa) causes silent failures or garbage output. Always verify format flags match your input file extensions.
- **Reference Not Found**: Forgetting to specify a reference structure (`-r`) results in an error stating "reference required". The tool cannot perform alignment without a target.
- **Score Threshold Too Strict**: Setting an excessively high similarity threshold (e.g., `--threshold 0.95` with low-quality queries) rejects all valid alignments, producing empty output files.
- **Case Sensitivity in SMILES**: SMILES string case is significant—lowercase 'cl' denotes chlorine atoms while uppercase 'Cl' is also chlorine; using the wrong case produces incorrect molecules or parse errors.
- **Memory with Large SDF Files**: Loading very large SDF files without the `--batch-size` limit can exhaust memory. Use batched processing for files with thousands of molecules.

## Examples

### Align a single SMILES query to a reference molecule
**Args:** `-r "CC(=O)Oc1ccccc1C(=O)O" -i query.smi -o aligned.smi`
**Explanation:** Aligns the input SMILES query to the aspirin reference structure (CC(=O)Oc1ccccc1C(=O)O), outputting aligned SMILES.

### Find maximum common substructure between molecules
**Args:** `--mode mcs -r reference.sdf -i querys.sdf --output-format sdf -o mcs_results.sdf`
**Explanation:** Uses MCS mode to find the largest shared substructure between reference and each query, outputting results in SDF format.

### Filter alignments by similarity threshold
**Args:** `--mode exact -r "c1ccccc1" -i compounds.smi --threshold 0.85 -o passed.smi`
**Explanation:** Retains only alignments with Tanimoto similarity ≥ 0.85, filtering out low-similarity matches from the output.

### Generate atom mapping for reaction parsing
**Args:** `--mode substruct -r reaction_template.smarts -i reactant.smi --map -o mapping.csv`
**Explanation:** Performs subgraph alignment and outputs atom-to-atom mappings in CSV format for reaction atom-tracking applications.

### Batch process with specific alignment algorithm
**Args:** `--mode mcs -r target.mol -i batch.sdf --method hungarian --scoring dice -o batch_aligned.sdf`
**Explanation:** Uses Hungarian algorithm for optimal MCS assignment with Dice coefficient scoring, processing all molecules in batch.sdf.