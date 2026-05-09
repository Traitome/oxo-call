---
name: cnvpytor
category: variant-calling/cnv
description: Copy Number Variation (CNV) calling from read depth and allele frequency analysis of aligned BAM/CRAM sequencing data.
tags: [cnv, read-depth, copy-number, snp, baf, segmentation]
author: AI-generated
source_url: https://github.com/abyzovlab/CNVpytor
---

## Concepts

- **Binary .pytor format**: CNVpytor stores all analysis data (read depth, BAF, SNVs, masks) in a single `.pytor` file organized by chromosome and genomic bin sizes (100bp, 500bp, 1kb, 5kb). This file is the primary input for all subsequent commands and supports incremental updates.
- **Two-signal CNV calling**: CNVpytor combines read depth (RD) signal with B-allele frequency (BAF) from heterozygous SNPs to distinguish between deletions, duplications, and copy-neutral events (e.g., absence of heterozygosity). Both signals are required for accurate CNV classification.
- **Two-pass BAM processing**: The `rd` command performs two passes over the BAM file — first for read depth calculation at each bin, then for GC bias correction. This ensures accurate normalization but means BAM files must be sorted and indexed beforehand.
- **Interactive Jupyter integration**: CNVpytor provides Jupyter magic commands (`%%cnvpytor`) and an interactive viewer (`cnvpytor -view`) for visual CNV inspection, enabling manual curation and refinement of called regions before downstream analysis.
- **Reference panel support**: When a VCF with known SNPs is supplied via `-refpanel`, CNVpytor uses allele frequencies from population variation to improve sensitivity for de novo CNV detection in family-trio analyses.

## Pitfalls

- **Forgetting to index BAM/CRAM files**: Running CNVpytor on unsorted or unindexed BAM files silently produces empty read depth output or crashes during the two-pass read. Always ensure BAM files are coordinate-sorted and indexed with `samtools index` before analysis.
- **Mismatched bin size between commands**: Creating a `.pytor` file with 1kb bins then requesting segmentation at 100bp causes errors or returns no calls. Bin sizes must be consistent across all analysis steps; specify the same bin size when invoking `-bin` for both `rd` and `call` commands.
- **Ignoring sex chromosome normalization**: By default, CNVpytor treats chrX and chrY using the same thresholds as autosomes. For male samples, this leads to spurious chrX deletions or female samples appearing as duplications. Explicitly set `-XY` (male) or `-XX` (female) when running `rd` and `call`.
- **Insufficient SNV density for BAF analysis**: CNVpytor requires ≥1 heterozygous SNP per 10 bins for reliable BAF segmentation. In low-coverage or low-divergence samples, the BAF signal will be sparse, causing false CNV boundaries or complete failure of `baf` segmentation.
- **Overwriting .pytor files during batch processing**: Using identical output names in scripts processes samples sequentially but corrupts data when run in parallel. Always use unique file names per sample or implement file locking when batch processing.

## Examples

### Initialise a CNVpytor session from a BAM file at 1kb bin resolution
**Args:** `-root sample.pytor -bam sample.bam -fasta reference.fa -bin 1000`
**Explanation:** Creates a new `.pytor` file, partitions the BAM into 1kb genomic bins, and calculates raw read depth statistics for all chromosomes in the reference FASTA.

### Segment called CNVs from precomputed read depth using RD signal only
**Args:** `-root sample.pytor -call 1000 rd_segmentation`
**Explanation:** Performs read depth segmentation at 1kb resolution using only the read depth signal, outputting regions with copy number gain or loss relative to the median depth.

### Import pre-existing SNV VCF file to enable BAF analysis
**Args:** `-root sample.pytor -vcf snvs.vcf.gz`
**Explanation:** Loads SNV/indel calls from a bgzip-compressed VCF file into the `.pytor` file, enabling joint RD+BAF CNV calling and LOH detection in downstream commands.

### Generate read depth histogram across all chromosomes at 5kb resolution
**Args:** `-root sample.pytor -his 5000`
**Explanation:** Calculates and stores the read depth distribution histogram for all chromosomes binned at 5kb resolution, which is used for visualizing coverage variance and identifying outlier regions.

### Call CNVs using joint read depth and BAF segmentation with Gaussian mixture model
**Args:** `-root sample.pytor -call 1000 baf_segmentation joint_germline`
**Explanation:** Executes combined RD+BAF segmentation at 1kb bins using a Gaussian mixture model to classify CNV types (deletion, duplication, copy-neutral LOH), suitable for germline CNV analysis.

### Export called CNV regions as a BED file for downstream annotation
**Args:** `-root sample.pytor -o cnv_calls.bed`
**Explanation:** Writes all previously called CNV regions from the `.pytor` file to a standard BED format file, preserving chromosome, start, end, and copy number ratio columns for use in UCSC or IGV visualization.