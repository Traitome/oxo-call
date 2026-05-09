---
name: biobb_structure_manager
category: Structural Biology / Molecular Dynamics
description: A tool from the BioPython Building Blocks (BioBB) package for managing, fetching, and manipulating molecular 3D structures from databases like PDB and RCSB. Supports operations including structure retrieval, chain extraction, ligand isolation, and structure validation.
tags: [pdb, molecular-structure, structure-management, protein-structure, ligand-extraction, rcsb, bioinformatics, biobb]
author: AI-generated
source_url: https://github.com/bioexcel/biobb_structure
---

## Concepts

- **Input/Output Formats**: Supports common molecular structure file formats including PDB (`.pdb`), MOL2 (`.mol2`), and PDBQT (`.pdbqt`) for docking preparations. The tool reads structure coordinates from local files or directly fetches them from the RCSB Protein Data Bank using entry IDs.
- **Structure Retrieval**: Can fetch entries from RCSB by providing a valid PDB ID (e.g., "1CRN", "4HHB"). Fetched structures are automatically saved in the specified output format. The tool can retrieve the full entry or specific chains as requested.
- **Chain and Ligand Extraction**: Enables extraction of specific protein chains, water molecules, or heteroatoms (ligands) from a complete structure. This is essential for preparing receptor files for docking or isolating ligand structures for parameterization.
- **Output Organization**: Automatically creates output directories if they do not exist. Output files are named according to the input structure ID and specified format, enabling easy tracking of processed structures.
- **Validation Warnings**: The tool checks for common structure issues like missing backbone atoms, non-standard residues, or incomplete chains. Warnings are printed to stderr but do not stop execution, allowing downstream processing to proceed.

## Pitfalls

- **Invalid PDB IDs**: Providing a non-existent or misspelled PDB ID results in an error from the RCSB server. The error message may be generic ("Structure not found"), requiring manual verification of the ID against the PDB database.
- **Mismatched File Formats**: Requesting output in a format not supported by the input structure (e.g., requesting PDBQT from a metal-containing structure without hydrogens) may produce corrupted output files that downstream tools cannot read.
- **Overwriting Existing Files**: The tool will silently overwrite existing output files without confirmation. This can cause loss of previously processed structures if the same output path is reused unintentionally.
- **Network Dependency**: Fetching structures from RCSB requires an active internet connection. Firewalls or proxy restrictions blocking RCSB access will cause all fetch operations to fail with connection errors.
- **Insufficient Disk Space**: Large structure files or batch processing multiple structures can consume significant disk space. Running without checking available space results in partial file writes and subsequent file integrity errors.

## Examples

### Fetch a protein structure from RCSB by PDB ID
**Args:** `--pdb-id 1CRN --output-format pdb --output-path ./structures/1crn.pdb`
**Explanation:** This retrieves the crystal structure of Crambin (PDB ID 1CRN) from the RCSB database and saves it in PDB format to the specified path for downstream analysis or docking preparation.

### Extract a single protein chain from a multi-chain structure
**Args:** --input-structure-file ./structures/4hhb.pdb --chain-id A --output-chain-only --output-path ./structures/4hhb_chainA.pdb
**Explanation:** This extracts only chain A from hemoglobin tetramer (4HHB), creating a monomeric receptor file suitable for single-chain docking protocols or interface analysis.

### Fetch a specific chain and save as MOL2 format
**Args:** --pdb-id 2AZW --chain-id B --output-format mol2 --output-path ./structures/2azw_chainB.mol2
**Explanation:** This fetches the complete entry 2AZW but extracts only chain B, converting it to MOL2 format which includes atom typing needed for quantum chemistry calculations.

### Extract all heteroatoms (ligands) from a structure
**Args:** --input-structure-file ./structures/1NAV --extract-ligands --output-format pdb --output-path ./structures/1nav_ligands.pdb
**Explanation:** This isolates all non-protein atoms including cofactors, metals, and water molecules from the Nav1.7 sodium channel structure, useful for ligand parameterization or metal center analysis.

### Fetch ligand-free protein (apo) structure by removing all heteroatoms
**Args:** --pdb-id 5TDP --remove-ligands --output-path ./structures/5tdp_apo.pdb`
**Explanation:** This retrieves the full entry but removes all heteroatoms, producing a clean apo protein structure standard for comparing bound and unbound conformations in structural biology studies.

### Convert an existing local PDB to MOL2 format
**Args:** --input-structure-file ./structures/my_structure.pdb --output-format mol2 --output-path ./structures/my_structure.mol2`
**Explanation:** This converts a previously downloaded or edited PDB file to MOL2 format without re-fetching from RCSB, enabling compatibility with molecular modeling tools requiring tripos atom types.

### Fetch multiple chains and keep only waters within 5 Angstroms
**Args:** --pdb-id 1L2Y --chain-id A --keep-waters-near 5.0 --output-path ./structures/1l2y_A_waters.pdb`
**Explanation:** This extracts chain A along with crystal waters within 5 Angstroms of the protein, preserving hydration information critical for binding site analysis or QM/MM calculations.