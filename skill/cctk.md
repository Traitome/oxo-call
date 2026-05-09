---
name: cctk
category: mass spectrometry / ion mobility
description: A toolkit for predicting collision cross section (CCS) values for metabolites, peptides, and other small molecules from mass spectrometry data. Supports multiple ion types and adducts.
tags:
  - CCS prediction
  - ion mobility
  - metabolomics
  - proteomics
  - mass spectrometry
  - IM-MS
author: AI-generated
source_url: https://github.com.com/yunkImBox/cctk
---

## Concepts

- **CCS as a molecular fingerprint**: Collision cross section (CCS) is a physical property that reflects the shape and size of an ion in the gas phase. cctk predicts CCS from molecular structure (SMILES, InChI) or molecular formula, making it a non-empirical way to annotate MS/MS spectra without requiring synthetic standards.
- **Input formats**: cctk accepts SMILES strings, InChI strings, molecular formulas, or SDF/MOL files as input. Each input mode selects a different prediction pipeline — chemical formula mode uses compositional heuristics, while SMILES mode invokes a trained graph neural network model.
- **Ion type and adduct handling**: Predictions are ion-specific; the user must specify the adduct form (e.g., [M+H]+, [M+Na]+, [M-H]-, [M+Cl]-). Using the wrong adduct results in CCS values that are not comparable to experimental measurements, which are always recorded for a specific ion type.
- **Output formats**: Results are written to CSV by default. The `--json` flag switches output to JSON Lines format. The `--summary` flag produces aggregate statistics (mean, std, range) when processing batch input files.
- **Model versioning**: cctk ships with a default trained model for metabolites (PCPS). Peptide CCS prediction requires the `--model peptide` flag, which loads a separate model trained on tryptic peptide datasets. Switching models without updating input adduct types causes silent mispredictions.

## Pitfalls

- **Forgetting to specify the adduct**: Without `--adduct`, cctk defaults to [M+H]+ for neutral molecules. If your analyte is natively an anion or a metal adduct, the predicted CCS will be systematically offset by 10–30 Å², rendering downstream annotation unreliable.
- **Mixing positive- and negative-mode inputs in a single batch**: Running a SMILES list containing both acidic and basic species without separating adduct types produces nonsense CCS values — negative-mode species predicted under a positive-mode adduct assumption can be off by >20%.
- **Using SDF files with implicit hydrogen counts**: SDF/MOL files may encode hydrogen counts ambiguously. cctk resolves hydrogens via the built-in Chemistry Development Kit (CDK) library; malformed SDF files with incorrect valences produce broken molecule objects and empty output rows without error messages.
- **Assuming CCS prediction is exact**: Machine learning-based CCS prediction has a typical mean absolute error (MAE) of 1.5–3.0 Å² on small metabolites. Treating predicted CCS as a precise measurement rather than a calibrated estimate leads to overconfident annotations, especially for novel scaffolds absent from training data.
- **Ignoring isotope patterns**: The `--isotope` flag controls whether a single monoisotopic mass or a pattern-weighted average is used when input is a molecular formula. Without this flag, formula mode defaults to monoisotopic, which can cause minor mismatches with experimental profiles acquired on high-resolution instruments.

## Examples

### Predict CCS for a single metabolite SMILES as [M+H]+
**Args:** `--smiles "CC(=O)Oc1ccccc1C(=O)O" --adduct "[M+H]+" --output aspirin_ccs.csv`
**Explanation:** This passes an aspirin SMILES string with the protonated adduct ion type and writes the predicted CCS value to a CSV output file.

### Batch-predict CCS for multiple compounds from a text file
**Args:** `--input metabolites.txt --mode smiles --adduct "[M+Na]+" --output batch_ccs.csv --format csv`
**Explanation:** This reads SMILES strings line-by-line from a file, applies sodium adduct ionization, and exports all predictions as a batch CSV.

### Predict CCS for a peptide using peptide-specific model
**Args:** `--smiles "NC(=O)C[NH+]CC(=O)N[C@@H](CCC(=O)O)C(=O)N[C@@H](CC(=O)O)C(=O)O" --adduct "[M+H]+" --model peptide --output peptide_ccs.csv`
**Explanation:** This uses the trained peptide CCS model instead of the default metabolite model, which is necessary for accurate tryptic peptide CCS values.

### Output predicted CCS in JSON Lines format for downstream API integration
**Args:** `--smiles "C[C@@H](N)C(=O)N[C@@H](CCC(=O)O)C(=O)O" --adduct "[M+H]+" --json --output results.jsonl`
**Explanation:** This produces one JSON object per line containing the input identifier, predicted CCS, and confidence interval, suitable for pipeline integration.

