---
name: biobb_io
category: molecular_dynamics_io
description: BioExcel Building Blocks I/O module for读写molecular structures and trajectories in MD workflows. Converts between formats (PDB, GRO, PSF, TOP, XTC, NC), handles AMBER/GROMACS file types, and manages workflow data staging.
tags:
  - molecular-dynamics
  - file-conversion
  - trajectory-analysis
  - gromacs
  - amber
  - pdb
  - gro
  - bioexcel
  - computational-chemistry
  - biopython
author: AI-generated
source_url: https://github.com/bioexcel/biobb_io
---

## Concepts

- **Multi-format structure I/O**: `biobb_io` reads and writes molecular structure files including PDB, GRO, AMBER TOP/RPR/CRD, GROMACS NDX, and CHARMM PSF/NAMD. File type is auto-detected from extension or explicitly set via `file_type` arguments; mismatched extensions trigger errors unless the correct format code is passed (e.g., `pdb` for `.ent` files).

- **Trajectory format handling**: Trajectory I/O supports GROMACS XTC/TRR/EDR, AMBER NC/NetCDF, and PDB/XTC-only readers. Coordinate-only formats (GRO, PDB) cannot carry velocity or force data; using them as input for velocity-dependent analyses silently drops those quantities.

- **Output file overwrite behaviour**: By default, `biobb_io` refuses to overwrite existing output files and raises an error. Setting `remove_tmp` or the equivalent flag is required to allow automatic cleanup of temporary staging files; failure to set this in pipeline loops causes disk-space accumulation in workflow temporary directories.

- **Data model compatibility**: `biobb_io` is built atop `MDAnalysis`, `MDTraj`, and `BioSimSpace` backends. Objects returned are dictionary-like containers with keys `output_tmp_path`, `properties`, and `binary` content; accessing raw NumPy arrays requires unwrapping the `binary` key rather than treating the return value as a plain file path.

## Pitfalls

- **Mismatched box information**: GRO format encodes box vectors on the last line of each frame; PDB does not. Converting a GRO trajectory to PDB without explicitly passing `box_vectors` properties results in simulations that lose box/periodic boundary data, breaking later MD runs.

- **Wrong AMBER format subtype**: AMBER topology files have distinct subtypes — TOP (standard), RPR (restrained), and CRD (coordinate). Passing a CRD file where a TOP is expected produces an error but using the wrong subtype in a pipeline silently causes parameter mismatches in the downstream MD engine.

- **Uncompressed vs compressed trajectory files**: XTC files may be gzip-compressed on disk. `biobb_io` readers detect compression by magic bytes; passing a `.xtc` extension on a file whose content is uncompressed raw binary results in truncated reads with no error message, just silently wrong trajectory data.

- **Large trajectory memory consumption**: Loading full XTC or TRR trajectories into memory via `biobb_io` without specifying `start`/`end`/`step` frame slices causes OOM kills on HPC nodes with limited RAM. Always set explicit frame ranges for large files (millions of frames).

- **Chiral centre / residue naming mismatches**: Converting between PDB and GRO renames residues using different conventions (PDB uses 3-letter codes, GRO uses 3-letter but truncated). Downstream GROMACS `gmx pdb2gmx` failures frequently trace to residue names changed by a prior `biobb_io` conversion step.

## Examples

### Convert a GRO structure file to PDB format
**Args:** `--input_gro_path structure.gro --output_pdb_path structure.pdb`
**Explanation:** Reads a GRO molecular structure file and writes an equivalent PDB-formatted output, preserving atom names and residue identifiers while reformatting coordinate precision.

### Extract frames 1–100 from a GROMACS XTC trajectory
**Args:** `--input_xtc_path md_run.xtc --output_xtc_path md_subset.xtc --initial_skip 0 --last_frame 100`
**Explanation:** Slices an XTC trajectory to frames 1–100 inclusive, reducing file size for downstream analysis without re-compressing the entire original file.

### Convert an AMBER topology-plus-coordinate to GRO format
**Args:** `--input_topology topology.top --input_coordinates coordinates.crd --output_gro_path output.gro`
**Explanation:** Combines an AMBER topology and coordinate file to produce a single GRO structure, used when bridging AMBER preparation steps into a GROMACS production pipeline.

### Read a PDB file and write a CHARMM PSF plus NAMD-compatible coordinate file
**Args:** `--input_pdb_path protein.pdb --output_psf_path protein.psf --output_crd_path protein.crd`
**Explanation:** Converts a PDB structure to CHARMM PSF (topology) and CRD (coordinate) files required by NAMD, preserving bond information extracted from the PDB atom connectivity.

### Concatenate two XTC trajectory files into a single output
**Args:** `--input_xtc_path md_part1.xtc --input_xtc_path_aux md_part2.xtc --output_xtc_path md_combined.xtc`
**Explanation:** Appends the second XTC trajectory to the first in chronological order, creating a continuous trajectory for post-processing without requiring `gmx trjcat`.

### Write a GRO trajectory while preserving periodic box vectors from an EDR file
**Args:** `--input_gro_path md.gro --input_edr_path md.edr --output_gro_path md_box.gro`
**Explanation:** Reads box vector information from an energy file (EDR) alongside GRO coordinates so that the output preserves periodic boundary conditions lost in GRO-only conversion pipelines.

### Extract atom indices from a GROMACS NDX index file
**Args:** `--input_ndx_path index.ndx --output_ndx_path protein_atoms.ndx --group_names Protein`
**Explanation:** Filters a GROMACS index file to retain only the named group(s), reducing index file size and simplifying group selection in subsequent `gmx make_ndx` or `biobb_io` processing steps.

---