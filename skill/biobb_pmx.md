---
name: biobb_pmx
category: Molecular Dynamics Analysis / GROMACS
description: Python building blocks for free energy calculations using GROMACS pmx tools. Provides CLI wrappers for analyzing thermodynamic properties, extracting ensemble data from MD simulations, and processing free energy perturbation (FEP) results.
tags:
  - molecular dynamics
  - free energy
  - GROMACS
  - thermodynamics
  - MD analysis
  - alchemical
author: AI-generated
source_url: https://github.com/bioexcel/biobb_pmx
---

## Concepts

- **Alchemical free energy analysis**: The pmx package analyzes results from alchemical free energy perturbation (FEP) calculations in GROMACS, extracting ΔG values from .xvg or .edr files and computing binding affinities or solvation free energies.

- **Key input formats**: Accepts GROMACS output files including .xvg (xdrace plotting), .edr (energy files), .log files, and plain text output from various GROMACS analysis tools. Outputs are typically CSV, JSON, or new .xvg files.

- **Ensemble averaging**: Computes statistically rigorous free energy estimates by averaging over multiple simulation windows or replicas, often requiring BAR, MBAR, or TI (thermodynamic integration) methods for accurate ΔG determination.

- **Companion binaries**: The package includes separate CLI tools (biobb_pmx-build, biobb_pmx-check) for preparing topology files and validating simulation inputs before running free energy analyses.

## Pitfalls

- **Incompatible energy file formats**: Using .xvg files from older GROMACS versions or different force fields without specifying the correct format version can produce NaN values or silent failures in ΔG calculations.

- **Insufficient equilibration**: Analyzing simulation windows that haven't reached equilibrium (insufficient sampling in early windows) propagates systematic errors into the final free energy estimate, making results unreliable.

- **Neglecting overlap**: Free energy methods (BAR/MBAR) require sufficient phase space overlap between adjacent windows. Skipping windows or using too few lambda points yields biased ΔG values with underestimated error bars.

- **Mixing units**: Confusing kJ/mol with kcal/mol or using temperature in Kelvin when the tool expects Celsius leads to incorrect numerical results without explicit warnings.

## Examples

### Analyze free energy data from an XVG file
**Args:** `--ixvg input_data.xvg --print_results`
**Explanation:** Reads the GROMACS .xvg output containing dhdl data and extracts free energy contributions using default MBAR analysis.

### Compute ΔG using the BAR method
**Args:** `--ixvg input_data.xvg --method bar --print_results`
**Explanation:** Applies the Bennett Acceptance Ratio method instead of the default MBAR, which is more robust when window overlap is limited.

### Process multiple energy files for ensemble averaging
**Args:** `--ixvg energy1.xvg --ixvg energy2.xvg --ixvg energy3.xvg --print_results`
**Explanation:** Combines three replicate simulations to compute a statistically averaged ΔG with improved confidence intervals.

### Export results to CSV format
**Args:** `--ixvg input_data.xvg --output_csv results.csv`
**Explanation:** Converts the analyzed free energy results into a CSV file for downstream processing or plotting in external tools.

### Use specific temperature for the analysis
**Args:** `--ixvg input_data.xvg --temperature 310 --print_results`
**Explanation:** Sets the system temperature to 310 K (physiological conditions) for accurate thermodynamic integration calculations.

### Build FEP topology for a small molecule transformation
**Args:** build --ifile mol.itp --ofile output.itp --atoms 12`
**Explanation:** Uses the companion biobb_pmx-build tool to generate a topology file interpolating between two ligand states for alchemical transformation.

### Validate FEP simulation setup before running
**Args:** check --input_tpr simulation.tpr --tpr_out validated.tpr`
**Explanation:** Uses the biobb_pmx-check tool to verify that the .tpr file contains correct lambda states and no corrupt parameters.