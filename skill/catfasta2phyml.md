---
name: catfasta2phyml
category: Phylogenetics / Alignment Processing
description: Concatenates multiple sequence alignments in FASTA format into a single interleaved alignment suitable for phylogenetic reconstruction with PhyML. Supports partitioned analyses by preserving partition information and handling missing sequences across datasets.
tags:
  - phylogenetics
  - alignment-concatenation
  - fasta
  - phyml-input
  - partitioned-analysis
  - multi-locus
  - sequence-alignment
author: AI-generated
source_url: https://github.com/jdyates/catfasta2phyml
---

## Concepts

- **Input format**: The tool expects one or more FASTA files containing sequence alignments. Each file may represent a single gene locus or marker. Sequences must be aligned (same length per file) but files can have different lengths representing different genes or partitions.

- **Partition handling**: When concatenating, the tool tracks partition boundaries by sequence position. Partitions can be labeled using the `-p` or `--partitions` flag with comma-separated names matching input file order. This partition information is critical for running partitioned phylogenetic models in PhyML.

- **Missing sequence handling**: If a sequence is absent from one partition but present in others, the tool inserts missing-data characters (`-` or `N`) to maintain positional alignment across all partitions. The `-m` flag controls how missing data characters are interpreted.

- **Output formats**: Supports standard interleaved FASTA output (default) and can produce output compatible with PhyML by arranging sequences in the specific interleaved block format PhyML expects. The `--interleaved` flag controls block size.

- **Sequence filtering**: The `-s` flag allows excluding sequences by name, useful for removing problematic taxa or outgroups from specific partitions before concatenation.

## Pitfalls

- **Mismatched sequence names across files**: If sequence labels differ slightly (e.g., "Gene1" vs "GENE1" or "Species_A" vs "Species-A") between input files, the tool will treat them as distinct taxa rather than concatenating them as the same sequence. This results in a concatenated alignment with more taxa than expected, breaking phylogenetic analyses. Always standardize names before concatenation using `sed` or `awk`.

- **Unequal sequence lengths within a single partition file**: Each FASTA file fed to catfasta2phyml must contain sequences of identical length (aligned). Feeding unaligned sequences will either produce an error or silently misalign the output, corrupting the phylogenetic matrix. Always verify alignment with ` alignments with `seqmagick` or `FASTA` dimensions before concatenating.

- **Incorrect partition order**: Partitions are processed in the order input files are provided. If the wrong file order is given, partition labels will be mismatched relative to the actual sequence positions. The resulting PhyML partition model will assign incorrect evolutionary models to each region, invalidating results. Always double-check file ordering matches the intended partition scheme.

- **Default missing-data character conflicts**: Some tools expect `N` for nucleotide missing data while others expect `-`. catfasta2phyml defaults to `-`, but if downstream tools like PhyML are configured to treat `-` as a gap (not ambiguity), results may be biased. Specify the correct character with `-m` to match your phylogenetic software's expectations.

- **Memory issues with very large alignments**: Concatenating hundreds of partitions with thousands of taxa can produce massive interleaved matrices. The tool loads all partitions into memory before writing output. On systems with limited RAM, this causes crashes or OOM kills. Process large datasets in batches using shell loops.

## Examples

### Concatenate two gene alignment FASTA files into a single interleaved alignment
**Args:** `gene1.fasta gene2.fasta > concatenated.fasta`
**Explanation:** Reads both FASTA files in order, concatenates sequences by taxon name, and writes interleaved output to stdout where sequences appear end-to-end across partitions.

### Concatenate three partitions and label them for a partitioned phylogenetic analysis
**Args:** `-p COI,12S,rpL16 partition1.fasta partition2.fasta partition3.fasta -o partitioned_output.fasta`
**Explanation:** Processes three input files and writes a concatenated alignment with partition labels COI, 12S, and rpL16 embedded in the output, enabling partitioned model specification in PhyML.

### Concatenate alignments excluding a problematic sequence from all partitions
**Args:** `-s Taxon_X -o clean_concat.fasta all_genes/*.fasta`
**Explanation:** Filters out any sequence named Taxon_X from all input FASTA files before concatenation, producing a clean matrix without manually removing entries from each file first.

### Generate a concatenated alignment with specific missing-data character for nucleotide analysis
**Args:** `-m N -o phyml_input.fasta locus*.fasta`
**Explanation:** Processes all matching locus FASTA files and outputs concatenated alignment using `N` instead of the default `-` character for missing positions, matching PhyML's nucleotide ambiguity expectation.

### Batch concatenate multiple gene sets across taxonomic groups
**Args:** `for f in species*/gene*.fasta; do echo "$f"; done | catfasta2phyml -o batch_concat.fasta`
**Explanation:** Uses shell expansion to collect multiple FASTA files from nested directories and pipes them to catfasta2phyml, useful when gene files are organized in per-species subdirectories rather than a single folder.