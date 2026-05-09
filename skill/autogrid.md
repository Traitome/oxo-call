---
name: autogrid
category: Molecular Docking / Computational Chemistry
description: A utility for computing atomic affinity grid maps for AutoDock4 molecular docking simulations. Autogrid pre-computes grids representing atomic interaction potentials for each atom type in a ligand, which significantly accelerates the subsequent docking calculation. It generates multiple files including the molecular affinity maps, electrostatic potential map, and desolvation potential map.
tags: autodock4, molecular docking, grid maps, computational chemistry, drug discovery, affinity maps, protein-ligand docking
author: AI-generated
source_url: https://autodock.scripps.edu/
---

## Concepts

- Autogrid processes PDBQT files containing protein structures with partial charges and atom type assignments. The input protein file must be in PDBQT format (converted from PDB using AutoDockTools), where each atom has assigned AutoDock4 atom types (e.g., C, A, N, O, S, P, H for non-polar or polar aliphatic, etc.).
- The tool generates separate .map files for each ligand atom type specified in the grid parameter file, a .electrostatics map using a Coulombic calculation with a distance-dependent dielectric, and a .desolvation map representing the desolvation energy contribution based on atomic solvation parameters.
- Grid dimensions are defined by the number of points in X, Y, Z directions with a specified spacing (default 0.375 Å). The total number of points directly impacts memory usage and runtime, with larger grids requiring more computation. The grid box must be sufficiently large to encompass the entire binding site and allow ligand flexibility during docking.

## Pitfalls

- Using incorrect atom type definitions in the grid parameter file causes runtime errors or generates incomplete maps. If the ligand contains atom types not defined in the grid parameter file, AutoDock4 will fail during docking with "FATAL: atom type X not defined in the grid map." Ensure all ligand atom types are explicitly listed in the .gpf file.
- Placing the grid center too far from the binding pocket results in poor docking results or missing critical interactions. The ligand may not be able to fully explore the binding site, or essential receptor-ligand interactions may fall outside the computed grids. Always center grids on known binding site residues or use reference ligand coordinates.
- Configuring excessive grid spacing (e.g., >0.5 Å) reduces map resolution and decreases docking accuracy. Coarser grids interpolate fewer atomic interactions, leading to false-positive docking poses or missed binding modes. Standard practice uses 0.375 Å spacing; never exceed 0.5 Å for production runs.

## Examples

### Generate affinity grids for a protein kinase binding site
**Args:** -p protein.gpf -l ligand.pdbqt
**Explanation:** This specifies the grid parameter file containing receptor atom types and defines the grid dimensions centered on the ligand binding site location.

### Specify custom grid dimensions and spacing
**Args:** -p protein.gpf -l ligand.pdbqt -o custom_grids
**Explanation:** Uses default grid parameter file values but allows output file naming. The .gpf file already contains the defined grid size, center coordinates, and spacing values.

### Run autogrid with explicit output directory
**Args:** -p protein.gpf -l ligand.pdbqt
**Explanation:** Executing from a working directory writes all output .map, .glg, and .fld files to the current directory. Ensure adequate disk space (typically 50-200 MB per full grid set).

### Compute grids for multiple ligand atom types
**Args:** -p protein.gpf -l ligand.pdbqt
**Explanation:** The grid parameter file lists all ligand atom types (HD, CA, N, OA, SA) that AutoGrid4 will generate individual affinity maps for, covering all potential interactions.

### Debug grid generation issues
**Args:** -p protein.gpf -l ligand.pdbqt - verbose
**Explanation:** Full logging output helps diagnose parameter file errors, atom type mismatches, or grid dimension problems. Check the .glg log file for specific error messages.

### Create grids for a flexible receptor side-chain study
**Args:** -p receptor_flex.gpf -l ligand.pdbqt
**Explanation:** When residues are defined as flexible in the receptor PDBQT, AutoGrid4 includes these atoms in the affinity calculations, allowing side-chain repositioning during docking.