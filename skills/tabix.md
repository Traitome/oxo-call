---
name: tabix
category: utilities
description: Index and query position-sorted genomic files (VCF, BED, GFF, GTF) for fast random-access lookups
tags: [indexing, vcf, bed, gff, compression, bgzip, tabular, utility]
author: oxo-call built-in
source_url: "http://www.htslib.org/doc/tabix.html"
---

## Concepts

- Tabix indexes tab-delimited genomic files that have been block-gzipped (bgzip) for fast random access queries.
- Always compress with bgzip (not gzip) before indexing: bgzip file.vcf creates file.vcf.gz.
- tabix -p vcf creates .tbi index for VCF; tabix -p bed for BED; tabix -p gff for GFF/GTF.
- Query a region: tabix indexed_file.vcf.gz chr1:1000-2000 > region.vcf
- tabix -h includes header lines in region query output; important for valid VCF output.
- Many tools (bcftools, GATK, samtools) require tabix-indexed .vcf.gz for region-specific operations.
- Use tabix -l to list all contigs/chromosomes in an indexed file.
- bgzip -d decompresses a bgzip file; bgzip -c compresses to stdout.

## Pitfalls

- gzip-compressed files CANNOT be indexed with tabix — always use bgzip instead of gzip.
- The file must be sorted by chromosome and position before bgzip+tabix — unsorted files cause errors.
- Without -h when querying, the VCF header is NOT included in output — this creates invalid VCF.
- Tabix index (.tbi or .csi) must be in the same directory as the data file.
- tabix -p vcf automatically detects VCF format; for BED use -p bed explicitly.
- The contig names in the query must EXACTLY match those in the file (chr1 vs 1 mismatch is common).

## Examples

### compress a VCF file with bgzip and create tabix index
**Args:** `variants.vcf.gz`
**Explanation:** first: bgzip variants.vcf → creates variants.vcf.gz; then: tabix -p vcf variants.vcf.gz → creates .tbi

### create tabix index for a bgzipped VCF file
**Args:** `-p vcf variants.vcf.gz`
**Explanation:** -p vcf specifies VCF format; creates variants.vcf.gz.tbi index file

### query a specific genomic region from an indexed VCF
**Args:** `-h variants.vcf.gz chr1:1000000-2000000 > chr1_region.vcf`
**Explanation:** -h includes header lines; queries chr1 from 1Mb to 2Mb; output redirected to new VCF

### create tabix index for a bgzipped BED file
**Args:** `-p bed regions.bed.gz`
**Explanation:** -p bed specifies BED format; BED file must be sorted by chromosome and start position

### list all chromosomes/contigs in an indexed VCF
**Args:** `-l variants.vcf.gz`
**Explanation:** -l lists all contig names in the index; useful for scripting region-based iteration
