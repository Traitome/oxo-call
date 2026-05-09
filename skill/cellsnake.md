---
name: cellsnake
category: genomics/sequence-analysis
description: cellsnake is a bioinformatics pipeline for converting and processing cellSNP-format sparse matrices and genotypes from single-cell RNA-seq (scRNA-seq) data. It provides tools for generating sparse genotype matrices, computingallelic counts, and producing input files for downstream tools such as Vireo for CRISPR screening or cell fraction inference.
tags:
  - single-cell
  - scRNA-seq
  - genotype-calling
  - sparse-matrix
  - allele-counting
  - vireo-input
  - crispr-screening
author: AI-generated
source_url: https://github.com/sINGLECellLab/cellsnake
---

## Concepts

- cellsnake operates on sorted BAM/SAM files tagged with cell barcodes (CB tags) and generates sparse matrices of observed alleles at genomic positions, using the cellSNP output format where rows are SNPs, columns are cells, and values are allele counts.
- Input to cellsnake typically requires a SAM/BAM file (aligned to a reference genome), a list of target SNP positions (VCF file), and a whitelist of valid cell barcodes; the pipeline emits both a sparse matrix (`.mtx`) and a TSV file mapping SNPs to genomic coordinates.
- The main workflow consists of two phases: a `count` phase that tallies reads supporting reference and alternative alleles per cell per SNP, and a `sparse` phase that filters low-coverage cells and SNPs and writes the results in HDF5 or market-matrix format for downstream consumption.
- cellsnake can process both single-sample and pooled multiplexed experiments; for multiplexed runs it requires a `samplesheet` CSV defining which barcodes belong to which sample so that cross-sample contamination is minimized during allele counting.
- Output file naming follows predictable conventions — for example `cellsnake.sparse.h5` for the HDF5 sparse matrix and `cellsnake.samples.tsv` for the per-cell sample assignment — and downstream tools like Vireo expect these exact filenames.

## Pitfalls

- Using a VCF file with alleles that do not match the reference genome version used to align the BAM file will cause allele parsing to silently assign wrong alleles; this leads to downstream genotype calls being dominated by mismatched positions and severely reduced signal in Vireo.
- If the cell barcode whitelist contains barcodes absent from the BAM file (e.g., derived from a different sequencing run or library), cellsnake will emit an output matrix with a very high proportion of zero-count cells, inflating output file size and wasting downstream compute time in Vireo.
- cellsnake's memory footprint scales with the number of reads covering SNP positions; for whole-genome SNP panels with thousands of positions and hundreds of thousands of cells, insufficient RAM will cause OOM kills, so monitoring `--max-records` or batching the BAM by chromosome is necessary.
- Misconfiguring the `--min-counts` threshold (setting it too low) allows ambient RNA contamination to register as real allele counts, causing false-positive genotype assignments; setting it too high discards genuine low-coverage cells, leading to underrepresentation of heterozygous sites.
- Running cellsnake on an unsorted BAM file without the `--sort` pre-step produces out-of-order read groups that cause the pipeline to skip or misassign reads to cells, resulting in missing or duplicate allele counts in the sparse matrix.

## Examples

### Count allele reads in a BAM file against a target VCF for a single sample
**Args:** `count --bam aligned.bam --vcf snps.vcf.gz --barcode-tag CB --out-dir output/`
**Explanation:** This runs the allele counting phase using the barcode tag CB from the BAM file, writing per-SNP per-cell counts into the output directory for downstream sparse-matrix construction.

### Generate a filtered sparse matrix from raw counts with cell and SNP coverage filters
**Args:** `sparse --count-dir output/ --min-cells 10 --min-reads 3 --out matrix.h5`
**Explanation:** This applies minimum coverage thresholds across all cells and SNPs, removes low-quality entries, and exports the resulting filtered sparse matrix in HDF5 format.

### Process a multiplexed experiment using a samplesheet to demultiplex cells before allele counting
**Args:** `count --bam multiplexed.bam --vcf snps.vcf.gz --samplesheet samples.csv --out-dir multi_out/`
**Explanation:** This assigns each barcode to its respective sample using the samplesheet, so that allele counts are stored per-sample rather than merged, which is required for Vireo CRISPR multiplexing workflows.

### Limit memory usage by splitting BAM processing by chromosome
**Args:** `count --bam large.bam --vcf snps.vcf.gz --chrom-list chr1 chr2 chr3 --out-dir chrom_split/`
**Explanation:** This restricts allele counting to a subset of chromosomes, reducing per-job memory consumption and enabling parallelization across chromosome-aware cluster jobs.

### Export sparse matrix in Market Matrix (.mtx) format for compatibility with downstream Python tools
**Args:** `sparse --count-dir output/ --format mtx --out matrix_exports/`
**Explanation:** This converts the internal HDF5 sparse representation into the text-based Market Matrix format with separate `.mtx`, `.barcodes.tsv`, and `.features.tsv` files, enabling ingestion by scanpy or Seurat without custom HDF5 parsers.