---
name: censtats
category: Bioinformatics/Genomics Statistics
description: Compute descriptive statistics for genomic features including central tendency measures (mean, median, mode), dispersion metrics (variance, standard deviation, quartiles), and distribution summaries for genomic coordinate files. Operates on BED, GTF, GFF, and chromosome length files to generate population-level statistics about feature lengths, densities, and genomic coverage.
tags:
  - genomics
  - statistics
  - genomic-features
  - summary-statistics
  - coverage-analysis
  - feature-lengths
author: AI-generated
source_url: https://github.com/placeholder/censtats
---

## Concepts

- **Input Format Flexibility**: Accepts standard genomic feature files (BED, GTF, GFF version 3) as primary input, with optional chromosome length files for normalization. Features are parsed by start/end coordinates and feature type annotation.

- **Central Tendency Metrics**: Computes weighted and unweighted mean, median, and mode of feature lengths across the entire input. For weighted calculations, each feature's contribution is scaled by its genomic span (end - start + 1).

- **Dispersion and Distribution Analysis**: Calculates variance, standard deviation, interquartile range (IQR), and provides histogram bins for feature length distributions. Outputs both raw counts and percentage-based frequency tables.

- **Coverage and Density Statistics**: When chromosome lengths are provided, computes feature density (features per Mb), total genomic coverage (percentage of genome spanned by features), and gaps per chromosome. Useful for assessing genome-wide annotation completeness.

- **Multi-feature-type Support**: Processes GTF/GFF files with multiple feature types (exon, gene, transcript, CDS) separately, producing stratified statistics for each feature class found in the input.

## Pitfalls

- **Zero-length Features**: Features where end coordinate equals start coordinate (or is less than start) produce NaN values for length-based statistics, causing downstream parsing errors in scripts that expect numeric output. Always validate coordinate ordering before analysis.

- **Mismatched Chromosome Naming**: Chromosome names in feature files must exactly match those in the chromosome length file (e.g., "chr1" vs "1"). Mismatches cause silent failures where features are excluded from all per-chromosome calculations without warning, leading to underestimated coverage.

- **Memory Consumption with Large Files**: Full file parsing loads all features into memory simultaneously. Files with >10 million features may cause memory exhaustion on systems with limited RAM, resulting in process termination without error output.

- **Incorrect Feature Type Filtering**: When processing GTF/GFF files without specifying feature types, all lines including non-coordinate lines (e.g., FASTA comments, metadata) are included in calculations, producing nonsensical statistics or parse failures.

## Examples

### Compute basic statistics for a BED file
**Args:** input.bed --output stats.txt
**Explanation:** Reads all features from input.bed, calculates mean/median/mode lengths and standard deviation, writes results to stats.txt for downstream analysis or reporting.

### Generate per-chromosome statistics with genome reference
**Args:** input.bed --genome genome.chrom.sizes --group-by chrom
**Explanation:** Groups feature statistics by chromosome using the provided chromosome length file, enabling comparative analysis across genomic compartments.

### Export histogram distribution data
**Args:** input.gtf --feature-type exon --histogram --bins 20 --output histogram.tsv
**Explanation:** Creates a 20-bin histogram of exon lengths from the GTF file, outputting frequency counts and percentage columns for visualization.

### Calculate coverage with normalization
**Args:** input.bed --genome genome.chrom.sizes --coverage --normalize
**Explanation:** Computes the percentage of each chromosome covered by features, normalizing by total chromosome length rather than feature count.

### Get density statistics per feature type
**Args:** input.gff --feature-type gene --density --genome genome.chrom.sizes
**Explanation:** Calculates gene density (genes per megabase) for each chromosome, useful for comparing gene richness across genomes.

### Output JSON format for programmatic use
**Args:** input.bed --format json --output stats.json
**Explanation:** Writes all computed statistics in JSON format, enabling automated pipelines to ingest results without custom parsing logic.