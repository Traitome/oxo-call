---
name: biofluff
category: k-mer analysis / taxonomic profiling
description: Biofluff is a fast k-mer counting and indexing tool for genome skimming and taxonomic classification. It builds compact k-mer databases from reference genomes, counts k-mers in sequencing reads or assemblies, and estimates taxonomic abundance or genome completeness against indexed references.
tags:
  - k-mer
  - genome-skimming
  - taxonomic-profiling
  - read-classification
  - completeness-estimation
  - reference-indexing
  - fm-index
  - bioinformatics
author: AI-generated
source_url: https://github.com/rambaut/biofluff
---

## Concepts

- Biofluff builds an FM-indexed k-mer reference database from one or more FASTA/FASTQ input genomes using `biofluff-build`. The resulting `.bfx` index file is compact, queryable, and reusable across multiple samples without rebuilding.
- K-mer counting in raw read files (`biofluff-count`) extracts canonical k-mers (reverse-complemented and min-hashed) from input data, producing a `.kcnt` binary histogram file. Canonical k-mers ensure each reverse-complement pair is counted as a single orientation.
- Taxonomic classification (`biofluff-classify`) maps read-derived k-mer sets against one or more `.bfx` reference indexes to estimate per-reference coverage, completeness percentage, and read assignment proportion, outputting a tab-delimited profile table.
- Biofluff uses a fixed memory budget via `--max-memory` (accepts values like `4G`, `500M`). If the working set exceeds this budget, excess k-mers are spill-loaded from disk; underestimating causes crashes; overestimating wastes RAM on systems with multiple concurrent jobs.
- The `-k` parameter sets k-mer size and must match between `biofluff-build` and `biofluff-count`/`biofluff-classify`. Sizes between 15 and 31 are common for bacterial genomes; smaller k values increase recall but raise false-positive assignments, while larger k values improve specificity but miss diverged homologs.

## Pitfalls

- Using mismatched k-mer sizes between index building and query stages silently produces zero meaningful matches, because k-mers extracted at one size will not exist in the reference database indexed at a different size.
- Providing assembled contigs instead of raw reads to `biofluff-count` when the intended workflow is taxonomic classification leads to heavily biased completeness estimates, as assembler-collapsed repeat k-mers skew read proportion calculations.
- Omitting `--canonical` when building the reference index causes `biofluff-classify` to double-count reverse-complement pairs during read mapping, artificially inflating estimated read proportions by up to 2× for circular genomes.
- Running `biofluff-build` on a reference set containing highly similar genomes (e.g., strains of the same species) without using `--distinct-k` results in an index dominated by shared k-mers, degrading discriminatory power for closely related taxa.
- Specifying input files with `.gz` compression extension but forgetting the `--gzip` flag causes `biofluff-count` to treat the file as plain text, resulting in malformed k-mer extraction or a crash when non-nucleotide characters are encountered.

## Examples

### Build a k-mer reference index from a single bacterial genome FASTA

**Args:** `build -k 25 -o myco_ref.bfx --canonical Mycobacterium_tuberculosis.fa.gz`
**Explanation:** Biofluff-build reads the gzip-compressed Mycobacterium genome, extracts all canonical 25-mers, and writes the FM-indexed `.bfx` database to `myco_ref.bfx`.

### Build a k-mer reference index from multiple genus-level genomes

**Args:** `build -k 21 -o genus_index.bfx --distinct-k species Salmonella_enterica_1.fa Salmonella_enterica_2.fa Shigella_flexneri.fa`
**Explanation:** The `--distinct-k` flag penalizes k-mers shared across input genomes during index construction, preserving strain-discriminative k-mers in the output database.

### Count canonical k-mers from paired-end read files

**Args:** `count -k 25 --gzip -o reads.kcnt --max-memory 2G sample_R1.fastq.gz sample_R2.fastq.gz`
**Explanation:** Biofluff-count extracts canonical 25-mers from both FASTQ files (treating read pairs as a merged pool), respecting a 2 GB memory ceiling and writing a binary k-mer histogram.

### Classify reads against a single reference index and save the profile

**Args:** `classify -k 25 -o profile.tsv reads.kcnt myco_ref.bfx`
**Explanation:** The k-mer count file `reads.kcnt` (counted at k=25) is mapped against the indexed reference, and per-reference completeness and read proportions are written tab-separated to `profile.tsv`.

### Classify reads against multiple reference indexes with lenient thresholds

**Args:** `classify -k 25 -o multi_profile.tsv --min-reads 2 --min-completeness 10.0 reads.kcnt genus_index.bfx another_index.bfx`
**Explanation:** Reads are mapped against all provided reference indexes simultaneously; only entries with at least 2 assigned reads and ≥10% estimated completeness appear in the output profile.

### Count k-mers from a single assembly contigs file

**Args:** `count -k 31 --canonical -o assembly.kcnt --max-memory 500M contigs.fa`
**Explanation:** Bioflufly-count processes a pre-assembled FASTA with canonical mode enabled, which is appropriate when analyzing haploid assemblies where duplicate k-mers from repeat collapse should not be normalized.