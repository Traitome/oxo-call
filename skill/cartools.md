---
name: cartools
category: Genomics/Assembly Tools
description: A suite of command-line utilities for consensus sequence assembly, variant calling, and genomic coordinate manipulation. Operates on FASTA, VCF, and BAM inputs to produce polished consensus sequences and phased haplotype assemblies.
tags: [consensus, variant-calling, assembly, haplotype, genomics, fasta, vcf]
author: AI-generated
source__url: https://github.com/bioinformatics-tools/cartools
---

## Concepts

- **Consensus Sequence Generation**: cartools builds consensus sequences from multiple sequence alignments (MSA) or BAM pileups by applying configurable base-quality thresholds (--min-base-qual) and coverage cutoffs (--min-coverage). The algorithm prefers IUPAC ambiguous base codes when heterozygosity is detected above the allele frequency threshold (--het-cutoff).
- **Input/Output Formats**: The tool accepts FASTA (reference and reads), VCF (variants), and sorted indexed BAM/CRAM files. Output formats are controlled by --out-format and include FASTA, FASTQ, VCF, and JSON. Variant calls are emitted as bgzip-compressed VCF with companion .tbi index unless --no-index is specified.
- **Haplotype Phasing**: cartools groups variants into phased blocks using read-backed evidence when --phase is enabled. Phase blocks are written to phased VCF with PS tags and to separate haplotype FASTA files (--haplotype-out). Phase accuracy is estimated from read coverage and anchoring k-mer uniqueness.
- **Coordinate System**: Genomic coordinates follow BED-like zero-origin, half-open intervals for interval operations. Contig names must match the reference dictionary exactly; case-sensitive matching is enforced unless --ignore-case is set.

## Pitfalls

- **Mismatched Reference Dictionary**: Specifying a FASTA reference without its companion .fai index causes cartools to abort with "Index file not found" instead of proceeding. Users must run `samtools faidx reference.fa` before any coordinate-based operation, or cartools will fail silently on small contigs while erroring on large ones.
- **Inconsistent Sample Names**: When processing multiple samples (--samples), VCF sample columns must match exactly. A trailing newline or whitespace mismatch in the sample list file causes cartools to skip that sample without warning, resulting in missing output haplotypes that are difficult to diagnose.
- **Memory Exhaustion on Deep Coverage**: BAM files with coverage exceeding 500× can cause cartools to allocate >32 GB RAM during pileup caching, leading to OOM kills. The --cache-size flag limits in-memory caching but degrades performance when set below 10% of average coverage depth.
- **Silent Overwriting of Output**: cartools does not check for existing output files by default. Specifying --out with a path to an existing VCF overwrites it atomically, which can corrupt pipelines that depend on atomic rename semantics.

## Examples

### Generate a consensus FASTA from a BAM file aligned to a reference

**Args:** consensus --ref reference.fa --bam alignments.bam --out consensus.fa --min-coverage 20 --min-base-qual 25
**Explanation:** This builds a consensus sequence requiring at least 20× coverage and base quality 25, emitting ambiguous bases for low-confidence positions.

### Call variants and output a bgzip-compressed VCF

**Args:** variants --ref reference.fa --bam tumor.bam --normal normal.bam --out tumor_vs_normal.vcf.gz --out-format vcf --filter "DP > 50"
**Explanation:** This performs tumor-normal calling, applying a read depth filter and writing a compressed VCF ready for bcftools or rtgtools downstream.

### Phase heterozygous variants into haplotype blocks

**Args:** phase --bam sample.bam --ref GRCh38.fa --out phased.vcf --haplotype-out haplotypes.fa --phase-window 50000
**Explanation:** This identifies heterozygous SNPs and assigns them to phase blocks spanning up to 50 kb using read-backed evidence, outputting phased VCF and separate haplotype sequences.

### Extract a genomic region and realign reads to a custom contig

**Args:** extract-ref --region chr12:1000000-2000000 --ref reference.fa --out extracted.fa && align --reads reads.fastq --ref extracted.fa --out aligned.bam
**Explanation:** This extracts a genomic window and realigns input reads against it, which is useful for扩增子 (amplicon) sequencing or viral quasispecies analysis.

### Generate a JSON report of called variants with annotations

**Args:** variants --ref reference.fa --bam sample.bam --out report.json --out-format json --include-ann --ann-db vep_cache/
**Explanation:** This outputs variant calls in JSON format with functional annotations pulled from a VEP-style cache, suitable for programmatic downstream pipelines.

### Merge multiple VCF files into a joint-callset

**Args:** merge-vcf --inputs sample1.vcf.gz sample2.vcf.gz sample3.vcf.gz --out joint.vcf.gz --normalize --split-multiallelics
**Explanation:** This combines multiple single-sample VCFs into a joint multisample VCF, applying left-normalization and splitting multiallelic sites to meet VCF specification requirements.