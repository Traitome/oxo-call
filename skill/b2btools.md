---
name: b2btools
category: Immunology / Repertoire Sequencing
description: A comprehensive toolkit for B-cell receptor (BCR) and T-cell receptor (TCR) repertoire analysis from NGS data. Supports V(D)J annotation, clonotype calling, diversity metrics, and statistical analysis of adaptive immune responses.
tags:
  - immunology
  - VDJ
  - CDR3
  - repertoire
  - NGS
  - bioinformatics
  - bcr
  - tcr
  - immune-repertoire
  - clonotyping
author: AI-generated
source_url: https://github.com/zzach/b2btools
---

## Concepts

- **V(D)J Recombination Model**: b2btools assigns Variable (V), Diversity (D), and Joining (J) gene segments to each read based on alignment scoring, accounting for junctional diversity (N-additions and P-nucleotides). The tool distinguishes between functional and non-functional sequences using stop codon and in-frame detection.

- **Input/Output Formats**: The toolkit accepts paired-end or single-end FASTQ reads and produces outputs in TSV, CSV, or JSON formats. The primary output table includes columns for: sequence ID, V gene, J gene, C gene, CDR3 nucleotide sequence, CDR3 amino acid translation, reads count, and clone frequency. Summary statistics are exported as JSON for downstream visualization.

- **CDR3 Detection Algorithm**: CDR3 loop boundaries are identified using conserved framework motifs (Cysteine-104 in V segment and Phenylalanine/Tyrosine-116 in J segment for BCR; similar conserved residues for TCR). The algorithm handles both forward and reverse complement orientations and supports multiple constant region annotations for multiplexed samples.

- **Clonal Grouping and Diversity Metrics**: Sequences with identical V and J gene assignments and identical CDR3 amino acid sequences are grouped into clonotypes. Diversity is quantified using established metrics including Shannon entropy, D50 clonality score, inverse Simpson index, and unique clone counts normalized by total reads.

- **Reference Database Compatibility**: b2btools uses IMGT database annotations for V, D, and J gene segment naming conventions (e.g., IGHV3-23*01, IGHJ4*02). Custom references can be provided in FASTA format for non-model species, with gene segment naming following standard bioinformatics convention.

## Pitfalls

- **Insufficient Sequencing Depth**: Samples with fewer than 1,000 reads per input file may produce unreliable clonotype calls due to stochastic sampling. Low-depth libraries result in under-sampling of rare clones, artificially inflating clonality metrics and missing low-frequency antigen-specific responses. Always evaluate library complexity using unique read counts before interpretation.

- **Misaligned Constant Region Annotations**: When using multiplexed samples with different constant region primers, failure to specify the correct C gene database results in incorrect sample demultiplexing and cross-contamination between clonotypes from different samples. Verify primer-barcode associations and specify corresponding constant region references.

- **Ignoring Ambiguous V(D)J Assignments**: Sequences with multiple equally-scoring V or J gene candidates default to the first match, which may not represent the biologically correct assignment. For high-sensitivity applications, re-run with relaxed alignment thresholds and filter results using assignment probability scores provided in the output.

- **Frame-Shift Artifacts from Junctional Diversity**: Non-templated nucleotide additions at V(D)J junctions can cause frameshift mutations that produce out-of-frame CDR3 translations. Failing to validate amino acid translations for biological plausibility (no internal stop codons, reasonable length of 5-20 amino acids) results in inclusion of non-functional sequences in downstream analysis.

- **Inconsistent Sample Labeling Across Pipeline Steps**: Using different label formats (e.g., SMPL-01 vs SMPL_01) in separate pipeline steps causes silent sample mismatches during merged analysis. Clonotype frequencies appear erroneous when samples fail to merge correctly. Establish and enforce a consistent naming convention before initiating any analysis.

## Examples

### Perform V(D)J annotation on paired-end reads

**Args:** `annotate -i sample1_R1.fastq.gz -j sample1_R2.fastq.gz -o sample1_annotated.tsv -p 0.01`
**Explanation:** Runs V(D)J gene assignment on paired-end FASTQ files, outputting annotated clonotypes with frequency threshold of 1% to filter low-abundance reads.

### Generate immune repertoire diversity statistics

**Args:** `diversity -i merged_clonotypes.tsv -m shannon -m simpson -m d50 -m unique --by-sample group_column`
**Explanation:** Computes Shannon entropy, inverse Simpson index, D50 clonality score, and unique clone counts grouped by sample identifier for comparative diversity analysis.

### Filter clonotypes by CDR3 length and amino acid composition

**Args:** `filter -i raw_clonotypes.tsv -o filtered_clonotypes.tsv --cdr3-len-min 8 --cdr3-len-max 18 --stop-codons exclude --out-frame exclude`
**Explanation:** Retains only functional clonotypes with CDR3 lengths between 8-18 amino acids, excluding sequences containing internal stop codons or out-of-frame junctions that cannot represent productive receptors.

### Generate clonotype visualization heatmap

**Args:** `plot -i filtered_clonotypes.tsv -o repertoire_heatmap.png --plot-type heatmap --top-clones 50 --color-scheme viridis --by V_gene --xaxis J_gene`
**Explanation:** Creates a two-dimensional heatmap showing V-J gene usage for the top 50 clonotypes, with viridis color scaling indicating clone frequency for visual exploration of repertoire skewing.

### Export clonotypes to AIRR-COMPLIANT JSON format

**Args:** `export -i filtered_clonotypes.tsv -o airr_repertoire.json --format airr --repertoire-id Sample001 --species human --中生`
**Explanation:** Converts clonotype table to Adaptive Immune Receptor Repertoire (AIRR) Community standard JSON format with required repertoire metadata for data sharing and repository submission.

### Merge multiple sample files for comparative analysis

**Args:** `merge -i sample1.tsv sample2.tsv sample3.tsv -o merged_repertoire.tsv --by-sample --deduplicate --cdr3-as-seqid`
**Explanation:** Combines three sample files, annotating origin sample, deduplicating identical clonotypes across samples, and using CDR3 nucleotide sequence as primary sequence identifier for merged analysis.