---
name: bio-ting
category: Bioinformatics / Sequence Analysis
description: A bioinformatics tool for extracting mathematical features from biological sequences to enable machine learning applications in genomics. Computes k-mer frequencies, GC content, codon usage, and derived statistical properties from DNA, RNA, or protein sequences.
tags:
- sequence-analysis
- feature-extraction
- machine-learning
- k-mers
- genomics
- bioinformatics
- dna-analysis
- rna-analysis
author: AI-generated
source_url: https://github.com/bio-ting/bio-ting
---

## Concepts

- **Input formats**: bio-ting accepts FASTA (.fa, .fasta), FASTQ (.fq, .fastq), and plain text (.txt) formats. Sequences can be DNA, RNA, or protein. Multi-sequence files are processed sequentially with one feature vector output per sequence.
- **K-mer counting**: The tool counts all possible k-length words in sequences using an efficient hash-based algorithm. K-mer sizes from 1 to 10 are supported. Frequencies are normalized by total k-mer count to produce probability distributions.
- **Output structure**: Results are output as tab-separated values with one row per input sequence. Columns include sequence identifier, length, GC content, k-mer frequency vectors, and derived statistics. Use `--output` to specify a file, otherwise stdout is used.
- **Companion binary**: bio-ting-build builds custom k-mer dictionaries from reference sequences. This dictionary can be passed to bio-ting with `--dict` to compute only specified k-mers, enabling consistent feature spaces across multiple datasets.

## Pitfalls

- **Mismatched sequence type**: Running DNA-specific analysis (e.g., codon usage with `--codon`) on protein sequences produces nonsensical output because protein amino acids cannot be interpreted as codons. Always verify your input sequence type matches the analysis mode.
- **Large k-mer sizes on short sequences**: Setting k > 6 with `--kmer` on sequences shorter than k produces zero counts for all k-mers. This results in feature vectors of all zeros, which degrades machine learning model training. Use k ≤ 6 for typical gene sequences (~100-1000 bp).
- **Memory exhaustion with large files**: Computing k-mer frequencies for entire genomes without using `--chunk` creates enormous hash tables. A human genome (~3 Gbp) with k=3 can require 50+ GB RAM. Process large genomes in chunks or use bio-ting-build to create a dictionary first.

## Examples

### Compute k-mer frequencies for a single FASTA sequence

**Args:** `--input sequences.fasta --kmer 3 --output kmer3.tsv`

**Explanation:** This extracts all 3-mers (trimers) from the input sequences and outputs their normalized frequencies, which are commonly used as encoding features for DNA classification tasks.

### Calculate GC content across multiple sequences

**Args:** `--input genes.fasta --gc --output gc_content.tsv`

**Explanation:** GC content is a fundamental genomic property that correlates with genome stability and gene density. This outputs the percentage of Guanine and Cytosine bases per sequence.

### Use a pre-built dictionary for consistent features

**Args:** `--input query.fa --dict custom_kmer.dict --output features.tsv`

**Explanation:** Using bio-ting-build created dictionaries ensures identical feature columns across all samples, which is required when combining feature matrices from multiple analyses for downstream machine learning.

### Process sequences in chunks for memory efficiency

**Args:** `--input genome.fa --kmer 4 --chunk 10000000 --output chunked_kmer.tsv`

**Explanation:** Chunking divides long sequences into windows, computing features separately for each chunk and outputting one row per chunk. This prevents memory exhaustion when processing whole chromosomes.

### Extract codon usage from coding sequences

**Args:** `--input cds.fasta --codon --output codon_usage.tsv`

**Explanation:** Codon usage patterns reflect species-specific translational efficiency and are valuable features for predicting gene expression levels or identifying highly expressed genes.