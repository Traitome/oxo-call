---
name: afplot
category: variant-analysis
description: A command-line tool for plotting allele frequency spectra from genetic variant data (VCF, BCF, or PLINK formats). Computes minor allele frequencies and generates publication-quality visualizations including histograms, cumulative distribution plots, and folded spectra.
tags: [bioinformatics, genetics, allele-frequency, VCF, visualization, variant-analysis]
author: AI-generated
source_url: https://github.com/smtha-tools/afplot
---

## Concepts

- **Input formats**: afplot accepts VCF, BCF, and PLINK PED/MAP files. For VCF/BCF, it either reads sample genotypes directly or uses pre-computed allele frequency annotations in the AF/INFO fields. The tool automatically detects the input format from the file extension.

- **Allele frequency computation**: When no AF annotation exists, afplot calculates allele frequencies by counting alternate alleles across all samples (AN = total alleles, AC = alternate count). Use `--field AF` to specify an existing INFO field containing pre-computed frequencies.

- **Output formats and options**: Generated plots support PNG, PDF, SVG, and EPS formats via `--format`. The tool can overlay multiple datasets using `--input` multiple times, with automatic legend generation. Customization includes bin counts (`--bins`), axis labels (`--xlabel`, `--ylabel`), and figure dimensions (`--width`, `--height`).

- **Filtering variants**: Apply `--min-af` and `--max-af` thresholds to exclude rare or common variants. Use `--chrom` to restrict analysis to specific chromosomes. The `--keep` and `--exclude` flags accept sample ID lists for cohort-specific frequency calculations.

- **Statistical overlays**: Add population genetics statistics to plots using `--add-stats`, which computes and displays Tajima's D, nucleotide diversity (π), and theta estimates. These overlay on the primary frequency histogram.

## Pitfalls

- **Missing genotype fields in VCF**: Running afplot on a VCF without FORMAT/GT fields or an AF annotation results in empty plots. Always verify your VCF contains genotype data (`FORMAT/GT`) or annotate with bcftools +fill-tags before plotting. Result: silent failure with zero-length output files.

- **Mismatched reference alleles**: If the allele column in your VCF does not match the genome build used for sample names, frequencies compute incorrectly. This produces misleading frequency distributions shifted toward specific allele classes. Always validate consistency between your VCF reference and sample population.

- **Non-numeric allele frequencies**: Some VCFs store AF as a string or multiple comma-separated values per alternate allele. Using `--field AF` on such files causes parsing errors. Use `--field AC,AN` instead to compute frequencies from counts, or preprocess with vcftools to clean annotations.

- **Memory exhaustion with large files**: Processing whole-genome VCFs with millions of variants without filtering (`--chrom`, `--min-af`) consumes excessive RAM. The tool may crash or hang on systems with limited memory. Apply strict frequency or chromosome filters before plotting to keep the dataset manageable.

- **Conflicting sample IDs**: When overlaying multiple VCF files, duplicate sample names cause frequency calculations to fail or produce incorrect results. Ensure each input file has unique sample identifiers or use `--sample-prefix` to disambiguate before merging.

## Examples

### Plot minor allele frequency spectrum from a VCF file
**Args:** `--input variants.vcf --plot maf --format png --output maf_spectrum.png`
**Explanation:** Generates a minor allele frequency histogram from genotypes in the input VCF, saved as a PNG image. Minor allele frequency is derived automatically from genotype counts.

### Compute and visualize allele frequency from AC/AN annotations
**Args:** `--input annotated.vcf --field AC,AN --plot af --bins 50 --format pdf`
**Explanation:** Uses pre-computed alternate allele counts (AC) and total allele numbers (AN) from the INFO fields to calculate allele frequencies, plotting them as a frequency distribution with 50 bins.

### Overlay frequency spectra from two population datasets
**Args:** `--input population_A.vcf --input population_B.vcf --plot folded --add-legend --format svg`
**Explanation:** Combines two VCF files to display folded allele frequency spectra on a single plot with an auto-generated legend distinguishing the populations.

### Filter rare variants and plot the distribution
**Args:** `--input rare_variants.vcf --min-af 0.01 --max-af 0.05 --plot hist --xlabel "Allele Frequency" --ylabel "Variant Count"`
**Explanation:** Restricts analysis to variants with minor allele frequencies between 1% and 5%, then creates a histogram with custom axis labels.

### Generate a cumulative distribution plot for publication
**Args:** `--input snps.vcf --plot cumulative --width 8 --height 6 --format pdf --title "Cumulative MAF Distribution"`
**Explanation:** Creates an 8x6 inch PDF with a cumulative minor allele frequency curve and a custom title, suitable for inclusion in published manuscripts.

### Add population genetics statistics overlay to frequency histogram
**Args:** `--input whole_genome.vcf --plot af --add-stats --format png --output with_stats.png`
**Explanation:** Computes Tajima's D, nucleotide diversity, and theta from the variant data and overlay these statistics as text annotations on the allele frequency histogram.

### Restrict analysis to autosomes only
**Args:** `--input WGS.vcf --chrom 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22 --plot maf`
**Explanation:** Filters the input to include only standard autosomes (1-22), excluding sex chromosomes and mitochondrial DNA from the frequency calculation.