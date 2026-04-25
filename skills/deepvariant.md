---
name: deepvariant
category: variant-calling
description: Deep learning-based germline variant caller from Google DeepMind for SNPs and indels
tags: [variant-calling, deep-learning, snp, indel, germline, vcf, illumina, pacbio, nanopore, gvcf, glnexus, deeptrio]
author: oxo-call built-in
source_url: "https://github.com/google/deepvariant"
---

## Concepts

- DeepVariant uses a deep convolutional neural network to call SNPs and indels from aligned reads.
- Run as a Docker/Singularity container or via the run_deepvariant.py script (recommended for ease of use).
- Use --model_type to specify data type: WGS, WES, PACBIO, ONT, HYBRID_PACBIO_ILLUMINA.
- Input: --input_bam (sorted, indexed BAM/CRAM); --ref (reference FASTA with .fai index).
- Output: --output_vcf for VCF, --output_gvcf for GVCF (for joint genotyping with GLnexus).
- Use --num_shards N to parallelize across CPU cores (N = number of CPUs available).
- DeepVariant outputs PASS-filtered variants in the VCF FILTER column — use bcftools filter -f PASS.
- For joint genotyping of multiple samples, use GLnexus to merge per-sample gVCFs.
- Three internal steps: make_examples (create pileup images), call_variants (DNN inference), postprocess_variants (format VCF).
- --intermediate_results_dir saves intermediate files for debugging or resuming.
- --dry_run prints commands without executing; useful for workflow debugging.
- GPU acceleration significantly speeds up call_variants step; use GPU Docker image.
- DeepTrio extends DeepVariant for trio/duo analysis with Mendelian consistency.

## Pitfalls

- DeepVariant model type must match the sequencing platform — using wrong model significantly reduces accuracy.
- The reference FASTA must match the genome build used for alignment exactly.
- GPU acceleration requires the GPU Docker image and NVIDIA Docker — much faster than CPU.
- DeepVariant outputs both PASS and non-PASS variants — filter with bcftools filter -f PASS for high-confidence calls.
- The --output_gvcf is needed for multi-sample analysis — don't skip it in cohort studies.
- DeepVariant is germline only — use other tools (Mutect2, Strelka2) for somatic variant calling.
- BAM files must be sorted and indexed; DeepVariant cannot read from stdin.
- BQSR (Base Quality Score Recalibration) slightly decreases accuracy; not recommended.
- Indel realignment has minimal effect on accuracy; not necessary.
- For PacBio long reads, use --norealign_reads --vsc_min_fraction_indels 0.12 flags.
- call_variants step benefits most from GPU; make_examples is CPU-only and should be parallelized.
- --writer_threads in call_variants auto-detects all CPUs; limit to avoid process limits on shared systems.

## Examples

### call variants from Illumina WGS data using DeepVariant
**Args:** `run_deepvariant --model_type WGS --ref reference.fa --input_bam sorted.bam --output_vcf output.vcf --output_gvcf output.g.vcf --num_shards 16`
**Explanation:** run_deepvariant script; --model_type WGS for Illumina whole-genome; --ref reference.fa reference FASTA; --input_bam sorted.bam input BAM; --output_vcf output.vcf output VCF; --output_gvcf output.g.vcf output GVCF; --num_shards 16 parallel CPU shards

### call variants from PacBio HiFi data
**Args:** `run_deepvariant --model_type PACBIO --ref reference.fa --input_bam hifi_sorted.bam --output_vcf pacbio_variants.vcf --output_gvcf pacbio.g.vcf --num_shards 16`
**Explanation:** run_deepvariant script; --model_type PACBIO for CCS/HiFi reads; --ref reference.fa reference FASTA; --input_bam hifi_sorted.bam input BAM; --output_vcf pacbio_variants.vcf output VCF; --output_gvcf pacbio.g.vcf output GVCF; --num_shards 16 parallel shards; dedicated model for long high-accuracy reads

