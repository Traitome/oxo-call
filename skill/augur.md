---
name: augur
category: phylogenetics
description: Command-line toolkit for phylogenetic analysis of pathogen genomes, part of the Nextstrain project. Performs sequence alignment, phylogenetic tree building, trait annotation, clade assignment, and mutation calling for pathogen surveillance and evolutionary studies.
tags: [phylogenetics, pathogen-genomics, nextstrain, sequence-alignment, tree-building, variant-calling, genomic-epidemiology]
author: AI-generated
source_url: https://github.com/nextstrain/augur
---

## Concepts

- **Alignment Subcommand**: The `align` command aligns pathogen sequences to a reference genome using MAFFT. It accepts FASTA input (both sequences and reference), outputs a FASTA alignment, and requires specifying the reference sequence via `--reference`. For insertions relative to the reference, use `--remove-outliers` to exclude sequences with excessive gaps.

- **Tree Building**: The `tree` command constructs phylogenetic trees from alignments using IQ-TREE (default) or RAxML. Provide the alignment via `--alignment`, set the output via `--output-tree`, and select the substitution model via `--model` (e.g., GTR for general time-reversible). The tool automatically identifies the best tree topology and bootstrap support values.

- **Metadata Integration**: Augur uses tab-delimited or comma-delimited metadata files (TSV/CSV) containing strain identifiers and trait columns (e.g., date, country, host). The `traits` subcommand maps metadata to tree nodes via `--metadata` and specifies trait columns with `--columns`. Strains in the metadata must exactly match tree tip labels.

- **Clade Assignment**: The `clades` command assigns clades based on predefined clade definitions stored in a JSON file. Provide the tree via `--tree`, alignment via `--alignment`, and clade definitions via `--clade-definitions`. Output is a node-data JSON with clade annotations for each tree node.

- **Export to JSON**: The `export` command generates the JSON format required for visualization in Auspice. It combines alignment, tree, and metadata into a single node-data JSON with geographic coordinates, timestamps, and mutation annotations. Requires specifying output via `--output`, tree via `--tree`, and alignment via `--alignment`.

## Pitfalls

- **Strain Name Mismatch**: If strain names in the metadata file do not exactly match the sequence headers in the FASTA alignment, the traits command will silently fail to annotate those sequences. Always verify that all tip labels in the tree have corresponding entries in the metadata file before running trait annotation.

- **Invalid Date Formats**: Augur requires dates in ISO 8601 format (YYYY-MM-DD) for temporal analysis. Dates in non-standard formats (e.g., MM/DD/YYYY or YYYY) will cause the `refine` command to fail during timetree estimation or produce incorrect branch lengths, leading to unreliable evolutionary conclusions.

- **Reference Sequence Errors**: Using a reference sequence that is significantly shorter than your input sequences or contains ambiguous bases (N) will cause the align subcommand to generate misaligned output. Always ensure the reference is a complete, high-quality sequence without ambiguous bases.

- **Missing Required Columns**: When running the `traits` command without specifying the `--columns` argument, augur cannot identify which metadata columns to use for annotation. This results in an empty output file with no trait data, making downstream export impossible.

- **Insufficient Sequence Diversity**: Building phylogenetic trees with alignments containing identical ornearly identical sequences (e.g., fewer than 3 variable sites) causes IQ-TREE to fail or produce unresolved trees. Always inspect alignments using `snp-dist` or similar tools before tree building.

## Examples

### Align sequences to a reference genome

**Args:** `align --sequences sequences.fasta --reference reference.fasta --output aligned.fasta --remove-outliers`

**Explanation:** This aligns input pathogen sequences to the reference genome using MAFFT, removes outliers with excessive insertions relative to the reference, and outputs a cleaned FASTA alignment for downstream phylogenetic analysis.

### Build a phylogenetic tree from an alignment

**Args:** `tree --alignment aligned.fasta --output-tree tree.nwk --model GTR`

**Explanation:** This constructs a maximum-likelihood phylogenetic tree from the alignment using IQ-TREE with the GTR substitution model, outputting a Newick-format tree file for tree visualization and further annotation.

### Annotate traits from metadata onto the tree

**Args:** `traits --tree tree.nwk --metadata metadata.tsv --columns date country --output node-data.json`

**Explanation:** This maps the date and country columns from the metadata file to tree nodes, creating a JSON file with trait annotations for each node that can be used in Auspice visualization.

### Assign clades to nodes based on clade definitions

**Args:** `clades --tree tree.nwk --alignment aligned.fasta --clade-definitions clades.json --output clades.json`

**Explanation:** This assigns pre-defined clades to tree nodes based on their mutation patterns as defined in the clade definitions file, outputting an annotated JSON for downstream export.

### Export combined data for Auspice visualization

**Args:** `export --tree tree.nwk --alignment aligned.fasta --metadata metadata.tsv --node-data node-data.json clades.json --outputauspice.json --lat-lon lat_lon.tsv`

**Explanation:** This combines the tree, alignment, metadata, and supplementary node annotations into a single JSON file readable by Auspice, including geographic coordinates for geographic visualization.

### Refine a tree with timetree estimation

**Args:** `refine --tree tree.nwk --alignment aligned.fasta --metadata metadata.tsv --timetree --clock-rate 0.001 --output-tree refined-tree.nwk --output-node-data refined-node-data.json`

**Explanation:** This refines the tree by rooting at the optimal location, estimating a timetree using a molecular clock rate of 0.001 substitutions per site per year, and outputting branch lengths in time units.

### Validate an alignment for analysis readiness

**Args:** `validate --alignment aligned.fasta --reference reference.fasta`

**Explanation:** This checks the alignment for validity by verifying that all sequences are the same length, no premature stop codons are present (for protein-coding sequences), and the alignment is compatible with the reference.