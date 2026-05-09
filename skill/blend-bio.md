---
name: blend-bio
category: bioinformatics-data-processing
description: A tool for processing, filtering, and blending biological sequence data and annotations. Supports operations on FASTA, FASTQ, VCF, andBED formats with streaming capabilities for large-scale genomic datasets.
tags:
  - sequence-analysis
  - data-filtering
  - format-conversion
  - genomics
  - stream-processing
author: AI-generated
source_url: https://github.com/bio-tools/blend-bio
---

## Concepts

- **Input format auto-detection**: blend-bio automatically detects input file formats (FASTA, FASTQ, VCF, BED) based on file extensions and magic bytes in the file header, so you do not need to manually specify the format for standard file extensions.
- **Streaming architecture**: The tool processes data in a streaming manner, which means it can handle files larger than available RAM by never loading the entire file into memory; use standard input redirection for pipeline integration.
- **Filter chaining**: Multiple filter operations can be chained together in a single invocation using the `--filter` flag multiple times or by specifying comma-separated filter expressions, and filters are applied in order from left to right.
- **Output format independence**: The output format is specified independently from the input format using the `--out-format` flag, allowing conversion between formats (e.g., FASTQ to FASTA) as part of a processing pipeline.

## Pitfalls

- **Mismatched filter criteria for file format**: Applying a QUAL filter on a FASTA file will silently produce zero output records because FASTA files do not contain quality scores, leading to seemingly successful but empty results.
- **Integer overflow in coordinate ranges**: Specifying very large values for `--max-length` or `--min-length` without the `K`, `M`, or `G` suffix may cause unexpected truncation since these values are parsed as raw integers, resulting in silently ignored records that exceed the actual limit.
- **Overwriting input files**: The default behavior of `--output` is to overwrite existing files without prompting, and combined with shell redirection, this can permanently destroy source data if the wrong filename is specified.
- **Loss of paired-end information**: When converting paired-end FASTQ files to single-end format using `--out-format fasta`, the tool discards read pair relationships, making downstream assembly impossible without maintaining the original pairing metadata.

## Examples

### Filter reads by length threshold
**Args:** `input.fastq --min-length 50 --out filtered.fastq`
**Explanation:** This removes all reads shorter than 50 nucleotides from the input FASTQ file, which is useful for removing low-complexity or adapter-derived sequences before assembly.

### Convert FASTQ to FASTA format
**Args:** `reads.fastq --out-format fasta --output reads.fasta`
**Explanation:** This converts a FASTQ file containing quality scores to a simple FASTA format, discarding quality information since FASTA does not support quality encoding.

### Extract reads matching a sequence pattern
**Args:** `dataset.fastq --filter "seq=ATCGATCG" --out matched.fastq`
**Explanation:** This extracts only reads containing the exact sequence "ATCGATCG" anywhere in the read, useful for primer detection or targeted sequence recovery.

### Convert VCF to BED format for genome browsers
**Args:** `variants.vcf --out-format bed --output variants.bed`
**Explanation:** This converts a VCF file containing genomic variants to BED format, which is directly compatible with UCSC Genome Browser and other visualization tools.

### Filter variants by quality score threshold
**Args:** `raw_variants.vcf --filter "QUAL>30" --output highq_variants.vcf`
**Explanation:** This removes all variants with a quality score below 30 from the VCF file, which is the recommended first step in variant filtering pipelines to reduce false positives.