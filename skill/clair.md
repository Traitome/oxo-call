---
name: Clair - Variant Caller for Oxford Nanopore Data
category: Variant Calling
description: A fast and accurate variant caller specifically designed for Oxford Nanopore sequencing data, leveraging neural network models to detect SNPs, indels, and structural variants from raw nanopore signals.
tags:
- variant-calling
- nanopore
- snp
- indel
- structural-variants
- real-time-analysis
author: AI-generated
source_url: https://github.com/nanopore-research/clarity
---

## Concepts

- **Input Format**: Clair accepts FAST5 files (raw nanopore signal data) and aligned BAM files as primary inputs. The tool processes either raw electrical signals directly or pre-aligned read data for variant detection.
- **Output Formats**: Generates variant call format (VCF) files with SNP and indel annotations, along with a binary variant database (VDB) for downstream query and filtering.
- **Model Architecture**: Uses ensemble neural network models trained on curated human variation databases, running inference either on GPU (CUDA-enabled) or CPU depending on configuration.
- **Streaming Capability**: Supports real-time variant calling via streaming mode when connected to MinKNOW or Bonito basecalling pipeline, enabling immediate variant detection during sequencing runs.

## Pitfalls

- **Coverage Thresholds**: Setting minimum coverage too low (below 10x) leads to high false-positive rates; variants below recommended thresholds lack statistical reliability for confident genotype calls.
- **Homopolymer Regions**: Failing to adjust parameters for homopolymer-rich genomic regions results in indel calling errors since nanopore signal analysis is sensitive to basecalling accuracy in repetitive sequences.
- **Model Mismatch**: Using a model trained on one reference genome version with a different reference leads to alignment artifacts and incorrect variant positions, generating false variants at known polymorphic sites.
- **Resource Allocation**: Running Clair without sufficient memory on systems with large FAST5 files causes process termination; allocate at least 16GB RAM for typical human genome datasets.

## Examples

### Call variants from a BAM file aligned to GRCh38
**Args:** `call -b aligned_reads.bam -r hg38.fa -o output.vcf`
**Explanation:** This command takes pre-aligned BAM reads and the human reference genome to produce a VCF file of called variants.

### Enable SNP and indel detection simultaneously
**Args:** `call -b sample.bam -r ref.fa --include-snps --include-indels -o variants.vcf`
**Explanation:** Specifying both SNP and indel flags ensures comprehensive variant detection across all variant types in a single run.

### Use GPU acceleration for faster processing
**Args:** `call -b sample.bam -r ref.fa --device cuda:0 --batch-size 32 -o output.vcf`
**Explanation:** Enabling CUDA device with increased batch size significantly speeds up inference for large datasets by parallelizing neural network computation.

### Set minimum coverage threshold for variant filtering
**Args:** `call -b sample.bam -r ref.fa --min-coverage 15 --min-qual 30 -o filtered.vcf`
**Explanation:** Requiring minimum 15x coverage and Phred quality 30 ensures only high-confidence variants appear in the final call set.

### Run with streaming mode for real-time analysis
**Args:** `stream -i fast5_dir/ -r ref.fa --model model.onnx -o stream.vcf`
**Explanation:** Streaming mode processes FAST5 files in real-time during sequencing, outputting variants as they are detected without waiting for complete dataset completion.

### Export variant database for query tools
**Args:** `call -b sample.bam -r ref.fa --make-vdb -o variant.vdb`
**Explanation:** Generating the VDB format allows use of variant query tools to filter and extract specific variant subsets after calling completes.

### Adjust homopolymer indel sensitivity
**Args:** `call -b sample.bam -r ref.fa --homopolymer-adjust -- indel-window 10 -o adjusted.vcf`
**Explanation:** Adjusting the homopolymer window parameter improves indel calling accuracy in repetitive regions prone to basecalling errors.