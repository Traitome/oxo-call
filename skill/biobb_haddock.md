---
name: biobb_haddock
category: biomolecular-docking
description: Python library and CLI wrapper for HADDOCK3 information-driven biomolecular docking. Provides tools to validate docking protocols, build input structures, generate topologies for small molecules, run ensemble docking, and analyze docking results.
tags: [protein-protein-docking, molecular-dynamics, docking-analysis, ambiguous-interaction-restraints, ensemble-docking, haddock3]
author: AI-generated
source_url: https://github.com/haddocking/biobb-haddock
---

## Concepts

- HADDOCK (High Ambiguity Driven biomolecular Docking) is an information-driven flexible docking approach that uses Ambiguous Interaction Restraints (AIRs) derived from experimental data (e.g., NMR chemical shift perturbation, mutagenesis, bioinformatic predictions) to guide the docking process. AIRs encode which residues are likely to participate in the interface, and their correct definition is the single most important factor determining docking quality. Without AIRs, HADDOCK defaults to a purely rigid-body search with significantly reduced sampling effectiveness.
- HADDOCK works with CNS (Crystallographic Neural Network / Crystallography Neighbourhood Standard) input file formats and produces `.pdb` coordinate files and `.txt` scoring/output files. PDB input files must have properly defined chain IDs (e.g., `A`, `B`, `C`) — not numerical chain labels — and all crystal waters, non-structural heteroatoms, and alternative conformers should be removed before docking to avoid crashes or spurious restraints.
- BioBB HADDOCK tools are organized as subcommands: `check_protocol` validates a HADDOCK run configuration file before execution; `build_model` assembles multi-chain PDB structures from individual chain files; `ligand_top` generates topology and parameter files for small-molecule ligands from an SDF or MOL2 file; `run` executes a HADDOCK3 docking protocol end-to-end; `analysis` computes interface metrics (L-RMSD, I-RMSD, Fnat, ILMRMSD) for a set of generated models against a reference structure.
- Ensemble docking in HADDOCK allows multiple conformers of the same biomolecule to be used simultaneously, capturing intrinsic flexibility. For ensemble input, all conformers must share identical chain IDs and residue numbering, and the ensemble size must be specified separately for each molecule in the run configuration. Ensemble docking substantially increases the computational cost (linearly with the number of conformers) but dramatically improves success rates on inherently flexible targets.
- HADDOCK scoring consists of an `HADDOCK-score` (weighted sum of intermolecular van der Waals, electrostatic, and AIR violation energies) and an `Edes` score (desolvation penalty). Lower HADDOCK scores indicate better models. Post-docking analysis tools compute `I-RMSD` (RMSD of interface Cα atoms after superposition on the reference complex), `L-RMSD` (RMSD of the ligand when the receptor is superimposed), and `Fnat` (fraction of native contacts within 5 Å), which are the primary metrics for model quality assessment.

## Pitfalls

- Specifying wrong or mismatched chain IDs in the HADDOCK run configuration will cause the docking to either crash or produce completely meaningless results, because HADDOCK maps PDB chain labels directly to its internal AIR definitions. Always verify that the chain IDs in `moleculetypes` and `molecules` sections of the config exactly match the chain identifiers in the input PDB files (e.g., `A` and `B`, not `0` and `1`).
- Failing to remove crystal waters, bound ions, and alternative conformers (ATOM records with `ALT` flags) before providing PDB structures to the docking pipeline will cause the `build_model` step to fail or generate spurious ambiguous restraints involving those non-interacting heteroatoms. Always preprocess input PDBs using a structure-cleaning tool to remove all `HOH`, `ION`, and `ALT` records.
- Running HADDOCK without any ambiguous interaction restraints for a protein-protein docking target results in a docking that skips the flexible refinement stage entirely and falls back to rigid-body sampling only. This severely limits the searchable conformational space and typically produces low-quality models even for strong interactors. Define at least a set of `active` and `passive` residues for each docking partner before running.
- Setting ensemble sizes incorrectly for ensemble docking — for example, specifying different numbers of conformers for the same molecule across different input files — will cause the `analysis` step to crash or generate misaligned cluster assignments. Ensemble sizes must be consistent and identical across all ensemble-related input parameters in the configuration.
- Forgetting to specify the correct CNS executable path in the `haddock3.cfg` file when HADDOCK is not in the system `$PATH` causes silent failures where the docking runs but produces no output structures, wasting all allocated compute time. Always verify the `haddock3` binary resolves correctly with `which haddock3` before submitting a large docking job.

## Examples

### Validate a HADDOCK docking protocol configuration before execution
**Args:** `check_protocol --config run.yaml --output validation_report.txt`
**Explanation:** The `check_protocol` subcommand parses the HADDOCK run configuration file and reports missing fields, malformed molecule paths, and invalid AIR definitions, preventing wasted compute time on a misconfigured job.

### Build a multi-chain PDB structure for docking by assembling individual chain files
**Args:** `build_model --input_pdb receptor.pdb --input_pdb ligand.pdb --output_structure complex.pdb`
**Explanation:** The `build_model` tool concatenates separate PDB chain files into a single structure with correctly ordered chains, which is the required input format for the HADDOCK `molecules` configuration section.

### Generate topology and parameter files for a small-molecule ligand from an SDF file
**Args:** `ligand_top --input_ligand molecule.sdf --output_top ligand.itp --output_par ligand.prm`
**Explanation:** The `ligand_top` tool converts an SDF MOLfile representation of a small organic molecule into GROMACS-style topology (`*.itp`) and parameter (`*.prm`) files that HADDOCK can use for ligand-parameterized docking.

### Execute a full HADDOCK3 docking protocol end-to-end
**Args:** `run --config run.yaml --output_folder results/ --nproc 8`
**Explanation:** The `run` subcommand launches the complete HADDOCK3 docking pipeline (rigid-body sampling, flexible refinement, water refinement, and scoring) using the specified configuration file and parallelization across 8 processors.

### Analyze docking models by computing interface RMSD and native contact metrics
**Args:** `analysis --models results/structures --ref reference.pdb --output analysis.csv`
**Explanation:** The `analysis` tool calculates I-RMSD, L-RMSD, Fnat, and ILMRMSD for every generated model against the reference complex, writing a CSV file that is used to rank and select top-scoring docking poses.

### Run ensemble docking using multiple conformers of the receptor molecule
**Args:** `run --config ensemble_run.yaml --output_folder ensemble_results/`
**Explanation:** When the configuration file specifies an ensemble of receptor conformers (via `ensemble_size` > 1), the `run` subcommand automatically expands the rigid-body search to include all ensemble combinations, producing a wider variety of docked poses.

### Generate ligand restraint file for flexible receptor-ligand docking
**Args:** `build_restraints --ligand mol.sdf --output_restraints ligand_rst AIR --cutoff 10.0`
**Explanation:** The `build_restraints` tool automatically generates ambiguous interaction restraints for a ligand in flexible receptor-ligand docking by defining accessible surface residues within a 10 Å cutoff of the ligand center, replacing manual restraint curation.