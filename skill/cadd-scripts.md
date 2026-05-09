---
name: cadd-scripts
category: variant prioritization
description: A framework for scoring variant deleteriousness by integrating diverse annotations into a single metric. CADD (Combined Annotation Dependent Depletion) trains a support vector machine on derived-human vs preserved-human variants and pathogenic vs common variants to produce genome-wide variant scores for SNVs, indels, and structural variants.
tags:
  - variant-scoring
  - deleteriousness-prediction
  - svm
  - vcf
  - genomics
author: AI-generated
source_url: https://github.com/kircherlab/cadd-scripts
---

## Concepts

- CADD scores are computed by an SVM model that contrasts evolutionary constraint (derived alleles observed in humans vs preserved in primates) with simulated de novo variants. A higher CADD score indicates a more deleterious variant; raw scores are scaled to PHRED-like values (sCADD) where 10-20% of possible SNVs exceed 20, and ~1% exceed 30.

- Input requires chromosome, position, reference allele, and alternate allele (minimum), plus external annotations like Ensembl transcript IDs, CADD pre-computed base scores, SIFT/PolyPhen predictions, conservation scores, and regulatory region flags. Without pre-computed base scores, annotation must be provided via VEP, ANNOVAR, or a custom annotation BED/WIG file.

- The workflow has three distinct entry points: (1) `CADD.sh` scores VCF or TSV inputs against pre-existing `.tsv.gz` annotations; (2) `cadd-scripts-annotate.sh` or `annotate.sh` generates the annotation BED/WIG file from raw input before scoring; (3) `cadd-scripts-calcSVLS.sh` scores large structural variants (copy number, inversion, translocation) using separate models from SNV/indel scoring.

- Output formats are flexible: tab-delimited TSV with one row per variant and columns for `Chrom`, `Pos`, `Ref`, `Alt`, `RawScore`, `PhredScore` plus any annotation columns carried through, or annotated VCF with a new `CADD` INFO field. The `--tsv` flag forces TSV output; omitting it returns annotated VCF when the input is VCF.

- Assembly version (GRCh37/hg19 vs GRCh38/hg38) is set with the `--genome` flag and must match both the input annotations and the pre-computed base scores file. Mixing assemblies produces silently incorrect scores because coordinate liftover and annotation tracks differ.

## Pitfalls

- Using the wrong `--genome` assembly causes coordinate mismatches and zero/null scores for variants at the edges of annotation tracks. The error does not fail loudly—check that the output column is non-empty for every scored variant.

- Omitting required annotations (especially conservation scores and transcript IDs) produces a `NA` Phred score with no warning. The output TSV will still be written, but every variant gets a missing value, making downstream filtering appear to have no callable sites.

- Scoring individual VCF records one at a time instead of batching by chromosome wastes compute time because the annotation index is reloaded per record. Process all variants on the same chromosome in a single invocation; use UNIX `awk` to split a large VCF by chromosome when needed.

- Forgetting the `--anno` flag when using a custom annotation BED/WIG file means the scoring engine ignores the annotations entirely, falling back to the base pre-computed score only. Custom annotations are only incorporated when `-a/--anno` is explicitly provided.

- Large structural variant files (>10,000 lines) without the `--sv` flag are silently processed with SNV/indel models, yielding scores optimized for single-nucleotide changes. Always use `cadd-scripts-calcSVLS.sh` for files containing `DEL`, `DUP`, `INV`, or `BND` records.

## Examples

### Score a small VCF file with pre-computed annotations
**Args:** `-g -f input.vcf -o output.tsv -s 30`
**Explanation:** The `-g` flag enables gene-aware scoring, `-f` specifies the input VCF, `-o` sets the output TSV path, and `-s 30` sets a per-sample score threshold for filtering during scoring.

### Batch-score a VCF by chromosome using awk splitting
**Args:** `-g -f