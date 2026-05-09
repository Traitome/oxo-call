---
name: clairvoyante
category: variant_calling
description: A neural network-based variant caller for long-read sequencing data (PacBio and Oxford Nanopore). Uses convolutional neural networks to detect SNPs, insertions, and deletions from haploid and diploid genomes with high accuracy.
tags: [variant-calling, long-reads, neural-network, nanopore, pacbio, snp, indel, germline-calling]
author: AI-generated
source_url: https://github.com/nanoporegenomics/clairvoyante
---

## Concepts

- **Multi-stage calling workflow**: Clairvoyante requires three distinct stages—training a model with `clairvoyante train`, generating candidate variants with `clairvoyante candidates`, and final variant calling with `clairvoyante call`. Skipping the candidate stage reduces accuracy as the caller relies on pre-generated candidate positions.
- **Platform-specific model architectures**: Different sequencing platforms (PacBio `--platform pb`, Oxford Nanopore `--platform ont`) require separate trained models. Using a model trained on wrong platform data results in high false-positive rates because base-calling error profiles differ significantly between technologies.
- **TensorFlow dependency and GPU acceleration**: Clairvoyante uses TensorFlow backend for neural network inference. Without CUDA-enabled GPUs, training and calling are extremely slow—genome-scale calling on CPU can take weeks. The `--chunkSize` parameter controls batch processing memory usage.
- **Input format requirements**: Inputs must be coordinate-sorted BAM files with proper indices and a reference genome in FASTA format. The reference must be the same version used for alignment, as mismatch between alignment reference and calling reference causes positional errors.
- **Output VCF specification**: Output VCFs from `call` contain genotype probabilities (FORMAT GP field) and read-backed phasing information (PS and PQ tags). The quality score (QUAL) is a neural network-derived probability, not a traditional Phred score.

## Pitfalls

- **Using CPU-only processing for large datasets**: Training without GPU can take 10-100x longer. A typical human genome training job that takes 2 hours on a single GPU may require 20+ hours on CPU, often leading to incomplete runs or abandoned workflows.
- **Mismatching training and calling platform**: Training on PacBio data and then calling with Oxford Nanopore reads produces high false-positive rates (often >10% FDR) because the neural network learns platform-specific noise patterns that don't transfer between technologies.
- **Insufficient training data for diploid calling**: Training with fewer than 10 samples per haplotype for diploid variant calling leads to unreliable genotype predictions. The model cannot learn the full spectrum of heterozygous patterns, resulting in systematic under-calling of het sites.
- **Reference genome mismatch**: Using a different reference FASTA than the one used for alignment causes positional errors in variant calling. Always verify that the MD5 hash of the reference matches between alignment and calling stages.
- **Forgetting candidate generation**: Running `call` without first running `candidates` uses a genome-wide scanning mode that is significantly less accurate and much slower. The candidate stage focuses the neural network on likely variant regions.

## Examples

### Train a variant calling model using PacBio CLR reads
**Args:** train --refPath reference.fa --bamPath aligned.bam --trainDataset train_data.vcf --modelFolder ./model --platform pb --epochs 20 --chunkSize 1000
**Explanation:** Trains a neural network model on PacBio data using 20 epochs with 1000 read chunks, saving the trained model to the specified folder for later variant calling.

### Generate candidate variant positions from an aligned BAM
**Args:** candidates --refPath reference.fa --bamPath aligned.bam --modelFolder ./model --candidatesOutput candidates.bed
**Explanation:** Uses a trained model to identify likely variant positions across the genome, outputting those candidates as a BED file for the calling stage.

### Call variants using Oxford Nanopore reads
**Args:** call --refPath reference.fa --bamPath aligned.bam --modelFolder ./model --candidates candidates.bed --vcfOutput variants.vcf --platform ont
**Explanation:** Calls variants on Nanopore data using the candidate regions, outputting variant calls in VCF format with genotype probabilities.

### Train a model with custom chunk size for memory-constrained systems
**Args:** train --refPath reference.fa --bamPath aligned.bam --trainDataset train_data.vcf --modelFolder ./model --platform pb --chunkSize 500 --epochs 15
**Explanation:** Reduces chunk size to 500 to fit in limited memory while training, trading some throughput for stability on systems with constrained RAM.

### Evaluate calling accuracy using a truth VCF
**Args:** evaluate --vcfPath called_variants.vcf --truthPath truth.vcf --refPath reference.fa --outputMetrics evaluation.txt
**Explanation:** Compares called variants against a truth set to generate precision, recall, and F1 score metrics for quality assessment.