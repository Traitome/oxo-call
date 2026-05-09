---
name: "argh"
category: "variant_calling"
description: "ARGh (ARGnome Haplotype) is a Bayesian haplotype-aware variant caller that detects SNPs, indels, and complex variants from aligned sequencing reads in BAM/CRAM format. It uses population-aware Bayesian modeling to call variants across the genome, supporting pooled and individual samples."
tags: ["variant_calling", "snps", "indels", "haplotype", "bayesian", "vcf", "bam", "cram", "genomics"]
author: "AI-generated"
source_url: "https://github.com/argh-toolkit/argh"
---

## Concepts

- **Input formats**: argh accepts aligned reads in BAM or CRAM format, optionally with an index file (.bai/.crai). A reference genome in FASTA format (with .fai index) is required for both input formats.
- **Output format**: All variant calls are emitted in VCF (Variant Call Format) version 4.2 to stdout by default, with a header containing sample names, filters applied, and annotation fields (DP, AF, AB, etc.).
- **Population and ploidy modeling**: argh supports arbitrary ploidy settings per-sample via the `--ploidy` flag, and pooled sampling modes where allele frequencies are estimated rather than called per-individual.
- **Haplotype-aware calling**: Unlike position-based callers, argh builds candidate haplotypes from read evidence and evaluates them jointly, enabling detection of complex events and indels.
- **Window-based processing**: argh processes the genome in overlapping sliding windows (controllable via `--window-size` and `--window-padding`) to balance sensitivity and computational burden.

## Pitfalls

- Using a CRAM input without a matching reference or with an incompatible reference causes the tool to fail silently or produce malformed alignments; always verify the `--reference` path matches exactly the FASTA used for the original alignment.
- Specifying a ploidy that exceeds the sample count in the input (e.g., `--ploidy 10` for a single individual) yields spurious allele frequency estimates and false-positive variants, as argh interprets this as a pooled sample; set `--ploidy` to 1 or 2 for non-pooled data.
- Omitting the required `--reference` flag when piping input from stdin or using bash globbing results in errors because argh cannot auto-detect the reference from the BAM/CRAM header in all configurations.
- Running without specifying output and forgetting to redirect stdout means variant calls are printed to the terminal and may be lost; always use `> output.vcf` or `--output output.vcf`.
- Setting `--min-supporting-reads` too low (e.g., 1 or 2) on high-coverage data dramatically increases false-positive indels and SNP calls; adjust this threshold based on coverage depth (typical minimum is 3–5 reads).

## Examples

### Call variants from a single BAM file using a reference genome
**Args:** `--reference hs37d5.fa --bam reads.bam --output variants.vcf`
**Explanation:** This specifies the reference FASTA and aligned BAM input, outputting called variants to a VCF file for downstream filtering.

### Run variant calling on a CRAM file with explicit reference
**Args:** `--reference grch38.fa --cram sample.cram --output called.vcf`
**Explanation:** Argh processes the CRAM file using the provided FASTA as the coordinate donor, producing VCF output relative to the GRCh38 assembly.

### Set diploidy for a human individual sample
**Args:** `--bam sample.bam --reference hg38.fa --ploidy 2 --output diploid_calls.vcf`
**Explanation:** Setting ploidy to 2 tells argh to assume two alleles per locus, which is correct for autosomal chromosomes in a diploid human sample.

### Call variants on pooled sequencing data with allele frequency estimation
**Args:** `--bam pool.bam --reference ref.fa --pooled --output pooled.vcf`
**Explanation:** The `--pooled` flag enables population frequency estimation mode, outputting allele frequencies rather than discrete genotype calls.

### Adjust the window size to improve sensitivity in repeat-rich regions
**Args:** `--bam reads.bam --reference ref.fa --window-size 50000 --window-padding 10000 --output repeat_region_calls.vcf`
**Explanation:** Larger windows capture more read context across repetitive or structural variant-prone regions, improving haplotype assembly at the cost of memory.

### Filter output by requiring minimum supporting reads at a variant locus
**Args:** `--bam NA12878.bam --reference hg38.fa --min-supporting-reads 5 --output filtered.vcf`
**Explanation:** Requiring at least 5 reads supporting the alternate allele reduces false-positive calls in low-coverage or PCR-duplicate regions.

### Enable standard VCF output to stdout for piping to other tools
**Args:** `--bam sample.bam --reference ref.fa`
**Explanation:** This outputs VCF to stdout, suitable for piping into bcftools or bgzip for compression and indexing in a bioinformatics pipeline.