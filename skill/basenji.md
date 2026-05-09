---
name: basenji
category: Genomic Variant Effect Prediction
description: A deep convolutional neural network tool for predicting the functional effects of genetic variants at high resolution using genomic sequences as input. Basenji models regulatory activity and variant impact across cell types.
tags:
  - variant-effect-prediction
  - deep-learning
  - genomics
  - regulatory-elements
  - neural-network
  - non-coding-variants
  - chromatin
  - tf-binding
author: AI-generated
source_url: https://github.com/calico/basenji
---

## Concepts

- Basenji operates on short genomic sequence windows (typically 131,072 bp or 524,288 bp depending on the model variant) extracted from a reference genome, requiring both a FASTA genome file and a chromosome sizes file for coordinate resolution and sequence extraction.
- Input variants are provided in standard VCF 4.x format, and Basenji outputs per-variant log fold-change predictions quantifying the expected change in regulatory activity (e.g., chromatin accessibility or TF binding) for the alt allele relative to the ref allele.
- The model must be trained on organism-specific reference data (human, mouse, etc.) using a配套 basenji-train pipeline that consumes a targets file describing which genomic activities are being predicted (e.g., histone marks, DNase hypersensitivity), and the resulting model checkpoints are passed to basenji-predict for inference.
- Basenji supports parallel batch prediction across chromosomal regions, and performance scales strongly with GPU memory — larger batch windows and wider genomes require more VRAM; a typical human model with 524,288 bp windows needs at least 16 GB VRAM for efficient inference.
- Output predictions are written in compressed HDF5 format (.h5 files) containing per-position, per-target activity scores for both ref and alt alleles, which must then be post-processed to extract variant-level effect summaries using basenji-saturate or custom Python scripts.

## Pitfalls

- Using a model checkpoint trained on a different genome build (e.g., hg38 model with hg19 coordinates) silently produces meaningless predictions because sequence coordinates and reference bases are mismatched — always verify the genome assembly matches between model training and variant calling.
- Omitting the --loci flag when specifying chromosome coordinates causes basenji-predict to process the entire genome by default, which can take hours on large genomes and fill disk with unwanted HDF5 output chunks; always specify the exact chromosome regions of interest.
- Specifying window sizes smaller than the model's training window (e.g., requesting 32,768 bp with a 131,072 bp model) causes the model to either error out or silently pad sequences, producing unreliable variant effect scores at sequence boundaries.
- Forgetting to index the input FASTA genome file (requiring .fai index) before running prediction causes I/O errors at runtime; always run `samtools faidx genome.fa` before the first prediction run.
- When processing multi-sample VCF files, failing to specify the --sample flag or passing the wrong sample name results in predictions being generated only for the first sample in the VCF, silently dropping all other sample genotypes.

## Examples

### Train a Basenji model on human genomic activity targets
**Args:** train.py --gti human_targets.tf --fai GRCh38.p14.fa.fai --out model_out/ --tfrs tfrs/train*.tfr --tfrs_val tfrs/val*.tfr --json hparams.json --device cuda:0
**Explanation:** This launches training using a targets file defining genomic activity tracks, indexed FASTA reference, HDF5 TensorFlow Records for training and validation splits, and a GPU device for acceleration.

### Predict variant effects for a GWAS loci VCF on GPU
**Args:** predict.py --model model_out/model-best.h5 --fai GRCh38.p14.fa.fai --vcf gwascatalog_loci.vcf.gz --lofi loci.bed --loci chr22:35000000-38000000 --tfrs tfrs/predict.tfr --out pred_out/ --device cuda:0 --batch_size 8
**Explanation:** This runs prediction on a specific chromosomal interval from a compressed VCF using a trained model checkpoint, an indexed genome, and a batch size tuned for GPU memory utilization.

### Saturate a genomic region to identify the causal variant among candidates
**Args:** saturate.py --model model_out/model-best.h5 --fai GRCh38.p14.fa.fai --vcf candidate_snps.vcf.gz --region chr3:37000000-37500000 --out saturation_out/ --samples 1-4 --threads 12
**Explanation:** This performs saturation mutagenesis across the specified region, testing all possible base substitutions at each position to quantify each candidate variant's regulatory impact.

### Convert BAM-based genomic activity data into TensorFlow Records for training
**Args:** basenji-data.py --fai GRCh38.p14.fa.fai --bfy_plus bam/plus/*.bam --bfy_minus bam/minus/*.bam --out tfrs/ --split 131072 --clip 2 --n耳 32
**Explanation:** This ingests stranded BAM coverage files (forward and reverse strand separately) and converts them into compressed TFRecord shards covering 131,072 bp windows, ready for model training ingestion.

### Post-process HDF5 predictions into a sortable variant effects table
**Args:** post.py --h5 pred_out/chr22_predictions.h5 --vcf gwascatalog_loci.vcf.gz --out effects.bed --stats mean
**Explanation:** This converts the raw HDF5 prediction outputs into a BED-style effect table by comparing alt vs. ref allele activity scores across the targets, generating mean effect magnitude statistics for downstream fine-mapping.

### Build a species-specific reference index for Basenji inference
**Args:** basenji-build.py --fasta GRCh38.p14.fa --out grch38_index/ --split 524288 --bed annotations regulatory_beds/
**Explanation:** This precomputes genomic sequence splits and regulatory annotations for fast random-access retrieval during inference, generating the indexed reference directory used by basenji-predict for efficient sequence lookup.

### Evaluate model performance on held-out test variants
**Args:** eval.py --model model_out/model-best.h5 --fai GRCh38.p14.fa.fai --vcf test_variants.vcf.gz --tfrs tfrs/test.tfr --out eval_out/ --metrics roc_pr auroc spearman
**Explanation:** This evaluates the trained model's predictions against held-out test variants using area under the ROC curve, precision-recall, and Spearman rank correlation to quantify prediction accuracy.

### Batch predict across multiple chromosomes with distributed processing
**Args:** predict.py --model model_out/model-best.h5 --fai GRCh38.p14.fa.fai --vcf multi_chr.vcf.gz --lofi all_loci.bed --out batch_out/ --device cuda:0 --distributed --n_gpus 4
**Explanation:** This distributes prediction workload across four GPUs by processing different chromosomal loci in parallel, significantly accelerating genome-wide effect scoring for large variant sets.