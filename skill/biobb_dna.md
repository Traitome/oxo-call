---
name: biobb_dna
category: DNA Structure Analysis
description: A Python package from BioExcel Building Blocks that provides wrappers for DNA structure analysis tools, including DNA curvature calculation, Monte Carlo simulations, and helical parameter analysis. Wraps binaries such as dnaMC and curves.
tags:
  - DNA structure
  - Monte Carlo simulation
  - DNA curvature
  - Helical parameters
  - Computational biochemistry
  - Molecular dynamics analysis
author: AI-generated
source_Url: https://biobb-dna.readthedocs.io/
---

## Concepts

- **Input Format Requirements**: biobb_dna primarily accepts DNA structures in PDB format or AMBER topology/parm files. For Monte Carlo simulations (dnaMC), a properly formatted AMBER leap-generated topology file is mandatory—using a PDB file alone will cause the wrapper to fail with a topology parsing error.

- **Output Generates Multiple Data Files**: Each analysis task produces separate output files including JSON result summaries, dat files with numerical data (e.g., curvature profiles, flexibility values), and optional plot-ready data for visualization. The `--output_json_path` parameter controls the main results file, while auxiliary outputs are written to the same directory with auto-generated names.

- **Helical Parameter Calculation**: The package computes roll, tilt, shift, slide, and rise values per base pair step using geometric transformations. These parameters require a reference ideal B-DNA structure for normalization—without specifying the `--ref_frame` option, relative deviations from canonical B-DNA are not calculated.

- **Temperature and Simulation Control**: For dnaMC simulations, temperature must be specified in Kelvin using `--temperature` (default 300K). Incorrect temperature values (e.g., using Celsius) will silently produce physically meaningless results without raising an error.

## Pitfalls

- **Topology File Mismatch**: Providing an AMBER topology file generated from a protein or RNA structure (instead of DNA) causes silent failures or garbage output because DNA-specific base pair and sugar parameters are absent. Always verify the molecule type in the topology file header before running simulations.

- **Missing base pair Step Information**: When analyzing helical parameters, if the input structure lacks complete base pair definitions (e.g., has chain breaks or non-standard residues), the wrapper outputs NaN values for affected steps without warning. Check input structures with a molecular viewer before analysis.

- **Output Directory Not Pre-created**: If the specified output directory does not exist, the wrapper raises a generic FileNotFoundError that does not indicate the directory is missing. Explicitly create output directories with `mkdir -p` before invoking the tool.

- **Force Field Incompatibility**: Using non-DNA-compatible AMBER force fields (e.g., ff19SB designed for proteins) with dnaMC produces simulation artifacts. The package expects DNA-specific parameters—always use DNA-optimized force fields like OL15 or BSC1 for the leap topology generation step.

## Examples

### Calculate DNA curvature from a PDB structure
**Args:** `--input_structure_path ./dna_curved.pdb --output_json_path ./curvature_results.json --output_dat_path ./curvature_profile.dat`
**Explanation:** Reads the DNA structure from the PDB file and computes local curvature angles at each base pair step, outputting numerical values to the dat file and a summary to JSON.

### Run DNA Monte Carlo simulation at physiological temperature
**Args:** `--input_topo_path ./dna topology.parm7 --input_coord_path ./dna.inpcrd --output_json_path ./mc_sim.json --temperature 310 --steps 10000 --output_traj_path ./trajectory.pdb`
**Explanation:** Performs a Monte Carlo simulation using the AMBER topology and coordinates, running 10,000 steps at 310K and saving the final trajectory to the specified PDB file.

### Compute helical parameters (roll, tilt, shift, slide) per base pair
**Args:** `--input_structure_path ./dna helix.pdb --output_json_path ./helical_params.json --ref_frame canonical_bdna`
**Explanation:** Extracts the six helical parameters for each base pair step relative to canonical B-DNA, outputting the complete parameter set as JSON for downstream analysis or plotting.

### Analyze DNA flexibility from an ensemble of structures
**Args:** `--input_ensemble_path ./dna_ensemble --input_format pdb --output_json_path ./flexibility.json --parameter slide --window_size 6`
**Explanation:** Analyzes flexibility by computing standard deviations of the specified parameter (slide) over a 6-base-pair sliding window across multiple PDB structures in the ensemble directory.

### Generate a DNA mechanical profile for molecular dynamics setup
**Args:** `--input_topo_path ./dna.parm7 --input_coord_path ./dna.rst7 --output_json_path ./mech_profile.json --calculation_type stiffness --output_dat_path ./stiffness_values.dat`
**Explanation:** Computes DNA stiffness constants from the input structure, generating a mechanical profile suitable for setting up mesoscopic DNA models or coarse-grained simulations.

### Prepare AMBER topology for DNA before simulation
**Args:** biobb_dna-build --input_pdb_path ./dna.pdb --output_topo_path ./dna.parm7 --output_lib_path ./dna.off --force_field OL15 --water_model TIP3P`
**Explanation:** Uses the companion build tool to generate a complete AMBER topology file and library for the DNA structure, employing the OL15 force field and TIP3P water model required for subsequent dnaMC simulations.