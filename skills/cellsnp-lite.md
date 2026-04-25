---
name: cellsnp-lite
category: single-cell
description: Efficient pileup of known SNPs from single-cell or bulk RNA/DNA-seq BAM files for demultiplexing and genotyping
tags: [single-cell, snp, pileup, allele-specific, genotyping, rna-seq, demultiplexing, vireo, variant-calling, 10x-genomics]
author: oxo-call built-in
source_url: "https://cellsnp-lite.readthedocs.io/"
---

## Concepts

- cellsnp-lite piles up reads at known SNP positions (Mode 1) or discovers de novo SNPs (Mode 2) in single-cell or bulk BAMs.
- Mode 1a: bulk BAM + known SNPs (-R); Mode 1b: 10x BAM + cell barcodes (-b) + known SNPs; Mode 2: de novo SNP discovery.
- Output is a set of sparse matrix files (AD.mtx, DP.mtx, OTH.mtx) in VCF + MTX format compatible with Vireo and other demultiplexers.
- The -R flag points to a VCF of known SNPs (e.g., 1000G or dbSNP); using a genome-matched SNP panel improves genotyping accuracy.
- Cell barcodes are supplied via -b (plain text file, one barcode per line) matching the CB tags in the BAM.
- --minMAF and --minCOUNT filter sparse positions; for genotyping applications use --minMAF 0.1 --minCOUNT 20.
- BAM files must be indexed (.bai) before running cellsnp-lite.
- --genotype flag enables genotyping in addition to counting, outputting GP (genotype probability) and GT (genotype) fields.
- --gzip compresses output files to BGZF format, saving disk space.
- --UMItag controls UMI handling: UB for 10x Genomics, None for bulk, Auto for automatic detection.

## Pitfalls

- BAM files must have CB (cell barcode) and UB (UMI) tags for single-cell mode; bulk BAMs without CB tags must use Mode 1a.
- Not filtering by mapping quality (--minMAPQ 20) allows multi-mapping reads to distort allele counts.
- Using too permissive --minMAF (e.g., 0) in Mode 2 generates millions of low-quality SNPs; use --minMAF 0.1 for reliability.
- The SNP VCF for -R must contain REF and ALT alleles; position-only files cause incorrect allele counting.
- Output directories must not exist; cellsnp-lite fails if -O points to an existing directory without --force.
- Single-cell mode requires the BAM index (.bai); run samtools index before cellsnp-lite.
- --regionsVCF (-R) uses indexed random access while --targetsVCF (-T) uses streaming; use -R for small SNP panels, -T for large VCFs.
- Default --exclFLAG filters duplicates when UMI is off but not when UMI is on; explicitly set --exclFLAG for consistent behavior.
- Mode 2 (de novo) requires a reference genome (-f) to determine REF alleles; missing -f causes errors or incorrect genotypes.
- Memory usage scales with --maxDEPTH; leave at default (0) for most cases, only reduce if memory is limited.

## Examples

### pileup known SNPs in a 10x Chromium scRNA-seq BAM with cell barcodes
**Args:** `-s possorted_genome_bam.bam -b barcodes.tsv -O cellsnp_out -R common_snps.vcf.gz -p 16 --minMAF 0.1 --minCOUNT 20`
**Explanation:** -s possorted_genome_bam.bam BAM input; -b barcodes.tsv cell barcodes file; -O cellsnp_out output directory; -R common_snps.vcf.gz known SNP VCF; -p 16 threads; --minMAF 0.1 minimum allele frequency; --minCOUNT 20 minimum count; quality filter for Mode 1b

### pileup SNPs in a bulk BAM without cell barcodes
**Args:** `-s bulk.bam -O bulk_snp_out -R common_snps.vcf.gz -p 16 --minMAF 0.05 --minCOUNT 10`
**Explanation:** -s bulk.bam BAM input; -O bulk_snp_out output directory; -R common_snps.vcf.gz known SNP VCF; -p 16 threads; --minMAF 0.05 minimum allele frequency; --minCOUNT 10 minimum count; Mode 1a: no -b flag for bulk; output has total AD/DP counts per SNP

### de novo SNP discovery in single-cell BAM (Mode 2)
**Args:** `-s possorted_genome_bam.bam -b barcodes.tsv -O denovo_snp_out -p 16 --minMAF 0.1 --minCOUNT 100 --gzip`
**Explanation:** -s possorted_genome_bam.bam BAM input; -b barcodes.tsv cell barcodes file; -O denovo_snp_out output directory; -p 16 threads; --minMAF 0.1 minimum allele frequency; --minCOUNT 100 minimum count; --gzip compresses output; Mode 2: no -R flag; discovers SNPs from data

