---
name: clove
category: variant-calling
description: A variant caller for Oxford Nanopore Technologies (ONT) sequencing data. Clove analyzes aligned Nanopore reads (BAM/CRAM format) to identify single nucleotide variants (SNVs) and small indels relative to a reference genome, outputting standard VCF files for downstream analysis.
tags: [nanopore, variant-calling, snv, indel, genomics, vcf, ont, sequencing]
author: AI-generated
source_url: https://github.com/rlorigro/clove
---

## Concepts

- **Input format**: Clove requires aligned Nanopore reads in BAM or CRAM format, with associated index files (.bai/.crai). The alignments must be generated against the same reference genome specified for variant calling. Unaligned FASTQ files are not accepted directly.
- **Reference genome**: A FASTA-format reference genome is mandatory. The sequence headers in the FASTA must exactly match the @SQ records in the BAM header. Using a different reference version or mismatched sequence names will cause silent false positives or missed variants.
- **Output format**: Clove emits a VCF 4.2-compliant file containing variant calls with QUAL scores, FILTER annotations, and INFO fields including allele depth (AD), genotype quality (GQ), and read support counts. Only variants meeting the minimum quality threshold are retained.
- **Sequencing technology specificity**: Clove is optimized for the error profile of Oxford Nanopore reads (higher indel error rate compared to Illumina), and applies Nanopore-specific basecalling quality scores for variant scoring. Do not use with Illumina or PacBio data without appropriate parameter adjustment.
- **Multi-sample analysis**: Clove supports joint genotyping of multiple samples from a single BAM, enabling population-scale variant discovery and shared variant identification across samples in one run.

## Pitfalls

- **Mismatched reference genome**: Using a reference genome whose sequence names or lengths do not exactly match the BAM @SQ lines will cause the variant caller to either fail with an error or silently produce incorrect calls. Always verify sequence name concordance with `samtools dict` or the BAM header before running.
- **Low Nanopore coverage**: Variant calling below 20x coverage dramatically increases false positive rates and reduces sensitivity for heterozygous variants. For reliable diploid calling, aim for minimum 30x coverage per sample.
- **Missing BAM index**: Clove requires a corresponding .bai index file in the same directory as the input BAM. Without it, the tool will fail to random-access the alignments. Generate with `samtools index` if missing.
- **Ignoring motif-related errors**: Nanopore sequencing has systematic errors at certain sequence motifs (e.g., homopolymers). Failing to filter these regions or set appropriate minimum allele frequency thresholds will include spurious indels in the output VCF.

## Examples

### Call variants from a single Nanopore BAM
**Args:** -b reads.bam -r reference.fasta -o variants.vcf
**Explanation:** This basic command specifies an aligned BAM file, reference genome, and outputs a VCF file with called SNVs and indels. Clove uses default quality thresholds and scoring parameters.

### Run with explicit minimum coverage threshold
**Args:** -b reads.bam -r reference.fasta -o variants.vcf -m 20
**Explanation:** The `-m 20` flag sets a minimum read depth of 20 supporting reads to call a variant. This reduces false positives in low-coverage regions but may miss true variants in under-sampled areas.

### Generate output with gVCF for joint analysis
**Args:** -b reads.bam -r reference.fasta -o variants.g.vcf -g
**Explanation:** The `-g` flag outputs a genomic VCF (gVCF) that includes reference (non-variant) calls at all callable positions, essential for joint genotyping workflows with multiple samples.

### Filter variants by quality score
**Args:** -b reads.bam -r reference.fasta -o variants.vcf -q 30
**Explanation:** Setting `-q 30` requires variants to have a minimum QUAL score of 30, corresponding to 99.9% confidence. This is a stringent filter that reduces false positives at the cost of potentially excluding true low-frequency variants.

### Specify output compression and index
**Args:** -b reads.bam -r reference.fasta -o variants.vcf.gz -z -i
**Explanation:** The `-z` flag compresses output to BCF (compressed VCF), and `-i` creates a corresponding index file (.tbi) for fast random access. Essential for large datasets to reduce disk usage and improve downstream querying speed.