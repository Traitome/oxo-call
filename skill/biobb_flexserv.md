---
name: biobb_flexserv
category: structure_analysis
description: Tool for analyzing molecular flexibility using normal mode analysis (NMA) on protein structures. Performs essential dynamics analysis on PDB trajectories to calculate atomic fluctuations, eigenvectors, and collectivity indices.
tags:
  - molecular-dynamics
  - flexibility-analysis
  - normal-modes
  - pdb
  - protein-structure
  - essential-dynamics
  - atomic-fluctuations
author: AI-generated
source_url: https://github.com/bioexcel/biobb_flexserv
---

## Concepts

- **Input Format:** Accepts PDB structure files containing atomic coordinates. The tool performs NMA on Calpha atoms for computational efficiency, requiring a properly processed structure with missing hydrogens or heavy atoms potentially filled.
- **Analysis Output:** Generates eigenvalue spectrum, eigenvectors, and atomic fluctuation profiles. Output includes essential dynamics data showing the number of modes required to describe specified variance (typically 90-95% for most flexible regions).
- **Method Variants:** Implements different normal mode methods (LNM, ANM, GNM) selectable via flags. The Gaussian Network Model (GNM) uses Kirchhoff matrix construction while Atomic NMA uses Hessian matrix for more detailed calculations.
- **CLI Invocation:** The tool is executed via the `biobb_flexserv` command with subcommands (e.g., `biobb_flexserv pcaserver`, `biobb_flexserv anm`). Configuration is typically JSON or YAML based, with input/output paths specified explicitly.

## Pitfalls

- **Using non-Calpha chains:** Specifying chains with only backbone atoms or incomplete residues causes the analysis to fail or produce meaningless results, as NMA requires Calpha coordinates for proper elastic network construction.
- **Overlooking missing residue numbering:** Sending structures with discontinuous residue numbering or missing residues leads to incorrect distance matrix calculations, corrupting the entire normal mode analysis.
- **Neglecting eigenvalue convergence:** Setting too few modes for analysis (e.g., less than required for 95% variance) misses critical flexible regions, producing incomplete flexibility profiles that misrepresent protein dynamics.
- **Ignoring output format mismatches:** Requesting output formats incompatible with downstream tools wastes computation. Ensure trajectory format matches subsequent molecular dynamics processing pipelines.

## Examples

### Calculate principal components from a PDB trajectory
**Args:** --input_pdb_path trajectories.pdb --output_pcs_json eigenvalues.json --num_modes 20
**Explanation:** Extracts principal components from trajectory frames using Calpha atoms, outputting the top 20 eigenvalue/eigenvector pairs for essential dynamics visualization.

### Run Anisotropic Network Model analysis
**Args:** --input_pdb_path protein.pdb --anm_output anm_results.csv --num_modes 50 --cutoff_distance 8.0
**Explanation:** Performs ANM with 8.0 Angstrom cutoff distance on all Calpha atoms, computing 50 normal modes and storing elastic network results for flexibility mapping.

### Generate Gaussian Network Model with custom chain selection
**Args:** --input_pdb_path protein.pdb --output_gnm gnm_modes.csv --chain_ids A,B --num_modes 30 --force_constant 1.0
**Explanation:** Runs GNM analysis selecting chains A and B only, using default force constant of 1.0 kcal/mol/A², producing 30 mode eigenvalues for specified chain flexibility.

### Compute atomic fluctuation profiles
**Args:** --input_pdb_path structure.pdb --output_fluctuations rmsf.csv --num_modes 100 --method lnm
**Explanation:** Calculates root mean square fluctuations using linear normal mode method across 100 modes, outputting per-residue fluctuation values for B-factor comparison.

### Extract eigenvalues for specific residue range
**Args:** --input_pdb_path protein.pdb --output_eigenvalues eigen.json --residue_range 50-200 --num_modes 25
**Explanation:** Restricts NMA calculation to residues 50-200, producing filtered eigenvalue spectrum for local domain flexibility analysis without full protein computation.