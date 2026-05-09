---
name: biopython.convert
category: sequence-format-conversion
description: Convert bioinformatics sequence files between common formats such as FASTA, GenBank, EMBL, and other supported biological sequence file types.
tags:
  - biopython
  - sequence
  - fasta
  - genbank
  - format-conversion
  - bioinformatics
  - python
author: AI-generated
source_url: https://biopython.org/wiki/SeqIO
---

## Concepts

- BioPython convert uses the SeqIO module internally to read input files in any supported format and write them to an output file in a different format, supporting FASTA, GenBank, EMBL, ABI, GFF3, PDB, and many others.
- The tool infers file formats automatically from file extensions when possible, but explicit format specification may be required when input/output files use non-standard or ambiguous extensions.
- Sequence quality scores are preserved during conversion when both input and output formats support them (e.g., FASTQ to Qual), but quality data is silently dropped when converting to formats that do not store quality information (e.g., FASTA or GenBank).
- BioPython's parse-and-write architecture means the entire file is not loaded into memory at once for large files, enabling conversion of moderately large datasets without excessive RAM usage.

## Pitfalls

- Converting a FASTQ file to FASTA will permanently discard all quality score information, which is required for many downstream tools like variant callers or quality trimmers; users may not realize this data loss until downstream analysis fails.
- If the output file already exists, the tool will silently overwrite it without prompting, potentially causing irreversible data loss on important pre-existing files.
- Specifying an incompatible output format for the sequence type (e.g., outputting protein sequences as FASTQ) will produce malformed or rejected output files that may not report clear errors during conversion.
- Using mismatched input and output format flags when file extensions are non-standard can result in silent failures where BioPython defaults to a generic parser, producing corrupted or truncated output files.

## Examples

### Convert a GenBank file to FASTA format
**Args:** input.gbk -o output.fasta
**Explanation:** Reads the GenBank file and writes all sequences in FASTA format, stripping annotation metadata like gene names and CDS coordinates that only exist in GenBank.

### Convert FASTQ to FASTA while preserving the input quality file
**Args:** sample.fastq -o sample.fasta --qual sample.qual
**Explanation:** Converts the sequence data to FASTA format while separately exporting quality scores to a Qual format file, ensuring no data is lost during the conversion.

### Batch convert multiple FASTA files to GenBank format
**Args:** file1.fasta file2.fasta -o converted.gbk -o file2_converted.gbk
**Explanation:** Reads two input FASTA files and writes them to separate GenBank output files, each retaining appropriate feature annotations where possible.

### Convert an EMBL file specifying the input format explicitly
**Args:** sequences.embl -i embl -o output.fasta
**Explanation:** Explicitly specifies the input format as EMBL to ensure correct parsing when the file extension is non-standard or missing, preventing format inference errors.

### Convert all sequences to a new format and specify output directory
**Args:** input/*.gbk -o output_directory/ -o fasta
**Explanation:** Uses glob pattern to select all GenBank files in a directory and converts them to FASTA format, outputting to a specified directory while inferring output format from the specified extension.