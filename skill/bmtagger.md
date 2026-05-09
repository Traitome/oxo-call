---
name: bmtagger
category: Sequencing/Demultiplexing
description: NCBI bmtagger is a barcode identification and demultiplexing tool that matches sequencing reads against a reference database of barcode or primer sequences to assign reads to samples. It is part of the NCBI SRA Toolkit and pairs with the bmtagger-build companion binary for constructing barcode reference indexes.
tags:
  - demultiplexing
  - barcode
  - fastq
  - ncbi
  - sratools
  - sequencing
  - illumina
author: AI-generated
source_url: https://github.com/ncbi/sra-tools
---

## Concepts

- **Barcode Reference Database Is Mandatory**: bmtagger requires a pre-built reference index created with the companion `bmtagger-build` binary. Without this index, the tool has no sequences to match against, and every read will be reported as unmatched.

- **I/O Format Is FASTQ In, Tagged FASTQ or SAM Out**: bmtagger accepts standard Sanger-scored FASTQ files (Phred+33 quality encoding). Output can be written as barcode-tagged FASTQ (each read annotated with the matched barcode ID) or optionally as SAM records for downstream alignment pipelines.

- **Mismatch Tolerance Is Configurable Per Run**: The `-m` flag controls how many mismatches are permitted when matching a read against barcodes. Setting this to 0 enforces exact matching only, while allowing 1–2 mismatches is common for barcodes with known sequencing errors, but increases the risk of cross-assignment in similar barcode sequences.

- **Paired-End Read Support Requires Two Input Files**: When processing paired-end Illumina data, pass both read files with `-1` and `-2`. The tool evaluates each end independently against the barcode database and reports both assignments; downstream scripts must resolve conflicts if the two ends receive different barcode assignments.

- **Multi-Mapped Reads Are Flagged Not Dropped**: If a read matches more than one barcode equally well (ties), bmtagger annotates the read as `MULTI` rather than silently assigning it to the first match. This flag is essential for detecting barcode bleed-through or library contamination.

## Pitfalls

- **Forgetting to Run bmtagger-build First Produces Zero Matches**: If you invoke bmtagger without a reference file (or with a file that was never indexed), the tool will process all reads without finding any barcodes, silently producing an output file full of `NONE` assignments and wasting an entire sequencing run.

- **Setting Mismatch Count Too High Causes Cross-Assignment**: Allowing 3 or more mismatches (`-m 3`) can cause a read with multiple sequencing errors to match the wrong barcode if two barcodes in your reference differ by only a few bases, resulting in sample bleed-through in your demultiplexed data.

- **Quality Score Offset Mismatch Leads to Corrupt Output**: Passing an Illumina 1.0/1.5 encoded FASTQ (Phred+64) to bmtagger without the `-I` flag causes the tool to interpret quality scores incorrectly, triggering false barcode matches and corrupting downstream variant calling or expression analysis.

- **Paired-End Files Processed as Single-End Causes Duplicate Assignments**: If you specify the same FASTQ file for both `-1` and `-2` by mistake, bmtagger will treat both read ends as independent but identical inputs, effectively doubling your read count and causing every read to appear twice in the output.

- **Output Directory Must Exist Before Invocation**: If you specify an output path to a directory that does not yet exist, bmtagger will fail at the write stage with a non-descriptive I/O error, and any progress made during the read processing phase is lost with no intermediate checkpoint.

## Examples

### Build a barcode reference index from a FASTA file
**Args:** `-build -o barcode_reference.sq -b 3