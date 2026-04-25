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
**Explanation:** varscan mpileup2snp subcommand; --min-coverage 8 minimum coverage; --min-reads2 2 minimum variant reads; --min-avg-qual 15 minimum base quality; --min-var-freq 0.01 minimum variant frequency; --p-value 0.99 p-value threshold; --output-vcf 1 VCF format output; pipe from samtools mpileup

### call somatic variants from tumor-normal pair
**Args:** `somatic normal_pileup.pileup tumor_pileup.pileup --output-snp somatic.snp.vcf --output-indel somatic.indel.vcf --output-vcf 1 --min-coverage 8 --min-var-freq 0.1 --somatic-p-value 0.05`
**Explanation:** varscan somatic subcommand; normal_pileup.pileup normal pileup file; tumor_pileup.pileup tumor pileup file; --output-snp somatic.snp.vcf SNP output; --output-indel somatic.indel.vcf indel output; --output-vcf 1 VCF format; --min-coverage 8 minimum coverage; --min-var-freq 0.1 minimum VAF; --somatic-p-value 0.05 somatic p-value

### filter somatic variants for high-confidence calls
**Args:** `processSomatic somatic.snp.vcf --min-tumor-freq 0.1 --max-normal-freq 0.05 --p-value 0.05`
**Explanation:** varscan processSomatic subcommand; somatic.snp.vcf input VCF; --min-tumor-freq 0.1 minimum tumor VAF; --max-normal-freq 0.05 maximum normal VAF; --p-value 0.05 p-value threshold; separates Somatic, LOH, Germline calls

### call germline SNPs with lower frequency threshold (sensitive)
**Args:** `mpileup2snp --min-coverage 8 --min-reads2 2 --min-var-freq 0.05 --p-value 0.99 --strand-filter 0 --output-vcf 1 > sensitive_snps.vcf`
**Explanation:** varscan mpileup2snp subcommand; --min-coverage 8 minimum coverage; --min-reads2 2 minimum variant reads; --min-var-freq 0.05 sensitive detection; --p-value 0.99 p-value threshold; --strand-filter 0 disables strand bias filter; --output-vcf 1 VCF output

### call indels from mpileup
**Args:** `mpileup2indel --min-coverage 8 --min-reads2 2 --min-var-freq 0.1 --p-value 0.99 --output-vcf 1 > indels.vcf`
**Explanation:** varscan mpileup2indel subcommand; --min-coverage 8 minimum coverage; --min-reads2 2 minimum variant reads; --min-var-freq 0.1 minimum VAF; --p-value 0.99 p-value threshold; --output-vcf 1 VCF output; indel calling from mpileup

### call consensus sequence with variants
**Args:** `mpileup2cns --min-coverage 8 --min-reads2 2 --min-var-freq 0.2 --p-value 0.99 --output-vcf 1 > consensus.vcf`
**Explanation:** varscan mpileup2cns subcommand; --min-coverage 8 minimum coverage; --min-reads2 2 minimum variant reads; --min-var-freq 0.2 minimum VAF; --p-value 0.99 p-value threshold; --output-vcf 1 VCF output; calls consensus and variants

### somatic calling with custom thresholds
**Args:** `somatic normal.pileup tumor.pileup --output-snp somatic.snp --output-indel somatic.indel --output-vcf 1 --min-coverage 10 --min-coverage-tumor 6 --min-var-freq 0.05 --somatic-p-value 0.01`
**Explanation:** varscan somatic subcommand; normal.pileup tumor.pileup pileup files; --output-snp somatic.snp SNP output prefix; --output-indel somatic.indel indel output prefix; --output-vcf 1 VCF format; --min-coverage 10 minimum coverage; --min-coverage-tumor 6 tumor coverage; --min-var-freq 0.05 VAF threshold; --somatic-p-value 0.01 higher stringency

### filter variants by coverage and frequency
**Args:** `filter snps.vcf --min-coverage 10 --min-reads2 3 --min-var-freq 0.2 --p-value 0.01 --output-file filtered_snps.vcf`
**Explanation:** varscan filter subcommand; snps.vcf input VCF; --min-coverage 10 minimum coverage; --min-reads2 3 minimum variant reads; --min-var-freq 0.2 minimum VAF; --p-value 0.01 p-value threshold; --output-file filtered_snps.vcf output VCF

### copy number analysis from tumor-normal
**Args:** `copynumber normal.pileup tumor.pileup --output-file copynumber.txt --min-coverage 20`
**Explanation:** varscan copynumber subcommand; normal.pileup tumor.pileup pileup files; --output-file copynumber.txt output file; --min-coverage 20 minimum coverage for CNV detection

### call somatic variants with direct pipe from samtools
**Args:** `somatic --mpileup 1 --output-snp somatic.snp.vcf --output-indel somatic.indel.vcf --output-vcf 1 --min-coverage 8 --min-var-freq 0.1 --somatic-p-value 0.05`
**Explanation:** varscan somatic subcommand; --mpileup 1 enables direct pipe from samtools mpileup; --output-snp somatic.snp.vcf SNP output; --output-indel somatic.indel.vcf indel output; --output-vcf 1 VCF format; --min-coverage 8 minimum coverage; --min-var-freq 0.1 minimum VAF; --somatic-p-value 0.05 somatic p-value; no intermediate pileup files

### call germline SNPs with Java heap size adjustment
**Args:** `mpileup2snp --min-coverage 8 --min-reads2 2 --min-var-freq 0.01 --p-value 0.99 --output-vcf 1 --java-mem 8G > snps.vcf`
**Explanation:** varscan mpileup2snp subcommand; --min-coverage 8 minimum coverage; --min-reads2 2 minimum variant reads; --min-var-freq 0.01 minimum VAF; --p-value 0.99 p-value threshold; --output-vcf 1 VCF output; --java-mem 8G sets JVM heap to 8GB; essential for large BAM files

### batch process multiple samples with consistent parameters
**Args:** `mpileup2snp --min-coverage 8 --min-reads2 2 --min-var-freq 0.01 --p-value 0.99 --output-vcf 1 --output-root sample1 > sample1_snps.vcf`
**Explanation:** varscan mpileup2snp subcommand; --min-coverage 8 minimum coverage; --min-reads2 2 minimum variant reads; --min-var-freq 0.01 minimum VAF; --p-value 0.99 p-value threshold; --output-vcf 1 VCF output; --output-root sample1 output prefix; useful for batch processing

### call high-confidence somatic variants with strict thresholds
**Args:** `somatic normal.pileup tumor.pileup --output-snp highconf.snp --output-indel highconf.indel --output-vcf 1 --min-coverage 15 --min-coverage-normal 10 --min-coverage-tumor 10 --min-var-freq 0.15 --somatic-p-value 0.001 --strand-filter 1`
**Explanation:** varscan somatic subcommand; normal.pileup tumor.pileup pileup files; --output-snp highconf.snp SNP output; --output-indel highconf.indel indel output; --output-vcf 1 VCF format; --min-coverage 15 coverage threshold; --min-coverage-normal 10 --min-coverage-tumor 10 coverage per sample; --min-var-freq 0.15 VAF threshold; --somatic-p-value 0.001 strict p-value; --strand-filter 1 strand bias filter; high-confidence somatic calls
