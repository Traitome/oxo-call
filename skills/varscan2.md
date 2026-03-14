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

## Pitfalls

- VarScan2 requires mpileup input — cannot take BAM directly; must pipe from samtools mpileup.
- The mpileup step with samtools must use -q (mapping quality) and -Q (base quality) filters.
- VarScan2 tumor-normal somatic requires normal BAM first, then tumor BAM in samtools mpileup.
- Without --output-vcf 1, VarScan2 outputs its own format, not standard VCF.
- VarScan2 is a Java tool — set JVM heap: java -Xmx4g -jar VarScan.jar.
- For somatic calling, use VarScan2 processSomatic after somatic to separate germline/LOH/somatic.

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
