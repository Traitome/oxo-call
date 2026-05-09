---
name: biobb_mem
category: molecular-dynamics / energy-minimization
description: BioBB module for performing energy minimization on molecular structures using popular molecular dynamics engines (Gromacs, Ambertools, OpenMM). Accepts structure and topology files, runs a minimization algorithm, and outputs a minimized structure.
tags: [energy-minimization, gromacs, amber, openmm, molecular-dynamics, structure-optimization, biobb]
author: AI-generated
source_url: https://biobb.readthedocs.io/
---

## Concepts

- **Input requires paired structure and topology files.** The module needs both a coordinate file (`.pdb`, `.gro`) and a matching topology file (`.top`, `.itp`, `.prmtop`) to define atoms and force-field parameters. Using mismatched or inconsistent structure/topology pairs produces unreliable forces during minimization.
- **Output is a minimized coordinate file in the same format as the input structure file.** After minimization, the tool writes a new `.pdb` or `.gro` file depending on the engine, overwriting any existing file at the target path. You must specify the output path explicitly to avoid accidental data loss.
- **Minimization parameters control convergence.** Key flags include `maxiter` (maximum iterations), `step_size` (minimization step), and `emtol` (force tolerance threshold). Tightening `emtol` improves structural quality but increases runtime; loosening it may finish quickly but leave the structure under-minimized.
- **Engine selection is runtime-critical.** The `--engine` flag switches the backend (e.g., `gromacs`, `ambertools`, `openmm`). Each engine requires a compatible input file format: Gromacs needs `.gro`+`.top`, Ambertools needs `.pdb`+`.prmtop`, OpenMM needs `.pdb`+`.xml`. Running the wrong engine with incompatible files will raise a format mismatch error at execution.
- **Output log files contain energy trajectory data.** Most engines produce a log or energy file (e.g., `ener.edr` for Gromacs) alongside the structure. These logs are required for later analysis steps (e.g., plotting energy convergence), so discarding or overwriting them breaks downstream analysis.

## Pitfalls

- **Passing Gromacs-formatted input to an Ambertools engine causes a fatal parse error.** Ambertools cannot read Gromacs `.gro`/`.top` files and will crash with an "unknown file type" message. Always verify that the input format matches the chosen `--engine`.
- **Omitting the `--output_path` flag silently overwrites the input structure.** If `output_path` is not set, the module writes the minimized structure to the same file path as the input, permanently destroying the original coordinates and making restarts impossible.
- **Setting `--maxiter` too low leaves the structure under-minimized.** If the algorithm stops before reaching the tolerance threshold (e.g., `--emtol 1000.0`), the RMS forces in the output may still be high, producing physically unreliable results in subsequent MD steps. Always check the log file's final force values.
- **Using an incompatible `.itp` include file with Gromacs causes a segmentation fault.** Gromacs minimization crashes with a segfault if the referenced `.itp` files define atoms not present in the coordinate file. Validate all `#include` directives in the topology before running.
- **Forgetting to specify `--output_log_path` makes debugging minimization failures difficult.** Without a saved log, convergence failures cannot be diagnosed from the CLI; you must re-run the minimization to reproduce the error. Always route log output to a persistent file.

## Examples

### Minimize a Gromacs structure using the Gromacs engine
**Args:** `--input_structure_path structure.gro --input_topology_path system.top --output_structure_path structure_min.gro --output_log_path minimization.log --engine gromacs --maxiter 10000 --emtol 1000.0`
**Explanation:** Runs up to 10000 iterations of energy minimization with a force tolerance of 1000.0 kJ/mol·nm on a Gromacs-formatted structure, producing a minimized `.gro` file and a log for convergence inspection.

### Minimize a Gromacs structure using a PDB output for downstream AMBER steps
**Args:** `--input_structure_path structure.gro --input_topology_path system.top --output_structure_path structure_min.pdb --output_log_path minimization.log --engine gromacs --maxiter 5000 --emtol 500.0`
**Explanation:** Performs 5000 iterations with a tighter force tolerance of 500.0 kJ/mol·nm and outputs the minimized structure in PDB format for use in Ambertools-based workflows.

