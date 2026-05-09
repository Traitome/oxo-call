---
name: biobb_pdb_tools
category: Structural Biology / PDB Manipulation
description: A collection of tools from the BioBuildingBlocks library for parsing, manipulating, and transforming PDB (Protein Data Bank) format structure files. Includes utilities for extracting chains, removing ligands/water, splitting models, centering structures, and other common PDB transformations.
tags: [pdb, protein structure, bioinformatics, molecular biology, file conversion, structure manipulation, biobb]
author: AI-generated
source_url: https://github.com/bioexcel/biobb_pdb
---

## Concepts

- **Input/Output Format**: All tools in this collection operate on PDB format files (.pdb, .ent). Some tools also support mmCIF format. Input files must be valid structure files with ATOM/HETATM records; malformed files will cause parsing failures.
- **Chain and Residue Selection**: Most tools support selecting specific chains using the `--chain` flag (single letter like A, B) or residue ranges using `--residue_number_start` and `--residue_number_end`. Ranges are 1-indexed and inclusive.
- **Atomic Coordinate Manipulation**: Tools operate on the coordinates section of PDB files (ATOM/HETATM records), preserving metadata like HEADER, TITLE, and REMARK sections unchanged unless specifically targeted for modification.

## Pitfalls

- **Invalid Chain Identifiers**: Specifying a chain letter that does not exist in the input PDB file produces no error but outputs an empty or incomplete structure; always verify chain IDs in the source file first using a PDB viewer or `grep` for chain identifiers.
- **1-indexed Residue Numbering**: PDB residue numbers are 1-indexed in the chain sequence; specifying start/end values incorrectly (e.g., using 0 as start) will silently select wrong residues or none at all.
- **Overwriting Input Files**: Many tools default to writing output with the same name as input if the `--output` flag is not specified, causing data loss; always explicitly set `--output` to preserve the original structure.

## Examples

### Extract a specific chain from a PDB structure
**Args:** `--input 1UBQ.pdb --chain A --output 1UBQ_chainA.pdb`
**Explanation:** Extracts only chain A from a multi-chain PDB file, useful for generating single-chain inputs for downstream docking or simulation.

### Remove all water molecules from a structure
**Args:** `--input 4ABC.pdb --output 4ABC_nohoh.pdb --remove_water true`
**Explanation:** Removes all HETATM records marked as water (HOH), reducing file size and preventing water interference in binding site analysis.

### Split a multi-model NMR structure into individual models
**Args:** --input 2KIH.pdb --output_model true --output 2KIH_models.pdb`
**Explanation:** Splits an NMR ensemble (multiple MODEL/ENDMDL sections) into separate PDB files (named with _model1, _model2 etc.), required for single-model analysis tools.

### Center the structure at the origin
**Args:** --input 1CRN.pdb --output 1CRN_centered.pdb --center true`
**Explanation:** Calculates the geometric center of all atoms and translates coordinates so the structure is centered at (0,0,0), essential for certain docking protocols requiring origin-centered inputs.

### Extract a residue range from a specific chain
**Args:** --input 1LYD.pdb --chain B --residue_number_start 10 --residue_number_end 50 --output 1LYD_B10-50.pdb`
**Explanation:** extracts residues 10 through 50 from chain B, creating a truncated structure file for focused analysis of a specific domain or binding region.

### Remove all ligands except crystallographic waters
**Args:** --input 3HXY.pdb --output 3HXY_clean.pdb --remove_ligands true --remove_water false`
**Explanation:** Removes all non-protein residues (ligands, ions, cofactors) while preserving water molecules, useful for preparing apo structures for comparative analysis.

### Sort atoms by residue number within each chain
**Args:** --input 1MEL.pdb --output 1MEL_sorted.pdb --sort_method atom`
**Explanation:** Reorders ATOM records to be sequential by residue number within each chain rather than the original PDB order, improving compatibility with tools expecting standard atom ordering.