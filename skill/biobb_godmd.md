---
name: biobb_godmd
category: Molecular Dynamics / Bioinformatics
description: Python wrapper for running GROMACS with the GOdmd (Grid-Optimized Molecular Dynamics) enhanced sampling method. This tool facilitates conformational sampling of biomolecules by distributing multiple parallel simulations on a grid of temperature or Hamiltonian exchange states.
tags:
  - molecular-dynamics
  - gromacs
  - enhanced-sampling
  - replica-exchange
  - protein-conformation
  - free-energy
  - biopython
author: AI-generated
source_url: https://github.com/bioexcel/biobb
---

## Concepts

- **GROMACS Input Structure**: biobb_godmd accepts standard GROMACS topology files (.tpr), coordinate files (.gro/.pdb), and index files (.ndx). The tool requires pre-processed simulation systems prepared with tools like biobb_gromacs or GROMACS built-in utilities for energy minimization and equilibration.
- **Output Ensemble**: The tool generates a collection of trajectory files (.xtc or .trr) across multiple replica states, combined with replica exchange statistics. Output includes energy files (.edr), and checkpoint files (.cpt) for continuation, enabling thorough post-analysis of conformational landscapes.
- **Parallel Temperature/Hamiltonian Grid**: GOdmd implements replica exchange on a grid of temperatures or Hamiltonian values. The tool manages communication between replicas using MPI, with exchange probabilities calculated based on the acceptance ratio at each grid point.
- **Configuration via YAML**: All simulation parameters (temperature range, exchange frequency, timestep, output intervals) are specified in a YAML configuration file following the BioBB schema, enabling reproducible and version-controlled simulation protocols.

## Pitfalls

- **Unprepared Topology Files**: Using unminimized or unequilibrated .tpr files causes immediate simulation crashes or produces physically meaningless conformational states. Always verify system preparation with energy minimization and equilibration steps before running GOdmd.
- **Incorrect Temperature Grid**: Setting temperature ranges too narrow results in low exchange acceptance rates (below 10-20%), defeating the purpose of enhanced sampling. Conversely, ranges too wide may cause phase space overlaps that obscure the relevant conformational basin.
- **Memory Exhaustion**: Running too many replica states without sufficient memory allocation causes MPI failures mid-simulation. Each replica requires separate trajectory storage; estimate 2-4 GB per replica depending on system size and output frequency.
- **Exchange Frequency Mismatch**: Setting exchange attempts too frequent (e.g., every 100 steps) for large systems wastes computational time without proper equilibration between exchanges, while too infrequent exchanges fail to sample the conformational space adequately (optimal range: 500-2000 steps depending on system).

## Examples

### Running GOdmd with a temperature grid from 300K to 450K
**Args:** --config_yaml go_md.yaml --input_tpr_path system.tpr --input_gro_path system.gro --output_replica_traj_path replica_trajectories --output_log_path godmd.log
**Explanation:** This runs enhanced sampling across a temperature replica grid, allowing the system to overcome energy barriers by exploiting higher-temperature excursions while preserving thermodynamic information at each temperature point.

### Specifying a custom exchange frequency of 1000 steps
**Args:** --config_yaml go_md.yaml --input_tpr_path system.tpr --input_gro_path system.gro --output_replica_traj_path replica_trajectories --output_log_path godmd.log
**Explanation:** Exchange attempts every 1000 steps balances sampling efficiency with computational overhead, giving replicas sufficient time to equilibrate locally before attempting coordinate exchanges.

### Running with GPU acceleration enabled
**Args:** --config_yaml go_md_gpu.yaml --input_tpr_path system.tpr --input_gro_path system.gro --output_replica_traj_path replica_trajectories --output_log_path godmd.log
**Explanation:** GPU-enabled GROMACS significantly accelerates MD steps; ensure the configuration specifies CUDA or OpenCL device IDs and that the compute environment has compatible GPU hardware.

### Continuing a previous GOdmd simulation from checkpoint
**Args:** --config_yaml go_md_continue.yaml --input_tpr_path system.tpr --input_gro_path system.gro --input_cpt_path prev_checkpoint.cpt --output_replica_traj_path replica_trajectories --output_log_path godmd.log
**Explanation:** The checkpoint continuation allows extending simulation walltime without losing conformational progress; verify the checkpoint file matches the current topology and number of replicas.

### Outputting trajectory files in XTC format with 10ps recording intervals
**Args:** --config_yaml go_md.yaml --input_tpr_path system.tpr --input_gro_path system.gro --output_replica_traj_path replica_trajectories --output_log_path godmd.log --output_xtc
**Explanation:** XTC provides compressed trajectory output suitable for long simulations, reducing storage requirements while maintaining atomic-resolution sampling data for post-analysis.

### Running with MPI parallelization across 8 replicas
**Args:** --config_yaml go_md.yaml --input_tpr_path system.tpr --input_gro_path system.gro --output_replica_traj_path replica_trajectories --output_log_path godmd.log --mpi_np 8
**Explanation:** MPI parallelization distributes replica calculations across multiple processors, enabling efficient scaling of replica exchange simulations on HPC clusters or multi-core workstations.