### Minimize a structure using the Ambertools engine with a prmtop topology
**Args:** `--input_structure_path structure.pdb --input_topology_path structure.prmtop --output_structure_path structure_min.pdb --output_log_path minimization.log --engine ambertools --maxiter 5000 --emtol 100.0 --step_size 0.01`
**Explanation:** Uses Ambertools with the matching LEaP-format topology to minimize 5000 steps with a smaller step size of 0.01 Å and a tighter tolerance of 100.0 kcal/mol·Å, outputting a PDB-formatted minimized structure.

### Minimize a structure and extract the final energy from the log for analysis
**Args:** `--input_structure_path structure.gro --input_topology_path system.top --output_structure_path structure_min.gro --output_log_path minimization.log --output_energy_path energies.xvg --engine gromacs --maxiter 10000 --emtol 1000.0`
**Explanation:** Runs the same minimization as above but also writes the energy trajectory to an `.xvg` file, enabling downstream plotting of the energy convergence curve across minimization steps.

### Generate a topology for a Gromacs minimization from a PDB and include files
**Args:** `--input_structure_path structure.pdb --input_topology_path system.top --output_structure_path structure_min.gro --output_log_path minimization.log --engine gromacs --maxiter 10000 --emtol 1000.0 --include_itp molecule.itp`
**Explanation:** Includes an additional `.itp` file defining a custom molecule during minimization, which is required when the coordinate file contains ligand atoms that are not in the base force-field topology.

### Minimize a structure with OpenMM using an XML force field
**Args:** `--input_structure_path structure.pdb --input_topology_path system_ff.xml --output_structure_path structure_min.pdb --output_log_path minimization.log --engine openmm --maxiter 5000 --emtol 50.0`
**Explanation:** Uses OpenMM as the backend with a custom force-field XML, minimizing 5000 steps with a force tolerance of 50.0 kJ/mol·nm and outputting a PDB-formatted minimized structure.

### Minimize only the solvent molecules while holding the solute fixed
**Args:** `--input_structure_path structure.gro --input_topology_path system.top --output_structure_path structure_min.gro --output_log_path minimization.log --engine gromacs --maxiter 5000 --emtol 1000.0 --select_group Protein`
**Explanation:** Runs minimization on only the solvent atoms while freezing the Protein group using Gromacs index groups, which is useful for relaxing solvent around a pre-minimized solute before full system equilibration.

---
name: biobb_amber
category: amber-tools / molecular-dynamics-utils
description: BioBB module wrapping Ambertools utilities (tleap, ptraj, cpptraj, nab) for system setup, trajectory analysis, and nucleic acid modeling. Operates on AMBER-format coordinate and topology files (.prmtop, .in, .crd).
tags: [amber, cpptraj, trajectory-analysis, leap, nucleic-acids, biobb]
author: AI-generated
source_url: https://biobb.readthedocs.io/
---

## Concepts

- **Ambertools expect AMBER-native file formats.** The module works with `.prmtop` (topology), `.in`/`mdin` (input scripts), `.mdcrd`/`.crd` (trajectories), and `.pdb` (structures). Using non-AMBER formats (e.g., Gromacs `.gro` or `.xtc`) requires an explicit format conversion step before use.
- **tleap handles system building and topology generation.** When setting up a new system, the module invokes `tleap` via the AmberTools wrapper. This requires correct residue definitions in the input PDB; non-standard residues or missing residues cause `tleap` to fail with an "Unknown residue" error.
- **cpptraj performs trajectory analysis.** The module routes analysis requests to `cpptraj`, which reads a `.prmtop` + trajectory pair and executes commands from the provided input script (e.g., `rms`, `radgyr`, `hbond`). The topology file must match the trajectory exactly; any atom-order mismatch produces incorrect or nonsensical results.
- **nab is used for nucleic acid structure modeling.** When the module detects nucleic acid residues (DNA/RNA), it delegates to `nab` for specialized structure prediction. Supplying a protein-only PDB to a nucleic-acid-mode command causes `nab` to throw a "no nucleic acid atoms found" exception.
- **Output file paths are always required.** The module does not infer default output paths. If `--output_prmtop_path` or `--output_traj_path` is omitted, the wrapper writes to the current working directory with an auto-generated name, risking file collisions in batch workflows.

