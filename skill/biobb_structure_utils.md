---
name: biobb_structure_utils
category: Structural Biology Utilities
description: A collection of command-line tools from the BioExcel Building Blocks library for manipulating 3D molecular structures. Provides utilities for chain extraction, ligand removal, water stripping, structure fixing, and format conversion operations on PDB and mmCIF files.
tags: ["bioinformatics", "molecular-structure", "pdb", "mmcif", "protein-structure", "biobb", "structural-biology"]
author: AI-generated
source_url: https://github.com/bioexcel/biobb_structure_utils
---

## Concepts

- **Data Model**: Works with molecular structure files in PDB and mmCIF formats. Each structure contains atoms, residues, chains, and optional ligands/waters that can be selectively manipulated or removed.
- **Input/Output**: Supports direct file path arguments for input structure and output destination. File formats are typically inferred from extensions (.pdb, .cif) but can be explicitly specified via flags.
- **Key Utilities**: The package includes multiple companion binaries for different operations - chain extraction (select_chain), ligand removal (remove_ligand), water stripping (remove_water), structure fixing (fix_structure), and more.
- **Atomic Selection**: Many operations use chain identifiers (e.g., --chain_id A) to target specific molecular components for inclusion or exclusion from output.

## Pitfalls

- **Input Format Mismatch**: Using an unrecognized file extension causes the tool to fail with a parsing error. Ensure your input file has a valid .pdb, .cif, or .mmcif extension.
- **Non-Existent Chain**: Specifying a chain identifier that does not exist in the input structure produces an empty output file without warning. Verify chain IDs in your input structure before processing.
- **Overwriting Input**: Specifying the same path for input and output without backup will overwrite the original structure, potentially losing data. Always use distinct output paths or create backups first.
- **Missing Dependencies**: Some advanced operations require external libraries (Biopython, MDAnalysis) that may not be installed. Missing dependencies cause cryptic import errors at runtime.

## Examples

### Extract a specific protein chain from a multi-chain PDB structure
**Args:** `--input_structure_path receptor.pdb --output_structure_path chain_a.pdb --chain_id A`
**Explanation:** This extracts only chain A from a multi-chain PDB file, useful for isolating a single protein subunit for further analysis.

### Remove all water molecules from a crystal structure
**Args:** `--input_structure_path hydrated.pdb --output_structure_path dry.pdb --remove_water true`
**Explanation:** This strips all HOH residues from the structure, which is often necessary before molecular dynamics simulations or binding analyses.

### Remove bound ligands while keeping protein and waters
**Args:** `--input_structure_path complex.pdb --output_structure_path protein_only.pdb --remove_ligand true`
**Explanation:** This removes all non-protein residues (ligands, ions, substrates) while preserving the protein chain and crystal waters.

### Convert a PDB file to mmCIF format
**Args:** `--input_structure_path protein.pdb --output_structure_path protein.cif --output_format cif`
**Explanation:** This converts the legacy PDB format to the modern mmCIF format, which supports more metadata and larger structures.

### Fix common structure issues in a PDB file
**Args:** `--input_structure_path broken.pdb --output_structure_path fixed.pdb --fix_issues true`
**Explanation:** This attempts to repair common problems in PDB files such as missing atoms, incorrect residue numbering, or chain break warnings.

### Extract multiple chains by specifying them as a comma-separated list
**Args:** `--input_structure_path complex.pdb --output_structure_path ab_heterodimer.pdb --chain_id A,B`
**Explanation:** This extracts chains A and B together, keeping both subunits of a heterodimer while discarding any other chains present in the structure.

### Remove all non-protein components including waters and ligands
**Args:** `--input_structure_path crystal.pdb --output_structure_path clean.pdb --remove_water true --remove_ligand true`
**Explanation:** This combines water and ligand removal in a single operation, leaving only the protein atoms for simplified processing.

### Generate a structure file with explicit output format specification
**Args:** `--input_structure_path input.cif --output_structure_path output.pdb --output_format pdb`
**Explanation:** This explicitly specifies the output format as PDB even when the input is mmCIF, ensuring compatibility with older analysis tools.