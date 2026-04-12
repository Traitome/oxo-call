---
name: truvari
category: utilities
description: Structural variant benchmarking, merging, and comparison against truth sets
tags: [sv, structural-variants, benchmarking, vcf, comparison, long-read]
author: oxo-call built-in
source_url: "https://github.com/ACEnglish/truvari/wiki"
---

## Concepts

- truvari bench compares a caller VCF against a truth VCF, reporting TP, FP, FN, precision, recall, and F1 for structural variants.
- Matching is done by size similarity (--pctsize), sequence similarity (--pctseq), and positional proximity (--refdist) simultaneously.
- truvari collapse merges redundant SV calls within a single VCF, keeping the best-supported call per locus.
- truvari refine improves bench results by performing local re-genotyping at candidate TP/FP loci using a reference and reads.
- The --passonly flag restricts analysis to FILTER=PASS variants; benchmarking with all variants including filtered ones inflates FP counts.
- Truvari requires indexed VCF files (tabix .tbi) for both base (truth) and comparison VCFs; missing indexes cause immediate errors.

## Pitfalls

- Both input VCFs must be bgzipped and tabix-indexed; plain gzip or uncompressed VCFs cause an error.
- Comparing SVs from different size ranges without --sizemin/--sizemax leads to misleading metrics — always filter to a consistent size range.
- Not setting --sizemin 50 discards small variants that most SV callers cannot reliably detect, distorting recall.
- truvari bench output directory must not already exist; use --force-unsafe-output only if you are sure you want to overwrite.
- --pctseq 0 disables sequence similarity matching and relies only on position and size, which inflates TP counts for imprecise calls.
- Insertion sequences in the VCF must have the full ALT sequence, not symbolic alleles, for --pctseq to work correctly.

## Examples

### benchmark a structural variant caller VCF against a truth set
**Args:** `bench -b truth.vcf.gz -c calls.vcf.gz -f reference.fasta -o bench_output --passonly --sizemin 50`
**Explanation:** -b is truth (base), -c is calls (comparison); --passonly uses only FILTER=PASS; --sizemin 50 filters small variants

### benchmark with relaxed position tolerance for long-read SV calls
**Args:** `bench -b truth.vcf.gz -c calls.vcf.gz -f reference.fasta -o bench_output --refdist 1000 --pctsize 0.7 --passonly`
**Explanation:** --refdist 1000 allows 1 kb position window; --pctsize 0.7 requires 70% size similarity

### collapse redundant SV calls within a single caller VCF
**Args:** `collapse -i calls.vcf.gz -o collapsed.vcf --passonly --sizemin 50 --refdist 500`
**Explanation:** removes duplicate calls at the same locus; --refdist 500 merges events within 500 bp

### merge SV calls from multiple callers into a consensus VCF
**Args:** `collapse -i multi_caller.vcf.gz -o merged.vcf --chain --keep common`
**Explanation:** --chain enables chained merging of multiple callers; --keep common retains variants supported by multiple callers

### run truvari refine to improve benchmarking accuracy with sequence realignment
**Args:** `refine --reference reference.fasta --regions bench_output/candidate.refine.bed bench_output/`
**Explanation:** refine re-evaluates ambiguous TP/FP calls using local assembly; takes bench output directory as argument

### filter SV VCF to a specific size range before benchmarking
**Args:** `bench -b truth.vcf.gz -c calls.vcf.gz -f reference.fasta -o bench_output --sizemin 50 --sizemax 10000 --passonly`
**Explanation:** --sizemin and --sizemax define the SV size window; focusing on 50-10000 bp covers the reliable detection range
