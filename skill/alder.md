---
name: alder
category: Variant Calling / Genomics
description: A command-line tool for detecting variants from sequencing data, supporting VCF/BCF input/output and multiple alignment formats. Designed for germline and somatic variant discovery in BAM/CRAM files.
tags: [variant-calling, genomics, vcf, bcf, bam, germline, somatic, snp, indel]
author: AI-generated
source_url: https://github.com/example/alder
---

## Concepts

- **I/O Formats**: alder accepts aligned reads in BAM or CRAM format and outputs variants in VCF (text) or BCF (binary) format. The tool automatically detects compressed files by `.gz` or `.bcf` extensions.
- **Data Model**: Variants are called using a Bayesian likelihood model that considers read base qualities, mapping qualities, and read positioning. The output includes INFO fields for metrics like QD, FS, and MQ.
- **Reference Indexing**: When using a FASTA reference, alder requires a FAI index file in the same directory. Without a valid index, the tool aborts with an error indicating missing reference annotations.
- **Multi-threading**: The `--threads` flag controls parallel processing, with each thread handling a genomic region chunk. Thread count directly impacts memory usage and runtime.

## Pitfalls

- **Missing Reference Index**: Running alder on a FASTA without a corresponding `.fai` index causes immediate failure. Always generate the index with `samtools faidx` before variant calling.
- **Incompatible Read Groups**: If BAM files lack read group tags (`@RG`), alder may silently merge samples incorrectly, producing misleading heterozygous ratios in pooled samples.
- **Memory Overcommitment**: Setting `--threads` higher than available CPU cores causes excessive context switching and can crash the process with out-of-memory errors, especially when processing large contigs.
- **Wrong Sample Naming**: Specifying `--samples` with names not present in the BAM header results in no variant calls for those samples without warning, leading to empty output files.

## Examples

### Call variants from a single BAM file against a reference
**Args:** `-R ref.fa -o output.vcf input.bam`
**Explanation:** Uses the FASTA reference to call variants from a single alignment file and writes results in VCF format.

### Call variants from multiple BAMs with specific sample names
**Args:** `-R ref.fa -S sample1,sample2 -o output.vcf align1.bam align2.bam`
**Explanation:** Explicitly specifies sample names in the output VCF, allowing differentiation when multiple BAMs contain the same sample identifier.

### Output in BCF format for downstream processing
**Args:** `-R ref.fa -Ov -o output.bcf input.bam`
**Explanation:** Outputs binary BCF format instead of text VCF, reducing file size and speeding up subsequent filtering with bcftools.

### Use 8 threads for parallel processing
**Args:** `-R ref.fa --threads 8 -o output.vcf input.bam`
**Explanation:** Enables multi-threaded execution to partition the genome into chunks processed concurrently, reducing runtime on multi-core systems.

### Filter variants by minimum quality threshold
**Args:** `-R ref.fa -o output.vcf -Q 30 input.bam`
**Explanation:** Only outputs variants with quality (QUAL) score >= 30, reducing false positive calls in the final VCF.

---

*Last updated: 2025-01-15*