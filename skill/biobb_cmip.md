---
name: biobb_cmip
category: Molecular Dynamics / MD Systems Preparation
description: BioBB CMIP is a module of the BioExcel Building Blocks suite that provides command-line wrappers for CMIP (Common Molecular Dynamics Pipeline) operations, enabling preprocessing, system building, and trajectory analysis for molecular dynamics workflows.
tags:
  - molecular-dynamics
  - AMBER
  - GROMACS
  - system-preparation
  - biobb
  - MD-simulation
  - trajectory-analysis
  - topology
  - conda
author: AI-Generated
source_url: https://biobb-cmip.readthedocs.io/en/latest/
---

## Concepts

- biobb_cmip exposes CMIP pipeline operations through Python-based wrappers that standardize input/output file formats (PDB, AMBER topology, GROMACS topology, trajectory files) and automate common MD preparation tasks such as adding hydrogens, setting protonation states, and solvating systems.
- The tool integrates with other BioBB modules (e.g., biobb_io, biobb_md) via shared configuration keys for `--input_coords_path`, `--output_traj_path`, and `--input_topology_path`, so pipeline arguments must reference existing files produced by upstream steps.
- biobb_cmip subcommands (`build`, `process`, `analyze`) correspond to distinct CMIP pipeline stages: `build` prepares the molecular system, `process` applies restraints or conversions, and `analyze` extracts properties from trajectories; each subcommand has its own mandatory and optional flag set.
- All input coordinate files should be pre-validated for bond integrity and residue naming conventions before passing to biobb_cmip, as the tool does not repair malformed PDB files or correct residue numbering gaps automatically.
- The tool supports both AMBER (prmtop/inpcrd) and GROMACS (gro/top) output formats controlled by the `--output_format` flag; mixing formats between steps (e.g., feeding a GROMACS box to an AMBER-specific subcommand) will cause runtime failures.

## Pitfalls

- Feeding a PDB file with alternate location identifiers or disordered residues directly to `build` without prior cleanup causes the system to be constructed with incorrect atom counts, leading to topology mismatches downstream.
- Omitting the `--output_format` flag defaults to AMBER format; users expecting GROMACS output (gro/top) who forget to set `--output_format gromacs` waste compute cycles rerunning the pipeline.
- Using mismatched `--input_topology_path` and `--input_coords_path` files (e.g., a prmtop from a different PDB than the supplied coordinate file) produces physically inconsistent systems with corrupted angles and dihedrals.
- Specifying an incorrect `--box_type` (e.g., `octahedron` when the solvating subcommand only supports `cubic` or `dodecahedron`) silently ignores the request or raises a generic I/O error with no diagnostic.
- Running parallel `biobb_cmip` subcommands on the same output directory without atomic file locking results in corrupted trajectory output files that are difficult to diagnose post-hoc.

## Examples

### Prepare a solvated AMBER system from a clean PDB
**Args:** `build --input_coords_path input.pdb --output_system_path system.prmtop --output_coords_path system.inpcrd --box_type dodecahedron --ionic_conc 0.15`
**Explanation:** The `build` subcommand reads the cleaned PDB, adds hydrogens, places the system in a dodecahedral water box at physiological ionic strength, and writes AMBER topology and coordinate files.

### Convert an AMBER trajectory to GROMACS format
**Args:** `process --input_traj_path md.nc --input_topology_path system.prmtop --output_format gromacs --output_traj_path md.gro`
**Explanation:** The `process` subcommand reads the AMBER NetCDF trajectory and AMBER topology, then converts them to GROMACS format, writing a GRO-style trajectory file suitable for GROMACS analysis tools.

### Analyze rmsd of a trajectory with respect to a reference structure
**Args:** `analyze rmsd --input_traj_path md_fit.xtc --input_ref_structure_path reference.pdb --input_topology_path md_top.mdp --output_plot_path rmsd.png`
**Explanation:** The `analyze rmsd` command calculates root-mean-square deviation of the fitted trajectory against the reference PDB, then writes a PNG plot of the rmsd time series.

### Build a periodic cubic box with explicit water ions
**Args:** `build --input_coords_path bstate.pdb --output_system_path system.prmtop --output_coords_path system.inpcrd --box_type cubic --ionic_conc 0.10 --positive_ion NA --negative_ion CL`
**Explanation:** The `build` subcommand places the molecular system in a cubic periodic box with sodium and chloride ions at 0.10 M concentration, producing an electrically neutral solvated system.

### Strip solvent and ions from a trajectory before analysis
**Args:** `process strip --input_traj_path md_full.xtc --input_topology_path system.top --output_traj_path md_nosol.xtc --strip_selection "not resname SOL and not resname NA and not resname CL"`
**Explanation:** The `process strip` subcommand removes water and ion atoms from the trajectory based on residue name selection, outputting a stripped trajectory file for faster downstream analysis.

### Generate an amber98-compatible topology from a PDB file
**Args:** `build --input_coords_path peptide.pdb --output_system_path amber98.prmtop --output_coords_path amber98.inpcrd --force_field amber98 --add_hydrogen yes`
**Explanation:** The `build` subcommand assigns AMBER98 force field parameters and adds explicit hydrogens to the peptide PDB, outputting an AMBER98-compatible topology and coordinate file pair.