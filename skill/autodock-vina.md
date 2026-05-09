---
name: autodock-vina
category: Molecular Docking
description: AutoDock Vina is a fast open-source molecular docking program based on efficient iterative gradient descent optimization. It accepts protein–ligand structures in PDBQT format, exhaustively searches binding poses within a user-defined search space, and outputs ranked binding affinities. Used in virtual screening, lead optimization, and structural biology research.
tags:
  - molecular docking
  - virtual screening
  - structure-based drug design
  - scoring function
  - protein–ligand interaction
  - receptor
  - ligand
  - PDBQT
  - binding affinity
  - computational chemistry
author: AI-generated
source_url: https://github.com/cdds-lab/vina
---

## Concepts

- AutoDock Vina operates on PDBQT files, a PDB-derived format that encodes atom types, partial charges, and rotatable dihedrals. Both the receptor (protein) and ligand must be pre-processed with the `prepare_receptor` and `prepare_ligand` utilities (from MGLTools) before docking. PDBQT is the primary I/O format throughout the workflow.
- The program defines a **search space** (a rectangular box in Ångströms) centered on a specific region of the receptor, specified by `--center_x/y/z` and `--size_x/y/z`. Docking is only performed within this box. A box that is too small produces unreliable poses; a box that is too large wastes computation. The exhaustiveness parameter (`--exhaustiveness`, default 8) controls the number of independent docking runs and directly trades off runtime against sampling quality.
- Binding affinity is reported in **kcal/mol** via a built-in empirical scoring function derived from the AutoDock4 force field. The lower (more negative) the score, the tighter the predicted binding. Multiple output poses are ranked by score, with `--num_modes` controlling how many are retained. Energy differences between top poses inform relative binding preference.

## Pitfalls

- **Forgetting to add non-polar hydrogen atoms before PDBQT conversion**: MGLTools `prepare_ligand` and `prepare_receptor` add hydrogens automatically, but if the input PDB file already contains hydrogens, they may be duplicated or placed incorrectly, corrupting atom types and ruining docking accuracy.
- **Specifying a search box that lies partially outside the receptor volume**: If `--center_*` is set to coordinates with no protein density and `--size_*` is too large, the box includes bulk solvent or empty space. This causes Vina to waste extensive sampling on irrelevant regions and yield lower-quality poses or false negatives.
- **Re-using a ligand PDBQT file with incorrect or missing rotatable bonds defined**: Vina relies on the rotatable bond annotations encoded in the PDBQT ATOM/HETATM records. If the ligand geometry was edited (e.g., via a molecular editor) after PDBQT conversion without regenerating the PDBQT, stale torsion settings cause severe pose distortions or scoring errors.
- **Assuming `--energy_range` controls pose diversity rather than energy cutoff**: `--energy_range` sets the maximum energy difference (in kcal/mol) between the best and worst output pose relative to the top-ranked pose. It does not define a fixed energy threshold. Setting it too narrow discards plausible alternative poses; setting it too wide floods output with irrelevant conformations.
- **Running batch docking without seeding the random number generator**: Each Vina invocation uses a random seed for its global search initialization. Without a fixed seed, reproducibility is lost across runs, making it impossible to compare results or verify convergence — a critical issue for publication-quality workflows.

## Examples

### Basic single-ligand docking into a rigid receptor
**Args:** `--receptor receptor.pdbqt --ligand ligand.pdbqt --center_x 15.3 --center_y 22.7 --center_z -8.1 --size_x 20 --size_y 20 --size_z 20 --out vina_output.pdbqt`
**Explanation:** This is the foundational docking command. Vina explores all ligand poses within the rectangular box centered at (15.3, 22.7, −8.1) with dimensions 20×20×20 Å, producing the default 9 ranked poses in PDBQT format.

### High-exhaustiveness docking for critical binding analysis
**Args:** `--receptor receptor.pdbqt --ligand ligand.pdbqt --center_x 15.3 --center_y 22.7 --center_z -8.1 --size_x 20 --size_y 20 --size_z 20 --out vina_output.pdbqt --exhaustiveness 32 --num_modes 20`
**Explanation:** Setting exhaustiveness to 32 dramatically increases sampling depth, and `--num_modes 20` returns more alternative poses for downstream comparison, making this suitable for studies where missing a near-native pose is unacceptable.

### Flexible receptor docking with side-chain torsion sampling
**Args:** `--receptor flexible_receptor.pdbqt --ligand ligand.pdbqt --center_x 15.3 --center_y 22.7 --center_z -8.1 --size_x 20 --size_y 20 --size_z 20 --out vina_output.pdbqt`
**Explanation:** When the receptor PDBQT file contains rotatable dihedrals (indicating flexible residues), Vina simultaneously optimizes ligand pose and receptor side-chain conformations, at significant computational cost, yielding more realistic binding mode predictions for induced-fit systems.

### Restricting output to top-scoring poses within an energy window
**Args:** `--receptor receptor.pdbqt --ligand ligand.pdbqt --center_x 15.3 --center_y 22.7 --center_z -8.1 --size_x 20 --size_y 20 --size_z 20 --out vina_output.pdbqt --energy_range 3.0 --log vina_log.txt`
**Explanation:** `--energy_range 3.0` filters output poses so that only those within 3 kcal/mol of the top pose are retained, and logging to `vina_log.txt` captures per-pose scores for quantitative analysis and plotting.

### Batch docking of multiple ligands using a shell loop
**Args:** `bash for lig in ligands/*.pdbqt; do name=$(basename $lig .pdbqt); vina --receptor receptor.pdbqt --ligand $lig --center_x 15.3 --center_y 22.7 --center_z -8.1 --size_x 20 --size_y 20 --size_z 20 --out results/${name}_out.pdbqt --log results/${name}_log.txt; done`
**Explanation:** A shell loop iterates over pre-processed ligand PDBQT files, running Vina against the same receptor and search box for each, writing individual output PDBQTs and logs. This pattern is the standard approach for medium-scale virtual screening across a compound library.
---