---
name: biobb_gromacs
category: Molecular Dynamics Simulation
description: Python wrappers for GROMACS molecular dynamics simulation tools. Provides modular interface to GROMACS commands for energy minimization, MD production, trajectory processing, and analysis.
tags:
  - molecular dynamics
  - gromacs
  - md simulation
  - protein dynamics
  - trajectory analysis
  - energy minimization
author: AI-generated
source_url: https://github.com/bioexcel/biobb_gromacs
---

## Concepts

- **Modular command structure**: biobb_gromacs consists of independent wrappers for individual GROMACS modules (gmx grompp, gmx mdrun, gmx energy, etc.), each invoked through separate Python classes or subcommands rather than a single monolithic interface.
- **Standard file formats**: Input files must be in GROMACS formats (.tpr for topology, .gro/.pdb for coordinates, .mdp for parameters); output files include trajectories (.xtc/.trr), energies (.edr), and structures (.gro/.pdb). Using incompatible formats (e.g., AMBER prmtop instead of GROMACS tpr) will cause parsing failures.
- **Configuration via MDP files**: Simulation parameters (temperature, pressure, timestep, cutoff distances) are controlled through .mdp files passed to the grompp module, which generates the portable binary run file (.tpr) consumed by mdrun.
- **Docker/Singularity support**: biobb_gromacs tools can run inside containers, with the container image specified via the `gromacs_image` parameter, ensuring reproducibility across different compute environments.

## Pitfalls

- **Mismatched force field**: Using a structure file and topology generated with different force fields (e.g., CHARMM-trained structure with AMBER-born parameters) produces physically nonsensical results. Always ensure coordinate files and .tpr topology derive from the same force field.
- **Neglecting periodic boundary conditions**: Running simulations without defining periodic boundary conditions in the .mdp file causes atoms to drift away from the simulation box, leading to artifacts in trajectory analysis and visualization.
- **Insufficient equilibration time**: Starting production MD immediately after energy minimization without adequate equilibration (position-restrained followed by unrestrained) results in system instability and energy spikes.
- **File path errors in containers**: When using Docker/Singularity, input files must be accessible inside the container at the specified paths—if mounting paths differ between host and container, the tool fails with file-not-found errors.

## Examples

### Perform energy minimization on a solvated structure

**Args:** `--input_structure_path ../../data/system_solvated.gro --input_topology_path ../../data/topol.tpr --output_minimization_path ../../data/system_minimized.gro --output_ener_edr ../../data/min_ener.edr --properties_files ../../data/minimization.mdp`

**Explanation:** Runs GROMACS energy minimization using the specified MDP parameters to remove steric clashes from the system before equilibration.

### Generate a portable binary run file (.tpr)

**Args:** `--input_structure_path ../../data/system_minimized.gro --input_topology_path ../../data/topol.tpr --input_md_config_path ../../data/equilibration.mdp --output_tpr_path ../../data/system_eq.tpr`

**Explanation:** Uses grompp to combine structure and topology with MDP parameters, producing a binary run file required for MD execution.

### Run molecular dynamics production simulation

**Args:** `--input_tpr_path ../../data/system_eq.tpr --output_trajectory_xtc ../../data/traj.xtc --output_ener_edr ../../data/ener.edr --output_log ../../data/md.log`

**Executes:** The GROMACS mdrun module to propagate system dynamics over the defined time, writing trajectory and energy outputs.

### Convert trajectory file format

**Args:** `-i ../../data/traj.xtc -o ../../data/traj_nojump.xtc -f ../../data/topol.tpr`

**Explanation:** Uses gmx trjconv to unwrap trajectories (removing periodic boundary jumps) for visualization, referencing the topology file for correct atom ordering.

### Extract energy components from .edr file

**Args:** `--input_energy_edr_path ../../data/ener.edr --output_energy_csv_path ../../data/energy_components.csv --terms Bond Angle Potential Kinetic-Energy Total-Energy`

**Explanation:** Parses the binary energy file and extracts specified terms (bond, angle, potential, kinetic, total energy) as a CSV file for downstream analysis.

### Calculate center of mass trajectory

**Args:** `-f ../../data/traj.xtc -s ../../data/system_eq.tpr -o ../../data/com_traj.xtc`

**Explanation:** Uses gmx trajcat or gmx trajectory operations to compute and output the center of mass motion over time.

### Analyze RMSD of a trajectory

**Args:** `--input_structure_path ../../data/system_eq.tpr --input_trajectory_path ../../data/traj.xtc --output_rmsd_xvg ../../data/backbone_rmsd.xvg --select_group_name "Backbone"`

**Explanation:** Computes root-mean-square deviation of backbone atoms relative to the reference structure, outputting XVG format for plotting.