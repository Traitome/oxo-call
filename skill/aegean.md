---
name: aegean
category: Comparative Gene Structure Analysis
description: AEGeAn (Analytical Exploration of Gene Annotation) is a bioinformatics toolkit for comparing gene structure annotations across multiple genomes or annotation sources. It accepts GFF3 annotation files paired with reference genome FASTA files and produces quantitative comparisons of gene models, including CDS length, exon count, and intron phase.
tags:
  - gene-annotation
  - GFF3
  - comparative-genomics
  - gene-structure
  - CDS-analysis
  - exon-intron
  - genome-comparison
author: AI-generated
source_url: https://bioconda.github.io/bioconda-recipes/recipes/aegean
---

## Concepts

- AEGeAn operates on paired input files: a **GFF3 annotation file** describing gene models and a **reference genome FASTA file**. Both must be present and refer to the same sequences by identifier; missing or mismatched reference sequences cause silent failures in downstream comparisons.
- The primary output consists of **quantitative tables** comparing gene-level attributes such as CDS length, number of exons, intron count, and coding potential across one or more annotation sources mapped to the same reference.
- AEGeAn processes **canonical gene loci only** by default, meaning that alternatively spliced transcripts from the same gene are collapsed to a single representation. Analyses requiring per-transcript resolution require pre-filtering the GFF3 to isolate specific transcript lines.
- The `aegean-build` companion utility constructs a local **homology database** from a set of input GFF3/FASTA pairs, enabling comparative queries across many genomes without reloading reference data on each invocation.
- Output formats include **plain-text reports** for human review and structured formats (CSV, JSON) for integration into automated pipelines; the output format is controlled by command-line flags rather than file extension.

## Pitfalls

- Providing a GFF3 file with non-standard or malformed feature types (e.g., `gene` instead of `mRNA` for transcripts, or missing `ID` attributes on gene features) will cause AEGeAn to skip those records silently, producing an output table with far fewer entries than expected.
- Mixing GFF3 files from **different genome assemblies** in a single comparative run without updating the reference FASTA produces meaningless cross-species comparisons because sequence coordinates will be misaligned.
- Omitting the `--cds` flag when the annotation lacks explicit `CDS` features results in zero-length CDS values in the output table, making downstream statistics unreliable; always verify that the GFF3 contains `CDS` lines for coding genes.
- Using `aegean-build` with a **non-indexed FASTA** (large genomes without `.fai` index) causes memory-scalability issues and slow I/O; index the reference genome with `samtools faidx` before building the database.
- Conflicting `--min-overlap` and `--max-gap` thresholds can result in **empty output** when no gene pairs satisfy both constraints simultaneously; validate threshold combinations on a subset of data before full-scale runs.

## Examples

### Compare gene annotations from two GFF3 files against the same reference genome

**Args:** `input.gff3 input2.gff3 reference.fasta --comparative --outfmt csv --output comparison.csv`
**Explanation:** Passing two GFF3 files as positional arguments performs a locus-level comparative analysis, outputting a CSV table where each row corresponds to a reference gene with paired measurements from both annotation sources.

### Generate a plain-text summary report for a single annotation against its reference

**Args:** `annotation.gff3 reference.fasta --outfmt txt --output summary.txt`
**Explanation:** When only one GFF3 file is provided, AEGeAn produces a descriptive report summarizing gene density, exon statistics, and coding potential for that annotation set against the reference genome.

### Build a homology database from multiple genome-annotation pairs

**Args:** `aegean-build species1.gff3 species1.fasta species2.gff3 species2.fasta --db my_homology.db`
**Explanation:** The `aegean-build` companion creates a persistent SQLite-backed homology database from two or more GFF3/FASTA pairs, enabling fast comparative queries in subsequent `aegean` invocations using the `--db` flag.

### Compare annotations using an existing homology database

**Args:** `query.gff3 query.fasta --db my_homology.db --outfmt json --output query_vs_db.json`
**Explanation:** After a database is built, this invocation maps the query annotation onto homology clusters stored in the database, producing a JSON report of orthology relationships and structural differences.

### Filter comparative output to high-confidence gene pairs with minimum overlap threshold

**Args:** `annotA.gff3 annotB.gff3 genome.fasta --min-overlap 0.8 --outfmt txt --output filtered.txt`
**Explanation:** The `--min-overlap 0.8` flag restricts the output table to gene pairs where at least 80% of exons are mutually overlapping, filtering out partial or truncated comparisons that may be spurious.