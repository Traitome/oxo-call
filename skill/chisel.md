---
name: chisel
category: variant-calling
description: Haplotype-based somatic SNV and LOH caller for pooled low-depth cancer sequencing. Uses allele-frequency modeling and haplotype phasing to call variants from read alignments and a reference panel of common SNPs.
tags: [somatic-snvs, pooled-sequencing, allele-frequency, loh, cancer, haplotype-phasing]
author: AI-generated
source_url: https://github.com/raphael-group/chisel
---

## Concepts

- Chisel takes a **BAM/CRAM file** (or a list of multiple BAMs for a pool) and a **reference panel** (BCF/VCF of known common SNP sites) to estimate allelic depths and model allele frequencies using a beta-binomial mixture. The reference panel provides haplotype phase information used to disambiguate overlapping reads.
- The core output is a **site list table** (or optionally VCF) with chromosome, position, reference allele, alternate allele, allele fraction, and a categorical call quality (e.g., "strong_alt", "weak_alt", "ambiguous", "ref"). These categories reflect the posterior probability of the alternate allele being present.
- Chisel performs **haplotype-aware frequency estimation** using a forward-backward algorithm along phased haplotype blocks from the reference panel. This reduces bias from read misalignment, especially in repetitive or duplicated regions where naive read-counting approaches fail.
- The **--max-depth** flag sets a per-site coverage ceiling; sites exceeding this threshold are excluded from analysis to avoid inflated statistics from mapping artifacts. The recommended setting is sample-specific and typically falls between 500x and 2000x.
- Companion tool **chisel-asm** runs local de novo assembly (using flowsolver) around candidate sites to rescue variants that fall in repetitive sequence, producing an assembly table consumed by the main chisel pipeline via the **--asm** flag.

## Pitfalls

- Using a reference panel that is **not indexed with bcftools index** causes chisel to abort at startup with a "could not open panel" error, and the entire run fails without producing any output. Always index the panel with `bcftools index panel.bcf` before passing it.
- Mismatch between **chromosome names in the BAM and the reference panel** silently drops entire chromosomes from analysis. For example, a BAM with "chr1" and a panel without the "chr" prefix results in zero shared genomic intervals. Verify exact chromosome naming with `samtools view -H sample.bam | grep SN:` and `bcftools view -h panel.bcf`.
- Setting **--max-depth too low** discards genuine highly-covered sites such as centromeres and segmental duplications, which are exactly the regions where assembly-based rescue via **--asm** is most valuable. Balance by setting **--max-depth** to a value above the 99th percentile of coverage across your sample.
- Not setting **--seed** leads to non-reproducible results when beta-binomial model parameters are bootstrapped, because the EM algorithm initialization differs across runs. This matters for reproducibility in research contexts; always fix the seed for publication pipelines.
- For **pooled samples with very unequal representation** (e.g., highly aneuploid tumors), using a uniform sample table causes biased frequency estimates for underrepresented clones. Use the **sample-specific read-depth column** in the samples file to weight clones appropriately.
- Passing a **BAM with unpaired reads** to a pipeline expecting paired reads may not raise an explicit error but results in degraded haplotype phasing and a higher rate of "ambiguous" calls at known heterozygous sites.

## Examples

### Calling somatic SNVs in a single pooled sample
**Args:** `call --bam tumor.bam --panel common_snps.bcf --out results.tsv --genome hg38`
**Explanation:** This runs the main chisel pipeline on a single BAM for a pooled tumor sample, using the common SNP reference panel for phasing and outputting a tab-delimited site list.

### Filtering for high-confidence alternate calls only
**Args:** `call --bam tumor.bam --panel common_snps.bcf --out filtered.tsv --min-ad 10 --min-af 0.05 --call-filter strong_alt`
**Explanation:** Restricts the output to sites with at least 10 alternate read counts, an allele fraction of at least 5%, and a "strong_alt" quality call, removing ambiguous and likely-reference sites.

### Detecting loss-of-heterozygosity regions
**Args:** `call --bam tumor.bam --panel common_snps.bcf --out loh.tsv --loh --loh-sites loh_regions.bed`
**Explanation:** Enables LOH detection mode, which identifies genomic intervals where the normal reference allele is entirely lost and the tumor is homozygous for the alternate allele, outputting BED-formatted coordinates.

### Running assembly-based rescue before variant calling
**Args:** `asm --bam tumor.bam --out asm_results.tsv --genome hg38`
**Explanation:** Invokes the chisel-asm companion tool to perform local de novo assembly around candidate sites, producing an assembly table used in the main call step via the `--asm` flag to rescue variants in repetitive regions.

### Multi-sample pooled analysis with a samples table
**Args:** `call --bam samples.list --panel common_snps.bcf --out multi.tsv --samples samples.txt --min-mean-depth 30`
**Explanation:** Runs chisel on multiple pooled samples defined in `samples.list`, using a samples table that assigns clones to subpopulations; only sites where all samples meet the minimum mean depth threshold of 30x are retained.

### Reproducible calling with a fixed random seed
**Args:** `call --bam tumor.bam --panel common_snps.bcf --out reproducible.tsv --seed 42 --min-af 0.02`
**Explanation:** Fixes the random seed for the beta-binomial model EM algorithm to ensure bit-identical results across reruns, while lowering the minimum allele fraction threshold to 2% to capture low-frequency subclones.