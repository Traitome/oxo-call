---
name: atol-reference-data-lookups
category: Bioinformatics - Reference Data
description: A command-line tool for performing fast lookups against reference genomic databases, including variant annotations, allele frequencies, and functional annotations.
tags:
  - reference-data
  - variant-lookup
  - genomic-database
  - annotation
  - bioinformatics
  - vcf
  - bed
author: AI-generated
source_url: https://github.com/atol-reference-data-lookups
---

## Concepts

- **Input Formats**: The tool accepts query files in BED, VCF, or plain text format (one genomic position per line as chr:start-end). Output can be returned as JSON, TSV, or VCF with annotations appended.
- **Reference Index**: All lookups require a pre-built reference index created by the companion tool `atol-reference-data-lookups-build`. Without a valid index, lookups will fail with an index not found error.
- **Annotation Sources**: The tool can query multiple annotation tracks simultaneously, including allele frequencies (gnomAD, TOPMed), pathogenicity scores (CADD, PolyPhen), and gene annotations (RefSeq, Ensembl).
- **Batch Processing**: Large query files are processed in chunks streaming from stdin to manage memory; the `--batch-size` flag controls the number of positions processed per chunk (default 1000).

## Pitfalls

- **Missing Index**: Running lookups without first building an index will produce a cryptic "index not found" error. Always run `atol-reference-data-lookups-build` before performing any lookups.
- **Coordinate Mismatch**: If query coordinates use 1-based indexing while the reference index was built with 0-based coordinates (or vice versa), results will be offset by one base pair, leading to incorrect annotations.
- **Uncompressed Input**: Passing gzipped BED/VCF files directly without decompressing first causes parse errors. Use `gunzip` or the `--input-format` flag to specify compression explicitly.
- **Memory Overflow**: Processing extremely large query files without adjusting `--batch-size` can exhaust RAM, especially when annotating many tracks. Reduce batch size for large inputs.

## Examples

### Look up variant annotations for a single genomic position
**Args:** `--index refdb --query 1:1234567-1234567 --output json`
**Explanation:** This queries the reference database at chromosome 1, position 1234567 and returns annotations in JSON format for programmatic downstream processing.

### Annotate variants from a VCF file using multiple tracks
**Args:** `--index refdb --input-vcf variants.vcf --tracks gnomad_exome,gnomad_genome,cadd --output-tsv annotated.tsv`
**Explanation:** This reads variants from a VCF file, annotates them with gnomAD exome and genome frequencies plus CADD scores, and writes results to a TSV file.

### Query allele frequencies for a list of positions
**Args:** `--index refdb --input-txt positions.txt --track gnomad_exome --output json`
**Explanation:** This reads genomic positions from a text file (one per line in chr:start-end format) and returns gnomAD allele frequency annotations.

### Build a reference index from a GFF and VCF database
**Args:** refdb_build --reference refgenome.fa --annotations annotations.gff --variants variants.vcf --out refdb.idx`
**Explanation:** This companion command builds a lookup index from a reference FASTA, GFF annotations, and variant VCF. The index is required for all subsequent lookups.

### Stream results to stdout for pipeline integration
**Args:** `--index refdb --input-vcf input.vcf --tracks topmed,refseq --format json --no-header`
**Explanation:** This outputs annotated results to stdout in JSON format without a header row, making it suitable for piping into other bioinformatics tools.

### Limit batch size for memory-constrained environments
**Args:** `--index refdb --input-vcf large_query.vcf --batch-size 100 --output-tsV result.tsv`
**Explanation:** This reduces memory usage by processing only 100 positions per batch, useful when running on systems with limited RAM.

### Filter results by functional consequence
**Args:** `--index refdb --query 22:123456-234567 --tracks cadd,polyphen --filter consequence=missense --output json`
**Explanation:** This filters lookups to only return annotations for missense variants, reducing output size and focusing on potentially deleterious mutations.