---
name: biobb_flexdyn
category: Molecular Dynamics
description: Analyze protein flexibility and conformational dynamics using elastic network models or normal mode analysis. Typically computes collective motions, residue flexibility indices, and generates animated trajectories representing low-frequency normal modes.
tags:
- protein-dynamics
- elastic-network-model
- normal-modes
- flexibility-analysis
- molecular-modeling
- conformational-changes
author: AI-generated
source_url: https://github.com/bioexcel/biobb_flexdyn
---

## Concepts

- **Input Structure Format**: biobb_flexdyn accepts PDB format structures (typically after removing water molecules and ligands), processing only the protein chain coordinates for flexibility computation.
- **Elastic Network Model**: The tool constructs a Gaussian network or anisotropic network model where residues are nodes and edges connect residues within a cutoff distance (usually 7-10 Å), preserving the native Fold topology.
- **Output Data Types**: The analysis produces eigenvector files containing normal mode amplitudes, animation trajectory files showing mode deformation, and textual output reporting fluctuation amplitudes per residue.

## Pitfalls

- **Missing Residues or Gaps**: Input PDB files with incomplete backbone atoms or chain breaks cause the elastic network model to fail, as the connectivity graph cannot be properly constructed across discontinuous segments.
- **Non-Standard Residues**: Modified or non-canonical amino acids (e.g., selenomethionine, post-translational modifications) in the input structure crash the analysis or produce incorrect results since they're omitted from standard connectivity matrices.
- **Overly Large Structures**: Applying flexibility analysis to structures with >10,000 residues dramatically increases computational time and memory usage, often causing timeout or memory errors on standard workstations.

## Examples

### Compute backbone flexibility using elastic network model
**Args:** --input_structure_path step1_protein.pdb --output_nm_path eigenvecs.nc --output_trajectory_path mode_animation.pdb --parameters_json '{"cutoff": 8.0, "nmodes": 10}'
**Explanation:** This generates the 10 lowest-frequency normal modes (collective motions) by constructing an elastic network with 8 Å distance cutoff between C-alpha atoms.

### Analyze hinge residues in a multi-domain protein
**Args:** --input_structure_path protein_4iz9.pdb --output_fluctuations_path fluctuats.csv --output_nm_path nmodes.nc --parameters_json '{"method": "anm", "cutoff": 7.5}'
**Explanation:** Using the anisotropic network method with tighter cutoff identifies flexible hinge regions often located at domain boundaries.

### Generate animated mode visualization for publication
**Args:** --input_structure_path target.pdb --output_trajectory_path mode7_animation.pdb --parameters_json '{"animation_mode": "both", "skip_unselected": true}'
**Explanation:** Creates a PDB trajectory showing both forward and backward excursions along mode 7, useful for visualizing conformational changes in molecular viewers.

### Extract residue-specific flexibility scores
**Args:** --input_structure_path enzyme.pdb --output_fluctuations_path residue_flex.csv --parameters_json '{"flatten_output": true}'
**Explanation:** Exports per-residue mean squared fluctuation values as a simple CSV table with residue numbers and corresponding flexibility indices.

### Compare dynamics between wild-type and mutant
**Args:** --input_structure_path mutant_complex.pdb --output_nm_path mutant_modes.nc --parameters_json '{"mass_weighted": true, "nmodes": 20}'
**Explanation:** Computing 20 normal modes with mass-weighting normalizes for heavier atoms, enabling direct comparison with wild-type dynamics results.

### Run with explicit output control for pipeline use
**Args:** --input_structure_path complex.pdb --output_nm_path modes.nmz --output_summary_path summary.json --parameters_json '{"write_summary": true, "hessian_output": false}'
**Explanation:** Explicitly controls output files and disables Hessian matrix output to reduce disk usage in automated workflows.

### Analyze allosteric pathway via domain motions
**Args:** --input_structure_path allosteric.pdb --output_fluctuations_path domain_flux.csv --parameters_json '{"domain1": [1, 150], "domain2": [300, 450], "cutoff": 9.0}'
**Explanation:** Defining two domains calculates inter-domain motions along normal modes, revealing potential allosteric communication pathways between distant regions.