---
name: bcftools
category: variant-calling
description: Tools for variant calling and manipulating VCF/BCF files, part of the samtools/htslib suite
tags: [vcf, bcf, variant-calling, snp, indel, genotyping, ngs, mpileup, filtering, annotation]
author: oxo-call built-in
source_url: "https://samtools.github.io/bcftools/bcftools.html"
---

## Concepts

- BCF is binary VCF (smaller/faster); bcftools works with both. Use -O b for BCF output, -O z for gzipped VCF (recommended), -O v for plain VCF.
- bcftools has 22+ subcommands organized into three groups: **Indexing** (index), **VCF/BCF manipulation** (annotate, concat, convert, head, isec, merge, norm, plugin, query, reheader, sort, view), and **VCF/BCF analysis** (call, consensus, cnv, csq, filter, gtcheck, mpileup, polysomy, roh, stats).
- bcftools view is the Swiss-army knife for VCF filtering: -s selects samples, -r for regions, -f for FILTER field, -i/-e for INFO/FORMAT expressions.
- bcftools call performs variant calling from bcftools mpileup output; use -m for multiallelic calling (modern), -v to output only variants.
- The standard variant calling pipeline: bcftools mpileup -f ref.fa bam | bcftools call -m -v -o variants.vcf
- bcftools norm deduplicates and normalizes indels (left-align, split multi-allelic); use -f ref.fa for proper normalization.
- VCF files must be bgzip-compressed and tabix-indexed for region queries: bgzip -c file.vcf > file.vcf.gz && tabix -p vcf file.vcf.gz
- bcftools query extracts custom fields with --format: '%CHROM\t%POS\t%REF\t%ALT\t%INFO/DP\n' enables tabular output for downstream analysis.
- bcftools isec computes set operations (intersection, union, complement) on multiple VCF files; -n controls how many files a site must appear in.
- bcftools consensus applies VCF variants to a reference FASTA, producing a consensus sequence; supports IUPAC codes for heterozygotes (-I).
- bcftools plugin system provides 41+ plugins (fill-tags, split-vep, vcf2table, etc.); invoke with `bcftools +pluginname`.
- mpileup -g produces gVCF blocks for non-variant sites, enabling joint genotyping across samples.

## Pitfalls

- bcftools ARGS must start with a subcommand (view, call, mpileup, norm, merge, annotate, filter, stats, index, concat, isec, roh, gtcheck, consensus, plugin, csq, reheader, convert, head, query, sort, cnv, polysomy) — never with flags like -O, -o, -f. The subcommand ALWAYS comes first.
- bcftools subcommands do NOT accept `--help`; run the subcommand without arguments to see its usage (e.g., `bcftools view` instead of `bcftools view --help`).
- bcftools view filters by FILTER field with -f; use -i 'QUAL>20' for INFO-based filtering (not -f QUAL).
- bcftools call requires sorted BAM input; use samtools sort first.
- Multi-sample VCF merge with bcftools merge requires all input VCFs to be bgzipped and tabix-indexed.
- bcftools stats outputs statistics, not a VCF — do not try to pipe it back into VCF tools.
- -O z outputs gzipped VCF but does NOT automatically tabix-index it — run tabix after.
- Region strings use the format 'chr:start-end' with 1-based coordinates (same as VCF).
- bcftools concat requires input files to be sorted by position; use -a --allow-overlaps if files may overlap, or --naive for simple concatenation without recompression.
- -r (regions) uses index-jumps (fast, requires indexed input); -t (targets) streams through the file (works without index but slower).
- Plugins are invoked with `+` prefix: `bcftools +fill-tags`, not `bcftools plugin fill-tags`.
- When combining -s (sample subsetting) with -i/-e (filtering) in bcftools view, filtering happens first by default — split into two commands if unsure, piping with -Ou.

## Examples

### call variants from a BAM file against a reference genome
**Args:** `mpileup -f reference.fa -Ou input.bam | bcftools call -m -v -O z -o variants.vcf.gz`
**Explanation:** -f specifies reference; mpileup -Ou pipes uncompressed BCF (fast); call -m uses multiallelic model; -v outputs only variant sites; -O z outputs gzipped VCF; -o writes output

### filter VCF to keep only high-quality SNPs (QUAL > 30, depth > 10)
**Args:** `view -i 'QUAL>30 && INFO/DP>10 && TYPE="snp"' -O z -o filtered.vcf.gz input.vcf.gz`
**Explanation:** -i applies INFO field filter expression; TYPE selects variant type; -O z outputs bgzipped VCF; -o writes output

