---
name: binspreader
category: Genomics / Metagenomics
description: A command-line tool for manipulating and processing genomic bins from metagenomic assembly data. binspreader enables extraction, filtering, and conversion of bin information for downstream analysis in microbial ecology studies.
tags:
  - bins
  - metagenomics
  - assembly
  - genomics
  - microbial-ecology
author: AI-generated
source_url: https://github.com/binspreader/binspreader
---

## Concepts

- **Bin File Formats**: binspreader accepts standard genomic bin formats including FASTA (.fa, .fna), tabular bin lists (.txt, .tsv), and coverage profiles (.cov, .depth). Each bin is identified by a unique identifier that must be consistent across input files.
- **Data Model**: The tool operates on a bin-centric data model where each genomic bin contains sequence data, coverage information, and quality scores. Bins can be filtered by minimum sequence length (default: 1000 bp), minimum completeness (default: 50%), and maximum contamination (default: 10%).
- **Input/Output Behavior**: binspreader processes input in streaming mode for large datasets and writes output sequentially. The tool preserves the original bin naming convention and appends processed suffixes unless explicitly renamed via the `--output-prefix` flag.
- **Multi-file Operations**: When processing multiple bin files, binspreader performs pairwise comparisons by default. Use `--parallel` flag to enable concurrent processing, which significantly reduces runtime on multi-core systems.

## Pitfalls

- **Mismatched Bin Identifiers**: If input bin files use different naming schemes, the tool will fail silently or produce empty output. Always verify bin IDs match across all input files using a preliminary check script before running binspreader.
- **Insufficient Memory for Large Bins**: Loading oversized genomic bins (>500 Mb) without specifying `--chunk-size` can cause memory allocation failures. Monitor system RAM and adjust chunk size to prevent process termination.
- **Incorrect File Permissions**: Attempting to write output to read-only directories or protected paths results in silent failures where no error message is displayed but the output file is not created. Always verify write permissions beforehand.
- **Ignoring Quality Thresholds**: Running binspreader without setting appropriate completeness/contamination thresholds produces low-quality bins that may contaminate downstream analysis. Default thresholds are overly permissive for high-quality metagenome-assembled genomes.

## Examples

### Extract bins meeting minimum completeness threshold
**Args:** `--input-bins assembly_bins.txt --min-completeness 90 --output-prefix high_quality_`
**Explanation:** Filters and outputs only bins with at least 90% completeness, useful for selecting high-quality metagenome-assembled genomes.

### Convert FASTA bins to coverage profile format
**Args:** `--input-fasta bins_directory/ --output-format cov --output-dir coverage_output/`
**Explanation:** Transforms genomic bin sequences into coverage profile format for downstream abundance analysis.

### Merge overlapping bins from co-assembly
**Args:** --input-bins combined_bins.txt --merge --overlap-threshold 0.85 --output-prefix merged_
**Explanation:** Combines bins with more than 85% overlap into single representative bins, reducing redundancy in final assemblies.

### Filter bins by sequence length
**Args:** --input-fasta genome_bins.fa --min-length 5000 --max-length 500000 --output-dir filtered/
**Explanation:** Retains only bins with sequence lengths between 5 kb and 500 kb, removing short contigs and excessively large scaffolds.

### Generate summary statistics for all bins
**Args:** --input-bins all_bins.txt --stats --output-file bin_statistics.tsv
**Explanation:** Produces a tabular summary including GC content, sequence count, N50, and quality metrics for each bin in the input set.

### Run parallel processing with 8 threads
**Args:** --input-bins large_collection/ --parallel --threads 8 --output-dir parallel_output/
**Explanation:** Enables multi-threaded processing to accelerate bin manipulation on large datasets, reducing total runtime significantly.

### Export bins in BED format for genomic visualization
**Args:** --input-fasta my_bins.fa --export-format bed --output-file bins_visualize.bed
**Explanation:** Converts bin sequences to BED format for visualization in genome browsers or integrative analysis platforms.