### Predict CCS from molecular formula with isotope averaging
**Args:** `--formula "C6H12O6" --adduct "[M+NH4]+" --isotope --output glucose_ccs.csv`
**Explanation:** This uses molecular formula mode with isotope pattern weighting enabled, which is more realistic for high-resolution MS experimental comparison than monoisotopic mass alone.

---

## Concepts

- **CCS as a molecular fingerprint**: Collision cross section (CCS) is a physical property that reflects the shape and size of an ion in the gas phase. cctk predicts CCS from molecular structure (SMILES, InChI) or molecular formula, making it a non-empirical way to annotate MS/MS spectra without requiring synthetic standards.
- **Input formats**: cctk accepts SMILES strings, InChI strings, molecular formulas, or SDF/MOL files as input. Each input mode selects a different prediction pipeline — chemical formula mode uses compositional heuristics, while SMILES mode invokes a trained graph neural network model.
- **Ion type and adduct handling**: Predictions are ion-specific; the user must specify the adduct form (e.g., [M+H]+, [M+Na]+, [M-H]-, [M+Cl]-). Using the wrong adduct results in CCS values that are not comparable to experimental measurements, which are always recorded for a specific ion type.
- **Output formats**: Results are written to CSV by default. The `--json` flag switches output to JSON Lines format. The `--summary` flag produces aggregate statistics (mean, std, range) when processing batch input files.
- **Model versioning**: cctk ships with a default trained model for metabolites (PCPS). Peptide CCS prediction requires the `--model peptide` flag, which loads a separate model trained on tryptic peptide datasets. Switching models without updating input adduct types causes silent mispredictions.

## Pitfalls

- **Forgetting to specify the adduct**: Without `--adduct`, cctk defaults to [M+H]+ for neutral molecules. If your analyte is natively an anion or a metal adduct, the predicted CCS will be systematically offset by 10–30 Å², rendering downstream annotation unreliable.
- **Mixing positive- and negative-mode inputs in a single batch**: Running a SMILES list containing both acidic and basic species without separating adduct types produces nonsense CCS values — negative-mode species predicted under a positive-mode adduct assumption can be off by >20%.
- **Using SDF files with implicit hydrogen counts**: SDF/MOL files may encode hydrogen counts ambiguously. cctk resolves hydrogens via the built-in Chemistry Development Kit (CDK) library; malformed SDF files with incorrect valences produce broken molecule objects and empty output rows without error messages.
- **Assuming CCS prediction is exact**: Machine learning-based CCS prediction has a typical mean absolute error (MAE) of 1.5–3.0 Å² on small metabolites. Treating predicted CCS as a precise measurement rather than a calibrated estimate leads to overconfident annotations, especially for novel scaffolds absent from training data.
- **Ignoring isotope patterns**: The `--isotope` flag controls whether a single monoisotopic mass or a pattern-weighted average is used when input is a molecular formula. Without this flag, formula mode defaults to monoisotopic, which can cause minor mismatches with experimental profiles acquired on high-resolution instruments.

## Examples

### Predict CCS for a single metabolite SMILES as [M+H]+
**Args:** `--smiles "CC(=O)Oc1ccccc1C(=O)O" --adduct "[M+H]+" --output aspirin_ccs.csv`
**Explanation:** This passes an aspirin SMILES string with the protonated adduct ion type and writes the predicted CCS value to a CSV output file.

### Batch-predict CCS for multiple compounds from a text file
**Args:** `--input metabolites.txt --mode smiles --adduct "[M+Na]+" --output batch_ccs.csv --format csv`
**Explanation:** This reads SMILES strings line-by-line from a file, applies sodium adduct ionization, and exports all predictions as a batch CSV.

### Predict CCS for a peptide using peptide-specific model
**Args:** `--smiles "NC(=O)C[NH+]CC(=O)N[C@@H](CCC(=O)O)C(=O)N[C@@H](CC(=O)O)C(=O)O" --adduct "[M+H]+" --model peptide --output peptide_ccs.csv`
**Explanation:** This uses the trained peptide CCS model instead of the default metabolite model, which is necessary for accurate tryptic peptide CCS values.

### Output predicted CCS in JSON Lines format for downstream API integration
**Args:** `--smiles "C[C@@H](N)C(=O)N[C@@H](CCC(=O)O)C(=O)N[C@@H](CC(=O)O)C(=O)O" --adduct "[M+H]+" --json --output results.jsonl`
**Explanation:** This produces one JSON object per line containing the input identifier, predicted CCS, and confidence interval, suitable for pipeline integration.

