---
name: varscan2
category: variant-calling
description: Variant detection in massively parallel sequencing data using threshold-based approach
tags: [variant-calling, somatic, germline, snp, indel, tumor-normal, mpileup, vcf]
author: oxo-call built-in
source_url: "https://varscan.sourceforge.net/"
---

## Concepts
- VarScan2 calls germline and somatic variants from samtools mpileup output.
- Main commands: mpileup2snp, mpileup2indel, mpileup2cns (for germline); somatic (for tumor-normal).
- Typical workflow: samtools mpileup -f ref.fa -q 20 sample.bam | varscan mpileup2snp > snps.vcf
- For somatic calling: samtools mpileup -f ref.fa normal.bam tumor.bam | varscan somatic --mpileup 1
- Use --strand-filter 0 to disable strand bias filter (can be too conservative).
- VarScan2 is a Java tool: varscan is an alias for java -jar VarScan.jar.
- Use --output-vcf 1 for VCF output; default is VarScan custom format.
- Somatic output: <prefix>.snp.vcf and <prefix>.indel.vcf files.
- --min-coverage sets minimum coverage for variant calling (default 8).
- --min-var-freq sets minimum variant allele frequency (default 0.20 for germline).
- --min-reads2 sets minimum supporting reads for variant allele (default 2).
- --p-value sets Fisher's exact test p-value threshold for germline variants.
- --somatic-p-value sets p-value threshold for somatic variant calling.

## Pitfalls
- VarScan2 requires mpileup input — cannot take BAM directly; must pipe from samtools mpileup.
- The mpileup step with samtools must use -q (mapping quality) and -Q (base quality) filters.
- VarScan2 tumor-normal somatic requires normal BAM first, then tumor BAM in samtools mpileup.
- Without --output-vcf 1, VarScan2 outputs its own format, not standard VCF.
- VarScan2 is a Java tool — set JVM heap: java -Xmx4g -jar VarScan.jar.
- For somatic calling, use VarScan2 processSomatic after somatic to separate germline/LOH/somatic.
- --min-var-freq default 0.20 may miss low-frequency variants; lower for sensitive detection.
- --strand-filter 1 (default) can be too conservative; disable with 0 for sensitive calling.
- --min-coverage should be adjusted based on sequencing depth; higher for high-coverage data.
- --p-value 0.99 is permissive; decrease for higher stringency.

## Examples

### call germline SNPs from a tumor or normal sample
**Args:** `mpileup2snp --min-coverage 8 --min-reads2 2 --min-avg-qual 15 --min-var-freq 0.01 --p-value 0.99 --output-vcf 1 > snps.vcf`
**Explanation:** pipe from: samtools mpileup -f ref.fa -q 20 sample.bam | varscan mpileup2snp [args]

### call somatic variants from tumor-normal pair
**Args:** `somatic normal_pileup.pileup tumor_pileup.pileup --output-snp somatic.snp.vcf --output-indel somatic.indel.vcf --output-vcf 1 --min-coverage 8 --min-var-freq 0.1 --somatic-p-value 0.05`
**Explanation:** samtools mpileup outputs separate pileup files; or pipe with --mpileup 1 flag

### filter somatic variants for high-confidence calls
**Args:** `processSomatic somatic.snp.vcf --min-tumor-freq 0.1 --max-normal-freq 0.05 --p-value 0.05`
**Explanation:** processSomatic separates Somatic, LOH, and Germline calls from VarScan2 somatic output

### call germline SNPs with lower frequency threshold (sensitive)
**Args:** `mpileup2snp --min-coverage 8 --min-reads2 2 --min-var-freq 0.05 --p-value 0.99 --strand-filter 0 --output-vcf 1 > sensitive_snps.vcf`
**Explanation:** --min-var-freq 0.05 for sensitive detection; --strand-filter 0 disables strand bias filter

### call indels from mpileup
**Args:** `mpileup2indel --min-coverage 8 --min-reads2 2 --min-var-freq 0.1 --p-value 0.99 --output-vcf 1 > indels.vcf`
**Explanation:** mpileup2indel for indel calling; same parameters as mpileup2snp

### call consensus sequence with variants
**Args:** `mpileup2cns --min-coverage 8 --min-reads2 2 --min-var-freq 0.2 --p-value 0.99 --output-vcf 1 > consensus.vcf`
**Explanation:** mpileup2cns calls consensus and variants; useful for generating consensus sequences

### somatic calling with custom thresholds
**Args:** `somatic normal.pileup tumor.pileup --output-snp somatic.snp --output-indel somatic.indel --output-vcf 1 --min-coverage 10 --min-coverage-tumor 6 --min-var-freq 0.05 --somatic-p-value 0.01`
**Explanation:** --min-coverage-tumor 6 for tumor; --somatic-p-value 0.01 for higher stringency

### filter variants by coverage and frequency
**Args:** `filter snps.vcf --min-coverage 10 --min-reads2 3 --min-var-freq 0.2 --p-value 0.01 --output-file filtered_snps.vcf`
**Explanation:** VarScan filter applies additional filtering; useful for removing false positives

### copy number analysis from tumor-normal
**Args:** `copynumber normal.pileup tumor.pileup --output-file copynumber.txt --min-coverage 20`
**Explanation:** copynumber command for CNV detection; requires higher coverage (20x)

### call somatic variants with direct pipe from samtools
**Args:** `somatic --mpileup 1 --output-snp somatic.snp.vcf --output-indel somatic.indel.vcf --output-vcf 1 --min-coverage 8 --min-var-freq 0.1 --somatic-p-value 0.05`
**Explanation:** --mpileup 1 enables direct pipe: samtools mpileup -f ref.fa normal.bam tumor.bam | varscan somatic [args]; no intermediate pileup files

### call germline SNPs with Java heap size adjustment
**Args:** `mpileup2snp --min-coverage 8 --min-reads2 2 --min-var-freq 0.01 --p-value 0.99 --output-vcf 1 --java-mem 8G > snps.vcf`
**Explanation:** --java-mem 8G sets JVM heap to 8GB; essential for large BAM files; prevents OutOfMemoryError

### batch process multiple samples with consistent parameters
**Args:** `mpileup2snp --min-coverage 8 --min-reads2 2 --min-var-freq 0.01 --p-value 0.99 --output-vcf 1 --output-root sample1 > sample1_snps.vcf`
**Explanation:** --output-root specifies output prefix; useful for batch processing with consistent naming

### call high-confidence somatic variants with strict thresholds
**Args:** `somatic normal.pileup tumor.pileup --output-snp highconf.snp --output-indel highconf.indel --output-vcf 1 --min-coverage 15 --min-coverage-normal 10 --min-coverage-tumor 10 --min-var-freq 0.15 --somatic-p-value 0.001 --strand-filter 1`
**Explanation:** strict thresholds: 15x coverage, 15% VAF, p-value 0.001; high-confidence somatic calls for clinical applications
