---
name: bart
category: epigenomics/sequence-analysis
description: A bioinformatics tool for building genomic indexes and analyzing bisulfite sequencing data. BART (Bisulfite Analysis and Recovery Tool) is commonly used in DNA methylation studies to create reference indexes and process aligned reads for epigenomic analysis.
tags:
  - methylation
  - epigenomics
  - index-building
  - bisulfite-sequencing
  - genomics
author: AI-generated
source_url: https://github.com/zhouwyl/BART
---

## Concepts

- **BART uses a companion binary `bart-build`** to create genomic indexes from FASTA reference files. The index enables efficient read alignment and k-mer lookup during subsequent analysis stages.
- **Input formats**: BART accepts FASTA files for index building and FASTQ/BAM for read analysis. For methylation data, it processes reads treated with bisulfite conversion where cytosines are converted to uracils (read as thymine).
- **Output formats**: BART generates binary index hash tables (`.bt2` extension by default) and may produce alignment SAM/BAM files or summary reports depending on the subcommand used.
- **K-mer length configuration**: The tool supports configurable k-mer sizes (typically -k 25 to -k 37) which affects memory usage and alignment sensitivity. Longer k-mers reduce memory but may miss divergent reads.

## Pitfalls

- **Using an incompatible reference genome version**: Providing a mismatched FASTA reference (e.g., hg38 index with hg19-aligned reads) produces meaningless alignments and corrupts downstream analysis results.
- **Setting k-mer length too short or too long**: A k-mer below 20 may cause excessive false-positive matches and memory exhaustion, while k-mers above 40 may fail to align legitimate reads with natural mutations.
- **Forgetting to index the reference before alignment**: Running alignment commands without a pre-built index yields errors or silently drops reads, wasting compute time on incomplete results.
- **Specifying wrong file formats for input**: Feeding FASTQ to a FASTA-required parameter or vice versa causes parsing failures and crashes the analysis pipeline.

## Examples

### Build a genomic index from a FASTA reference file
**Args:** bart-build -k 25 -t 8 reference.fa genome_index
**Explanation:** Creates a 25-mer index using 8 threads for parallel computation, producing the base index files needed for alignment.

### Build an index with 32-mer k-mers for higher specificity
**Args:** bart-build -k 32 -t 4 reference.fa genome_index_32
**Explanation:** Uses longer 32-mers which increase alignment specificity but require more memory, employing 4 parallel threads.

### Override the default output filename base
**Args:** bart-build -o custom_index_name reference.fa
**Explanation:** Stores the resulting index files under the custom basename rather than deriving names from the input filename.

### Reserve additionalthreads for parallel indexing
**Args:** bart-build -k 28 -t 16 reference.fa genome_index
**Explanation:** Allocates 16 CPU threads to accelerate the indexing step when working with large genomes or deep coverage datasets.

### Specify a memory-saver mode for large genomes
**Args:** bart-build --packed -k 30 reference.fa genome_index
**Explanation:** Uses packed memory representation to reduce RAM consumption at the cost of slightly slower lookup during alignment.