### Predict CCS from molecular formula with isotope averaging
**Args:** `--formula "C6H12O6" --adduct "[M+NH4]+" --isotope --output glucose_ccs.csv`
**Explanation:** This uses molecular formula mode with isotope pattern weighting enabled, which is more realistic for high-resolution MS experimental comparison than monoisotopic mass alone.

---

## Concepts

- **CCS as a molecular fingerprint**: Collision cross section (CCS) is a physical property that reflects the three-dimensional shape and size of a gas-phase ion. cctk predicts CCS values from molecular structure, enabling annotation of ion mobility spectrometry (IMS) data without requiring physical standards.
- **Input formats**: cctk accepts SMILES, InChI, molecular formulas, and SDF/MOL files as structural input. SMILES mode uses a trained machine learning model; formula mode applies compositional CCS heuristics. Providing the wrong input type for your data causes prediction failures or silent model mismatches.
- **Ion type specificity**: CCS is always measured for a specific ionic form (adduct). cctk requires the `--adduct` flag to specify the charge carrier (e.g., [M+H]+, [M+Na]+, [M-H]-, [M+NH4]+). Predictions without explicit adduct default to [M+H]+, which is incorrect for anionic or metal-adducted species.
- **Output formats**: Default output is CSV containing input, predicted CCS (in Å²), and prediction uncertainty. JSON Lines output (`--json`) is available for programmatic pipelines. Summary statistics mode (`--summary`) aggregates batch results.
- **Model selection**: Default model is trained on metabolite collision cross sections. The `--model peptide` flag loads a separate model for tryptic peptides, which has different training data and produces non-comparable CCS ranges if used on small molecules.

## Pitfalls

- **Omitting the adduct specification**: Running cctk without `--adduct` silently assumes [M+H]+. For negative-mode metabolites or metal-coordinated species, this produces CCS values offset by 10–30 Å² from experimental measurements, leading to incorrect annotations.
- **Using the wrong input mode**: Mixing `--formula` with a SMILES string (or vice versa) causes parsing errors. Similarly, formula mode cannot handle stereochemistry; providing a stereochemically-specific SMILES under formula mode silently discards stereochemical information.
- **Applying peptide model to small molecules**: The peptide CCS model is optimized for 500–3000 Da tryptic peptides. Using `--model peptide` on metabolites produces systematically inflated CCS predictions because the model was not trained on small-molecule conformational ensembles.
- **Accepting predictions without checking uncertainty**: Each prediction includes an uncertainty estimate (Å²). Ignoring this and matching predictions directly to experimental CCS with sub-1 Å² tolerance causes false negatives for compounds near the model's accuracy boundary.
- **Batch processing mixed ionization modes together**: A batch file containing compounds from both positive and negative ionization modes, processed with a single adduct flag, produces mixed-output quality. Positive-mode adduct assumptions on negative-mode species cause >20% relative CCS error.

## Examples

### Predict CCS for a single metabolite SMILES as the protonated ion
**Args:** `--smiles "CC(=O)Oc1ccccc1C(=O)O" --adduct "[M+H]+" --output aspirin_ccs.csv`
**Explanation:** This runs cctk on an aspirin SMILES with explicit protonated adduct specification and writes the predicted CCS to a CSV file.

### Batch-predict CCS from a file of SMILES strings with sodium adduct
**Args:** `--input compounds.txt --mode smiles --adduct "[M+Na]+" --output batch_ccs.csv --format csv`
**Explanation:** This reads SMILES from a line-separated file, applies sodium adduct ionization across all entries, and writes the batch results to CSV.

### Predict peptide CCS using the peptide-trained model
**Args:** `--smiles "NC(=O)C[NH+]CC(=O)N[C@@H](CCC(=O)O)C(=O)N[C@@H](CC(=O)O)C(=O)O" --adduct "[M+H]+" --model peptide --output peptide_ccs.csv`
**Explanation:** This loads the peptide-specific CCS model via the model flag and predicts CCS for a doubly-charged peptide sequence.

### Export results in JSON Lines format for pipeline integration
**Args:** `--smiles "c1ccc2c(c1)[NH+]c3ccccc3O2" --adduct "[M+H]+" --json --output results.jsonl`
**Explanation:** This outputs machine-readable JSON Lines instead of CSV, including the predicted CCS value and uncertainty estimate per line.

### Predict CCS from molecular formula with isotope-weighted averaging
**Args:** `--formula "C6H12O6" --adduct "[M+NH4]+" --isotope --output glucose_ccs.csv`
**Explanation:** This uses formula input mode with isotope pattern weighting, which produces more realistic CCS estimates for high-resolution IMS instruments.