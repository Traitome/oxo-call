---
name: callerpp
category: Variant Calling / Genomics
description: A high-performance variant caller for detecting SNVs, indels, and structural variants from aligned sequencing reads (BAM/SAM/CRAM). Supports multi-sample joint calling, contamination estimation, and produces standard VCF/BCF output.
tags:
- variant-calling
- genomics
- snp
- indel
- vcf
- bcf
- sequence-analysis
author: AI-generated
source_url: https://github.com/oxo-call/callerpp
---

## Concepts

- **Input formats**: Accepts aligned reads in BAM, SAM, or CRAM format, optionally with index files (.bai/.crai). Multi-sample analysis uses a list file (one path per line) to specify multiple input BAMs.
- **Output formats**: Produces compressed VCF (.vcf.gz) or BCF (.bcf) by default, with optional separate variant-only output (--regions or --sites-only). A GZIP or BGF compression is determined by the --output-fmt option.
- **Key algorithms**: Uses a Bayesian mixture model for genotype likelihoods (--model bayes), with optional read-backed phasing (--phase) and structural variant detection via split-read assembly (--assemble).
- **Filtering tiers**: Applies read depth (--min-depth), allele balance (--ab-ratio), and base quality (--min-qual) filters at call time. Use --filter-expression for custom VCF FILTER annotations using SCEpr language.
- **Parallel execution**: Uses chunk-based decomposition (--regions) for distributed calling. Chunk size controlled by --chunk-size and memory budget by --max-mem; scales linearly with input file count.

## Pitfalls

- **Unindexed BAM files cause silent failures**: Running callerpp on BAMs without corresponding .bai index files produces no error but generates empty VCF output. Always pre-index with samtools index before calling.
- **Mismatched read groups trigger sample misdetection**: If input BAMs have missing or inconsistent @RG read group tags, callerpp falls back to treating all reads as a single sample (--sample-name becomes mandatory to override).
- **Low-quality bases inflate false positives**: Default --min-qual threshold (20) may be insufficient for PacBio/Nanopore data which have higher per-base error rates. Adjust to --min-qual 15 or use --model noise-aware for the respective technology.
- **Memory exhaustion on large cohort joint calling**: Joint calling 100+ samples without chunking can exceed available RAM and crash. Use --regions to split by chromosome and call in batches, then merge with bcftools concat.
- **Duplicate reads create allele balance distortion**: If duplicates are not marked or --remove-duplicates is omitted, heterozygous calls may show skewed allele ratios (0.33 or 0.66 instead of 0.5), leading to false filtered variants.

## Examples

### Call variants from a single BAM file with default settings
**Args