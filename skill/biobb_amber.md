---
name: biobb_amber
category: Molecular Dynamics Simulation
description: Python wrapper library for AMBER MD simulation tools including tleap, pmemd, sander, cpptraj, antechamber, and parmed. Provides both Python API and CLI interfaces for biomolecular structure preparation, energy minimization, molecular dynamics simulation, and trajectory analysis.
tags: [molecular dynamics, AMBER, protein simulation, MD simulation, biomolecular modeling, computational chemistry, structure preparation, trajectory analysis]
author: AI-generated
source_url: https://biobb.readthedocs.io/en/latest/source/available/biobb_amber.html
---

## Concepts

- **Input Format and Configuration:** BioBB AMBER tools accept JSON or dictionary-based configuration for simulation parameters. Structure input typically requires PDB or MOL2 files, while topology and coordinate files (.prmtop, .inpcrd, .crd) are generated during the workflow. Each tool wrapper has specific mandatory and optional properties documented in the API.

- **Output File Dependencies:** Many BioBB AMBER tools produce intermediate files that serve as inputs for subsequent steps in a workflow. For example, tleap produces topology (.prmtop) and coordinate (.inpcrd) files required by sander or pmemd for MD simulations. Output directories must exist before execution or be created programmatically using the `make_dir` parameter.

- **Tool Path Requirements:** BioBB AMBER requires AMBERTools to be installed and accessible in the system PATH. The wrappers call underlying executables (tleap, sander, pmemd.cuda, cpptraj, antechamber, parmed) based on the installed version. Verify installation with `ambertools --version` before running workflows. Environment variables like `AMBERHOME` should be properly configured.

- **Docker and Singularity Support:** BioBB AMBER provides containerized execution through Docker images (`biocharmb/amber:latest`) and Singularity recipes. When using containers, mount input directories correctly using `-v /host/path:/container/path` and set working directories appropriately to ensure file accessibility.

## Pitfalls

- **Incorrect File Path Mounts in Containers:** When running BioBB AMBER in Docker, incorrect volume mount paths cause FileNotFoundError exceptions. Using `-v $(pwd):/app/data` instead of `-v $(pwd):/home/biobb/data` will make output files unavailable in the container. Always verify mount paths match the working directory specified in the wrapper.

- **Missing Periodic Box Definition:** Running MD simulations without defining a periodic boundary condition box (water box, octahedral, or rectangular) causes energy minimization failures or unstable trajectories. Ensure the structure preparation step (tleap) includes `setWaterBox` or equivalent commands with appropriate padding values (typically 10-12 Å).

- **Mismatched Topology and Trajectory Files:** Attempting to analyze a trajectory using a topology file generated for a different system or with different atom ordering produces incorrect results or crashes in cpptraj. Always verify that the topology (.prmtop) file corresponds exactly to the system used to generate the trajectory (.nc, .mdcrd, .netcdf).

- **Insufficient System Charge Neutralization:** Failure to neutralize the system charge before running MD simulations causes simulation instability or crashes in PME calculations. Use the structure preparation tools (tleap or parmed) to add counterions with appropriate concentration or replacement of specific residues.

- **Overwriting Existing Output Files:** BioBB AMBER tools do not overwrite existing output files by default and will raise an error. Either delete existing files manually or use the `remove_tmp_files` and `check_outfiles` parameters to manage file conflicts when re-running workflows.

## Examples

### Prepare a protein structure in a water box with tleap
**Args:** `{"input_path": "protein.pdb", "output_path": "structure.parmtop", "output_inpcrd_path": "structure.inpcrd", "output_crd_path": "structure.crd", "topology_format": "parm7", "force_field": ["leaprc.protein.ff14SB", "leaprc.water.tip3p"], "water_type": "TIP3PBOX", "water_box": {"x": 10.0, "y": 10.0, "z": 10.0}, "ions": [{"name": "NA", "amount": 1}], "set_title": "protein_structure"}`
**Explanation:** This invokes the tleap wrapper to load a PDB file, apply the ff14SB force field and TIP3P water model, add a neutralizing sodium ion, and generate AMBER topology and coordinate files for subsequent MD simulations.

### Run energy minimization with sander
**Args:** `{"input_coordinates_path": "structure.inpcrd", "input_topology_path": "structure.parmtop", "output_coordinates_path": "min.rst7", "output_log_path": "min.out", "md_input": {"maxcyc": 1000, "ncyc": 500, "ntr": 1, "restraint_wt": 10.0, "restraintmask": ":1-20"}}`
**Explanation:** This runs a two-stage energy minimization with sander, first using steepest descent and then conjugate gradient, with positional restraints on the first 20 residues to allow solvent relaxation while keeping the protein core fixed.

### Perform MD simulation with GPU-accelerated pmemd
**Args:** `{"input_coordinates_path": "min.rst7", "input_topology_path": "structure.parmtop", "output_coordinates_path": "md.nc", "output_rst_state_path": "md.rst7", "output_log_path": "md.out", "md_input": {"ntx": 1, "irest": 0, "ntpr": 1000, "ntwx": 1000, "ntwr": 10000, "nstlim": 1000000, "dt": 0.002, "temp0": 300.0, "ntp": 1, "taup": 2.0}, "gpu_id": 0}`
**Explanation:** This executes a 2-nanosecond NPT MD simulation at 300 K using pmemd.cuda with isotropic pressure coupling, writing coordinates every 2 ps and maintaining temperature with Langevin dynamics. Using pmemd instead of sander provides significantly faster performance on GPUs.

### Analyze a trajectory with cpptraj
**Args:** `{"input_topology_path": "structure.parmtop", "input_coordinates_path": "md.nc", "output_data_path": "rmsf.csv", "cpptraj_input": {"trajin": "md.nc", "rms": "first :1-200@CA", "atomicfluct": "byres :1-200@CA out rmsf.dat"}}`
**Explanation:** This performs RMSD alignment to the first frame using CA atoms of residues 1-200, then calculates per-residue atomic fluctuations (RMSF) and writes the results to a CSV file for visualization or downstream analysis.

### Calculate charges with antechamber
**Args:** `{"input_path": "ligand.mol2", "output_path": "ligand.ac", "output_ac_type": "ac", "input_format": "mol2", "tool": "antechamber", "additional_params": ["-c", "bcc", "-at", "gaff2"]}`
**Explanation:** This assigns AM1-BCC semi-empirical charges to a small molecule ligand using antechamber with the GAFF2 atom type definition, generating an AMBER charge (ac) file suitable for inclusion in leap topology generation.