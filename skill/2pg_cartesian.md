---
name: 2pg_cartesian
category: Molecular Structure Processing
description: Converts molecular structure files to Cartesian coordinate representations. Extracts and reformats 3D atomic coordinates from PDB, MOL, or other molecular file formats into standardized Cartesian coordinates suitable for further computational analysis or visualization.
tags: [molecular-structures, cartesian-coordinates, pdb-processing, structural-conversion, computational-chemistry, structural-analysis]
author: AI-generated
source_url: https://github.com/3d-e-chem/2pg-cartesian
---

## Concepts

- **Input Format Support**: Accepts standard PDB (Protein Data Bank) files, MOL/SDF format files, and XYZ coordinate files. Multi-model files (e.g., NMR ensembles) are processed by selecting specific models or reading all models sequentially.
- **Output Coordinate Systems**: Produces standard Cartesian coordinates (x, y, z) in Ångströms for each atom, preserving element type, residue information, and atom naming. The output can be filtered by atom type, residue, or chain identifier.
- **Coordinate Precision and Units**: Default output precision is 3 decimal places (0.001 Å), sufficient for most molecular modeling tasks. Units are always in Angströms regardless of input file units.
- **Model and Frame Selection**: For trajectories or multi-model files, specific frames or model numbers can be selected using 1-based indexing. Without selection, only the first model/frame is processed by default.
- **Atom Selection Syntax**: Supports standard PDB atom selection language for filtering atoms to include in output. Selections can be based on atom name, residue number, chain ID, or element type.

## Pitfalls

- **Incorrect Model Indexing**: Using a model number that exceeds the total number of models in the input file causes silent failure or empty output. Multi-model PDB files from NMR structures may contain 20+ models; always verify model count first.
- **Inconsistent Atom Naming**: PDB files from different sources may use variant atom naming conventions (e.g., "CA" vs "CA" for alpha-carbon, "OP1" vs "O1'" for phosphate oxygens). Output may miss atoms if selection criteria don't match the actual naming in your file.
- **Missing Chain or Residue Information**: If the input PDB lacks chain identifiers (common in older files), chain-based filtering will fail. The tool preserves whatever annotation exists in the input but cannot infer missing metadata.
- **Non-Standard Elements**: Files containing elements not in the periodic table or malformed element symbols cause parsing errors. Always validate input files with a molecular viewer before processing.
- **Whitespace and Delimiter Issues**: Tab-separated vs space-separated fields in output may cause downstream parsing failures. The tool outputs space-delimited fields by default; specify alternative delimiters explicitly if required by downstream tools.

## Examples

### Extract all atoms from a single PDB file
**Args:** input.pdb -o output.xyz
**Explanation:** Reads all atoms from input.pdb and writes Cartesian coordinates in XYZ format to output.xyz for visualization or further processing.

### Process only alpha-carbon atoms from a protein structure
**Args:** protein.pdb --selection "name CA" -o ca_atoms.xyz
**Explanation:** Filters the PDB to include only CA (alpha-carbon) atoms, producing a simplified backbone representation suitable for structural comparisons.

### Select a specific model from an NMR ensemble
**Args:** nmr_ensemble.pdb --model 10 -o model_10.pdb
**Explanation:** Extracts model number 10 from an NMR ensemble file containing multiple structural models, useful for comparing individual conformations.

### Convert MOL format to Cartesian coordinates
**Args:** molecule.mol -o molecule_cartesian.xyz
**Explanation:** Converts a MOL/SDF format file to XYZ Cartesian coordinates, preserving atom types and 3D positions in a simple text format.

### Filter atoms by chain identifier
**Args:** multi_chain.pdb --chain A -o chain_a.pdb
**Explanation:** Extracts only chain A from a multi-chain PDB file, isolating a specific subunit for independent analysis or docking studies.

### Specify custom output precision
**Args:** input.pdb -o output.xyz --precision 4
**Explanation:** Writes Cartesian coordinates with 4 decimal places (0.0001 Å) precision, useful when high accuracy is required for quantum mechanical calculations.

### Process only a specific residue range
**Args:** protein.pdb --selection "resid 10 to 50" -o residues_10_50.xyz
**Explanation:** Extracts atoms within residues 10 through 50, useful for focusing on a particular domain or functional region of a protein.

### Write output without header information
**Args:** input.pdb --noheader -o clean_output.xyz
**Explanation:** Outputs only the atomic coordinate lines without the XYZ file header, suitable for piping directly into tools that require header-free coordinate streams.