## Pitfalls

- **Running cpptraj with a Gromacs trajectory (.xtc) without prior conversion causes a fatal read error.** cpptraj cannot natively parse `.xtc` files. Attempting this will produce an "unrecognized file type" error. Convert the trajectory to `.mdcrd` using `biobb_structure_conversion` before analysis.
- **Providing a PDB with altloc indicators to tleap results in duplicate atom definitions.** tleap interprets altloc characters (A/B) as distinct atom positions, generating a topology with duplicate atoms and causing crashes in subsequent minimization or MD steps. Strip altloc flags with a preprocessing step before system building.
- **Using an outdated AMBER force field specifier causes tleap to issue warnings and fall back to a generic force field.** If the `--forcefield` flag uses an old or misspelled specifier (e.g., `ff19SB` vs `ff19SB`), tleap silently substitutes a default and your system uses incorrect parameters. Always verify the specifier against the installed Ambertools version.
- **Omitting `--output_prmtop_path` auto-writes to the current directory, overwriting any prior `prmtop` file.** In sequential workflows (e.g., system setup followed by equilibration), the auto-generated filename may collide with the input topology, silently replacing it and breaking the equilibration step.
- **Analyzing a trajectory without first fitting it to a reference causes misleading RMSD values.** If the `--reference` flag is omitted in an RMSD calculation, cpptraj uses the first frame as reference, which may already be misaligned. Report RMSD relative to a known crystallographic reference structure for meaningful comparison.

## Examples

### Build a solvated system using tleap with the ff19SB force field
**Args:** `--input_structure_path protein.pdb --output_prmtop_path system.prmtop --output_crd_path system.crd --output_inpcrd_path system_inpcrd --forcefield ff19SB --solvent_model OPC --temp0 300.0 --remove_water False`
**Explanation:** Invokes tleap to generate a solvated, parameterized AMBER topology using the ff19SB protein force field and the OPC water model, outputting a complete system topology and coordinate file for subsequent equilibration.

### Strip water and ions from a trajectory for analysis
**Args:** `--input_prmtop_path system.prmtop --input_traj_path production.mdcrd --output_traj_path dry.mdcrd --input_cpptraj_commands 'strip :WAT,Na+,Cl- outtraj dry.mdcrd' --engine cpptraj`
**Explanation:** Uses cpptraj to remove all water molecules and ions from the production trajectory, producing a smaller trajectory file containing only the solute for downstream RMSD or hydrogen-bond analysis.

### Compute RMSD of a trajectory aligned to a reference structure
**Args:** `--input_prmtop_path system.prmtop --input_traj_path production.mdcrd --output_data_path rmsd.dat --reference protein_ref.pdb --engine cpptraj --commands 'rms R1 :1-100@CA ref R1 :1-100@CA out rmsd.dat'`
**Explanation:** Aligns frames to the alpha carbons of residues 1–100 in the reference PDB and computes RMSD per frame, writing the values to a columnar text file for plotting or statistical analysis.

### Calculate radius of gyration for a protein over a production trajectory
**Args:** `--input_prmtop_path system.prmtop --input_traj_path production.mdcrd --output_data_path rg.dat --engine cpptraj --commands 'radgyr Rg :1-200 out rg.dat'`
**Explanation:** Computes the radius of gyration for atoms 1–200 across all frames in the trajectory using cpptraj, outputting a single-column data file for folding stability analysis or time-series plotting.

