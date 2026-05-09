---
name: biobb_vs
category: Virtual Screening / Drug Discovery
description: A bioinformatics tool from the BioExcel Building Blocks suite for virtual screening workflows, including binding site analysis, ligand docking, and protein-ligand interaction characterization.
tags: [molecular-docking, virtual-screening, drug-discovery, proteins, ligands, bioinformatics, biobb]
author: AI-generated
source_url: https://github.com/bioexcel/biobb_vs
---

## Concepts

- **Modular Subcommand Architecture**: biobb_vs provides distinct subcommands for different virtual screening tasks (e.g., binding site detection, ligand docking, post-docking analysis), each accepting specific input parameters and producing task-specific output files.
- **Configuration-Driven Workflows**: The tool uses JSON or YAML configuration files to define molecular system parameters (receptors, ligands, docking boxes, scoring functions), allowing reproducible and scalable screening pipelines without hardcoding values in CLI arguments.
- **Standard Molecular Formats**: Input/output uses established bioinformatics formats including PDB for receptor structures, SDF/MOL2 for ligand libraries, and PDBQT for docking-ready formats, ensuring interoperability with other molecular modeling tools like AutoDock Vina or Open Babel.
- **Output Aggregates**: Docking results are typically emitted as ranked lists in CSV or SDF format, containing binding scores (G_scores), predicted poses, and auxiliary metadata (RMSD, interactions), enabling downstream filtering or visualization.

## Pitfalls

- **Mismatched Receptor-Ligand Atom Types**: Providing a receptor PDB file with non-standard atom names (e.g., from certain minimization programs) causes parsing failures in docking preparation; always use PDB files with standard residue names or run them through structure preparation tools first.
- **Undersized Docking Box**: Setting the search space (box dimensions) too small excludes viable binding poses, leading to false negatives; the box should encompass the entire binding site plus a margin of at least 4–6 Å in each dimension.
- **Insufficient Ligand Protonation**: Failing to properly ionize ligands at physiological pH produces unrealistic binding scores and poses; use explicit protonation states or rely on the tool's built-in pKa estimation prior to docking.
- **Ignoring Water Molecules**: Retaining crystal water molecules inside the binding site without explicit handling can either help or hinder docking accuracy; decide purposefully (keep structured waters, remove bulk waters) rather than defaulting arbitrarily, as the choice directly impacts pose quality.
- **Version Incompatibilities**: Using mismatched biobb_vs and underlying docking engine versions (e.g., AutoDock Vina) causes parameter parsing errors; verify version compatibility when building reproducible workflows.

## Examples

### Identify binding sites on a receptor structure
**Args:** `--input_receptor_pdb_code 1ABC --output_binding_site_pdb ./binding_site.pdb`
**Explanation:** This downloads or retrieves the PDB structure for receptor 1ABC and identifies and outputs the predicted binding sites to a PDB file for downstream docking.

### Prepare a receptor for docking by adding hydrogens and defining the binding box
**Args:** `--input_receptor ./receptor.pdb --input_ligands ./ligands.sdf --box_center 12.5,34.2,8.7 --box_size 20.0,20.0,20.0 --output_receptor_pdbqt ./receptor_p.pdbqt --output_ligands_pdbqt ./ligands_p.pdbqt`
**Args:** `--input_receptor ./receptor.pdb --input_ligands ./ligands.sdf --box_center 12.5,34.2,8.7 --box_size 20.0,20.0,20.0 --output_receptor_pdbqt ./receptor_p.pdbqt --output_ligands_pdbqt ./ligands_p.pdbqt`
**Explanation:** This prepares both receptor and ligands for docking by adding hydrogens and defining a cubic docking box centered at coordinates (12.5, 34.2, 8.7) with 20 Å edges in each dimension.

### Run virtual screening docking on a ligand library
**Args:** --input_receptor_pdbqt ./receptor_p.pdbqt --input_ligands_pdbqt ./ligands_p.pdbqt --exhaustiveness 16 --num_poses 5 --output docking_results.sdf
**Args:** --input_receptor_pdbqt ./receptor_p.pdbqt --input_ligands_pdbqt ./ligands_p.pdbqt --exhaustiveness 16 --num_poses 5 --output docking_results.sdf
**Explanation:** This executes the docking simulation using exhaustiveness 16 (higher than default for thorough sampling) and retains the top 5 poses per ligand, outputting results to an SDF file for analysis.

### Filter docking results by binding score threshold
**Args:** --input_docking_results ./docking_results.sdf --score_cutoff -7.5 --output filtered_results.sdf
**Args:** --input_docking_results ./docking_results.sdf --score_cutoff -7.5 --output filtered_results.sdf
**Explanation:** This filters the docking output to keep only poses with binding scores (G_scores) stronger than -7.5 kcal/mol, outputting the filtered ligand poses to a new SDF file.

### Calculate RMSD between docked poses and a reference crystal ligand
**Args:** --input_reference ./crystal_ligand.sdf --input_docked ./docking_results.sdf --output rmsd_analysis.csv
**Args:** --input_reference ./crystal_ligand.sdf --input_docked ./docking_results.sdf --output rmsd_analysis.csv
**Explanation:** This computes the RMSD of each docked pose against the known crystal ligand conformation and writes numerical RMSD values to a CSV file for validation and ranking.

### Extract and visualize top-ranked poses with their interaction fingerprints
**Args:** --input_docking_results ./docking_results.sdf --top_n 10 --output interactions.txt
**Args:** --input_docking_results ./docking_results.sdf --top_n 10 --output interactions.txt
**Explanation:** This extracts the top 10 scoring poses from the docking results and writes their interaction fingerprints (hydrogen bonds, hydrophobic contacts) to a text file for visualization or further analysis.

### Export docking results to a CSV summary with scores and ligand IDs
**Args:** --input_docking_results ./docking_results.sdf --output_summary ./results_summary.csv
**Args:** --input_docking_results ./docking_results.sdf --output_summary ./results_summary.csv
**Explanation:** This converts the SDF-formatted docking results into a tabular CSV containing ligand identifiers, binding scores, and pose IDs for easier downstream data processing.