### pileup multiple BAMs from different samples at shared SNP positions
**Args:** `-s sample1.bam,sample2.bam,sample3.bam -O multi_sample_out -R common_snps.vcf.gz -p 16 --minMAF 0.1 --minCOUNT 20`
**Explanation:** -s sample1.bam,sample2.bam,sample3.bam comma-separated BAM list; -O multi_sample_out output directory; -R common_snps.vcf.gz known SNP VCF; -p 16 threads; --minMAF 0.1 minimum allele frequency; --minCOUNT 20 minimum count; multi-sample bulk pileup; useful for genotyping donors for Vireo

### restrict pileup to specific chromosomes to reduce runtime
**Args:** `-s possorted_genome_bam.bam -b barcodes.tsv -O chr1_out -R chr1_snps.vcf.gz --chrom 1 -p 8 --minMAF 0.1 --minCOUNT 20`
**Explanation:** -s possorted_genome_bam.bam BAM input; -b barcodes.tsv cell barcodes file; -O chr1_out output directory; -R chr1_snps.vcf.gz known SNP VCF; --chrom 1 restricts to chromosome 1; -p 8 threads; --minMAF 0.1 minimum allele frequency; --minCOUNT 20 minimum count; useful for testing or chromosome-level parallel jobs

### pileup with strict base quality filter for high-confidence allele counts
**Args:** `-s possorted_genome_bam.bam -b barcodes.tsv -O hq_out -R snps.vcf.gz -p 16 --minMAF 0.1 --minCOUNT 20 --minMAPQ 30`
**Explanation:** -s possorted_genome_bam.bam BAM input; -b barcodes.tsv cell barcodes file; -O hq_out output directory; -R snps.vcf.gz known SNP VCF; -p 16 threads; --minMAF 0.1 minimum allele frequency; --minCOUNT 20 minimum count; --minMAPQ 30 filters low-quality alignments; cellsnp-lite does not have --minBQ flag; use --minMAPQ and --minLEN for read filtering

### genotype cells in addition to counting alleles
**Args:** `-s possorted_genome_bam.bam -b barcodes.tsv -O genotyped_out -R snps.vcf.gz -p 16 --minMAF 0.1 --minCOUNT 20 --genotype --gzip`
**Explanation:** -s possorted_genome_bam.bam BAM input; -b barcodes.tsv cell barcodes file; -O genotyped_out output directory; -R snps.vcf.gz known SNP VCF; -p 16 threads; --minMAF 0.1 minimum allele frequency; --minCOUNT 20 minimum count; --genotype outputs genotype probabilities (GP) and genotypes (GT); --gzip compresses output files

### use streaming mode for large SNP panels
**Args:** `-s possorted_genome_bam.bam -b barcodes.tsv -O streaming_out -T large_panel.vcf.gz -p 16 --minMAF 0.1 --minCOUNT 20`
**Explanation:** -s possorted_genome_bam.bam BAM input; -b barcodes.tsv cell barcodes file; -O streaming_out output directory; -T large_panel.vcf.gz SNP panel (streaming mode); -p 16 threads; --minMAF 0.1 minimum allele frequency; --minCOUNT 20 minimum count; -T uses streaming instead of indexed access; better for large SNP panels

### pileup without UMI counting (read-level)
**Args:** `-s bulk.bam -O read_counts_out -R snps.vcf.gz -p 16 --minMAF 0.05 --minCOUNT 10 --UMItag None`
**Explanation:** -s bulk.bam BAM input; -O read_counts_out output directory; -R snps.vcf.gz known SNP VCF; -p 16 threads; --minMAF 0.05 minimum allele frequency; --minCOUNT 10 minimum count; --UMItag None counts reads instead of UMIs; useful for bulk RNA-seq or when UMI information is not available

### use sample list for multi-sample bulk analysis
**Args:** `-S bam_list.txt -O multi_sample_out -R snps.vcf.gz -p 16 --minMAF 0.1 --minCOUNT 20 -i sample_ids.txt`
**Explanation:** -S bam_list.txt file listing BAM files; -O multi_sample_out output directory; -R snps.vcf.gz known SNP VCF; -p 16 threads; --minMAF 0.1 minimum allele frequency; --minCOUNT 20 minimum count; -i sample_ids.txt file with corresponding sample IDs; alternative to comma-separated -s
