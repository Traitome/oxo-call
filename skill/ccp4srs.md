---
name: ccp4srs
category: Macromolecular Crystallography
description: A CCP4 program for structural refinement and optimization of crystal structures. Performs iterative least-squares refinement against crystallographic data to improve model coordinates and atomic parameters.
tags: [crystallography, refinement, protein-structure, ccp4, x-ray]
author: AI-generated
source_url: https://www.ccp4.ac.uk/ccp4wiki/
---

## Concepts

- **MTZ Reflection Data**: ccp4srs requires input MTZ (mergeable translate) files containing observed structure factor amplitudes and phases. The program reads columns labeled with FOBS, SIGFOBS, and optionally HL coefficients for maximum-likelihood refinement.

- **PDB Coordinate Input**: Starting atomic models must be provided in PDB format with correct residue numbering and chain identifiers. The program expects standard atom names (CA, CB, N, C, O) matching the monomer library definitions.

- **Refinement Strategy Control**: The program supports isotropic and anisotropic atomic displacement parameter (ADP) refinement, TLS (translation-libration-screw) motion modeling, and optional automated water addition cycles. Resolution-dependent weight optimization is critical for stable convergence.

- **Output Data Types**: Successful refinement produces updated PDB coordinates, an MTZ file with calculated structure factors and phases, and optionally refinement statistics in the log output. The program generates TLS groups automatically if requested.

## Pitfalls

- **Mismatched MTZ Column Labels**: Specifying incorrect or non-existent column labels (e.g., using "F" instead of "FOBS") causes immediate failure with a cryptic error message about missing columns. Always verify MTZ contents with `mtzinfo` before running refinement.

- **Resolution Cutoff Mismatch**: Applying overly aggressive high-resolution cutoffs (beyond the data quality limits) produces unstable refinement with unrealistic atomic.adp values. The data must support the resolution cutoff chosen.

- **Incorrect Space Group**: Providing a space group inconsistent with the input MTZ symmetry causes the program to reject the data. Verify space group consistency between all input files before refinement begins.

- **Missing Monomer Library**: Running without defining the monomer library (CIF or restraint file) results in complete failure with "No monomer library found" errors. Specify the appropriate chemistry library for all ligand residues present.

## Examples

### Refine a protein structure against native data

**Args:** -mtzin native.mtz -pdbin model.pdb -pdbout refined.pdb -xyzin model.cif -resolution 2.0 -ncycle 5

**Explanation:** Runs five cycles of refinement against native data at 2.0 Å resolution, using the specified monomer library for restraints.

### Refine with anisotropic ADPs

**Args:** -mtzin data.mtz -pdbin start.pdb -pdbout refined.pdb -anisotropic -ncycle 3

**Explanation:** Enables anisotropic refinement of atomic displacement parameters for three cycles, producing more physically reasonable ADPs for higher-resolution data.

### Refine with TLS groups

**Args:** -mtzin data.mtz -pdbin start.pdb -pdbout refined.pdb -tls -tlsin tls_groups.pdb -ncycle 4

**Explanation:** Performs TLS (translation-libration-screw) motion refinement using pre-defined TLS groups to model domain motions more accurately.

### Add waters automatically after refinement

**Args:** -mtzin data.mtz -pdbin model.pdb -pdbout final.pdb -water -resolution 2.5 -ncycle 3

**Explanation:** Automatically adds water molecules after each refinement cycle at 2.5 Å resolution, accepting peaks above the configured sigma cutoff.

### Refine with hydrogen atoms

**Args:** -mtzin data.mtz -pdbin model.pdb -pdbout refined.pdb -refine ALL -hydrogen YES -ncycle 5

**Explanation:** Explicitly includes hydrogen atoms in the refinement with all coordinates and ADPs optimized, for high-resolution structures where hydrogens are observable.

### Control refinement weights interactively

**Args:** -mtzin data.mtz -pdbin start.pdb -pdbout refined.pdb -weight 0.5 -ncycle 4

**Explanation:** Sets the weight between X-ray and geometry terms to 0.5, increasing the influence of geometric restraints to prevent overfitting of weak data.

### Refine as_job using a script file

**Args:** -mtzin data.mtz -pdbin model.pdb -pdbout refined.pdb -jobin refine_script.txt

**Explanation:** Runs a scripted refinement sequence from an external job file, allowing complex multi-stage refinement strategies with different parameters at each stage.

---