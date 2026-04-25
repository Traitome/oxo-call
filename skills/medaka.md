---
name: medaka
category: variant-calling
description: Sequence correction and variant calling for Oxford Nanopore sequencing using neural network models
tags: [nanopore, long-read, polishing, variant-calling, consensus, ont, neural-network]
author: oxo-call built-in
source_url: "https://github.com/nanoporetech/medaka"
---

## Concepts
- Medaka polishes Oxford Nanopore assemblies and calls variants using neural network models trained by Oxford Nanopore.
- Use 'medaka consensus' for per-read consensus; 'medaka stitch' to merge; or use 'medaka_consensus' pipeline.
- The medaka_consensus pipeline (all-in-one) is easiest: medaka_consensus -i reads.fastq -d draft.fasta -o output/ -t N.
- Model selection is critical: match the model to the basecalling model used (e.g., r941_min_hac_g507).
- Use 'medaka tools list_models' to see available models; models are named by flow cell, chemistry, and basecaller version.
- Medaka requires minimap2 for alignment; the medaka_consensus pipeline handles alignment automatically.
- For variant calling: medaka_haploid_variant (haploid) or medaka_variant (diploid) pipelines.
- GPU acceleration is supported with CUDA — dramatically speeds up medaka.
- medaka features generates training data from aligned reads; medaka inference runs the neural network on features.
- Chunking parameters (--chunk_len, --chunk_ovlp) control memory usage; reduce for low-memory systems.
- --regions allows targeted analysis of specific genomic regions or BED file input.
- --save_features preserves intermediate HDF5 files for debugging or re-running inference with different models.
- --check_output validates output file integrity after inference completion.
- medaka sequence stitches consensus from inference output; medaka vcf creates variant calls from diploid inference.

## Pitfalls
- medaka ARGS must start with a subcommand (consensus_from_features, compress_bam, features, train, inference, smolecule, tandem, consensus_from_features, fastrle, sequence, vcf, tools) — never with flags like -i, -d, -o. The subcommand ALWAYS comes first. Note: medaka_consensus, medaka_haploid_variant, medaka_variant are separate binary wrappers, not subcommands.
- Using the wrong model (-m) gives inaccurate polishing — always match the model to your basecaller and chemistry.
- Medaka requires conda installation with tensorflow dependencies — environment conflicts are common.
- For diploid variant calling, use medaka_variant, not medaka_consensus.
- The medaka_consensus pipeline overwrites the output directory — use a fresh directory each run.
- Medaka polishing is CPU/GPU intensive — use -t 8 or more threads and consider GPU for large assemblies.
- Running too many polishing rounds (>2) with Medaka does not improve quality and may introduce errors.
- Default chunk_len (10000) may cause OOM on low-memory systems; reduce to 5000 or 2000 with --chunk_len.
- --regions BED file must be 0-based, tab-delimited; incorrect format causes silent failures.
- GPU and CPU models are not interchangeable; CPU-only installation lacks GPU capability.
- medaka inference output is HDF5 format; requires medaka sequence or medaka vcf to convert to FASTA/VCF.
- Version 2.0+ uses PyTorch instead of TensorFlow; models are not backward compatible with v1.x.

## Examples

### polish an ONT assembly with Medaka (all-in-one pipeline)
**Args:** `medaka_consensus -i reads.fastq.gz -d draft_assembly.fasta -o medaka_output/ -t 8 -m r941_min_hac_g507`
**Explanation:** medaka_consensus command; -i reads.fastq.gz ONT reads; -d draft_assembly.fasta draft assembly; -o medaka_output/ output directory; -t 8 threads; -m r941_min_hac_g507 model matching basecaller version

### call variants from ONT reads (haploid)
**Args:** `medaka_haploid_variant -i reads.fastq.gz -r reference.fasta -o medaka_variants/ -t 8 -m r941_min_hac_g507`
**Explanation:** medaka_haploid_variant command for haploid variant calling (bacteria, viruses); -i reads.fastq.gz ONT reads; -r reference.fasta reference FASTA; -o medaka_variants/ output directory; -t 8 threads; -m r941_min_hac_g507 model

### list available Medaka models
**Args:** `tools list_models`
**Explanation:** medaka tools subcommand; list_models lists all available models; select the appropriate model for your flowcell and basecaller version

### run Medaka consensus with GPU acceleration
**Args:** `medaka_consensus -i reads.fastq.gz -d draft.fasta -o medaka_gpu/ -t 2 -m r1041_e82_400bps_hac_v4.2.0 --gpu`
**Explanation:** medaka_consensus command; -i reads.fastq.gz ONT reads; -d draft.fasta draft assembly; -o medaka_gpu/ output directory; -t 2 threads; -m r1041_e82_400bps_hac_v4.2.0 model; --gpu uses CUDA for faster neural network inference

### run targeted variant calling with region BED file
**Args:** `medaka_variant -i reads.fastq.gz -r reference.fasta -o targeted_variants/ -t 8 -m r1041_e82_400bps_hac_v4.2.0 --regions targets.bed`
**Explanation:** medaka_variant command; -i reads.fastq.gz ONT reads; -r reference.fasta reference; -o targeted_variants/ output directory; -t 8 threads; -m r1041_e82_400bps_hac_v4.2.0 model; --regions targets.bed limits analysis to specified BED regions

### reduce memory usage for large genomes
**Args:** `medaka_consensus -i reads.fastq.gz -d draft.fasta -o medaka_lowmem/ -t 4 -m r1041_e82_400bps_hac_v4.2.0 --chunk_len 5000 --chunk_ovlp 500`
**Explanation:** medaka_consensus command; -i reads.fastq.gz ONT reads; -d draft.fasta draft assembly; -o medaka_lowmem/ output directory; -t 4 threads; -m r1041_e82_400bps_hac_v4.2.0 model; --chunk_len 5000 and --chunk_ovlp 500 reduce memory footprint

### save intermediate features for model comparison
**Args:** `inference --save_features --model r1041_e82_400bps_hac_v4.2.0 aligned.bam output.hdf`
**Explanation:** medaka inference subcommand; --save_features preserves feature HDF5; --model r1041_e82_400bps_hac_v4.2.0 neural network model; aligned.bam input BAM; output.hdf output HDF5

### run inference on specific chromosomes only
**Args:** `inference --regions chr1 chr2 chr3 --model r1041_e82_400bps_hac_v4.2.0 aligned.bam chr1-3_output.hdf`
**Explanation:** medaka inference subcommand; --regions chr1 chr2 chr3 specifies which contigs to process; --model r1041_e82_400bps_hac_v4.2.0 neural network model; aligned.bam input BAM; chr1-3_output.hdf output HDF5

### stitch consensus from inference HDF5 output
**Args:** `sequence output.hdf consensus.fasta`
**Explanation:** medaka sequence subcommand; output.hdf input HDF5; consensus.fasta output FASTA; converts inference HDF5 to FASTA consensus

### create VCF from diploid inference output
**Args:** `vcf output.hdf variants.vcf reference.fasta`
**Explanation:** medaka vcf subcommand; output.hdf input HDF5; variants.vcf output VCF; reference.fasta reference FASTA; generates VCF from diploid inference

### compress BAM to RLE format for storage
**Args:** `compress_bam -t 8 aligned.bam compressed.bam`
**Explanation:** medaka compress_bam subcommand; -t 8 threads; aligned.bam input BAM; compressed.bam output BAM; creates run-length encoded BAM
