---
name: biobloomtools
category: Read Classification
description: A Bloom filter-based bioinformatics tool for fast classification, filtering, and membership testing of sequencing reads against indexed reference sequences. Optimized for metagenomics and taxonomic profiling workflows.
tags: [bloom-filter, classification, metagenomics, k-mer, fastq, fasta, taxonomy, sequence-analysis]
author: AI-generated
source_url: https://github.com/biobloomtools/biobloomtools
---

## Concepts

- **Bloom Filter Indexing**: Use `biobloomtools-build` to create a bloom filter index from one or more reference FASTA/FASTQ files. The index stores k-mers from all reference sequences and enables fast O(1) membership queries without storing the full sequences.
- **Read Classification**: The `biobloomtools-filter` (or `biobloomtools-mapprofile`) subcommand classifies input reads by querying the bloom filter index. Each read is classified based on the proportion of its k-mers matching the reference, with configurable threshold parameters.
- **K-mer Size Critical**: The `-k` parameter defines the k-mer size used for both building the filter and classification. This must be consistent between index creation and filtering—typical values range from 20-64 bp depending on read length and specificity requirements.
- **Multiple Reference Support**: Build bloom filters from multiple reference sequences (e.g., different species or strains) by providing multiple input files or a directory. The tool distinguishes between references using header metadata and outputs classification scores per reference.
- **Output Formats**: Filtered outputs can be written in FASTQ, FASTA, or SAM/BAM format. The tool reports classification results including matched k-mer counts, coverage percentage, and assigned taxonomy labels.

## Pitfalls

- **Mismatched K-mer Size**: Using a different `-k` value when filtering than what was used to build the bloom filter will produce meaningless results—classification will fail to find any matches because k-mers are hash-computed with the specified size.
- **Insufficient Memory Allocation**: Bloom filter size (`-s` parameter, number of bits) that is too small relative to the total unique k-mers in references causes high false positive rates, leading to incorrect classifications. The tool may warn about this but still run.
- **Wrong File Formats**: Attempting to use compressed `.fastq.gz` or `.fq.gz` files without first decompressing them causes errors—BioBloomTools requires uncompressed FASTQ/FASTA inputs unless explicitly configured with decompression handling.
- **Threshold Too Strict**: Setting the classification threshold (`-t` or `-f`) too high may reject valid reads that genuinely belong to references but have sequencing errors or partial matches, resulting in false negatives.
- **Missing Index Recalibration for New Data**: Reusing a pre-built bloom filter index for a different read dataset without verifying that the k-mer characteristics align can degrade classification accuracy significantly.

## Examples

### Build a bloom filter index from a single bacterial reference genome
**Args:** `-f ecoli_ref.fasta -o ecoli_bf -k 31 -s 100000000`
**Explanation:** Creates a bloom filter named `ecoli_bf` using k-mer size 31 (standard for Illumina reads) and allocates 100 million bits (~12.5 MB) of filter space from the E. coli reference genome file.

### Build a bloom filter from multiple viral genomes for metagenomics
**Args:** `-f Virus1.fasta -f Virus2.fasta -f Virus3.fasta -o viral_panel -k 25 -s 50000000`
**Explanation:** Combines three viral reference genomes into a single bloom filter index `viral_panel` with smaller k-mer size (25) for increased sensitivity to short viral reads and 50 million bits of allocated space.

### Filter and classify reads against a pre-built bloom filter
**Args:** `-d ecoli_bf -i reads.fastq -o classified_output -t 0.8`
**Explanation:** Queries the pre-built `ecoli_bf` bloom filter against input reads in FASTQ format, outputting results to `classified_output` and requiring at least 80% k-mer match rate for positive classification.

### Output classified reads with detailed statistics
**Args:** `-d viral_panel -i sample1.fastq -o sample1_classified -t 0.5 -S -g -F fastq`
**Explanation:** Uses a relaxed 50% threshold for classification, enables statistics output (`-S`), prints per-read classification details to stdout (`-g`), and writes matched reads in FASTQ format.

### Adjust sensitivity using a smaller threshold for noisy reads
**Args:** `-d bacterial_db -i noisy_sequences.fasta -o result -t 0.3 -m 10`
**Explanation:** Applies a low 30% k-mer match threshold to account for sequencing errors in the input, specifying minimum k-mer count of 10 (`-m`) to reduce spurious matches from low-complexity regions.