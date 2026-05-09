---
name: clair3-illumina
category: Variant Calling
description: A deep learning-based variant caller optimized for Illumina short-read sequencing data, part of the Clair3 suite. It calls germline and somatic single-nucleotide variants (SNVs) and small indels using tensor-based neural networks trained on Illumina platforms.
tags:
  - variant-calling
  - illumina
  - deep-learning
  - snv
  - indel
  - germline
  - short-reads
  - vcf
author: AI-enerated
source_url: https://github.com/HKU-BAL/Clair3
---

## Concepts

- **Input format**: Clair3-illumina accepts sorted and indexed BAM files aligned to a reference genome. The BAM must contain valid read group tags (`@RG`) and the MD:Z tag for mismatch tracking. Unmapped reads or secondary alignments are automatically filtered during calling.
- **Output format**: The primary output is a bgzip-compressed VCF file (`.vcf.gz`) with an accompanying index (`.vcf.gz.tbi`). The VCF uses GRCh38 notation by default and follows VCF 4.3 specification, with `PASS`, `LowGQ`, or `LowDepth` in the FILTER field indicating variant reliability.
- **Model architecture**: Clair3 uses a tensor-based convolutional neural network with a three-step pileup variant candidate selection strategy. Models are organized by flowcell and chemistry (e.g., `IlluminaStandard`, `NovaSeq`), and an incorrect model choice reduces accuracy significantly.
- **Phasing behavior**: By default, Clair3 enables read-backed phasing using WhatsHap. Disabling phasing with `--no_phasing` may reduce accuracy in heterozygous calls and impairs the detection of complex variants.
- **Runtime dependencies**: Clair3 requires Python 3.8+, TensorFlow 2.x, WhatsHap, Cython, and Samtools. The companion binary `clair3-illumina-build` is used to compile the neural network engine and is not a separate variant caller.

## Pitfalls

- **Mismatched reference genome**: Using a reference genome build (e.g., hg38) that does not match the BAM header results in silent incorrect calls or errors at the contig-level. Always verify that `samtools view -H input.bam | grep -E '^@SQ'` matches the reference used with `--ref_fn`.
- **Incorrect model selection**: Running with the wrong model directory (e.g., NovaSeq model on a HiSeq BAM) produces high false-positive rates for SNVs, particularly in low-complexity regions. Check the flowcell type in the read group tag or `RG:PL` field before selecting `--model_path`.
- **Memory exhaustion on large genomes**: Specifying `--include_all_ctgs` without enough RAM (>16 GB for human genomes) causes the process to be killed by the OOM killer. Use `--ctg_name` to process one contig at a time when memory is limited.
- **Conflicting pileup and full-alignment modes**: Setting `--pileup_only` together with a full-alignment mode flag produces undefined behavior or crashes. These modes are mutually exclusive; choose exactly one strategy per run.
- **Missing index files**: Running without a pre-existing `.bai` index for the BAM or `--ref_fn` without a `.fai` index causes samtools to fail silently or produce truncated output. Always run `samtools index input.bam` and `samtools faidx ref.fa` before calling.

## Examples

### Calling variants on a single contig using pileup mode with a specific model
**Args:** `--bam_fn NA12878.bam --ref_fn GRCh38.fa --model_path /models/ Clair3/IlluminaStandard --ctg_name chr20 --pileup_only --output_fn NA12878_chr20.vcf.gz`
**Explanation:** This restricts calling to chromosome 20 only, uses the standard Illumina pileup model, and writes a bgzipped VCF for downstream compatibility with bcftools or GATK.

### Running full variant calling on all contigs with multi-threading
**Args:** `--bam_fn sample.bam --ref_fn GRCh38.fa --model_path /models/Clair3/NovaSeq --include_all_ctgs --threads 16 --output_fn sample_variants.vcf.gz`
**Explanation:** The `--include_all_ctgs` flag processes every contig in the BAM, and `--threads 16` parallelizes the tensor operations across 16 CPU cores to reduce wall-clock time.

### Enabling read-backed phasing for diploid variant calls
**Args:** `--bam_fn tumor_normal.bam --ref_fn GRCh38.fa --model_path /models/Clair3/HiSeq --ctg_name chr1 --output_fn phased_chr1.vcf.gz`
**Explanation:** By default, phasing is enabled when no `--no_phasing` flag is provided. WhatsHap reconstructs haplotype blocks, improving genotype accuracy for heterozygous positions.

### Disabling phasing to speed up calling in validation runs
**Args:** `--bam_fn validation.bam --ref_fn GRCh38.fa --model_path /models/Clair3/IlluminaStandard --ctg_name chr1 --no_phasing --output_fn no_phasing_chr1.vcf.gz`
**Explanation:** The `--no_phasing` flag skips the WhatsHap phasing step entirely, which reduces runtime by 20-30% but may degrade genotype quality in regions with allele imbalance.

### Running in fast mode with reduced tensor resolution
**Args:** `--bam_fn rapid_run.bam --ref_fn GRCh38.fa --model_path /models/Clair3/IlluminaStandard --ctg_name chr22 --fast_mode --output_fn fast_chr22.vcf.gz`
**Explanation:** Fast mode downsamples read pileups and reduces the neural network tensor size, cutting runtime by roughly half at the cost of lower sensitivity in low-coverage or repetitive regions.

### Filtering low-confidence variants post-calling using bcftools
**Args:** `--bam_fn sample.bam --ref_fn GRCh38.fa --model_path /models/Clair3/NovaSeq --ctg_name chr1 --output_fn raw_chr1.vcf.gz && bcftools filter -s LowQual -m + -e 'FILTER="."' raw_chr1.vcf.gz -o filtered_chr1.vcf.gz`
**Explanation:** After Clair3 produces a VCF, bcftools applies an expression-based filter to annotate any variants without an explicit FILTER assignment as `LowQual`, which is useful for strict downstream filtering pipelines.

### Batch processing multiple samples using a shell loop
**Args:** `for bam in sample_list/*.bam; do id=$(basename "$bam" .bam); clair3-illumina --bam_fn "$bam" --ref_fn GRCh38.fa --model_path /models/Clair3/IlluminaStandard --ctg_name chr1 --output_fn "results/${id}_chr1.vcf.gz"; done`
**Explanation:** This shell loop iterates over all BAM files in a directory and calls variants for chromosome 1 only, using the same reference and model, which is typical for targeted validation studies.