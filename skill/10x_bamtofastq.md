---
name: 10x_bamtofastq
category: Bio informatics / Sequencing
description: Converts a 10x Genomics BAM file back to FASTQ format, extracting reads and preserving cell barcodes and UMIs. This tool reverses the basecalling and alignment process, reproducing the original FASTQ files from which the BAM was generated.
tags:
- 10x-genomics
- bam-to-fastq
- single-cell
- scrna-seq
- scatac-seq
- converting
- sequencing-files
author: AI-generated
source_url: https://support.10xgenomics.com/single-cell/software/pipelines/latest/output/fastq-files
---

## Concepts

- The tool reads a 10x-aligned BAM file and demultiplexes it into separate FASTQ files for each read type (R1, R2, I1, I2), reconstructing the original input FASTQ files from the aligned reads.
- Cell barcodes and Unique Molecular Identifiers (UMIs) are encoded in the BAM CB and UB tags; when converting back to FASTQ, these are written back to the read headers in FASTQ format.
- The output FASTQs use a naming convention that encodes the 10x barcode and UMI information, enabling compatibility with other 10x downstream pipelines like Cell Ranger.

## Pitfalls

- Using a non-10x BAM file (one not aligned using the 10x pipeline) will fail or produce meaningless output, as the required CB/UB tags are missing from standard BAM files.
- If the reference genome specified does not match the reference used for alignment, the tool may error out or produce corrupted FASTQ files with mismatched headers.
- For large BAM files, failing to allocate sufficient disk space for output will cause the process to crash midway; the tool writes FASTQ files directly and requires free space equal to the original FASTQ size.

## Examples

### Converting a 10x BAM file to FASTQ format in a specific output directory
**Args:** `--bam example.bam --output-dir ./fastq_output --sample-name sample1`
**Explanation:** This converts example.bam into FASTQ files, placing them in the ./fastq_output directory with the prefix "sample1" in the output filenames.

### Converting a BAM file with a specific reference genome
**Args:** `--bam sample.bam --output-dir ./fastqs --reference /path/to/refdata-cellranger-GRCh38-3.0.0`
**Explanation:** Specifying the reference path ensures the tool uses the correct reference metadata for proper FASTQ reconstruction.

### Processing a BAM file while limiting the number of read threads
**Args:** `--bam input.bam --output-dir ./out --threads 8 --logging-level INFO`
**Explanation:** Using 8 threads speeds up processing for large BAM files while setting INFO logging provides detailed progress feedback.

### Converting a BAM and forcing overwrite of existing output files
**Args:** `--bam run.bam --output-dir ./fastqs --force`
**Explanation:** The --force flag allows overwriting any existing FASTQ files in the output directory without prompting.

### Processing a BAM file and generating output for a specific sample prefix
**Args:** `--bam cellranger_output/possorted_bam.bam --output-dir ./converted_fastqs --sample-prefix P12345`
**Explanation:** This extracts FASTQs from a Cell Ranger output BAM, using the specified sample prefix in the output FASTQ headers for tracking.