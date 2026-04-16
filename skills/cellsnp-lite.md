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
**Explanation:** -s BAM; -b cell barcodes file; -O output dir; -R known SNP VCF; -p threads; --minMAF and --minCOUNT for quality filter

### pileup SNPs in a bulk BAM without cell barcodes
**Args:** `-s bulk.bam -O bulk_snp_out -R common_snps.vcf.gz -p 16 --minMAF 0.05 --minCOUNT 10`
**Explanation:** Mode 1a: no -b flag for bulk; -R provides known SNP positions; output has total AD/DP counts per SNP

### de novo SNP discovery in single-cell BAM (Mode 2)
**Args:** `-s possorted_genome_bam.bam -b barcodes.tsv -O denovo_snp_out -p 16 --minMAF 0.1 --minCOUNT 100 --gzip`
**Explanation:** Mode 2: no -R flag; discovers SNPs from the data; --gzip compresses output matrix files; higher --minCOUNT for quality

### pileup multiple BAMs from different samples at shared SNP positions
**Args:** `-s sample1.bam,sample2.bam,sample3.bam -O multi_sample_out -R common_snps.vcf.gz -p 16 --minMAF 0.1 --minCOUNT 20`
**Explanation:** comma-separated BAM list for multi-sample bulk pileup; useful for genotyping donors for Vireo demultiplexing

### restrict pileup to specific chromosomes to reduce runtime
**Args:** `-s possorted_genome_bam.bam -b barcodes.tsv -O chr1_out -R chr1_snps.vcf.gz --chrom 1 -p 8 --minMAF 0.1 --minCOUNT 20`
**Explanation:** --chrom 1 restricts to chromosome 1; useful for testing or chromosome-level parallel jobs

### pileup with strict base quality filter for high-confidence allele counts
**Args:** `-s possorted_genome_bam.bam -b barcodes.tsv -O hq_out -R snps.vcf.gz -p 16 --minMAF 0.1 --minCOUNT 20 --minMAPQ 30`
**Explanation:** --minMAPQ 30 filters low-quality alignments; cellsnp-lite does not have --minBQ flag; use --minMAPQ and --minLEN for read filtering

### genotype cells in addition to counting alleles
**Args:** `-s possorted_genome_bam.bam -b barcodes.tsv -O genotyped_out -R snps.vcf.gz -p 16 --minMAF 0.1 --minCOUNT 20 --genotype --gzip`
**Explanation:** --genotype outputs genotype probabilities (GP) and genotypes (GT) in addition to allele counts; --gzip compresses output files

### use streaming mode for large SNP panels
**Args:** `-s possorted_genome_bam.bam -b barcodes.tsv -O streaming_out -T large_panel.vcf.gz -p 16 --minMAF 0.1 --minCOUNT 20`
**Explanation:** -T uses streaming instead of indexed access; better for large SNP panels (millions of SNPs) where -R would be slow

### pileup without UMI counting (read-level)
**Args:** `-s bulk.bam -O read_counts_out -R snps.vcf.gz -p 16 --minMAF 0.05 --minCOUNT 10 --UMItag None`
**Explanation:** --UMItag None counts reads instead of UMIs; useful for bulk RNA-seq or when UMI information is not available

### use sample list for multi-sample bulk analysis
**Args:** `-S bam_list.txt -O multi_sample_out -R snps.vcf.gz -p 16 --minMAF 0.1 --minCOUNT 20 -i sample_ids.txt`
**Explanation:** -S provides a file listing BAM files; -i provides corresponding sample IDs; alternative to comma-separated -s
