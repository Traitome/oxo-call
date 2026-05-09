---
name: biobb_analysis
category: Molecular Dynamics Analysis
description: A bioinformatics building blocks library for analyzing molecular dynamics trajectories and protein structures. Provides tools for calculating RMSD, RMSF, radius of gyration, SASA, hydrogen bonds, secondary structure (DSSP), and residue contacts.
tags: [molecular-dynamics, trajectory-analysis, rmsd, rmsf, gyration, sasa, hbonds, dssp, protein-structure, bioinformatics]
author: AI-generated
source_url: https://github.com/bioexcel/biobb_analysis
---

## Concepts

- **Input formats**: Accepts PDB, GRO, and molecular dynamics trajectory formats (XTC, TRR, CDC, HDF5) for analyzing conformational changes over time.
- **Python API with CLI wrappers**: Primary interface is through Python classes (`Rmsd`, `Rmsf`, `RadiusGyration`, `Sasa`, `Hbonds`, `DSSP`), but command-line wrappers enable batch processing.
- **Reference structure support**: Many analyses (RMSD, RMSF) support a separate reference structure to compare against, enabling precise conformational difference measurements.
- **Output formats**: Generates numerical results as CSV, TSV, or JSON files, plus visualization plots in PNG or EPS formats for publication-quality figures.
- **Feature selection**: Supports residue selection strings (e.g., "protein and name CA") to restrict analysis to specific atoms or residues, improving performance and relevance.

## Pitfalls

- **Mismatched atom selections**: Using different selection strings between reference and target structures causes index mismatch errors, resulting in "Array shape mismatch" exceptions that halt analysis.
- **Missing trajectory frames**: Trajectories with missing frames or corrupted XTC/TRR files produce silent failures or NaN values in output, leading to incorrect conclusions if not visually inspected.
- **Insufficient trajectory length**: Analyzing fewer than 100 frames produces statistically unreliable RMSF and average structure calculations, as short trajectories have high statistical noise.
- **Ignoring periodic boundary conditions**: Not using wrap-around options causes molecules to appear split across box edges, artificialy inflating radius of gyration and SASA values.
- **Inconsistent water/ion removal**: Failing to strip water and ions consistently across reference and target structures introduces artifacts in RMSD calculations and hydrogen bond analysis.

## Examples

### Calculate RMSD between a trajectory frame and reference structure
**Args:** --input_structure_path protein_md.xtc --input_ref_pdb protein_ref.pdb --output_csv_path rmsd_results.csv --output_figure_path rmsd_plot.png
**Explanation:** Compares each frame of an MD trajectory against a reference PDB to quantify structural deviation across simulation time.

### Compute RMSF (flexibility) per residue from a trajectory
**Args:** --input_structure_path md_trajectory.xtc --input_starting_structure protein_start.pdb --output_csv_path rmsf_per_residue.csv --output_figure_path rmsf_plot.png
**Explanation:** Analyzes atomic fluctuations to identify flexible loop regions and stable secondary structure elements, useful for understanding protein dynamics.

### Calculate radius of gyration over simulation time
**Args:** --input_structure_path traj.xtc --input_starting_structure structure.gro --output_csv_path rg_time_series.csv --output_figure_path rg_plot.png
**Explanation:** Monitors protein compaction and folding by measuring mass-weighted distance from center of mass, detecting folding or unfolding events.

### Analyze solvent-accessible surface area
**Args:** --input_structure_path md_frames.xtc --input_starting_structure protein.pdb --output_csv_path sasa_results.csv --output_figure_path sasa_plot.png --surface_selection "protein and name CA"
**Explanation:** Computes SASA to identify buried versus exposed residues, important for binding site analysis and drug accessibility studies.

### Determine secondary structure evolution via DSSP
**Args:** --input_structure_path trajectory.xtc --input_starting_structure protein.pdb --output_csv_path dssp_evolution.csv --output_figure_path dssp_timeline.png
**Explanation:** Assigns secondary structure (alpha-helix, beta-sheet, coil) for each frame using DSSP algorithm, visualizing structural transitions over simulation.

### Calculate hydrogen bond occupancy
**Args:** --input_structure_path md.xtc --input_starting_structure protein.pdb --output_csv_path hbonds_occupancy.csv --output_figure_path hbonds_plot.png --donor_selection "protein and name N" --acceptor_selection "protein and name O"
**Explanation:** Counts hydrogen bonds and computes their occupancy to identify stable intra-protein or protein-ligand interactions.

### Generate residue-residue contact maps
**Args:** --input_structure_path traj.xtc --input_starting_structure protein.pdb --output_csv_path contacts_matrix.csv --output_figure_path contacts_map.png --cutoff_distance 5.0
**Explanation:** Identifies residue pairs within a cutoff distance to map stable contacts, helping characterize protein domains and allosteric networks.

### Compute distance between two atom groups
**Args:** --input_structure_path frame.pdb --output_csv_path distance_results.csv --group1_selection "resname LIG and name C1" --group2_selection "protein and name CA and resid 50"
**Explanation:** Measures the distance between specified atom groups, useful for monitoring ligand binding pocket proximity or domain movements.