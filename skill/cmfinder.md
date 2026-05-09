---
name: cmfinder
category: RNA Structure Analysis
description: Tool for searching sequence databases against RNA covariance models (CMs) to identify structural RNA homologs. Part of the Infernal toolkit for RNA analysis.
tags:
  - RNA
  - covariance model
  - infernal
  - homology search
  - ncRNA
  - structural RNA
author: AI-generated
source_url: http://infernal.org/
---

## Concepts

- **Covariance Models (CMs)**: Statistical models representing the conserved secondary structure and sequence of RNA families. Each position in a CM can account for compensatory mutations that maintain base-pairing in RNA stems.
- **Input Requirements**: cmfinder requires two primary inputs - a query CM database (built by cmbuild) and a target sequence database (FASTA format). The sequence database should contain the sequences to search against the models.
- **Output Formats**: Results are typically reported as alignments showing the predicted RNA structure with column annotations indicating base-pairing consistency and statistical significance (E-value).
- **Scoring System**: Uses a forward-backward algorithm to compute log-odds scores. Higher scores indicate better matches; negative scores suggest the match is worse than expected by chance.
- **Database indexing**: Before searching large databases, CM databases must be indexed using cmbuild to create efficient lookup structures. The index file significantly accelerates searches.

## Pitfalls

- **Unsorted or unindexed CM databases**: Searching against CM databases that haven't been built with cmbuild produces incorrect results or errors. Always ensure the CM database was created with proper indexing.
- **Sequence format issues**: Using sequences with ambiguous characters (N, X) or without proper FASTA formatting leads to skipped sequences or parsing errors. Clean and validate input sequences before searching.
- **Overly permissive E-value thresholds**: Setting E-value cutoffs too high (e.g., 1.0 or above) returns many false positives, polluting downstream analysis with spurious hits that don't represent real RNA homologs.
- **Searching protein sequences against DNA-trained CMs**: DNA-trained covariance models will produce meaningless scores when searched against protein sequences. Match the molecule type (DNA/RNA) between queries and targets.
- **Ignoring the tabular output for parsing**: Pipeline scripts that parse stdout text formatting are fragile. Use explicit output format flags (--tabfile) for machine-readable results.

## Examples

### Search a sequence database against a CM database
**Args:** --notrunc -o cmfinder_output.cm /path/to/cmdb /path/to/seqdb.fasta
**Explanation:** This runs a complete search of all sequences in seqdb.fasta against all models in cmdb, writing full output to the specified file.

### Run a quick search with tabular output for downstream parsing
**Args:** --tblout results.tbl mymodels.cm input.fasta
**Explanation:** Produces a concise tabular output file suitable for parsing by scripts, containing hit coordinates and scores.

### Search with strict E-value threshold to reduce false positives
**Args:** -E 0.001 myrna.cm sequences.fasta
**Explanation:** Only reports hits with E-value below 0.001, significantly reducing the number of spurious matches in large database searches.

### Search a single sequence against multiple CMs with verbose reporting
**Args:** -v --noali myrna.cm one_sequence.fasta
**Explanation:** Prints verbose progress information but suppresses the full alignment output, useful for quick presence/absence detection.

### Search using DNA alphabet models against DNA sequences
**Args:** --dna -T 50 mydna.cm dna_seqs.fasta
**Explanation:** Explicitly uses DNA alphabet scoring (not RNA) and requires minimum bit score of 50, appropriate when searching for DNA elements.

### Search protein sequences with protein-trained covariance models
**Args:** --rna -E 0.01 protein_cms.cm protein_seqs.fasta
**Explanation:** Uses RNA-trained models on protein sequence input (valid for protein-CMs built with --tprotein) with moderate E-value cutoff.