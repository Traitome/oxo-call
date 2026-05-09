---
name: afpdb
category: protein-structure-analysis
description: A Python library and command-line tool for parsing, extracting, and manipulating AlphaFold PDB output files. Provides chain filtering, residue range selection, B-factor analysis, and structural data export utilities.
tags:
  - protein-structure
  - alphafold
  - pdb
  - structural-biology
  - colabfold
  - python
author: AI-generated
source_url: https://github.com/YoshitakaMo/afpdb
---

## Concepts

- **PDB format parsing:** `afpdb` reads standard PDB files produced by AlphaFold2 or ColabFold, handling the `ATOM` and `HETATM` record types. It constructs an in-memory model of chains, residues, and atoms, enabling programmatic access to atomic coordinates, B-factors, and per-residue plDDT scores stored in the B-factor column.

- **Chain and residue filtering:** You can extract specific chains using `--chain` flags and restrict to residue ranges via `--resnum` (accepting PDB residue numbering, not model indices). Filters are applied lazily before any output is written, so downstream tools receive only the requested subset without intermediate file I/O.

- **B-factor / plDDT reinterpretation:** In AlphaFold PDBs, the B-factor column stores the per-residue confidence score (plDDT, 0–100). `afpdb` can report these values, color structures by plDDT, or replace B-factors with custom values for visualization in PyMOL, ChimeraX, or other viewers.

- **Output formats:** The tool writes filtered PDBs to stdout by default, making it composable in Unix pipes. It also supports JSON export for downstream Python scripts and CSV export for spreadsheet analysis of residue-level metrics.

- **Python API:** Beyond the CLI, `afpdb` exposes a Python module (`import afpdb`) allowing scripts to load a PDB, iterate over chains/residues/atoms, and apply filters programmatically. This is useful for batch-processing pipelines or embedding into Jupyter notebooks.

## Pitfalls

- **Residue numbering mismatch:** Using `--resnum` with model numbering (e.g., sequential index) instead of PDB/ICL residue labels causes silent zero-output or truncated files. Always verify residue IDs by first running `--report` or `--dump-resnums` to confirm the correct numbering scheme.

- **No B-factor plDDT preservation on modification:** When using `--replace-bfac` or `--set-bfac`, the original plDDT scores are permanently overwritten in the output PDB. If you later need plDDT for confidence analysis, save an unmodified copy before applying B-factor modifications.

- **Missing chains in output:** If the requested chain identifier does not exist in the PDB (common with monomer AlphaFold jobs that produce a single-chain file), `afpdb` exits with code 0 but writes an empty file. Always check output file size or use `--require-chain` to raise an error instead.

- **Inconsistent handling of alternate location atoms:** AlphaFold PDBs rarely contain alternate location (`ALT`) flags, but if present, `afpdb` may only retain the first occurrence, silently discarding others. Verify with a structural viewer if your analysis depends on alternate conformations.

- **Python version and dependency conflicts:** `afpdb` requires Python 3.8+. Installing via pip into a shared conda environment can cause dependency conflicts with NumPy or Biopython versions required by other tools. Use a virtual environment or `--user` pip install to isolate.

## Examples

### Print residue-level plDDT scores for a single chain

**Args:** `input AF2_result.pdb --report --chain A --score`
**Explanation:** Reads the AlphaFold PDB, extracts all plDDT scores (stored in the B-factor column) for chain A, and prints a residue-number–versus–score table to stdout for quality assessment.

---

### Extract chains B and D into a new PDB file

**Args:** `input AF2_multimer.pdb --chain B --chain D --output filtered.pdb`
**Explanation:** Filters the input PDB to retain only chains B and D, writing the result to `filtered.pdb`. Useful for separating multimeric AlphaFold predictions into individual chain files for downstream docking or analysis.

---

### Export per-residue plDDT as JSON for a Jupyter notebook

**Args:** `input AF2_result.pdb --json --output scores.json`
**Explanation:** Dumps the full plDDT profile as a JSON file with residue numbers, chain IDs, and confidence scores. This JSON can be imported directly into Python or R scripts for plotting or statistical analysis.

---

### Replace B-factors with CA atom distances to a reference chain

**Args:** `input AF2_result.pdb --set-bfac --chain A --output colored.pdb`
**Explanation:** Recomputes the B-factor column using CA distances (or a configured metric) for chain A and writes the modified PDB. The output can be loaded into ChimeraX or PyMOL to color by the new metric instead of plDDT.

---

### Validate chain existence and report summary without writing a PDB

**Args:** `input AF2_result.pdb --dump-resnums --chain A`
**Explanation:** Prints all residue numbers present in chain A without modifying or writing any file. Use this to confirm exact residue labeling before applying `--resnum` filters in a subsequent command.

---

### Filter to a specific residue range within chain A

**Args:** `input AF2_result.pdb --chain A --resnum 10-50 --output domain.pdb`
**Explanation:** Extracts residues with PDB residue numbers 10 through 50 from chain A into `domain.pdb`. This is commonly used to isolate a specific domain or structured region for separate analysis or visualization.