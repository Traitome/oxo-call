---
name: biobb_structure_checking
category: structure_validation
description: A BioExcel Building Blocks (BioBB) tool for checking and validating molecular structures (PDB files). It detects issues such as missing atoms, chain breaks, alternate locations, occupancy problems, and other structural anomalies. Can also apply fixes to resolve identified problems.
tags: [molecular-structure, pdb-validation, structural-biology, protein-structure, quality-control, biobb]
author: AI-generated
source_url: https://github.com/bioexcel/biobb_structure_checking
---

## Concepts

- **Input Format**: Accepts PDB format files (both standard PDB and PDBx/mmCIF) for structural validation and checking. The tool reads molecular coordinate data and analyzes structural properties.
- **Output Reports**: Generates detailed reports on structural issues found in the input structure, including missing atoms, chain breaks, alternate location indicators, and occupancy problems. Reports can be in text or JSON format.
- **Fix Capabilities**: Beyond detection, the tool can automatically apply corrections for common issues like adding missing atoms, removing water molecules, or handling alternate locations based on specified criteria.
- **Flexible Checking Options**: Users can select specific checks to run (e.g., `--check-only missing_atoms`, `--check-only chain_break`) or run all checks simultaneously, making it suitable for both targeted and comprehensive structure validation.

## Pitfalls

- **Not Specifying Input Format**: Failing to specify the input format with `--input-format pdb` or `--input-format pdbx` may cause the tool to fail when reading non-standard PDB files or PDBx files.
- **Running Without --output-path**: Omitting the `--output-path` flag means corrected structures are written to the same file as input, potentially overwriting valuable original data.
- **Confusing --check-only with --fix-only**: Using `--check-only` runs validation without applying any fixes, while `--fix-only` attempts to correct all detected issues automatically—these are mutually exclusive behaviors that produce different outcomes.
- **Ignoring --non-interactive Flag**: When using fix capabilities in scripts or pipelines, not including `--non-interactive` may cause the tool to prompt for user confirmation, blocking automated workflows.

## Examples

### Check a PDB file for all structural issues
**Args:** --input-structure-path structure.pdb --output-path checked_structure.pdb
**Explanation:** Runs all available structure checks on the input PDB file and writes the (optionally fixed) structure to the output path.

### Check for missing atoms only without making changes
**Args:** --input-structure-path protein.pdb --check-only missing_atoms --output-path protein_checked.pdb
**Explanation:** Validates only whether atoms are missing in the structure without applying any automatic corrections to keep the output unchanged.

### Fix chain breaks in a structure
**Args:** --input-structure-path broken_chain.pdb --fix-only chain_break --output-path fixed_chain.pdb
**Explanation:** Detects discontinuities in protein chains and applies automatic gap-filling or linkage corrections to resolve chain break issues.

### Remove alternate location indicators (altloc)
**Args:** --input-structure-path altloc_structure.pdb --fix-only altloc --output-path no_altloc.pdb
**Explanation:** Removes alternate location identifiers from the structure, keeping only the first occurrence for each atom with multiple positions.

### Check for water molecules and optionally remove them
**Args:** --input-structure-path with_waters.pdb --check-only water --output-path no_waters.pdb
**Explanation:** Identifies water molecules (HOH, WAT, H2O) present in the structure and can remove them when using the appropriate fix option.

### Validate structure and output results in JSON format
**Args:** --input-structure-path complex.pdb --output-path validated.pdb --output-json results.json
**Explanation:** Runs structural validation and saves both the processed structure and a JSON report containing detailed findings from all checks performed.

### Apply multiple fixes simultaneously using non-interactive mode
**Args:** --input-structure-path messy.pdb --fix-only missing_atoms --fix-only water --non-interactive --output-path clean.pdb
**Explanation:** Combines multiple fix operations (adding missing atoms and removing waters) in a single run without prompting for user confirmation.