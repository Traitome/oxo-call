---
name: baqlava
category: Bioinformatics / Sequence Analysis
description: A high-performance bioinformatics tool for processing and analyzing genomic sequences, supporting variant calling, alignment filtering, and sequence indexing operations.
tags: [genomics, variant-calling, sequence-analysis, bioinformatics, dna-analysis]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/baqlava
---

## Concepts

- **Data Model:** baqlava processes FASTQ/FASTA input files and produces VCF/BED output formats; internally uses a compressed read index stored in memory-mapped `.bqi` files for rapid random access to genomic coordinates.
- **I/O Formats:** Supports interleaved paired-end FASTQ (`--interleaved`), gzipped inputs (`-z`), multi-sample VCF merging (`--merge-vcf`), and BED annotation overlay (`--annotate-bed`).
- **Key Behaviors:** The tool operates in three modes: `index` (build reference index), `call` (variant discovery), and `filter` (post-processing); each mode has independent flag sets and output expectations.
- **Indexing:** Companion binary `baqlava-build` creates reference indexes that baqlava `call` mode automatically discovers using the `.bht` index registry file in the working directory.

## Pitfalls

- **Mismatched reference indices:** Using a reference genome for `call` mode without first running `baqlava-build` on that exact reference produces silent incorrect variant calls without error messages.
- **Memory overrun with large FASTQ files:** Specifying `--threads` without considering memory-per-thread (default 512MB) causes OOM kills on systems with limited RAM when processing >10M reads.
- **VCF output naming collisions:** Overwriting existing VCF files with `--output` is not prevented; the tool silently appends new variants, requiring manual cleanup.
- **Compressed input assumptions:** Passing both gzipped and plain FASTQ inputs in a batch without using `--force-raw` results in parser errors on the compressed files.

## Examples

### Building a reference genome index for variant calling
**Args:** `build --reference hs37d5.fa --index-dir ./baqlava_idx`
**Explanation:** Creates a memory-mapped index in the specified directory, enabling fast random access during subsequent `call` mode operations on the hs37d5 human reference.

### Calling variants from a paired-end FASTQ dataset
**Args:** `call --reads sample_R1.fq.gz --reads2 sample_R2.fq.gz --reference hs37d5.fa --output sample.vcf`
**Explanation:** Performs variant discovery on the provided paired reads against the reference, outputting variants in standard VCF 4.1 format.

### Filtering called variants by minimum quality score
**Args:** `filter --input sample.vcf --min-qual 30 --min-depth 10 --output sample_filtered.vcf`
**Explanation:** Removes variants with quality scores below 30 or read depth below 10x, producing a higher-confidence variant set.

### Running variant calling with multi-threading for speed
**Args:** `call --reads sample.fq --reference hg38.fa --threads 8 --output sample.vcf`
**Explanation:** Enables parallel processing across 8 threads, significantly reducing runtime on multi-core systems at the cost of higher memory usage.

###Annotating variants with a BED file of gene regions
**Args:** `call --reads sample.fq --reference hg38.fa --annotate-bed genes.bed --output annotated.vcf`
**Explanation:** Overlays gene region annotations from the BED file onto called variants, adding INFO field tags for nearby genes.

### Merging multiple VCF files into a single cohort file
**Args:** `filter --merge-vcf sample1.vcf sample2.vcf sample3.vcf --output cohort.vcf`
**Explanation:** Combines variants from multiple individual sample VCFs into a single cohort-level VCF with multi-sample genotype columns.