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
- tabix -C creates CSI index instead of TBI; required for chromosomes >512 Mb or for coordinate ranges >2^29.
- tabix -R restricts queries to regions listed in a BED file; useful for batch region extraction.
- tabix -T streams through a file rather than index-jumping; slower but works without an index.
- tabix -0 indicates zero-based coordinates (BED format); default is 1-based (VCF/GFF).

## Pitfalls

- CRITICAL: tabix has NO subcommands. ARGS starts directly with flags (e.g., -p, -h, -l, -C) or with the input file for indexing/querying. Do NOT put a subcommand like 'index' or 'query' before flags.
- gzip-compressed files CANNOT be indexed with tabix — always use bgzip instead of gzip.
- The file must be sorted by chromosome and position before bgzip+tabix — unsorted files cause errors.
- Without -h when querying, the VCF header is NOT included in output — this creates invalid VCF.
- Tabix index (.tbi or .csi) must be in the same directory as the data file.
- tabix -p vcf automatically detects VCF format; for BED use -p bed explicitly.
- The contig names in the query must EXACTLY match those in the file (chr1 vs 1 mismatch is common).
- CSI index (-C) is required for chromosomes >512 Mb; TBI has a 512 Mb limit per chromosome.
- BED files are 0-based; use -0 flag when indexing BED files to ensure correct coordinate interpretation.
- -R and -T both accept BED files, but -R uses index jumping (fast) while -T streams (slow, no index needed).

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

### create a CSI index for large genomes with contigs >512 Mb
**Args:** `-C variants.vcf.gz`
**Explanation:** -C creates a CSI index instead of TBI; required for chromosomes longer than 512 Mb (e.g., human chr1 in some assemblies)

### query multiple regions at once from an indexed VCF
**Args:** `-h variants.vcf.gz chr1:1000000-2000000 chr2:500000-1000000 > multi_region.vcf`
**Explanation:** multiple region arguments are supported; -h includes the header; output is concatenated into one file

### index a bgzipped GFF3 annotation file
**Args:** `-p gff annotation.gff3.gz`
**Explanation:** -p gff specifies GFF/GFF3 format; file must be sorted by seqname and start; creates annotation.gff3.gz.tbi

### fetch a remote indexed VCF region without downloading the whole file
**Args:** `-h https://example.com/variants.vcf.gz chr1:1000-2000 > remote_region.vcf`
**Explanation:** tabix supports HTTP/HTTPS/FTP URLs; only the relevant index blocks are fetched; requires the .tbi file at the same URL

### reindex a tabix file using a custom sequence dictionary order
**Args:** `-s 1 -b 2 -e 3 custom_format.bed.gz`
**Explanation:** -s specifies the sequence name column, -b the start column, -e the end column; use for non-standard tab-delimited files that tabix cannot auto-detect

### query regions from a BED file list
**Args:** `-h -R regions.bed variants.vcf.gz > subset.vcf`
**Explanation:** -R restricts to regions listed in BED file; useful for batch extraction of multiple regions; -h includes header

### stream through a file without using index
**Args:** `-T regions.bed variants.vcf.gz > subset.vcf`
**Explanation:** -T streams rather than index-jumps; works without .tbi index but slower; useful when index is unavailable

### print only header lines from indexed VCF
**Args:** `-H variants.vcf.gz`
**Explanation:** -H prints only header lines; useful for checking VCF version, sample names, and contig definitions

### create CSI index with custom min-shift for very large chromosomes
**Args:** `-C -m 16 variants.vcf.gz`
**Explanation:** -C creates CSI index; -m 16 sets minimal interval size to 2^16 (65536); useful for extremely large genomes

### index a SAM file for region queries
**Args:** `-p sam alignments.sam.gz`
**Explanation:** -p sam specifies SAM format; creates index for SAM alignments; useful for quick region extraction from SAM files

### query with multiple threads for faster retrieval
**Args:** `-h -@ 4 variants.vcf.gz chr1:1000000-2000000 > region.vcf`
**Explanation:** -@ 4 uses 4 threads for decompression; speeds up queries on large compressed files
