---
name: cmip
category: Protein Structure Analysis
description: A tool for analyzing conformational motifs and protein interface properties from molecular structures, supporting PDB input processing and interaction profile generation.
tags:
  - protein-structure
  - molecular-dynamics
  - interface-analysis
  - structural-biology
  - residue-profiling
author: AI-generated
source_url: https://github.com/cmip/cmip
---

## Concepts

- **Input Format**: CMIP accepts standard PDB files (`.pdb`) containing atomic coordinate data. Multi-chain PDBs are processed chain-by-chain, and the tool automatically detects residue numbering schemes from SEQRES records versus atomic records when present.
- **Output Data Model**: The tool generates per-residue interaction profiles (`.rip` files) containing fields for residue index, chain ID, secondary structure assignment, solvent accessibility values, and interaction partner counts. These profiles can be aggregated across multiple structures for statistical analysis.
- **Interface Detection Threshold**: CMIP uses a distance-based cutoff (default 5.0 Å between any atom pair) to classify residues as interface or non-interface. This threshold is configurable via the `--cutoff` flag and affects all downstream statistical calculations including interface composition percentages.
- **Companion Binary**: The `cmip-build` companion binary constructs reference profile databases from training sets of known structures. These databases are required for classification mode and must be indexed using the same residue numbering scheme as query structures.

## Pitfalls

- **Mismatched Residue Numbering**: Running classification on structures that use different residue numbering schemes than the reference database will produce nonsensical output scores (typically all zeros or random values) because residue indices cannot be matched correctly.
- **Ignoring Missing Electron Density**: CMIP processes all coordinates present in the PDB file without checking B-factor quality. Including structures with high B-factors (>40 Å²) in interface analysis leads to unreliable residue assignments since atomic positions may be poorly resolved.
- **Omitting the `-p` Flag for Multi-chain PDBs**: For PDB files containing multiple chains, forgetting the `-p` (process-all) flag causes CMIP to process only the first chain encountered, silently discarding interface data from remaining chains.
- **Insufficient Training Data for `cmip-build`**: Building a reference database with fewer than 50 structures produces unstable classification models with high false-positive rates in interface prediction, particularly for rare secondary structure types.

## Examples

### Calculate interface residues from a single PDB structure
**Args:** `input 1K5N.pdb --mode interface --cutoff 4.5`
**Explanation:** This command identifies residues forming the protein-protein interface using a tighter 4.5 Å cutoff than the default, writing results to `1K5N_interface.rip`.

### Generate interaction profiles for a multi-chain complex
**Args:** `input 2HR1.pdb -p --output-dir ./profiles --format json`
**Explanation:** This processes all chains in the viral capsid structure and writes per-chain interaction profiles in JSON format to the specified output directory for downstream bioinformatics workflows.

### Compare interface composition between two related structures
**Args:** `compare 3V1O.rip 4HHB.rip --metric composition --stats ttest`
**Explanation:** This performs a two-sample t-test comparing the amino acid composition of interface residues between the reference and mutant structures to identify statistically significant changes.

### Build a reference database from a training set for classification
**Args:** `cmip-build training_set/*.pdb --out reference_db.rib --min-structures 75`
**Explanation:** This constructs a reference interaction profile database from at least 75 labeled structures, which is required for subsequent classification analysis and improves statistical reliability.

### Batch process multiple PDB files and aggregate statistics
**Args:** `input ./structures/*.pdb --batch --aggregate --out aggregate_stats.csv`
**Explanation:** This processes all PDB files in the directory and aggregates interface statistics across the entire dataset, producing a single CSV file with mean and standard deviation values per residue position.