---
name: athena_meta
category: Bioinformatics Utilities
description: A command-line tool for extracting and managing metadata from bioinformatics data files, supporting multiple formats including VCF, BAM, and FASTQ headers.
tags: [metadata, vcf, bam, fastq, header-parsing, genomics]
author: AI-generated
source_url: https://github.com/athena-ecosystem/athena_meta
---

## Concepts

- **VCF/BAM Header Parsing**: athena_meta extracts metadata from VCF headers (##INFO and ##FORMAT fields) and BAM @SQ/@RG/@PG dictionary records, allowing inspection of sample identifiers, read groups, and pipeline versioning.

- **Multi-Format Support**: The tool processes FASTQ @HDR headers, SAM/BAM optional fields, and VCF INFO/FORMAT annotations, outputting structured JSON or TSV for downstream pipelines.

- **Batch Processing**: Multiple input files can be processed simultaneously using glob patterns or explicit file lists, with output aggregation into a single metadata summary table.

- **Field Filtering**: Users can specify inclusion (-i/--include) or exclusion (-e/--exclude) filters using regex patterns to extract only relevant metadata keys, reducing output noise.

- **Output Formatting**: Results can be exported as JSON (-o json), TSV (-o tsv), or YAML (-o yaml), with customizable column headers and nested field flattening.

## Pitfalls

- **Missing Header Tags**: If an input file lacks expected metadata tags (e.g., no @RG in BAM), athena_meta silently omits them; always verify output row counts match expected sample numbers.

- **Duplicate Keys in Multi-Sample VCF**: When processing multi-sample VCFs with duplicate INFO tags, output may contain conflicting values; use --merge-strategy "last" or "first" to disambiguate.

- **Path Globbing Mistakes**: Wildcard patterns like `*.vcf` relative to the wrong directory produce empty output; always verify with --dry-run before batch processing.

- **Encoding Issues**: Non-UTF8 characters in custom VCF headers cause parsing failures; sanitize input files or use --encoding "latin1" as a workaround.

- **Memory Limits on Large Files**: Very large VCF files (>10GB) may exceed default memory limits; increase with --max-memory "16g" or process in chunks using --chunk-size.

## Examples

### Extract all metadata from a VCF file
**Args:** input.vcf --output-format json
**Explanation:** Parses all ##INFO and ##FORMAT header lines from the VCF file and outputs them as JSON for easy programmatic consumption.

### List sample IDs from a multi-sample BAM
**Args:** sample.bam --include "^@RG.*SM" --output-format tsv
**Explanation:** Filters BAM read group headers to extract only sample name (SM) tags, outputting as tab-separated values for spreadsheet analysis.

### Find all pipeline versions in a directory of VCFs
**Args:** *.vcf --include ".*PG.*VN" --merge --output-format yaml
**Explanation:** Uses glob pattern to process multiple VCF files, extracts versioning info from the ##PG lines, and merges duplicates into a single YAML document.

### Export read group metrics to JSON
**Args:** input.bam -e "^@SQ" --output json -o read_groups.json
**Explanation:** Excludes SQ (sequence) dictionary entries but includes all @RG (read group) metadata, outputting to a specific JSON file.

### Extract specific INFO fields from VCF
**Args:** input.vcf --include "^INFO\\..*DP$" --columns CHROM,ID,INFO_DP
**Explanation:** Uses regex to extract only depth (DP) INFO fields and formats output with specific column headers for targeted analysis.

### Batch process FASTQ files with gzip compression
**Args:** *.fastq.gz --input-format fastq --output merged_meta.tsv
**Explanation:** Processes multiple gzipped FASTQ files simultaneously, extracting @HDR headers and appending all results to a single TSV output file.

### Filter to specific metadata keys
**Args:** dataset.vcf -i "^##SAMPLE" --output-format json
**Explanation:** Includes only SAMPLE meta information lines from the VCF header, useful for quickly inspecting sample-level annotations.

### Handle encoding variations in custom headers
**Args:** legacy.vcf --encoding "latin1" -o json
**Explanation:** Specifies latin1 encoding to parse non-standard UTF8 characters in custom VCF header fields, preventing parsing failures.

### Check what's in a file without outputting
**Args:** input.vcf --dry-run
**Explanation:** Runs in preview mode to show what metadata would be extracted without writing output, useful for testing filter patterns.

### Process with increased memory for large files
**Args:** huge.vcf --include ".*" --max-memory "16g" -o output.json
**Explanation:** Processes a very large VCF file with explicit memory allocation to prevent out-of-memory errors during parsing.