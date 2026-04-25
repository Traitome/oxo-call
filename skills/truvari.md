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
- truvari anno adds annotations (GC content, repeat overlap, etc.) to SV calls in a VCF.
- truvari consistency generates consistency reports between multiple VCF files.
- truvari vcf2df converts VCF files to pandas DataFrames for analysis.
- truvari divide splits VCFs into shards for parallel processing.
- truvari phab harmonizes variant representations using multiple sequence alignment.
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
**Explanation:** truvari bench subcommand; -b truth.vcf.gz truth (base) VCF; -c calls.vcf.gz comparison VCF; -f reference.fasta reference genome; -o bench_output output directory; --passonly uses only FILTER=PASS; --sizemin 50 filters small variants

### benchmark with relaxed position tolerance for long-read SV calls
**Args:** `bench -b truth.vcf.gz -c calls.vcf.gz -f reference.fasta -o bench_output --refdist 1000 --pctsize 0.7 --passonly`
**Explanation:** truvari bench subcommand; -b truth.vcf.gz truth VCF; -c calls.vcf.gz comparison VCF; -f reference.fasta reference genome; -o bench_output output directory; --refdist 1000 allows 1 kb position window; --pctsize 0.7 requires 70% size similarity; --passonly FILTER=PASS only

### collapse redundant SV calls within a single caller VCF
**Args:** `collapse -i calls.vcf.gz -o collapsed.vcf --passonly --sizemin 50 --refdist 500`
**Explanation:** truvari collapse subcommand; -i calls.vcf.gz input VCF; -o collapsed.vcf output VCF; --passonly FILTER=PASS only; --sizemin 50 minimum SV size; --refdist 500 merges events within 500 bp; removes duplicate calls

### merge SV calls from multiple callers into a consensus VCF
**Args:** `collapse -i multi_caller.vcf.gz -o merged.vcf --chain --keep common`
**Explanation:** truvari collapse subcommand; -i multi_caller.vcf.gz input VCF; -o merged.vcf output VCF; --chain enables chained merging; --keep common retains variants supported by multiple callers

### run truvari refine to improve benchmarking accuracy with sequence realignment
**Args:** `refine --reference reference.fasta --regions bench_output/candidate.refine.bed bench_output/`
**Explanation:** truvari refine subcommand; --reference reference.fasta reference genome; --regions bench_output/candidate.refine.bed BED file of candidates; bench_output/ bench output directory input; re-evaluates ambiguous TP/FP calls

### filter SV VCF to a specific size range before benchmarking
**Args:** `bench -b truth.vcf.gz -c calls.vcf.gz -f reference.fasta -o bench_output --sizemin 50 --sizemax 10000 --passonly`
**Explanation:** truvari bench subcommand; -b truth.vcf.gz truth VCF; -c calls.vcf.gz comparison VCF; -f reference.fasta reference genome; -o bench_output output directory; --sizemin 50 --sizemax 10000 size window; --passonly FILTER=PASS only; covers 50-10000 bp range

### annotate SV calls with repeat regions
**Args:** `anno -i calls.vcf.gz -r repeats.bed -o annotated.vcf`
**Explanation:** truvari anno subcommand; -i calls.vcf.gz input VCF; -r repeats.bed repeat regions BED; -o annotated.vcf output VCF; annotates SVs with overlapping repeat regions

### generate consistency report between multiple VCFs
**Args:** `consistency -i vcf_list.txt -o consistency_report.txt`
**Explanation:** truvari consistency subcommand; -i vcf_list.txt input file with VCF paths; -o consistency_report.txt output report; compares multiple VCF files for consistency

### convert VCF to pandas DataFrame for analysis
**Args:** `vcf2df -i calls.vcf.gz -o calls.df.pkl`
**Explanation:** truvari vcf2df subcommand; -i calls.vcf.gz input VCF; -o calls.df.pkl output pickle file; useful for custom analysis and plotting

### divide VCF into independent shards for parallel processing
**Args:** `divide -i calls.vcf.gz -o shards/ --shard-size 1000`
**Explanation:** truvari divide subcommand; -i calls.vcf.gz input VCF; -o shards/ output directory for shards; --shard-size 1000 variants per shard; useful for parallel processing

### perform phasing-aware SV harmonization
**Args:** `phab -b truth.vcf.gz -c calls.vcf.gz -f reference.fasta -o phased_output/`
**Explanation:** truvari phab subcommand; -b truth.vcf.gz truth VCF; -c calls.vcf.gz comparison VCF; -f reference.fasta reference genome; -o phased_output/ output directory; harmonizes variant representations using MSA
