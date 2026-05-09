---
name: Bioconvert
category: bioinformatics/file-conversion
description: A Python tool for converting between bioinformatics file formats including FASTA, FASTQ, SAM, BAM, VCF, CSV, TSV, BED, GTF, GFF3, and many others. Automatically detects formats based on file extensions and supports batch conversion.
tags: [conversion, format-converter, bioinformatics, fasta, fastq, sam, bam, vcf, bed, gtffile]
author: AI-generated
source_url: https://github.com/bioconvert/bioconvert
---

## Concepts

- Bioconvert automatically detects input and output formats based on file extensions, so you typically only need to specify the input and output files without explicit format flags.
- The tool supports many sequence formats (FASTA, FASTQ, GENBANK, EMBL), alignment formats (SAM, BAM, CRAM), variant formats (VCF, BCF), and annotation formats (BED, GTF, GFF3, BEDGRAPH).
- Bioconvert uses sequential or parallel pipelines for conversion: some formats require intermediate steps (e.g., FASTA → SAM requires going through FASTQ and alignment).
- Batch conversion is supported by providing multiple input files or using glob patterns, allowing processing of many files at once with the `--batch` flag.
- Conversion can be parallelized using built-in threading or multiprocessing to speed up processing of large files or many files.

## Pitfalls

- Specifying incorrect output file extensions will cause Bioconvert to fail or produce invalid output files, as it relies on extension-based format detection rather than explicit format arguments in most cases.
- Attempting to convert between formats that lack a direct or indirect conversion pathway will fail—for example, converting directly from unaligned BAM to BED is not supported without intermediate steps.
- Running Bioconvert without sufficient disk space for intermediate or temporary files can cause crashes, especially when converting to formats that require temporary storage (e.g., some compression operations).
- Using outdated Bioconvert versions may cause compatibility issues with newer file format specifications (e.g., VCF 4.3 specifications) and missing format support.
- Failing to install optional dependencies (like pysam for SAM/BAM support, or pyVCF for VCF support) will result in silent failures or incomplete conversions for those specific formats.

## Examples

### Convert a FASTA file to FASTQ with dummy quality scores

**Args:** `input.fasta output.fastq`

**Explanation:** This converts a FASTA file to FASTQ format, automatically assigning a default quality score (typically 'I' or 40) to all bases since FASTQ requires quality scores not present in FASTA.

### Convert a SAM file to BAM format

**Args:** `input.sam output.bam`

**Explanation:** This converts a SAM (text) alignment file to BAM (binary compressed) format, significantly reducing file size while preserving all alignment information.

### Convert a VCF file to BCF format

**Args:** `input.vcf output.bcf`

**Explanation:** This converts a text VCF file to binary BCF format, which is smaller and faster to process but requires compatible tools for reading.

### Convert a CSV file to TSV format

**Args:** `input.csv output.tsv --informat csv --outformat tsv`

**Explanation:** This explicitly converts a comma-separated CSV file to tab-separated TSV format, specifying both input and output formats when extensions are ambiguous.

### Convert multiple FASTQ files to FASTA using batch mode

**Args:** `*.fastq output.fasta --batch`

**Explanation:** This uses glob pattern matching with batch mode to convert all FASTQ files to FASTA format, processing them in sequence and concatenating output.

### Convert a BED annotation file to GTF format

**Args:** `input.bed output.gtf`

**Explanation:** This converts a BED format annotation file to GTF format, translating genomic coordinates and features while preserving gene and transcript annotations where possible.

### Compress a FASTQ file using gzip

**Args:** `input.fastq output.fastq.gz`

**Explanation:** This compresses a FASTQ file directly to gzip format, automatically detecting the compression based on the .gz extension.