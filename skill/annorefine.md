---
name: annorefine
category: Variant Annotation Refinement
description: A tool for refining and filtering variant annotations in VCF files using pre-built reference databases. It processes VCF input, applies annotation criteria, and outputs a refined VCF with updated INFO fields.
tags: [vcf, variant-annotation, filtering, genomics, snp, indel]
author: AI-Generated
source_-url: https://github.com/_refresh/annorefine
---

## Concepts

- **VCF Annotation Model**: `annorefine` reads VCF 4.0+ files and refines the INFO field annotations by comparing variant positions against pre-built reference databases (created using `annorefine-build`). The tool matches on chromosome, position, ref allele, and alt allele to locate corresponding annotations.
- **Input/Output Formats**: Input must be a valid VCF file (`.vcf` or `.vcf.gz`) containing at least CHROM, POS, ID, REF, ALT, QUAL, FILTER, and INFO columns. Output is a VCF written to stdout or a specified file, with original INFO fields preserved and supplemented by database annotations.
- **Database Architecture**: Reference databases are SQLite files produced by `annorefine-build`. These databases index genomic regions with associated annotation tracks, allowing fast lookup of overlapping or exact-match variants during the refinement process.
- **Filtering Criteria**: Variants can be filtered by allele frequency thresholds, annotation presence, genomic region ranges, and filter status flags. Multiple criteria may be combined using logical AND operations specified via command-line arguments.

## Pitfalls

- **Missing Reference Database**: Running `annorefine` without building a reference database first (using `annorefine-build`) will cause the tool to terminate with a "database not found" error and produce no output. Always build the database before running refinement jobs.
- **Chromosome Naming Mismatch**: If chromosome names in the input VCF (e.g., "chr1") do not match those in the reference database (e.g., "1"), the tool will fail to match variants and the output VCF will have empty annotation fields. Ensure consistent naming conventions before processing.
- **Compressed VCF Input Without Index**: Providing a bgzip-compressed VCF (`.vcf.gz`) without a corresponding `.tbi` index file will cause `annorefine` to read only the header and skip all variant records. Always create a tabix index using `bgzip` and `tabix` before processing compressed files.
- **Overwriting Output File**: Specifying an output file that already exists with the `-o` flag will silently overwrite the file without warning, potentially losing previously refined data. Use shell redirection or explicitly check for file existence beforehand.
- **Large VCF Memory Consumption**: Processing very large VCF files (millions of variants) without setting an appropriate batch size can cause excessive memory usage and potential OOM termination. Use the `-b` flag to control the number of variants processed per chunk.

## Examples

### Refine a single VCF file against a reference database
**Args:** `-d variants.db input.vcf.gz -o refined.vcf`
**Explanation:** This command reads the compressed VCF file, looks up each variant in the SQLite database built by `annorefine-build`, and writes the annotated output to a new VCF file.

### Refine with allele frequency filtering applied
**Args:** `--af-tag AF --af-max 0.01 -d variants.db sample.vcf -o rare_variants.vcf`
**Explanation:** This command filters variants so that only those with an allele frequency annotation below 1% (as stored in the AF INFO field tag) are included in the output file.

### Refine variants in a specific genomic region
**Args:** `-d variants.db -r chr1:1000000-2000000 input.vcf -o region_filtered.vcf`
**Explanation:** This command restricts processing to variants located within chromosomal coordinates 1,000,000 to 2,000,000 on chromosome 1, dramatically reducing processing time for large files.

### Output to compressed bgzip VCF with automatic tabix indexing
**Args:** `-d variants.db input.vcf -o refined.vcf.gz -z -i`
**Explanation:** This command writes the output as a bgzip-compressed VCF and automatically generates a corresponding tabix index file (`.vcf.gz.tbi`) for efficient downstream querying.

### Batch processing with explicit chunk size control
**Args:** `-d variants.db -b 50000 large_cohort.vcf -o batch_refined.vcf`
**Explanation:** This command processes the input VCF in batches of 50,000 variants per chunk, reducing peak memory consumption when handling millions of variants in a single file.