---
name: beacon2-import
category: Genomic Data Import/Integration
description: Imports genomic variant data (VCF, BCF, or other formats) into a Beacon-compatible database for query and sharing. Handles variant annotation, indexing, and metadata extraction.
tags: [genomics, variant-calling, beacon, vcf, data-import, ga4gh, database]
author: AI-generated
source_url: https://github.com/ga4gh-beacon/beacon-python
---

## Concepts

- **Input Formats**: Supports VCF (Variant Call Format), compressed VCF (.vcf.gz), and BCF (Binary VCF) as primary inputs, automatically detecting format via file extension and magic bytes.
- **Indexing Strategy**: Creates positional indexes using companion binary `beacon2-import-build` for rapid range queries; indexes are mandatory for Beacon server queries to function correctly.
- **Metadata Extraction**: Parses INFO and FORMAT fields from VCF headers, extracting sample genotypes, variant quality scores (GQ), and read depths (DP) into structured database tables.
- **Reference Genome Alignment**: Requires a compatible reference genome (FASTA) to be specified for coordinate validation; mismatched references cause query failures.

## Pitfalls

- **Mismatched Reference Genome**: Supplying a reference genome FASTA that doesn't match the VCF contig names results in silent coordinate mismatches, causing all Beacon queries to return empty results.
- **Forgetting to Build Indexes**: Running `beacon2-import` without subsequently running `beacon2-import-build` leaves the database unindexed, making queries extremely slow or timing out entirely.
- **Compressed VCF Without Index**: Using bgzip-compressed VCF files (.vcf.gz) without a corresponding .tbi (tabix) index causes import to fail; always ensure `.vcf.gz.tbi` exists alongside the data file.
- **Duplicate Sample Names**: If the VCF contains duplicate sample names in the header, the import process will overwrite earlier records silently, leading to data loss.

## Examples

### Import a single uncompressed VCF file into the database
**Args:** `--input variants.vcf --output beacon.db --reference hg38.fa`
**Explanation:** Imports a standard VCF file into a SQLite database using hg38 as the reference, creating a new database at beacon.db.

### Import a bgzip-compressed VCF file with automatic index detection
**Args:** `--input dataset.vcf.gz --output beacon.db --reference hg38.fa --force`
**Explanation:** Imports a bgzip-compressed VCF file, overwriting any existing database; the tool automatically locates the companion .tbi index file.

### Import BCF format with explicit reference specification
**Args:** --input multisample.bcf --output beacon.db --reference GRCh38.fa --sample-names sample1,sample2
**Explanation:** Imports a binary VCF file containing multiple samples, explicitly listing sample names to control metadata extraction order.

### Rebuild indexes after schema changes
**Args:** --reindex --database beacon.db --output beacon_index.idx
**Explanation:** Rebuilds the positional index on an existing database, useful when the underlying schema has changed or indexes are corrupted.

### Import with verbose logging for debugging import failures
**Args:** --input test.vcf --output test.db --reference hg38.fa --log-level debug --verbose
**Explanation:** Imports with debug-level logging enabled, outputting detailed parsing information to stderr for troubleshooting malformed VCF records.

### Import only specific chromosomes to reduce database size
**Args:** --input chr_subset.vcf --output beacon.db --reference hg38.fa --regions chr1,chr2,chr3
**Explanation:** Restricts import only to chromosomes 1, 2, and 3, ignoring all other contigs in the VCF file to create a smaller targeted database.

### Export imported data back to VCF for verification
**Args:** --export --database beacon.db --output verified_export.vcf --export-format vcf
**Explanation:** Exports the internal database representation back to VCF format, allowing verification that import handled all records correctly.