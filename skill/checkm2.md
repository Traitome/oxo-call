---
name: checkm2
category: Metagenome Assembly Quality Assessment
description: A deep learning tool for assessing completeness and contamination of metagenome-assembled genomes (MAGs). CheckM2 uses neural networks to predict genome quality from k-mer frequency profiles and gene content, providing more accurate estimates than traditional marker gene-based methods.
tags:
- metagenomics
- MAG
- quality assessment
- binning
- completeness
- contamination
- deep learning
- genome assembly
author: AI-generated
source_url: https://github.com/chklovski/CheckM2
---

## Concepts

- **Input Format**: CheckM2 accepts genome bins as FASTA files (containing contig sequences) placed in a single input directory. Each bin should be a separate file with sequences in FASTA format; the tool automatically identifies bins and processes them independently.
- **Prediction Model**: CheckM2 uses a deep neural network trained on k-mer frequencies and gene presence/absence patterns to predict completeness and contamination, requiring neither explicit marker gene sets nor reference databases for typical bins.
- **Output Columns**: The tool outputs completeness, contamination, and a combined quality score (completeness - 5×contamination) along with genome size, number of contigs, and N50 statistics for each bin analyzed.
- **Parallelization**: CheckM2 supports multi-threaded execution via the --threads flag, enabling faster processing of large bin sets; the tool can process multiple bins concurrently when sufficient threads are allocated.
- **Companion Binary**: The checkm2 package includes checkm2-build for building custom models, but for standard MAG quality assessment the main checkm2 binary is used directly.

## Pitfalls

- **Low-Quality Bins**: Bins with fewer than 10 contigs or genome sizes below 200 kbp may produce unreliable predictions because the neural network was trained on higher-quality genomes, leading to inaccurate completeness/contamination estimates.
- **Partial Genomes**: CheckM2's model is optimized for microbial genomes and may underperform on eukaryotic MAGs, viral contigs, or highly fragmented assemblies, resulting in completeness estimates biased toward lower values.
- **Memory Requirements**: Large bin sets processed with many threads can consume significant RAM; insufficient memory causes crashes or swapping that dramatically slows processing, particularly when input files are large.
- **File Permissions**: Writing output to directories without proper write permissions fails silently or produces errors; always verify write access to the output directory before running CheckM2.
- **Redundant Input**: Running CheckM2 on the same input directory multiple times without --force overwrites existing results, causing confusion about which version of results is current.

## Examples

### Assess genome quality for all bins in a directory

**Args:** -i ./bins -o ./checkm2_output

**Explanation:** This runs CheckM2 on all FASTA files in the ./bins directory and writes results to ./checkm2_output, automatically detecting all genome bins and predicting their completeness and contamination.

### Process bins with custom file extension

**Args:** -i ./bins -o ./results --extension .fna

**Explanation:** This processes only files with the .fna extension in the input directory, useful when the bin directory contains mixed file types or other file formats that should be excluded.

### Use multiple threads for faster processing

**Args:** -i ./bins -o ./results --threads 16

**Explanation:** This allocates 16 threads to process bins in parallel, significantly reducing runtime on multi-core systems when analyzing large bin sets; adjust thread count based on available CPU cores.

### Force overwrite existing results

**Args:** -i ./bins -o ./results --force

**Explanation:** This overwrites any existing output files in the results directory without prompting, necessary when re-running analysis with updated parameters or after fixing input data.

### Specify a custom temporary directory

**Args:** -i ./bins -o ./results --tmpdir /scratch/checkm2_tmp

**Explanation:** This uses /scratch/checkm2_tmp for temporary files during processing, useful when the default system temp directory has insufficient space or when running on cluster systems with dedicated scratch storage.