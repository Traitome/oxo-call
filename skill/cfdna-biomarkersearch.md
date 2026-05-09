---
name: cfdna-biomarkersearch
category: Genomics / Sequence Analysis
description: A bioinformatics tool for searching and identifying cell-free DNA (cfDNA) biomarkers from high-throughput sequencing data. Supports sequence alignment, motif discovery, and variant annotation for liquid biopsy applications.
tags:
  - cfDNA
  - biomarker
  - liquid-biopsy
  - genomics
  - sequence-analysis
  - variant-calling
author: AI-generated
source_url: https://github.com/cfdna-tools/cfdna-biomarkersearch
---

## Concepts

- **Input Formats:** The tool accepts raw sequencing reads in FASTQ format, aligned reads in BAM format, and variant calls in VCF format. It also supports BED files for genomic regions of interest.
- **Indexing:** Before searching, a reference genome must be indexed using the companion tool `cfdna-biomarkersearch-build`. The index enables fast sequence retrieval and alignment-free searching of potential biomarkers.
- **Output Formats:** Results are produced in multiple formats including TSV (tabular), JSON (machine-readable), and VCF (variant call format) depending on the search mode. Summary statistics are provided in a companion log file.
- **Paired-end Support:** When processing paired-end reads, the tool correctly handles inner mate distances and fragment length distributions, which are critical for cfDNA fragmentomics analysis.

## Pitfalls

- **Using unindexed references:** Attempting to search against a reference genome without first building an index with `cfdna-biomarkersearch-build` will cause the tool to fail or produce incorrect results due to lack of efficient sequence lookup structures.
- **Mismatched read lengths:** Specifying a mismatch tolerance that exceeds the read length will filter out all reads, producing empty output files. Always ensure the mismatch threshold is less than the read length.
- **Ignoring fragment length filtering:** cfDNA has a characteristic fragment length distribution (~166 bp peaks). Failing to filter reads by fragment length before biomarker search will include fragmented genomic DNA contamination and reduce thebiological signal.
- **Overwriting output files:** Running the tool with the same output prefix without moving previous results will silently overwrite existing files, causing data loss in batch processing workflows.
- **Incompatible coordinate systems:** Mixing zero-based and one-based coordinate systems between input BED files and reference coordinates will result in off-by-one errors in biomarker positions.

## Examples

### Search for known biomarkers in a FASTQ file
**Args:** -i sample.fastq -r hg38.fa -o biomarker_hits.tsv --search-mode known
**Explanation:** Searches for pre-defined biomarker sequences in the input FASTQ reads against the indexed hg38 reference genome.

### Build an index for a reference genome
**Args:** build -r GRCh38.fa -o hg38_idx/
**Explanation:** Creates a searchable index of the GRCh38 reference genome using the companion build tool, enabling fast subsequent biomarker searches.

### Discover novel motifs with customizable length
**Args:** -i sample.bam -r hg38.fa -o novel_motifs.tsv --search-mode discover -k 6 --min-freq 0.05
**Explanation:** Discovers new 6-bp motifs present in at least 5% of cfDNA fragments, outputting novel candidate biomarkers.

### Filter by fragment length before search
**Args:** -i sample.fastq -r hg38.fa -o filtered_hits.tsv --min-frag-length 100 --max-frag-length 200
**Explanation:** Filters input reads to the typical cfDNA fragment length range before searching, improving biomarker specificity.

### Export results in VCF format for downstream analysis
**Args:** -i sample.bam -r hg38.fa -o variants.vcf --output-format vcf --annotate
**Explanation:** Exports identified biomarkers as variant calls in VCF format with annotation tracks suitable for integration with other genomics tools.

### Use paired-end mode for improved mapping
**Args:** -i sample_R1.fastq -i2 sample_R2.fastq -r hg38.fa -o pe_hits.tsv --paired-end --mate-inner-dist 50
**Explanation:** Processes paired-end reads with specified inner mate distance, enabling more accurate alignment especially for short cfDNA fragments.