### merge multiple VCF files from different samples
**Args:** `merge -O z -o merged.vcf.gz sample1.vcf.gz sample2.vcf.gz sample3.vcf.gz`
**Explanation:** -O z outputs gzipped VCF; -o writes output; all inputs must be bgzip'd and tabix-indexed; outputs merged multi-sample VCF

### extract a specific sample from a multi-sample VCF
**Args:** `view -s SAMPLE_NAME -O z -o sample.vcf.gz multisample.vcf.gz`
**Explanation:** -s specifies sample name; -O z outputs gzipped VCF; -o writes output; use -s ^SAMPLE to exclude instead

### normalize indels and split multi-allelic variants
**Args:** `norm -m -any -f reference.fa -O z -o normalized.vcf.gz input.vcf.gz`
**Explanation:** -m -any splits all multi-allelic records; -f enables left-normalization of indels; -O z outputs gzipped VCF; -o writes output

### compute variant statistics for a VCF file
**Args:** `stats input.vcf.gz > stats.txt`
**Explanation:** outputs detailed statistics including ts/tv ratio, indel lengths, quality distributions; use plot-vcfstats to visualize

### select only SNPs from a VCF file
**Args:** `view -v snps -O z -o snps.vcf.gz input.vcf.gz`
**Explanation:** -v snps selects only SNP records; -O z outputs gzipped VCF; -o writes output; use -v indels for indels only; -V excludes types

### annotate VCF with a reference VCF (add ID field from dbSNP)
**Args:** `annotate -a dbsnp.vcf.gz -c ID -O z -o annotated.vcf.gz input.vcf.gz`
**Explanation:** -a is the annotation source; -c specifies which columns to annotate; -O z outputs gzipped VCF; -o writes output

### find variants shared between two VCF files (intersection)
**Args:** `isec -p output_dir -n=2 input1.vcf.gz input2.vcf.gz`
**Explanation:** -p writes per-file subsets to directory; -n=2 outputs sites present in both files; use -n+2 for sites in at least 2 files

### extract custom fields from VCF as TSV
**Args:** `query -f '%CHROM\t%POS\t%REF\t%ALT\t%INFO/DP\t[%GT\t]\n' input.vcf.gz`
**Explanation:** -f specifies format string; %CHROM/%POS are fixed fields; [%GT] iterates over samples; brackets [] wrap per-sample FORMAT fields

### create consensus sequence by applying VCF to a reference
**Args:** `consensus -f reference.fa -I -o consensus.fa input.vcf.gz`
**Explanation:** -f specifies reference FASTA; -I outputs IUPAC codes for heterozygotes; -o writes output; use -s to select sample, -H to choose haplotype

### concatenate chromosome VCFs into one file
**Args:** `concat -a --allow-overlaps -O z -o all_chr.vcf.gz chr1.vcf.gz chr2.vcf.gz chr3.vcf.gz`
**Explanation:** -a allows overlapping positions; --allow-overlaps handles overlapping regions; -O z outputs gzipped VCF; -o writes output; inputs must be sorted; use --naive for fast concatenation without recompression

### rename samples in a VCF file
**Args:** `reheader -s new_names.txt -o renamed.vcf.gz input.vcf.gz`
**Explanation:** -s takes a file with old_name\tnew_name per line; -o writes output; use -n for comma-separated list; -f to update sequence dictionary from .fai

### run a plugin (fill-tags to add AF, AC, AN, HWE)
**Args:** `+fill-tags -O z -o tagged.vcf.gz input.vcf.gz`
**Explanation:** + prefix invokes plugin; fill-tags computes AF, AC, AC_Het, AC_Hom, AN, HWE, MAF, NS; -O z outputs gzipped VCF; -o writes output; use -- -t AF,AN,AC to compute specific tags only

### gVCF calling for joint genotyping workflow
**Args:** `mpileup -f reference.fa -g 10 -Ou input.bam | bcftools call -m -v -O z -o variants.vcf.gz`
**Explanation:** -f specifies reference; -g 10 groups non-variant sites with DP>=10 into gVCF blocks; -Ou pipes uncompressed BCF; call -m uses multiallelic model; -v outputs only variants; -O z outputs gzipped VCF; -o writes output; enables scalable joint genotyping of many samples
