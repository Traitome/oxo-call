---
name: biobb_model
category: Structure Modeling / Protein Structure Prediction
description: A Python library from the BioBB (BioExcel Building Blocks) suite for building and refining 3D molecular structures, including homology modeling, hydrogen addition, peptide building, and energy minimization workflows.
tags:
  - structure-building
  - homology-modeling
  - protein-model
  - molecular-dynamics
  - 3d-structure
  - loop-modeling
  - biobb
  - bioinformatics
author: AI-Generated
source_url: https://biobb-model.readthedocs.io/
---

## Concepts

- `biobb_model` constructs 3D protein structures from sequence or template alignment data, supporting homology modeling, de novo peptide building, and model refinement. Input sequences must be provided in standard single-letter FASTA format (one header line starting with `>` followed by the sequence on subsequent lines), and any template structures must be in valid PDB format with properly numbered residues.
- The primary workflow (`build_model`) relies on template structure selection and alignment quality: higher sequence identity between target and template directly improves model accuracy, and the alignment file (typically Clustal Omega or MAFFT output in `.aln` format) must have no terminal gaps in conserved core regions or the resulting model will have truncated backbone segments.
- Output structures are written in PDB format withATOM/HETATM records; downstream tools like `biobb_amin` (energy minimization) or MD engines expect specific chain IDs, correct residue numbering, and complete backbone/sidechain coordinates — any missing atoms in the output will cause failures in subsequent modeling or dynamics steps.
- `biobb_model` is orchestrated via Python class wrappers (not direct CLI flags), so workflow scripts configure IO paths and method-specific parameters (e.g., `template_pdb`, `offset`, `inherent_bs`) through YAML or JSON configuration files passed with `--config`.
- Energy minimization with the `model_minimization` module uses BioBB's internal force-field wrappers; the convergence threshold (`ftolerance`) and maximum iterations (`maxit`) must be tuned for structure size — small values may terminate before convergence for large proteins, while excessively large values waste compute time.

## Pitfalls

- Providing a FASTA sequence with non-standard amino acid codes (e.g., `X`, `B`, `Z`) or lowercase letters without specifying `ignore_warnings=True` causes the builder to reject the input silently and produce an empty output PDB file, breaking any pipeline that depends on it.
- Using a template PDB with multiple chains or heteroatoms (ligands, waters) without filtering them via the `select` parameter results in a hybrid model that inherits unwanted chains — this distorts downstream docking or MD simulations that expect a single protein chain.
- Misaligned target-template sequences — for example, alignment files with large gaps in helix-forming regions — produce models with distorted secondary structure; the `build_model` tool does not validate alignment plausibility and will proceed regardless, generating physically unrealistic backbone conformations.
- Setting `maxit` too low in minimization workflows leads to structures that are not relaxed, retaining high internal strain; this manifests as abnormally short vdw contacts that cause crashes in subsequent MD equilibration phases.
- Running `build_model` without specifying `loop_refine=True` on targets with insertions/deletions relative to the template leaves unmodeled loop regions as sparse or absent coordinates — these gaps cause Modeler or GROMACS to fail with "atoms not found" errors.

## Examples

### Build a 3D protein model from a single FASTA sequence
**Args:** `--config config_homology.yaml`
**Explanation:** Specifying a YAML configuration file triggers the homology modeling workflow, which reads the target FASTA sequence and template PDB path from `config_homology.yaml` and writes the predicted 3D model to the configured output path.

### Build a peptide model from multiple aligned fragments
**Args:** `--config config_peptide.yaml`
**Explanation:** The peptide building module reads fragment coordinates and connectivity from `config_peptide.yaml`; the fragments must have consistent chain IDs and sequential residue numbering to be joined into a continuous peptide chain.

### Add missing hydrogen atoms to an existing PDB structure
**Args:** `--config config_addh.yaml`
**Explanation:** The hydrogen addition module in `config_addh.yaml` reads an experimental PDB structure (from X-ray or cryo-EM) that lacks hydrogens and adds them at physiological pH, producing a complete all-atom model suitable for electrostatics calculations.

### Refine a coarse-grained model with loop rebuilding
**Args:** `--config config_refine.yaml`
**Explanation:** Setting `loop_refine: true` in `config_refine.yaml` activates the loop modeling algorithm for regions where the target-template alignment contains gaps; without this flag, insertion-bearing loops are left unmodeled in the output.

### Minimize a刚刚 built模型的结构能
**Args:** `--config config_minimize.yaml`
**Explanation:** The energy minimization configuration in `config_minimize.yaml` sets convergence criteria (`ftolerance: 0.01`) and maximum iterations (`maxit: 500`) so that the model is relaxed enough for MD equilibration without excessive runtime.