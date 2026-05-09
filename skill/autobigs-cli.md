---
name: autobigs-cli
category: genomics
description: Command-line interface for automated genomic sequence binning and MAG quality estimation.
tags:
  - auto-binning
  - MAG
  - metagenomics
  - genomic-sequences
  - contigs
  - pipeline
  - quality-estimation
author: AI-generated
source_url: https://github.com/example/autobigs
---

## Concepts

- autobigs-cli organizes unbinned contigs from assembly files into draft MAGs using coverage and sequence composition signals. Input contig FASTA files are processed against a reference database (imported via `autobigs-cli import`) containing taxonomic markers and lineage-specific marker gene sets. The binner computes tetranucleotide frequencies and cross-maps reads to estimate coverage depth per contig before clustering.
- Quality filtering thresholds control which bins are retained. `--min-len` discards contigs below a base-pair length cutoff (often 1500 bp for细菌MAG standards), `--comp` enforces a minimum completion score, and `--cont` enforces a maximum contamination score. Bins failing either threshold are excluded from the final output by default unless `--force-include` is set.
- Output formats differ by downstream use case. TSV export includes bin membership per contig with quality statistics, which is suitable for manual review and spreadsheet analysis. FASTAGZ archives each bin as a separate FASTA file, retaining sequence data compressed for archive storage or re-analysis.
- autobigs-cli operates in three phases when using the `run` subcommand: (1) reference indexing, (2) coverage profiling across all contigs, and (3) expectation-maximization clustering. Checkpoint files allow resuming interrupted runs via `--resume`, avoiding recomputation of expensive coverage steps.
- Batch processing across multiple samples uses a manifest file listing one sample per line. When a global reference is available, all samples share the same marker gene set, but each sample's coverage profile is computed independently.

## Pitfalls

- Using mismatched `--ref-db` versions between `import` and `run` silently downgrades binning accuracy. The reference database carries a version hash that is checked at runtime; a mismatch produces a warning but allows execution with degraded marker coverage. Always rebuild the reference database when upgrading autobigs-cli.
- Feeding low-quality or highly fragmented assemblies causes oversized bins with inflated completion and contamination scores. Contigs shorter than `--min-len` are excluded from clustering entirely, so a low threshold increases false positives in downstream completeness estimates. Use CheckM or GUNC for orthogonal validation.
- Skipping the `--comp` and `--cont` filters results in draft MAGs that fail standard MIMAG quality tiers (e.g., high-quality, medium-quality). Even when completion is acceptable, bins with contamination above 10 % are flagged as contaminant-rich and should not be submitted to public repositories without manual review.
- Running `run` without `--cpus` on shared HPC nodes causes memory thrashing when large contig sets are processed. The binner loads the entire coverage matrix into RAM. Always specify `--cpus 8` or higher on multi-core nodes and monitor peak RSS usage in the job log.
- Attempting to resume a pipeline run after modifying input contigs without clearing the checkpoint directory produces inconsistent clusters. The checkpoint files embed contig hashes; a hash mismatch triggers an automatic deletion with a warning message, but the `--resume` flag alone does not validate input integrity.

## Examples

### Import a reference database for a bacterial clade
**Args:** `import --ref-db /data/gtdb-r214 --out /refs/gtdb-r214.db`
**Explanation:** The import subcommand downloads and indexes the specified reference database into a local binary cache, which is used by all subsequent binning steps.

### Run the binner with custom length and quality thresholds
**Args:** `run --contigs /data/sample_contigs.fna --ref-db /refs/gtdb-r214.db --out /output/sample1 --min-len 1500 --comp 90 --cont 5`
**Explanation:** Binning is performed with a minimum contig length of 1500 bp and quality filters requiring at least 90 % completion and no more than 5 % cross-contamination.

### Filter and retain only medium-quality MAG bins
**Args:** `filter --bins /output/sample1/bins/ --comp-min 50 --cont-max 10 --out /output/sample1/medium-q`
**Explanation:** The filter subcommand reads the existing bin set and writes a new bin directory containing only those bins meeting the medium-quality MIMAG threshold.

### Export bin results as a TSV table for manual review
**Args:** `export --bins /output/sample1/bins/ --format tsv --out /output/sample1/bins.tsv`
**Explanation:** Each bin is summarized with its contig membership, completion and contamination scores, and total base count, providing a tab-delimited file suitable for spreadsheets or downstream R scripts.

### Generate coverage and GC content scatter plots per bin
**Args:** `plot --bins /output/sample1/bins/ --type scatter