### Count native hydrogen bonds per frame over a trajectory
**Args:** `--input_prmtop_path system.prmtop --input_traj_path production.mdcrd --output_data_path hbond.dat --engine cpptraj --commands 'hbond HBDist acceptors DON :1-100 mask :101-200 mask out hbond.dat series uuseries'`
**Explanation:** Identifies and counts hydrogen bonds between donor residues 1–100 and acceptor residues 101–200 across all frames, outputting per-frame counts for stability or binding interface analysis.

---
name: biobb_structure
category: structure-analysis / validation
description: BioBB module for analyzing, validating, and manipulating molecular structures. Provides tools for checking residue geometry, computing RMSD, generating structure superpositions, and repairing missing atoms or chains in PDB files.
tags: [structure-analysis, pdb, rmsd, superposition, validation, biobb]
author: AI-generated
source_url: https://biobb.readthedocs.io/
---

## Concepts

- **Input structures must be valid PDB ormmCIF files.** The module parses standard PDB-format files (including multi-model NMR structures) but rejects non-standard atom naming, malformed residue codes, or missing CRD1 records in CIF files. Pre-validate structures with `pdb_validate` before running analysis.
- **RMSD computation requires a residue or atom selection on both input and reference structures.** The `--select` flag (e.g., `:1-50@CA`) must be resolvable in both the target and the reference. Mismatched selections (e.g., reference has chain A but target has chain B) produce a residue-not-found error.
- **Missing atom repair operates on residue templates.** When `--repair_mode` is set, the module adds missing atoms based on Dunbrack rotamer libraries or backbone templates. If the residue type is unknown (non-canonical or modified residues), repair silently skips those residues without warning.
- **Superposition uses a rigid-body least-squares algorithm.** The module applies a rotation + translation to the target structure to minimize the RMSD to the reference over the selected atoms. No energy minimization or side-chain repacking is performed; only the backbone and selected atoms are moved.
- **Output structures are written in PDB format by default.** Even if a Gromacs `.gro` or AMBER `.crd` file is provided as input, the superimposed or repaired structure is output as a `.pdb` file. This may introduce minor formatting differences (e.g., atom names, segids) that need downstream reconciliation.

## Pitfalls

- **Superimposing structures with different chains without specifying chain mapping produces incorrect RMSD values.** If chains are not explicitly mapped, the module maps them by order of appearance, which can invert chain A in the target with chain B in the reference, yielding an artificially high or low RMSD. Always specify `--chain_map` when chain orders differ.
- **Repairing structures with modified residues (e.g., phosphorylated serines) silently skips those residues.** The built-in Dunbrack templates do not cover post-translational modifications. Failing to repair these residues means the output structure still has missing atoms, which will cause downstream MD engines to reject the topology.
- **Computing RMSD across an entire multi-chain complex without specifying an atom selection inflates the RMSD.** Including disordered loops or flexible termini in the RMSD calculation dramatically increases the value, making it hard to distinguish genuine conformational changes from noise. Restrict the selection to structured elements (e.g., alpha-helices, beta-sheets).
- **Writing the superimposed output to the same path as the reference overwrites the original reference.** If `--output_structure_path` is set to the same file as the reference, the superposition operation permanently replaces the reference structure, making re-superimpositions or alternative alignments impossible.
- **Using NMR ensemble structures without the `--ensemble_mode` flag causes frame-averaged results.** When a multi-model PDB is provided, the module averages coordinates across all models by default, obscuring individual conformational states. Enable ensemble mode to analyze each model independently.

## Examples

### Superimpose a trajectory frame to a reference using C-alpha atoms
**Args:** `--input_structure_path frame.pdb --input_reference_path reference.pdb --output_structure_path aligned.pdb --select ':1-200@CA' --chain_map A:A,B:B'
**Explanation:** Applies a rigid-body rotation/translation to align the C-alpha atoms of chains A