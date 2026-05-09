---
name: chromhmm
category: Epigenomics
description: A tool for learning and predicting chromatin states from epigenomic data using hidden Markov models. ChromHMM analyzes combinatorial patterns of histone modifications and other chromatin-associated factors to segment genomes into biologically meaningful states.
tags: chromatin-states, histone-modifications, hidden-markov-model, epigenomics, segmentation, chip-seq
author: AI-generated
source_url: https://compbio.mit.edu/ChromHMM/
---

## Concepts

- ChromHMM uses a multivariate hidden Markov model (HMM) to segment the genome into a user-specified number of chromatin states, where each state is defined by a distinct combination of histone modification enrichments.
- Input data must be provided as "tag directories" created by the companion binary `makeTagDirectory` from BAM or BED files, containing the genomic coordinates of aligned reads or peaks.
- The tool requires a genome chromosome sizes file (e.g., from UCSC) to define the chromosomal boundaries and can process multiple cell types jointly to learn consistent states across datasets.
- ChromHMM outputs several files including a BED segment file annotating each genomic bin with the most probable state, an emission parameter file describing each state's histone modification signature, and a transition matrix file.
- The number of states (`-p`) is a critical parameter: too few states may merge biologically distinct states, while too many may overfit and reduce interpretability.

## Pitfalls

- Forgetting to specify the bin size (`-bin`) results in using the default (which may be inappropriate for your genome and data resolution), leading to either fragmented states or overly coarse segmentation.
- Using reads without proper normalization or failing to provide control (input) datasets can cause the model to learn biases from experimental noise rather than genuine enrichment patterns.
- Attempting to learn too many states (>15-20) without sufficient input tracks often leads to unstable or uninterpretable states, as the HMM may split coherent biological states into sub-states with high variance.
- Not providing a genome chromosome sizes file (`-gc`) or providing an incorrect one causes the tool to fail or produce malformed output that cannot be mapped back to the reference genome.
- Running ChromHMM on low-quality or insufficiently deep ChIP-seq data yields states that reflect background noise rather than true chromatin biology, as the model cannot distinguish weak signals from artifact.

## Examples

### Learning a 5-state chromatin model from a single cell type

**Args:** `BinarizeSignal -gc hg19.chrom.sizes ./input_dir ./output_dir -b 200 makeTagDirectory input_dir/ H3K4me1.bam H3K4me3.bam H3K27ac.bam H3K4me2.bam H3K27me3.bam H3K9me3.bam` `LearnModel -p 5 ./output_dir/hg19_200kb_binarized.txt ./model_5state ./output_dir`

**Explanation:** This command first binarizes the ChIP-seq data into 200kb bins using the provided tag directory and genome sizes, then trains a 5-state HMM model on the binarized data to identify major chromatin categories.

### Generating a 15-state model for detailed epigenomic annotation

**Args:** `BinarizeSignal -gc hg38.chrom.sizes ./data ./binary_out -b 200 makeTagDirectory data/ H3K4me1.bam H3K4me3.bam H3K27ac.bam H3K36me3.bam H3K27me3.bam H3K9me3.bam H3K9ac.bam`

**Explanation:** The binarization step converts continuous ChIP-seq enrichment to binary presence/absence in 200kb bins, which is required before model training with LearnModel using 15 states.

### Using peak files instead of tag directories as input

**Args:** `BinarizeSignal -gc mm10.chrom.sizes -peaks ./peak_dir ./binary_out -b 100 makeTagDirectory peak_dir/ H3K4me1_peaks.bed H3K4me3_peaks.bed H3K27ac_peaks.bed`

**Explanation:** This uses pre-called peak BED files as input rather than aligned reads, which is useful when working with published peak sets or when compute resources are limited.

### Extracting state assignments for downstream analysis

**Args:** `MakeSegmentation -gc hg19.chrom.sizes ./binary_out/hg19_200kb_binarized.txt ./model_15state ./segmentation_15state`

**Explanation:** After training a model, this command applies the learned emission and transition parameters to assign the most probable state to each genomic bin, producing a BED file for visualization or overlap analysis.

### Comparing chromatin states across two cell types jointly

**Args:** `BinarizeSignal -gc hg19.chrom.sizes celltype1_tags/ celltype2_tags/ ./combined_binary -b 200 makeTagDirectory celltype1_tags/*_cell1.bam makeTagDirectory celltype2_tags/*_cell2.bam`

**Explanation:** This binarizes data from two different cell types together, allowing the joint LearnModel to learn states that are consistent and comparable across both cell types, facilitating cross-cell-type epigenomic analysis.

### Producing a detailed emission table for biological interpretation

**Args:** `WriteEmissions ./model_12state_emit.txt ./emissions_12state.txt`

**Explanation:** After model training, this helper command extracts the emission probabilities for each state, showing the characteristic histone modification combination for interpretating each chromatin state's biological function.

### Adjusting bin size for high-resolution segmentation

**Args:** `BinarizeSignal -gc hg19.chrom.sizes -b 50 ./input_tags ./highres_binary makeTagDirectory input_tags/ *bam`

**Explanation:** Using a smaller 50kb bin size provides higher genomic resolution for the chromatin state segmentation, suitable when working with datasets with narrow regulatory elements like promoters and enhancers.