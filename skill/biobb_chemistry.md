---
name: biobb_chemistry
category: computational chemistry / molecular dynamics
description: A BioBB (BioExcel Building Blocks) Python module providing wrappers for small-molecule chemistry operations used in molecular dynamics workflows. Includes format conversion, partial charge calculation, Amber topology generation via ACPYPE, and molecular descriptor computation. All tools are invoked as Python imports from the `biobb_chemistry` package hierarchy.
tags:
  - bioexcel
  - small molecule
  - ligand parametrization
  - ACPYPE
  - Amber
  - SDF
  - MOL
  - PDBQT
  - mol2
  - GROMACS
  - molecular dynamics
  - python
author: AI-generated
source_url: https://github.com/bioexcel/biobb_chemistry
---

## Concepts

- **Python import invocation model**: biobb_chemistry tools are not shell executables but Python functions imported from the `biobb_chemistry` package hierarchy, such as `from biobb_chemistry.acpype import actupype_par` or `from biobb_chemistry.gamess.input import gamess_input`. Scripts must run in a Python environment where `biobb_chemistry` is installed via conda or pip, and they must set up an `inputProps` dictionary and an `outputProps` dictionary to pass configuration and file paths between steps.
- **ACPYPE for Amber topology generation**: ACPYPE (Antecedent Partitioning for Peptide and small-molecule Exchange) is the central tool for converting molecular structure files into GROMACS-compatible topology and coordinate files. It accepts MOL, SDF, or MOL2 input and requires either AM1-BCC partial charges (computed externally) or an explicit `-ac` charge specification; ACPYPE generates both `*_GMX.itp` and `*_GMX.gro` output files for direct GROMACS use.
- **Multi-format molecular I/O**: biobb_chemistry functions read and write molecular file formats including MOL (MDL Molfile), SDF (MDL Molfile series), PDBQT (AutoDock), MOL2 (Tripos), and GRO (GROMACS trajectory). The correct format pair must be matched between adjacent steps in a pipeline; for example, a GAMESS charge calculation step must output in a format ACPYPE can parse.
- **External binary dependency**: ACPYPE and several other wrapped tools depend on external compiled binaries (AMBER tools, GAMESS, Open Babel) that are not bundled inside the Python package. These must be installed separately in the conda environment, and their executable paths must be accessible via the `PATH` environment variable or configured via the `conda_path` property in the input dictionary.

## Pitfalls

- **Missing external binary runtime**: Calling an ACPYPE function when the `acpype` binary is not in `PATH` produces no error at the Python layer but generates empty or zero-byte output files, silently failing the topology step. Always verify that `acpype --version` responds in a fresh environment before running a pipeline.
- **Molecule perception failure on malformed SDF/MOL**: If the input structure file has non-standard atom types, incorrect bond orders, or missing hydrogen atoms, the library will raise a `BabelchemError` or return a malformed output molecule that causes downstream GROMACS to crash with `Invalid atom type` or `Cannot guess ATOM names`. Always validate input molecules with `obabel` before passing them to biobb_chemistry.
- **Wrong format pair between pipeline steps**: Connecting a step that produces PDBQT to a step expecting MOL2 will cause a format parse error at runtime, even though no type-checking guard exists in the Python API. Explicitly verify that `inputProps['input_format']` and `inputProps['output_format']` are set to matching literal strings for each step.
- **Silent zero-byte output when `output_path` is misconfigured**: If the `output_path` key is omitted from `outputProps`, most functions write to the current working directory using an auto-generated filename, which may overwrite prior results or write to an unintended location. Always populate `output_path` explicitly.
- **Ignoring the `charge` property**: ACPYPE requires a non-zero charge specification for the molecule to generate correct net charge in the topology. Passing `charge=0` when the actual ligand has a non-zero charge will produce topology files with incorrect total charge, causing GROMACS to fail with `Fatal error: No default MM parameters` during energy minimization.

## Examples

### Generate GROMACS topology and coordinate files for a ligand from an MOL2 input using ACPYPE

**Args:** `from biobb_chemistry.acpype import acpype_params; acpype_params(input_path='ligand.mol2', input_format='mol2', output_path='ligand_gmx', output_format='gro')`
**Explanation:** This imports the ACPYPE wrapper and converts an MOL2 structure file into GROMACS-compatible ITP and GRO files by passing explicit format strings so ACPYPE knows how to parse the input and name the output.

### Convert an SDF multi-molecule file to individual PDBQT files for AutoDock Vina docking

**Args:** `from biobb_chemistry.ad4process import process_ligand; process_ligand(input_path='compounds.sdf', output_path_prefix='dock_', output_format='pdbqt')`
**Explanation:** The AD4 process wrapper reads each molecule record from an SDF file and writes each as a separate PDBQT file with the given prefix, enabling batch docking without manual format conversion for each compound.

### Calculate Amber partial charges on a ligand using GAMESS before ACPYPE topology generation

**Args:** `from biobb_chemistry.gamess import gamess_input; gamess_input(input_path='ligand.mol', output_path='ligand_gamess.out', charge=0, method='pm3', task='gradient')`
**Explanation:** This runs a GAMESS semi-empirical gradient calculation to obtain electron density for AM1-BCC charge derivation, storing the output for extraction by an ACPYPE charge-reading step that follows in the pipeline.

### Compute molecular descriptors (MW, logP, HBA, HBD) for a set of molecules from an SDF file

**Args:** `from biobb_chemistry.pmx import pmx_atomtype; pmx_atomtype(input_path='molecules.sdf', input_format='sdf', output_path='molecules_desc.csv', output_format='csv')`
**Explanation:** The pmx atomtype wrapper computes and aggregates molecular descriptors for each record in the input SDF and writes them as a CSV table, which is useful for filtering a virtual screening library before parametrization.

### Generate GROMACS input from an MOL2 file with an explicit net charge for a charged ligand

**Args:** `from biobb_chemistry.acpype import acpype_params; acpype_params(input_path='drug.mol2', input_format='mol2', output_path='drug_gmx', charge=-1, output_format='gro')`
**Explanation:** Setting `charge=-1` ensures ACPYPE records the correct total charge in the topology ITP file so that GROMACS adds the correct number of counterions during system solvation, preventing fatal charge-mismatch errors later.

### Validate a converted PDBQT file by checking that all torsion entries are present for AutoDock Vina

**Args:** `from biobb_chemistry.ad4process import process_ligand; process_ligand(input_path='ligand.sdf', output_path='ligand_validated.pdbqt', output_format='pdbqt', detect_torsions=True)`
**Explanation:** Enabling `detect_torsions=True` causes the AD4 process wrapper to log the number of rotatable bonds detected, letting the user confirm that a ligand intended to be rigid has zero rotatable bonds before docking, which is required for accurate binding affinity estimation.

### Check that ACPYPE is available before running any pipeline step in a new environment

**Args:** `import subprocess; subprocess.run(['acpype', '--version'], check=True, capture_output=True)`
**Explanation:** This validates that the external ACPYPE binary is reachable via the shell path before any biobb_chemistry function that depends on it is called, catching the missing-binary pitfall early during environment setup rather than after partial output files are generated.