---
name: biobb_wf_mutations
category: workflow/bioinformatics
description: BioExcel Building Blocks workflow for protein point mutation analysis, including structure preparation, mutation introduction, and structural relaxation.
tags: [protein, mutation, bioinformatics, molecular-dynamics, workflow]
author: AI-generated
source_url: https://biobb.readthedocs.io/en/latest/
---

## Concepts

- The workflow chains multiple BioBB building blocks sequentially: structure preparation, point mutation introduction, and structural relaxation (energy minimization or short MD). Each step consumes outputs from the previous step as inputs, creating strict data dependencies.

- Input structures must be provided as PDB files with correct atom naming and residue labeling conventions. The workflow does not re-index residues; the configuration must use the same residue numbering as present in the input structure.

- The mutation configuration requires specifying the chain identifier, original residue, and target residue using three-letter amino acid codes. Wild-type and mutant residues must be valid PDB residue names (e.g., ALA, GLY, LEU, ARG).

- Output is written to a user-specified output directory, organized into subdirectories per workflow step (e.g., structure preparation outputs, mutation outputs, relaxation outputs). Each step also generates a log file for debugging.

- The workflow supports running individual steps separately for testing or reprocessing. Configuration files for each step are written to the output directory as JSON files, enabling step reuse or modification.

## Pitfalls

- Providing structures without hydrogen atoms causes the preparation step to fail or produce incorrect protonation states, leading to distorted bond lengths and angles during mutation introduction. Always ensure input structures are properly protonated before running.

- Using non-standard residue numbering in the configuration (e.g., insertion codes or shifted numbering) results in mutations being introduced at the wrong positions, producing incorrect structural models with potentially severe steric clashes.

- Specifying a non-existent output directory path causes the workflow to fail immediately. The parent directory must exist before running; the workflow does not create directories automatically.

- Requesting mutations on chains or residues not present in the input structure causes the mutation step to terminate with an error. Validate chain identifiers and residue positions against the input PDB file before running.

- Using configuration files with incorrect YAML syntax (e.g., indentation errors, missing colons) causes the workflow to fail at the parsing stage with obscure Python errors. Always validate YAML structure before execution.

## Examples

### Basic point mutation on a single chain
**Args:** `--input_protein_structure ./structures/lysozyme.pdb --mutation_chain A --mutation_residue 50 --mutation_type ALA --output_dir ./results/mutation_A50A`
**Explanation:** This runs the complete mutation workflow on chain A, changing residue 50 to alanine, using default settings for all steps and writing results to the specified output directory.

### Multi-step mutation with energy minimization
**Args:** `--input_protein_structure ./structures/1ake.pdb --mutation_chain A --mutation_residue 102 --mutation_type GLY --output_dir ./results/A102G --relaxation_step em --max_iterations 1000`
**Explanation:** This introduces the A102G mutation and applies energy minimization for structural relaxation, increasing iterations from the default 500 to 1000 for better convergence on glycine substitutions.

### Mutation with explicit pH for protonation
**Args:** `--input_protein_structure ./structures/enzyme.pdb --mutation_chain B --mutation_residue 31 --mutation_type LYS --output_dir ./results/B31K --ph 7.0 --ionization_mode consistent`
**Explanation:** This specifies pH 7.0 and consistent ionization mode to ensure lysine residues remain protonated and histidine residues are assigned appropriately for the mutation step.

### Test run skipping relaxation step
**Args:** `--input_protein_structure ./structures/test.pdb --mutation_chain A --mutation_residue 25 --mutation_type SER --output_dir ./results/test_only --skip_relaxation`
**Explanation:** This runs only structure preparation and mutation steps without relaxation, useful for testing that the mutation is introduced correctly before committing to expensive minimization.

### Mutation with custom force field parameters
**Args:** `--input_protein_structure ./structures/complex.pdb --mutation_chain C --mutation_residue 88 --mutation_type VAL --output_dir ./results/C88V --force_field amber99sb --water_model tip3p`
**Explanation:** This specifies the AMBER99SB force field and TIP3P water model for both mutation introduction and relaxation steps, ensuring compatibility with downstream analysis requiring AMBER parameters.