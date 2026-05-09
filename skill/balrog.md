---
name: balrog
category: Antibody Repertoire Analysis
description: A tool for analyzing antibody and B-cell receptor sequences from next-generation sequencing data. Balrog processes raw sequence reads, identifies germline V/D/J gene segments, estimates somatic hypermutation rates, and clusters sequences into clonal groups for immune repertoire profiling.
tags:
  - antibody
  - BCR
  - immune-repertoire
  - NGS
  - VDJ-recombination
  - somatic-hypermutation
  - clonality
  - immunology
  - germline-assignment
author: AI-generated
source_url: https://bitbucket.org/kleinstein/balrog
---

## Concepts

- **Input formats:** Balrog accepts raw unaligned reads (FASTA/FASTQ), IMGT-aligned sequences, and IgBLAST output. Sequence headers should carry meaningful metadata (e.g., sample ID, UMI tags) because balrog uses this information for clonal grouping and filtering downstream.
- **Germline assignment pipeline:** The tool performs V, D, and J gene assignment using a curated reference database. Assigning is probabilistic — scores are emitted alongside the call so users can set thresholds to filter low-confidence assignments.
- **Clonal grouping and SHM estimation:** After germline assignment, balrog clusters sequences by common VDJ origin and junction length, then computes somatic hypermutation (SHM) counts relative to the assigned germline. These estimates are strand-aware and account for insertions and deletions.
- **Output formats:** Results are written as tab-delimited tables (clonal assignments, mutation counts, gene calls) and optionally JSON for downstream integration. Column headers are stable but can be toggled between human-readable and programmatic aliases via flags.
- **Reference database versioning:** Balrog ships with a default IGHV/IGKV/IGLV germline set, but users can supply a custom database via `--germline` or `--db`. Using mismatched database versions between analysis runs is a common source of batch inconsistency.

## Pitfalls

- **Mixing database versions across samples:** If you supply germline references from different species or database releases (e.g., mixing IMGT 2022 with an older custom set), gene assignments and SHM counts become incomparable across samples, corrupting comparative analyses.
- **Low-quality sequences not pre-filtered:** Feeding balrog raw FASTQ without quality trimming or read merging produces inflated SHM counts and spurious V-gene assignments, because sequencing errors are counted as mutations against the germline.
- **Inconsistent UMI barcode handling:** Sequences multiplexed with UMI barcodes must be collapsed (e.g., using `fgbio` or `umi_tools`) before input. Passing UMI-tagged but uncollapsed reads double-counts clones and skews clonal size distributions.
- **Threshold misunderstanding on `--min-score`:** Setting `--min-score` too high silently discards legitimate sequences from minority clones, creating a false impression of reduced diversity. Users often set this value without cross-validation against their expected repertoire complexity.
- **Paired-end read configuration:** When reads span the V(D)J junction (common in 2×250 or 2×300 kits), balrog requires correctly oriented and merged input. Passing unmerged mate pairs without `--paired` or proper `--fragment-length` leads to chimeric or dropped assignments.
- **Output column naming instability across versions:** Column aliases changed between balrog releases (e.g., `mut_nt` → `nucleotide_mutations`). Pipelines that rely on hardcoded column names break after upgrades without a migration step.

## Examples

### Analyze a FASTA file of antibody sequences with default settings
**Args:** `analyze-seqs --input ./sequences.fasta --output ./results.tsv`
**Explanation:** Runs the full pipeline — quality filtering, germline assignment, SHM estimation, and clonal grouping — on a plain FASTA file and writes results to a tab-delimited output table.

### Assign VDJ genes using a custom germline reference set
**Args:** `analyze-seqs --input ./reads.fq --germline ./custom_germlines.fasta --species mouse --output ./custom_results.tsv`
**Explanation:** Overrides the default human germline database with a user-supplied mouse reference, ensuring correct gene calls when working with murine immune repertoires.

### Filter low-confidence gene assignments by setting a minimum score threshold
**Args:** `analyze-seqs --input ./rep1.fasta --output ./filtered.tsv --min-score 60`
**Explanation:** Discards gene assignments with probabilistic scores below 60, reducing false-positive VDJ calls at the cost of losing genuine low-abundance clones that may be biologically meaningful.

### Export results in JSON format for programmatic downstream analysis
**Args:** `analyze-seqs --input ./rep2.fasta --output ./results.json --format json`
**Explanation:** Produces machine-readable JSON output containing per-sequence gene calls, mutation counts, and clone IDs, suitable for integration into R or Python免疫 repertoire pipelines.

### Process paired-end reads with explicit fragment length specification
**Args:** `analyze-seqs --input ./pe_reads --paired --fragment-length 300 --output ./paired_results.tsv`
**Explanation:** Configures balrog to expect mate-pair reads spanning the full V(D)J region with an average fragment length of 300 bp, ensuring correct assembly and junction resolution before germline calling.

### Run clonal grouping only, reusing pre-computed gene assignments
**Args:** `clone-seqs --input ./preassigned.tsv --output ./clones.tsv --clonal-threshold 0.05`
**Explanation:** Skips the germline assignment step and directly clusters sequences into clonal groups using junction-length similarity and SHM distance, with a 5% divergence threshold defining clonal boundaries.

### Summarize a repertoire with basic diversity statistics
**Args:** `summarize --input ./results.tsv --output ./summary_report.txt`
**Explanation:** Generates a human-readable report summarizing V-gene usage, clone size distributions, and SHM statistics from a completed analysis, without re-running the full pipeline.