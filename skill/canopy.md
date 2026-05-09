---
name: canopy
category: Bioinformatics/Metagenomics
description: A software package for amplicon sequencing analysis including OTU clustering, taxonomy annotation, and microbial diversity estimation from 16S rRNA, ITS, and other marker gene sequences.
tags: [amplicon, metagenomics, Microbiome, OTU-clustering, taxonomy, diversity-analysis, 16S-rRNA, ITS]
author: AI-generated
source_url: https://github.com/codingforfermentation/canopy
---

## Concepts

- **Data Model**: Canopy operates on demultiplexed FASTQ or FASTA files containing marker gene sequences (16S rRNA, ITS, etc.) and produces OTU tables, representative sequences, and taxonomy assignments as primary data outputs.
- **Input Formats**: Accepts standard sequencing formats including FASTQ (with quality scores), FASTA (sequence only), and manifest files mapping samples to files; sequences must be pre-processed (filtered, trimmed) for optimal performance.
- **Key Behaviors**: The clustering engine uses a choice of de novo (UCLUST, USEARCH) or reference-based (CLOSOT) methods with configurable similarity thresholds (default 97% for species-level OTUs); abundance filtering removes singletons and low-coverage sequences to reduce false positives.
- **Companion Binaries**: canopy-build constructs custom reference databases for taxonomy annotation; canopy-index creates searchable indexes for rapid sample assignment; both accept FASTA/CSV reference files.
- **Output Formats**: Generates BIOM-format OTU tables for compatibility with QIIME2, representative sequence FASTA files, taxonomy TSV files with confidence scores, and summary statistics in JSON format.

## Pitfalls

- **Skipping Quality Filtering**: Running canopy on unfiltered FASTQ files with low-quality bases produces spurious OTUs from sequencing errors, inflating diversity estimates and creating artifactual clusters that dominate rarefaction curves.
- **Misconfiguring Similarity Thresholds**: Using 97% for all marker genes is inappropriate—ITS regions require 85-90% thresholds due to higher intra-species variability, while 16S typically needs 97-99%; wrong thresholds create over-clustered or under-clustered OTUs.
- **Ignoring Sample Nomenclature**: Canopy relies on consistent sample naming across input files; mismatched names between manifest and sequence headers cause samples to be dropped silently, leaving incomplete OTU tables that appear valid but lack expected samples.
- **Reference Database Mismatch**: Using a reference database built for bacteria to annotate fungal ITS sequences (or vice versa) results in zero taxonomy annotations despite high-quality OTUs, because the database lacks matching sequences.

## Examples

### Cluster sequences into OTUs using de novo method
**Args:** --input sequences.fasta --output otus --method denovo --similarity 0.97 --min-length 200
**Explanation:** This performs de novo clustering at 97% similarity, filtering sequences shorter than 200bp to remove fragments and primer dimers.

### Build custom taxonomy reference database
**Args:** canopy-build --reference ref_seqs.fasta --taxonomy tax_mapping.csv --output canopyDB --format csv
**Explanation:** Constructs an indexed reference database from FASTA sequences paired with taxonomy mappings in CSV format for use in annotation.

### Annotate OTU representative sequences
**Args:** canopy-annotate --input otus/representatives.fasta --reference canopyDB --output taxonomy.tsv --confidence 0.8
**Explanation:** Assigns taxonomy to representative sequences using the custom database, keeping only annotations with 80% or higher confidence.

### Calculate alpha diversity metrics
**Args:** canopy-diversity --input otus/table.biom --metric shannon --output alpha_div.tsv --rarefy-depth 1000
**Explanation:** Computes Shannon diversity index across samples, rarefying to 1000 sequences per sample to standardize comparisons.

### Generate beta diversity distance matrix
**Args:** canopy-diversity --input otus/table.biom --metric bray-curtis --output beta_div.tsv --transpose
**Explanation:** Calculates Bray-Curtis dissimilarity between samples, outputting a distance matrix suitable for PCoA or clustering analysis.

### Filter OTUs by abundance to remove noise
**Args:** canopy-filter --input otus/table.biom --min-abundance 10 --min-samples 3 --output filtered.biom
**Explanation:** Removes OTUs appearing fewer than 10 times in fewer than 3 samples, reducing false clusters while preserving rarefaction depth.

### Process multiplexed FASTQ with sample mapping
**Args:** canopy --input manifest.txt --output processed --demultiplex --format fastq --threads 8
**Explanation:** Demultiplexes sequences using sample mapping file, processing in 8 parallel threads for speed.