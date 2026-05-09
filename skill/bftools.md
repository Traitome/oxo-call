---
name: bftools
category: Bioinformatics Utilities
description: A suite of command-line utilities for viewing, converting, filtering, and manipulating common bioinformatics file formats including BED, VCF, BAM, and FASTQ. Provides efficient streaming operations for large genomic datasets.
tags:
  - bioinformatics
  - genomics
  - file-conversion
  - data-processing
  - vcf
  - bed
  - bam
  - fastq
author: AI-generated
source_url: https://github.com/placeholder/bftools
---

## Concepts

- **Input/Output Format Handling**: bftools reads/writes multiple bioinformatics formats (BED, VCF, BAM, FASTQ) with automatic format detection based on file extensions; use `-f` flag to specify format explicitly when extensions are ambiguous.
- **Streaming Pipeline Architecture**: Most bftools commands support stdin/stdout for Unix pipe integration, enabling efficient multi-step workflows without intermediate files; works with tools like `sort`, `grep`, and `awk`.
- **Region-Based Operations**: Many commands accept genomic coordinates in chromosome:start-end format for extracting or processing specific genomic regions; coordinates are 1-based and inclusive for BED files.
- **Index Dependency**: Commands operating on sorted BED/VCF files can use indexed `.tbi` or `.bai` files for rapid random access; always index large files with companion tools before region queries.

## Pitfalls

- **Forgetting to Sort Before Indexing**: Attempting to create a tabix index on unsorted BED/VCF files causes silent indexing failures leading to failed or incorrect region queries later; always sort input files first.
- **Mismatched Chromosome Naming**: Mixing chromosome names with and without 'chr' prefixes (e.g., 'chr1' vs '1') between query and index causes zero results with no error; ensure consistent naming conventions.
- **Writing to Input File**: Using the same file for output as the input (e.g., `bftools view input.bed > input.bed`) truncates the file before reading completes, resulting in data loss; always write to a different file or use `-o` flag for in-place updates.
- **Ignoring Header Lines**: Processing VCF/BED files without preserving headers using appropriate flags removes metadata essential for downstream tools; use `-h` flag to preserve headers in output.

## Examples

### Viewing a BED file with headers
**Args:** `view -h input.bed`
**Explanation:** Displays the BED file contents including all header lines (track definitions in BED format), preserving metadata needed for visualization or downstream processing.

### Extracting records for a specific genomic region
**Args:** `filter -i chr1:1000000-2000000 -f bed input.bed`
**Explanation:** Extracts all BED records overlapping chromosome 1 positions 1,000,000 to 2,000,000, enabling targeted analysis of specific genomic intervals without loading entire files.

### Converting VCF to BED format
**Args:** `convert -i input.vcf -o bed -o output.bed`
**Explanation:** Transforms VCF variant calls into BED format, making variant data compatible with tools that require genomic interval inputs like bedtools or IGV.

### Counting FASTQ sequences in a file
**Args:** `stats -c -f fastq input.fastq`
**Explanation:** Counts the total number of FASTQ sequences (reads) in the file by counting record identifiers, useful for quality control and dataset sizing before alignment.

### Merging overlapping BED intervals
**Args:** `merge -d 10 input.bed -o merged.bed`
**Explanation:** Combines overlapping or adjacent BED intervals within 10bp distance, reducing redundant intervals and creating a non-overlapping segment file for coverage analysis.

### Filtering VCF by QUAL score threshold
**Args:** `filter -q 30 -f vcf raw.vcf -o filtered.vcf`
**Explanation:** Retains only VCF variants with quality (QUAL) scores of 30 or higher, removing low-confidence variant calls to improve downstream genotyping accuracy.

### Extracting specific sample from multi-sample VCF
**Args:** `extract -s NA12878 -f vcf multisample.vcf -o NA12878.vcf`
**Explanation:** Extracts all variant records for sample NA12878 from a multi-sample VCF file, enabling individual-level analysis without carrying unnecessary sample data.