### call variants from Oxford Nanopore reads
**Args:** `run_deepvariant --model_type ONT --ref reference.fa --input_bam ont_sorted.bam --output_vcf ont_variants.vcf --output_gvcf ont.g.vcf --num_shards 16`
**Explanation:** run_deepvariant script; --model_type ONT for Nanopore reads; --ref reference.fa reference FASTA; --input_bam ont_sorted.bam input BAM; --output_vcf ont_variants.vcf output VCF; --output_gvcf ont.g.vcf output GVCF; --num_shards 16 parallel shards; trained on ONT-specific error profiles

### call variants from WES data
**Args:** `run_deepvariant --model_type WES --ref reference.fa --input_bam wes_sorted.bam --regions targets.bed --output_vcf wes_variants.vcf --output_gvcf wes.g.vcf --num_shards 8`
**Explanation:** run_deepvariant script; --model_type WES for whole-exome; --ref reference.fa reference FASTA; --input_bam wes_sorted.bam input BAM; --regions targets.bed restricts calling to target BED regions; --output_vcf wes_variants.vcf output VCF; --output_gvcf wes.g.vcf output GVCF; --num_shards 8 parallel shards

### run with intermediate results for debugging
**Args:** `run_deepvariant --model_type WGS --ref reference.fa --input_bam sorted.bam --output_vcf output.vcf --output_gvcf output.g.vcf --intermediate_results_dir /tmp/intermediate --num_shards 16`
**Explanation:** run_deepvariant script; --model_type WGS for Illumina whole-genome; --ref reference.fa reference FASTA; --input_bam sorted.bam input BAM; --output_vcf output.vcf output VCF; --output_gvcf output.g.vcf output GVCF; --intermediate_results_dir /tmp/intermediate saves make_examples and call_variants outputs; --num_shards 16 parallel shards; useful for debugging or resuming

### dry run to preview commands
**Args:** `run_deepvariant --model_type WGS --ref reference.fa --input_bam sorted.bam --output_vcf output.vcf --dry_run=true`
**Explanation:** run_deepvariant script; --model_type WGS for Illumina whole-genome; --ref reference.fa reference FASTA; --input_bam sorted.bam input BAM; --output_vcf output.vcf output VCF; --dry_run=true prints all commands without executing; useful for workflow validation and debugging

### hybrid PacBio-Illumina data
**Args:** `run_deepvariant --model_type HYBRID_PACBIO_ILLUMINA --ref reference.fa --input_bam hybrid.bam --output_vcf hybrid.vcf --output_gvcf hybrid.g.vcf --num_shards 16`
**Explanation:** run_deepvariant script; --model_type HYBRID_PACBIO_ILLUMINA model for combined long and short read data; --ref reference.fa reference FASTA; --input_bam hybrid.bam input BAM; --output_vcf hybrid.vcf output VCF; --output_gvcf hybrid.g.vcf output GVCF; --num_shards 16 parallel shards

### run with specific region
**Args:** `run_deepvariant --model_type WGS --ref reference.fa --input_bam sorted.bam --regions chr20:10,000,000-10,100,000 --output_vcf region.vcf --output_gvcf region.g.vcf --num_shards 4`
**Explanation:** run_deepvariant script; --model_type WGS for Illumina whole-genome; --ref reference.fa reference FASTA; --input_bam sorted.bam input BAM; --regions chr20:10,000,000-10,100,000 limits calling to specific genomic region; --output_vcf region.vcf output VCF; --output_gvcf region.g.vcf output GVCF; --num_shards 4 parallel shards; useful for testing or targeted analysis

### GPU-accelerated variant calling
**Args:** `run_deepvariant --model_type WGS --ref reference.fa --input_bam sorted.bam --output_vcf output.vcf --output_gvcf output.g.vcf --num_shards 1 --writer_threads=6`
**Explanation:** run_deepvariant script; --model_type WGS for Illumina whole-genome; --ref reference.fa reference FASTA; --input_bam sorted.bam input BAM; --output_vcf output.vcf output VCF; --output_gvcf output.g.vcf output GVCF; --num_shards 1 for GPU runs (one GPU); --writer_threads=6 limits CPU threads for output writing
