---
name: cath-tools
category: Protein Structure Analysis
description: A suite of tools for classifying protein domains using the CATH (Class, Architecture, Topology, Homologous superfamily) database. Includes utilities for building CATH databases, scanning structures, resolving hits, and generating structural classifications.
tags: [protein-structure, domain-classification, CATH, structural-bioinformatics, HMM, protein-domains]
author: AI-generated
source_url: https://github.com/UCLOrengoGroup/cath-tools
---

## Concepts

- **CATH Hierarchy**: Proteins are classified into four levels: Class (secondary structure composition), Architecture (spatial arrangement of secondary structures), Topology (connectivity of secondary structures), and Homologous superfamily (evolutionary relationship). A typical classification like `1.10.10.10` indicates Class 1, Architecture 10, Topology 10, Homologous superfamily 10.

- **Input Formats**: cath-tools accepts multiple input formats including PDB files (atomic coordinates), FASTA sequences, HMM profiles, and CATH's binary database files. For sequence-based classification, input is compared against pre-built HMMs representing CATH domains. For structure-based classification, PDB files are scanned against the CATH structure database.

- **Domain Assignment**: The tool identifies which CATH domains a query protein contains by aligning against CATH HMMs or scanning against CATH structural representatives. Each domain receives a CATH classification along with alignment scores and e-values indicating confidence. Multi-domain proteins receive multiple classifications.

- **Companion Binaries**: cath-tools provides several companion binaries—cath-scan (scans structures against CATH), cath-resolve-hits (converts HMM hits to domain definitions), cath-get-predictions (generates classifications), cath-cluster (clusters structures), and cath-tools-build (creates CATH database files).

- **Output Representations**: Results are returned as text with domain classifications, alignment boundaries (start-end residues), bitscores, e-values, and confidence estimates. Output can include structural superimpositions in PDB format for visualization.

## Pitfalls

- **Missing Database Files**: Running cath-tools without downloading or building the CATH database files results in errors. The tool requires the full CATH data directory (containing HMMs, PDB chains, and definitions) to be present and accessible via the CATH_TOOLS_DB environment variable.

- **Incorrect Sequence Input**: Using low-quality or incomplete sequences leads to misclassification. Sequences with missing residues, unknown amino acids (X characters), or contamination produce unreliable domain assignments. Always validate sequences with tools like Biopython before submission.

- **Confusing Scan vs. Resolve**: Running cath-scan without subsequent cath-resolve-hits produces raw alignments rather than definitive domain boundaries. The scan step identifies similarities, but resolve-hits converts these to concrete domain Definitions (S35, S60, S95, S100) with precise cutoffs.

- **Insufficient Memory for Large Scans**: Scanning large PDB files or many sequences against the full CATH database requires significant memory. Processing whole proteomes or large protein complexes can exhaust RAM, causing crashes or extreme slowdowns. Consider batch processing or using representative subsets.

- **File Path Errors**: Pointing to non-existent PDB files, incorrect working directories, or misconfigured CATH_TOOLS_DB paths produces cryptic errors. Always verify file existence with ls and confirm environment variables are set correctly before running analysis.

## Examples

### Scan a PDB file against the CATH structure database
**Args:** `-i 1aoi.pdb scan`
**Explanation:** The `-i` flag specifies the input PDB file, and the `scan` subcommand compares the structure against CATH's library of domain representatives to find structural neighbors and their classifications.

### Classify a protein sequence using HMMs
**Args:** `-i protein.fasta get-fun-of-hits`
**Explanation:** Submits a FASTA sequence to be searched against CATH's HMM profiles, returning which CATH domains the sequence potentially contains along with alignment statistics.

### Resolve raw HMM hits to definitive domain boundaries
**Args:** `-i hits.raw resolve-hits`
**Explanation:** Takes output from a previous HMM search (hits.raw) and applies CATH's domain definition cutoffs (S35, S60, S95, S100) to convert fuzzy alignments into precise domain boundaries with confidence levels.

### Generate predictions with specific bit-score thresholds
**Args:** `-i protein.fasta get-predictions --bit-score 80`
**Explanation:** Produces domain classifications for a sequence using an 80-bit score threshold, filtering out weaker matches to produce higher-confidence predictions with fewer false positives.

### Build a custom CATH database from structures
**Args:** `-i my_structures/ build`
**Explanation:** The `build` subcommand processes a directory of PDB files to create custom CATH binary files, HMMs, and domain definitions for use in subsequent analyses.

### Cluster structures by CATH classification
**Args:** `-i proteins.txt cluster --cutoff 95`
**Explanation:** Clusters input protein sequences or structures by their CATH topology (T-level), grouping proteins that share at least 95% identical domain architectures and classifications.

### Run scan with JSON output for programmatic parsing
**Args:** `-i 1abc.pdb scan --out json_out.json`
**Explanation:** Outputs results in JSON format instead of plain text, facilitating integration with automated pipelines and downstream bioinformatics workflows that require structured data.