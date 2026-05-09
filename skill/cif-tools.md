---
name: cif-tools
category: Structure Format Conversion
description: A suite of tools for converting between macromolecular Crystallographic Information File (mmCIF) and PDB formats, and for validating and manipulating macromolecular structural data. Provides subcommands for format conversion, structure extraction, and bulk processing of structural entries.
tags:
  - mmCIF
  - PDB format
  - structure conversion
  - crystallography
  - macromolecular structure
  - PDBx/mmCIF
  - structural biology
author: AI-Generated
source_url: https://github.com/ihmdev/cif-tools
---

## Concepts

- **Input/Output Formats**: The primary data model is macromolecular CIF (mmCIF), also known as PDBx/mmCIF, which is the IUCr/HMDB standard format for depositing and distributing macromolecular structures. Output options include legacy PDB format, mmCIF remapping, and JSON representations. Conversions preserve atom names, residues, and chain identifiers wherever possible.
- **Multi-Model and Multi-Chain Handling**: mmCIF files from NMR experiments or crystal structures with multiple models are fully supported. Individual chains or asymmetric units can be selected using `--chain` and `--asym` flags, and the `--model` flag limits output to specific models in multi-model files.
- **Companion Binaries**: The suite includes `cif2pdb`, `cif2pdb-gap`, `cif-ddl`, and `cif-rna-view` as separate executables. The `cif2pdb` binary performs the core CIF-to-PDB conversion, while `cif-ddl` handles Data Dictionary validation. Each companion binary has its own argument interface and accepts input via `--input` or positional arguments.
- **ID Mapping and Remapping**: Structures deposited in the PDB exist as both mmCIF and legacy PDB files. The `--remap` flag produces remapped mmCIF files from PDB-format input by regenerating equivalent mmCIF using the structural coordinates and remapped identifiers. Chain IDs in PDB format use single alphabetic characters, which differ from mmCIF chain labels.
- **Coordinate Precision and Insertion Codes**: mmCIF stores atomic coordinates with full precision (decimal, up to 4 decimal places). PDB format truncates coordinates to 3 decimal places. Insertion codes in PDB format (column 27) map to the `_struct_conf.conf_extension_id` and `_struct_asym_gen.record_id` fields in mmCIF.

## Pitfalls

- **Format Extension Mismatch**: mmCIF supports over 4,000 data categories, but the legacy 3-column PDB format can represent only a subset. Features like hydrogen atoms on multiple residues, multi-conformer side chains, and anisotropic B-factors may be silently dropped during CIF-to-PDB conversion. Consequence: a structurally important water molecule or alternative location is absent from the output PDB without warning.
- **Incorrect Chain and Asym Unit Selection**: Using `--chain` with a PDB-format chain ID (e.g., `A`) when the input mmCIF file uses a non-pdbx chain label (e.g., `BA` in the `_struct_asym` table) selects the wrong chain or none at all. Consequence: the output file is empty or contains the wrong polypeptide.
- **Decimal Truncation in PDB Output**: mmCIF coordinates (e.g., `38.12345`) are rounded to 3 decimal places in PDB format, causing sub-0.001 Å drift per atom. For high-resolution structures used in structure refinement, this accumulated drift can degrade model quality. Consequence: refined structures may not pass validation after re-conversion.
- **Ambiguous Input File Type**: cif-tools auto-detects file format by extension, but mmCIF and PDB files both use `.cif` extensions in some repositories. Consequence: the wrong parser is used, producing invalid output or errors like `Error: Unexpected END` or `Error: Category '_atom_site' not found`.
- **Missing `--input` Flag**: Some subcommands (notably `cif-ddl`) require the `--input` flag explicitly. Passing the file path as a bare positional argument causes a parse error or silent failure. Consequence: the operation completes without processing any data.

## Examples

### Convert an mmCIF file to legacy PDB format
**Args:** `--input 1abc.cif --output 1abc.pdb`
**Explanation:** The `--input` flag specifies the mmCIF source file, and `--output` redirects the PDB-format result to a new file. If the structure exceeds 99,999 ATOM records (the PDB format limit), only the first 99,999 are written and a truncation warning is emitted.

### Extract a single chain from an mmCIF file
**Args:** `--input 1abc.cif --output chain_B.pdb --asym B`
**Explanation:** The `--asym` flag selects the asymmetric unit labeled `B` from the input mmCIF `_struct_asym` table. This is the correct field for chain selection from mmCIF, not `--chain`, which applies a different filter.

### Download and convert a full PDB entry by ID
**Args:** `--input 1abc --output 1abc.pdb`
**Explanation:** When a bare 4-character PDB ID is provided as the input argument without a file extension, cif-tools resolves it to the mmCIF archive at the RCSB PDB and converts it to PDB format on the fly. This eliminates the need for a prior download step.

### Convert a multi-model NMR mmCIF file to PDB, limiting to model 3
**Args:** `--input 1abc.cif --output model3.pdb --model 3`
**Explanation:** The `--model` flag selects only the third structural model from a multi-model NMR ensemble. Each model in mmCIF is assigned a sequential `_atom_site.model_id`. Without this flag, all models are concatenated in PDB output, which many downstream tools cannot handle correctly.

### Batch-convert multiple mmCIF files to PDB format
**Args:** `--input dir/*.cif --output dir/converted/`
**Explanation:** When the `--input` argument contains a glob pattern, cif-tools iterates over every matching mmCIF file and converts each one individually, preserving the base filename with a `.pdb` extension in the output directory. The output directory must exist beforehand.

### Remap a legacy PDB file back to mmCIF format
**Args:** `--input 1abc.pdb --output 1abc_remapped.cif --remap`
**Explanation:** The `--remap` flag instructs cif-tools to parse a PDB-format file and regenerate an equivalent mmCIF representation. This is the inverse of the standard conversion and is useful for standardizing legacy PDB deposits into the modern mmCIF format before further processing.