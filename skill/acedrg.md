---
name: acedrg
category: Chemistry/Molecular Format Conversion
description: Converts between molecular structure formats (SMILES, SDF, MOL, MOL2) with optional canonicalization. Part of the ACEDB package for handling chemical structure data in bioinformatics workflows.
tags:
  - chemistry
  - molecular-format
  - format-conversion
  - small-molecules
  - sdf
  - mol
  - smiles
author: AI-generated
source_url: https://cedgen.univ-lyon1.fr/acedb/acedrg.html
---

## Concepts

- **Input/Output Formats**: acedrg supports SMILES (single-line), SDF/MOL (multi-molecule), and MOL2 formats. The tool automatically detects input format from file extension or can be explicitly specified using `-s` for input and `-o` for output format.
- **Batch Processing**: When processing multi-molecule SDF/MOL files, acedrg retains all molecule records and appends them to the output file. The tool iterates through each molecule independently for format conversion.
- **Canonicalization Option**: Using `-c` triggers SMILES canonicalization, which produces a standardized (unique) SMILES representation. Canonical SMILES ensures identical molecules always yield the same string, critical for database deduplication.
- **Structure Validity**: The tool validates chemical valence and connectivity before conversion. Invalid structures are reported to stderr with line numbers, but processing continues for valid records in batch files.
- **Implicit Hydrogens**: Output formats differ in hydrogen handling—MOL/MOL2 explicitly list all hydrogens, while SMILES notation implies them. The `-a` flag adds explicit hydrogens to SMILES output.

## Pitfalls

- **Mismatched Format Flags**: Specifying an output format (`-o`) that is incompatible with the input data causes silent failures or truncated output. Always verify format compatibility before running batch conversions.
- **Large SDF Files Without Memory Management**: Processing very large SDF files (>100MB) can cause memory issues because acedrg loads the entire file into memory before conversion. Split large files into smaller chunks to avoid crashes.
- **Duplicate Molecule Names**: When converting SDF to MOL2 with `-w` (overwrite), molecules with identical names in the input get overwritten in output, causing data loss. Use unique identifiers or enable append mode.
- **Inconsistent Stereochemistry Notation**: Stereochemical descriptors may be lost during format conversion if the target format does not support that type of notation. Always verify stereochemistry fidelity in output using a molecular viewer.
- **Missing aromaticity handling**: By default, acedrg treats aromatic rings as Kekulé form in SMILES output. Use `-r` to preserve aromatic notation, or canonicalization will produce non-aromatic Kekulé SMILES.

## Examples

### Convert a SMILES string to MOL format
**Args:** `-s smiles -o mol "CCO" -f ethanol.mol`
**Explanation:** This converts a SMILES string for ethanol into MOL format, writing the output to the specified file.

### Batch convert anSDF file to multiple MOL2 files
**Args:** `-s sdf -o mol2 -d output_dir/ -i molecules.sdf`
**Explanation:** Converts each molecule in an SDF file to separate MOL2 files, placing them in the designated output directory.

### Generate canonical SMILES from MOL input
**Args:** `-s mol -o smiles -c -f example.mol`
**Explanation:** Converts an MOL file to canonical SMILES, ensuring the output SMILES is unique and standardized for the molecule.

### Preserve aromaticity in SMILES output
**Args:** `-s mol -o smiles -r -f benzene.mol -n "benzene.arom.smi"`
**Explanation:** Converts an MOL file to aromatic SMILES notation using the `-r` flag to maintain aromatic ring representation.

### Convert MOL2 to SDF with explicit hydrogens
**Args:** `-s mol2 -o sdf -a -f input.mol2 -n output.sdf`
**Explanation:** Converts MOL2 format to SDF, adding explicit hydrogen atoms to all atoms in the output using the `-a` flag.

### Process a multi-molecule SDF and filter invalid structures
**Args:** `-s sdf -o sdf -v -f mixed.sdf -n valid_only.sdf`
**Explanation:** Processes an SDF file containing both valid and invalid structures, only outputting valid molecules to the result file.

### Convert SMILES to MOL2 for molecular docking
**Args:** -s smiles -o mol2 -f ligand.smi -n ligand.mol2
**Explanation:** Converts a SMILES string to MOL2 format, which retains bond order and atom type information required by docking software.