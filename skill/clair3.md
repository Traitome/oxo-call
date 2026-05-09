---
name: clair3
category: Variant Calling
description: Long-read variant caller using deep learning for Oxford Nanopore, PacBio CLR, and PacBio HiFi sequencing data. Detects SNPs and small indels with high accuracy using neural network-based pileup analysis.
tags: [variant-calling, long-reads, nanopore, pacbio, deep-learning, vcf, bioinformatics]
author: AI-generated
source_url: https://github.com/HongnanJ/Clair3
---

## Concepts

- Clair3 performs variant calling by analyzing pileup images generated from aligned reads (BAM/CRAM) against a reference genome. It requires input from `samtools mpileup` or directly from a BAM file when using the `--bam_file` option.
- The tool supports three sequencing platforms: `--platform ont` for Oxford Nanopore, `--platform pb` for PacBio CLR, and `--platform hifi` for PacBio HiFi. You must specify the correct platform as the model was trained on platform-specific data.
- Output is produced in VCF or GVCF format, with optional genotyping across multiple samples. The `--output` flag specifies the output VCF path, and `--gvcf` enables GVCF output for joint genotyping.
- Clair3 requires a pre-trained model trained on the same sequencing platform (available via `clair3-model` or download). Models are platform-specific and using an ont model for pb data (or vice versa) yields poor accuracy.
- For large genomes, use chunked calling with `--chunk_size` and `--chunk_cov` to parallelize across genomic regions and reduce memory consumption.

## Pitfalls

- **Omitting the `--platform` flag**: Defaults may not match your data source, leading to drastically reduced accuracy because Clair3's default model may be trained on a different platform's signal characteristics.
- **Using the wrong reference FASTA**: Alignment and variant calling require consistent reference sequences. Mismatched references between BAM and VCF cause incorrect variant positions and genotypes.
- **Running on unsorted or unindexed BAM files**: Clair3 requires position-sorted and indexed BAM files. Unsorted BAMs produce malformed pileup and no variant calls.
- **Specifying an incorrect or missing model path**: Running without `--model` or with an invalid path crashes with an error. Always verify the model directory exists and contains the required files.
- **Ignoring memory constraints for whole-genome analysis**:Whole-genome calling on large genomes (human) without chunking can exhaust memory. Use `--chunk_size` to limit regions processed at once.

## Examples

### Call variants from Oxford Nanopore FASTQ data
**Args:** `--platform ont --bam_fn /data/alignments.bam --ref_fn /data/reference.fasta --output /results/variants.vcf --model /models/ont`
**Explanation:** Standard command that specifies the Nanopore platform, input BAM, reference FASTA, output VCF path, and platform-specific trained model.

### Call variants from PacBio HiFi reads
**Args:** `--platform hifi --bam_fn /data/hifi_alignments.bam --ref_fn /data/reference.fasta --output /results/hifi_variants.vcf --model /models/hifi`
**Explanation:** Use for PacBio HiFi reads which have higher accuracy, requiring the HiFi-specific model trained on circular consensus sequences.

### Output GVCF for joint genotyping
**Args:** `--platform ont --bam_fn /data/sample.bam --ref_fn /data/ref.fasta --output /results/sample.g.vcf --gvcf --model /models/ont`
**Explanation:** Adding `--gvcf` outputs a GVCF file with genotype quality scores and allele depth, enabling joint variant calling across multiple samples.

### Limit memory with chunked calling on a specific chromosome
**Args:** `--platform ont --bam_fn /data/sample.bam --ref_fn /data/ref.fasta --output /results/chr20.vcf --model /models/ont --chunk_size 500000 --chunk_cov 5 --chrobj chr20`
**Explanation:** Use `--chunk_size` to process 500kb regions and `--chrobj` to target chromosome 20 directly, reducing peak memory and runtime on large genomes.

### Enable alternative allele detection and heterozygous filtering
**Args:** `--platform pb --bam_fn /data/pb_align.bam --ref_fn /data/ref.fasta --output /results/pb_vars.vcf --model /models/pb --enable_long_indel`
**Explanation:** The `--enable_long_indel` flag improves detection of insertions and deletions longer than 50bp, common in PacBio data, at the cost of slightly higher false positive rate.