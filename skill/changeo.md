---
name: changeo
category: Immunology / Immune Repertoire Analysis
description: A suite of tools for processing AIRR-seq (Adaptive Immune Receptor Repertoire Sequencing) data to analyze B-cell receptor (BCR) and T-cell receptor (TCR) repertoires. Provides functionality for database creation from IgBLAST results, clonal assignment based on V/J gene calls and CDR3 sequence similarity, and germline sequence mapping.
tags: immunology, airr-seq, bcr, tcr, immune-repertoire, igblast, clonality, antibody, repertoire
author: AI-generated
source_url: https://changeo.readthedocs.io/
---

## Concepts

- **Data Model**: changeo operates on tab-separated or FASTA-formatted files containing immune receptor sequences. Sequences must include the junction (CDR3) amino acid sequence, V gene call, J gene call, and C gene call for proper processing. The tool maintains all metadata columns from IgBLAST or other annotation sources throughout processing.

- **Clonal Assignment Logic**: The clone assignment algorithm clusters sequences sharing identical V and J gene assignments, identical CDR3 length, and CDR3 nucleotide similarity above a user-defined threshold (default 0.85). This combines sequences likely originating from the same somatic hypermutation event into clonal groups.

- **Input/Output Formats**: Input files are typically tab-separated files with sequence data and annotations (AIRR format), or FASTA files for sequences only. Outputs are tab-separated files with additional columns for clone IDs, germline sequences, and other computed values. The changeo-db subcommand converts IgBLAST tabular outputs into the changeo database format.

- **Companion Tools**: changeo works within the Immcantation framework alongside igblastn (for sequence annotation), presto (for sequence preprocessing), and shazam (for somatic hypermutation analysis). Database files created by changeo-db serve as input for downstream tools like shazam and tigger.

- **Germline Mapping**: The changeo-germline command replaces somatic mutations in V gene sequences with the corresponding germline bases, producing the full-length germline-reconstructed sequence. This is essential for analyzing somatic hypermutation loads and affinity maturation.

## Pitfalls

- **Missing IgBLAST Annotations**: Attempting to process sequences without proper V, D, J, and C gene calls results in empty or incorrect output. changeo requires these annotations from IgBLAST or equivalent alignment tools; raw sequences without annotation cannot be processed directly.

- **Inappropriate Clonal Threshold**: Using a similarity threshold that is too high (e.g., 1.0) splits clones that should be grouped together due to hypermutation diversity, while too low (e.g., 0.5) incorrectly merges distinct clones. The default 0.85 threshold is optimized for BCR data; TCR data may require different values.

- **CDR3 Detection Failures**: Sequences with ambiguous nucleotides (N bases) in the CDR3 region, or those where IgBLAST fails to identify a junction, will be excluded from clonal analysis. This can lead to significant data loss if input quality is poor.

- **Inconsistent Gene Naming**: Using gene names that don't match the germline database nomenclature causes alignment failures. The germline database must be consistent with the V/D/J gene calls in your input data.

- **Memory with Large Datasets**: Processing very large repertoire datasets (millions of sequences) can require substantial memory for clone assignment. Batch processing or filtering to high-confidence sequences first is recommended.

## Examples

### Create a changeo database from IgBLAST tabular output
**Args:** `db --db database.tsv --imed input_igblast.tsv --id call --seq sequence --v_call v_call --j_call j_call --c_call c_call --junction junction`
**Explanation:** This command converts IgBLAST tabular output into the changeo database format, specifying which columns contain the sequence identifiers, nucleotide sequences, V/J/C gene calls, and CDR3 junction sequences.

### Assign clonal groups based on CDR3 similarity
**Args:** `clone --db processed_db.tsv --out clones.tsv --sym fwr --mid 0.85 --nproc 4`
**Explanation:** This assigns clones to sequences using a 0.85 CDR3 nucleotide similarity threshold, focusing on framework regions and using 4 processor cores for parallel execution.

### Map sequences to germline V genes
**Args:** `germline --db clones.tsv --out germlined.tsv --germline /path/to/germline.fasta --exec igblastn --organism human --arm 3`
**Explanation:** This reconstructs germline sequences by replacing somatic mutations in V gene segments, using the specified germline FASTA database and IgBLAST executable for human heavy chain sequences.

### Export sequences in specific AIRR format
**Args:** `export --db clones.tsv --format tsv --out AIRR_subset.tsv --fields sequence_id,v_call,j_call,junction,clone_id`
**Explanation:** This exports a subset of columns from the database in AIRR-compliant TSV format, selecting only the sequence ID, V/J calls, junction, and clone assignment columns.

### Filter database by sequence count per clone
**Args:** `filter --db clones.tsv --out filtered_clones.tsv --cloned --min 2`
**Explanation:** This removes clones with fewer than 2 sequences, retaining only clonal groups with at least two members to focus on expanded clones for downstream analysis.

### Add taxonomy fields for mixed-species data
**Args:** `db --db database.tsv --imed mixed_calls.tsv --id sequence_id --seq sequence --v_call v_call --j_call j_call --c_call c_call --junction junction --species human,mouse,rhesus`
**Explanation:** This creates a database from files containing sequences from multiple species, specifying the species order to correctly annotate gene calls in the output.

### Generate FASTA output from database
**Args:** `convert --db clones.tsv --format fasta --out sequences.fasta`
**Explanation:** This converts the database into FASTA format, useful for creating input files for phylogenetic tools or sequence logo generation.

### Run clone assignment with amino acid sequence
**Args:** `clone --db processed_db.tsv --out clones_aa.tsv --sym fwr --mid 0.80 --aa`
**Explanation:** This performs clonal assignment using amino acid sequence similarity at a 0.80 threshold, which may be more appropriate for some TCR datasets or distant species comparisons.