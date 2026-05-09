---
name: autodock
category: Structure-Based Drug Design
description: AutoDock is a molecular docking software suite that predicts how small molecule ligands bind to a receptor protein of known 3D structure. It uses a Lamarckian genetic algorithm to explore ligand conformations and optimize binding affinity predictions.
tags:
- molecular docking
- ligand-protein interaction
- drug discovery
- computational chemistry
- binding affinity
- virtual screening
author: AI-generated
source_url: https://autodock.scripps.edu/
---

## Concepts

- **Input Data Model**: AutoDock uses pre-prepared receptor PDBQT files (rigid protein with polar hydrogens) and ligand PDBQT files (flexible ligands with torsion bonds defined). Docking parameters are specified in a DPF (Docking Parameter File) that defines search settings, while a GPF (Grid Parameter File) defines the 3D binding site box for energy calculations.

- **Output Formats**: The primary output is a DLG (Docking Log File) containing predicted binding modes, energies, and cluster analysis. Results are written as PDBQT files for individual poses. The .dlg file includes binding energy estimates (in kcal/mol), cluster histograms showing pose distributions, and ligand RMSD values relative to reference poses.

- **Key Behaviors**: AutoDock performs flexible ligand docking against a rigid receptor by mapping ligand torsions onto the receptor's pre-calculated Grid maps (electrostatic, van der Waals, hydrogen bonding, desolvation, and torsion). The Lamarckian GA evolves population members by applying genetic operators then locally optimizing each offspring before evaluation, combining global exploration with local refinement.

- **Parameter Independence**: Each docking requires independent specification of three parameter files (DPF, GPF, and GLG maps) that must be generated using autodock tools or similar preprocessing. Ligand and receptor atoms have specific PDBQT atom types matching AutoDock's force field, requiring careful preparation to avoid type mismatches that cause silent failures or unreliable energies.

---

## Pitfalls

- **Incomplete Ligand Preparation**: Using incorrectly typed atoms or missing polar hydrogens leads to unpredictable force field interactions and inaccurate binding scores. Ligands must have all hydrogen atoms (including polar hydrogens on heteroatoms) and use AutoDock atom type conventions (e.g., OA for oxygen, NA for nitrogen, S for sulfur).

- **Grid Box Misalignment**: If the defined binding site box (via GPF) does not encompass all relevant receptor binding site residues, optimal ligand poses may land outside the searchable space, producing false negative results where the best poses appear artificially poor due to truncated interactions.

- **Insufficient Search Sampling**: Setting the population size or number of evaluations too low causes premature convergence to local minima, producing unreliable binding mode predictions. Small ligand sets require at least 50,000 energy evaluations, while large virtual screening runs need 1,000,000+ evaluations for statistical significance.

- **Ignoring Cluster Analysis**: Assuming the lowest numerical binding energy represents the correct pose without considering cluster population sizes leads to overinterpretation of single poor-quality poses. Clusters containing >10% of sampled poses are more statistically robust than isolated low-energy outliers.

---

## Examples

### Run a basic ligand docking with standard genetic algorithm settings
**Args:** `-l ligand.pdbqt -r receptor.pdbqt -o docking.dlg -p docking.dpf -g grid.dpf`
**Explanation:** Specifies ligand and receptor input files, names the output log file, references the docking parameter file for search algorithm settings, and references the grid parameter file defining the binding site box.

### Specify high-throughput docking with reduced evaluations for virtual screening
**Args:** `-popfile newsearch.pdb -nruns 10 -nlev 25000`
**Explanation:** Uses a parameter override to set 10 independent docking runs per ligand with 25,000 energy evaluations each, balancing speed and sampling quality appropriate for screening thousands of compounds.

### Analyze clustering results from a completed docking log
**Args:** `-t 1 docking.dlg > best_cluster1.txt`
**Explanation:** Extracts the first travel cluster (lowest energy cluster) from the docking log file using the -t 1 flag, redirecting the clustered poses summary to a text file for downstream analysis.

### Resume an interrupted docking from checkpoint files
**Args:** `-l ligand.pdbqt -r receptor.pdbqt -o docking_resume.dlg -p resuming.dpf -e checkpoint.dlg.RESTART`
**Explanation:** Continues a docking run by specifying the original ligand, receptor, and parameter files while using the `-e` flag to load the last checkpoint state from the previous run's .dlg.RESTART file.

### Generate affinity maps for a specific binding site before docking
**Args:** `-i receptor.pdbqt -p gridParameter.dpf -o receptor.maps.fld`
**Explanation:** Uses AutoGrid (the companion program) to pre-compute interaction energy maps across the defined grid box, creating the .fld file needed by AutoDock for efficient energy calculations during docking.

### Set up flexible docking allowing specific ligand torsions
**Args:** `-l ligand_with_torsions.pdbqt -o flexible.dlg -p torsions.dpf -r receptor.pdbqt`
**Explanation:** Performs flexible ligand docking by referencing a ligand PDBQT file that has active torsion bonds defined (detected by autodock tools during preparation), allowing the genetic algorithm to rotate those bonds during conformational search.