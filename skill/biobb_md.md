---
name: biobb_md
category: Molecular Dynamics Analysis
description: A bioinformatics tool from the BioExcel Building Blocks (biobb) library for molecular dynamics simulation analysis. Provides command-line utilities for extracting and analyzing energies, trajectories, RMSD/RMSF, hydrogen bonds, contacts, radius of gyration, SASA, and diffusion coefficients from GROMACS and other MD simulation outputs.
tags:
  - molecular dynamics
  - MD analysis
  - GROMACS
  - trajectory analysis
  - protein dynamics
  - computational chemistry
  - biobb
author: AI-generated
source_url: https://github.com/bioexcel/biobb_md
---

## Concepts

- **Input Format**: biobb_md accepts multiple MD data formats including GROMACS TRR/XTC trajectories, GRO/PDB structures, EDH energy files, and XVG output from GROMACS analysis tools. Trajectory files must be pre-processed with tools like gmx trjconv or gmx make_ndx if periodic boundary conditions need handling.
- **Output Types**: The tool produces XVG (XMGrace) plot files for energy profiles and analysis, CSV/JSON for numeric results, and PDB/DCD files for trajectory snapshots. Each subcommand (energy, rmsd, hbond, etc.) generates format-specific outputs compatible with visualization tools like VMD, PyMOL, or XMGrace.
- **Command Structure**: biobb_md uses subcommands for distinct analysis tasks (e.g., `biobb_md energy`, `biobb_md rmsd`, `biobb_md hbond`). The tool name prefix is required only when invoking from a shell; when specifying arguments for prompt generation, use only the subcommand and its flags.
- **Key Subcommands**: Common analysis modules include `energy` (extracts potential/kinetic/total energy), `rmsd` (calculates backbone RMSD against a reference), `hbond` (computes hydrogen bond occupancy), `contacts` (analyzes inter-chain contacts), `radius_gyration` (measures compactness), `sasa` (solvent-accessible surface area), and `diffusion` (mean square displacement for diffusion coefficients).

## Pitfalls

- **Incorrect Reference Structure for RMSD**: Specifying a mismatched reference PDB for RMSD calculations produces meaningless values. Using a crystal structure when the trajectory contains a different conformation, or forgetting to align structures before RMSD computation, yields incorrect RMSD profiles that invalidate downstream interpretation.
- **Missing Periodic Boundary Treatment**: Failing to use `-pbc nojump` or `-pbc cluster` when处理 trajectories with molecules crossing periodic boundaries causes trajectory discontinuities. This breaks hydrogen bond calculations and contact analysis, producing artificially broken bonds or missing contacts across box edges.
- **Mismatched Selection Strings**: Using incorrect atom selection syntax (e.g., `protein` when the trajectory uses different naming like `Protein` or `PROTEIN`) causes the entire analysis to fail or return zero contacts/hbonds. GROMACS selection engine is case-sensitive and requires exact matches to the topology atom names.
- **Unit Errors in Diffusion Analysis**: Specifying wrong time units (picoseconds vs nanoseconds) or distance units (nanometers vs angstroms) in diffusion calculations leads to incorrect diffusion coefficients by factors of 1000 or more. Always verify output units match expected physical values (typically D in 10⁻⁵ cm²/s).

## Examples

### Calculate the potential energy profile from a GROMACS energy file
**Args:** `energy -edh energy.edh -o energy.xvg`
**Explanation:** Extracts potential energy time series from an EDH energy file and writes an XVG plot for visualization; this is essential for assessing MD simulation stability and detecting energy drift.

### Compute RMSD of a protein backbone against a reference crystal structure
**Args:** `rmsd -ttr trajectory.trr -s reference.pdb -o rmsd.xvg -select "Backbone" -nofit`
**Explanation:** Calculates backbone RMSD without least-squares fitting to the reference; using -nofit is appropriate when structures are already aligned to preserve the biological interpretation of conformational change.

### Analyze hydrogen bond occupancy between two chain groups
**Args:** `hbond -s structure.tpr -ttr trajectory.xtc -select "chain A and resid 10 to 50" -select2 "chain B and resid 60 to 100" -o hbond.xvg`
**Explanation:** Computes hydrogen bond occupancy and distances between specified residue ranges across chains over the entire trajectory; essential for studying protein-protein or protein-ligand interface stability.

### Calculate radius of gyration for a protein to assess folding state
**Args:** `radius_gyration -s structure.tpr -ttr trajectory.xtc -select protein -o rog.xvg`
**Explanation:** Computes the radius of gyration time series to monitor protein compactness; values below ~2.0 nm typically indicate folded states while unfolded proteins show Rg > 2.5 nm.

### Compute solvent-accessible surface area for a protein-ligand complex
**Args:** `sasa -s structure.tpr -ttr trajectory.xtc -select "protein and resname LIG" -o sasa.xvg`
**Explanation:** Calculates SASA for the ligand binding site to quantify solvent exposure during binding/unbinding events; this data is critical for understanding dehydration penalties in molecular recognition.

### Calculate inter-chain residue contacts over a trajectory
**Args:** `contacts -s structure.tpr -ttr trajectory.xtc -o contacts.xvg -output "json" -cutoff 0.6`
**Explanation:** Identifies residue-residue contacts between chains using a 0.6 nm cutoff and outputs results in JSON format for downstream network analysis; useful for mapping allosteric communication pathways.

### Extract diffusion coefficient from mean square displacement
**Args:** `diffusion -trj trajectory.xtc -select "resname NA+" -o diffusion.json -begin 100 -end 1000`
**Explanation:** Calculates the diffusion coefficient for sodium ions using MSD analysis from frame 100 to 1000 (skipping equilibration); the JSON output provides D values in proper units for validation against experimental ionic mobility data.