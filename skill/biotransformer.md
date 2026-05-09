---
name: biotransformer
category: Cheminformatics / Metabolism Prediction
description: Predicts biotransformation (metabolic) pathways of chemical compounds using machine learning models. Supports Phase I, Phase II, and microbial transformations. Takes SMILES or SDF input and outputs predicted metabolic products with transformation probabilities.
tags: [metabolism, biotransformation, cheminformatics, toxicology, drug-discovery, machine-learning, chemical-compounds]
author: AI-generated
source_url: https://biotransformer.org
---

## Concepts

- **Input formats**: BioTransformer accepts chemical structures in SMILES string format (single-line) or SDF file format (multiple compounds). Each SMILES must represent a valid chemical structure with correct atom valencies. SDF files contain multiple molecules with their associated data fields.

- **Transformation types**: The tool supports three main biotransformation categories — Phase I (oxidations, reductions, hydrolytic reactions performed by CYP enzymes), Phase II (conjugation reactions including glucuronidation, sulfation, glutathione binding), and microbial transformations (gut/environmental metabolism). Selecting the correct model type (`--model` flag) is essential for relevant predictions.

- **Output formats and interpretation**: Results are exported as SDF, CSV, or JSON files. Each predicted product includes a transformation probability score (0-1 range), the likely enzyme/ mechanistic class, and the transformed SMILES. Higher probability scores indicate more confident predictions but do not guarantee experimental detection.

- **Batch processing**: Multiple compounds can be processed by providing an SDF input file or multiple SMILES entries. The tool iterates through each structure and generates transformation products. Batch size may be limited by available system memory when processing very large molecules or extensive SDF collections.

## Pitfalls

- **Invalid or chemically impossible SMILES**: Providing malformed SMILES strings (e.g., unmatched ring closures, incorrect valency, invalid atomic symbols) causes the tool to skip that compound without raising a clear error. Always validate SMILES with a molecular viewer before processing, otherwise you may miss entire subsets of your input data.

- **Mismatched transformation model**: Using the wrong model flag (e.g., running Phase II model on a compound that only undergoes Phase I metabolism) yields unrealistic or no predictions. Each model is trained on enzyme-specific reaction data, so confirm your biological context before selecting `--model phase1`, `--model phase2`, or `--model microbial`.

- **Overwriting output files silently**: BioTransformer writes output files without prompting for confirmation. If you specify an output path that already exists, the tool silently overwrites it. Always verify your output path or use unique filenames to prevent accidental data loss.

- **Insufficient memory for large SDF files**: Processing large SDF files with thousands of compounds can exhaust system memory, especially when generating multiple transformation branches per parent compound. Monitor RAM usage during batch processing; consider splitting large SDF files into smaller chunks to avoid crashes.

## Examples

### Predict Phase I biotransformation for a single SMILES

**Args:** `--input "CC(=O)Oc1ccccc1C(=O)O"` `--model phase1` `--output aspirine_metabolites.sdf`

**Explanation:** This runs the Phase I model on the SMILES for aspirin (acetyl salicylic acid), predicting oxidative metabolites such as salicylic acid and gentisic acid. The result is written to an SDF file containing all predicted products with transformation probabilities.

### Process multiple compounds from an SDF input file

**Args:** `--input compounds.sdf` `--model phase2` `--output conjugated_products.sdf`

**Explanation:** This reads all chemical structures in `compounds.sdf` and applies the Phase II (conjugation) model, predicting glucuronidated, sulfated, or glutathione-conjugated products. Each molecule in the SDF generates transformation outputs, maintaining the original molecule data fields.

### Export predictions as CSV with probability scores

**Args:** `--input "c1ccc2[nH]ccc2c1" `--model microbial` `--output indole_microbial.csv` `--format csv`

**Explanation:** This processes the indole SMILES using the microbial metabolism model and writes results in CSV format. Each row contains the parent SMILES, product SMILES, transformation probability, and predicted enzyme class. CSV format is useful for downstream statistical analysis.

### Limit the number of transformation steps

**Args:** `--input "CCCO" `--model phase1` `--output propanol_tx.sdf` `--maxsteps 2`

**Explanation:** This limits the biotransformation recursion depth to 2 steps, generating direct products and second-generation metabolites. Controlling depth prevents combinatorial explosion and reduces output file size for complex transformation networks.

### Run with verbose logging for debugging

**Args:** `--input test.sdf` `--model phase1` `--output test_output.sdf` `--verbose`

**Explanation:** The `--verbose` flag enables detailed logging, printing each compound processed, skipped molecules with reasons, and transformation tree construction. Use this when debugging unexpected output or missing predictions in batch runs.