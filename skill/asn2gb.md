---
name: asn2gb
category: Format Conversion
description: Converts ASN.1 formatted sequence data from NCBI databases to GenBank flat file format. Essential for transforming structured sequence records into the widely-used GenBank text representation.
tags:
- asn1
- genbank
- format-conversion
- sequence-data
- ncbi
- bioinformatics
- nucleotide
- protein
author: AI-generated
source_url: https://www.ncbi.nlm.nih.gov/
---

## Concepts

- **Input Format (ASN.1)**: asn2gb accepts ASN.1 (Abstract Syntax Notation One) formatted sequence data, which is NCBI's binary/intermediate format for storing structured biological sequence records including sequences, features, annotations, and metadata.

- **Output Format (GenBank Flat File)**: The tool produces GenBank flat file format output, a human-readable text format that encodes nucleotide or protein sequences with their features, annotations, source information, and references in a structured, line-based layout.

- **Sequence Selection**: You can extract specific sequences from a multi-sequence ASN.1 input using the `-seq` flag with an accession number, gi number, or specific sequence identifier to convert only the desired record.

- **Output Redirect**: By default, asn2gb outputs to standard output (stdout); use shell redirection (`>`) or the `-out` flag to write the GenBank output to a specific file for downstream analysis or storage.

- **Format Variants**: The tool supports outputting in different GenBank display styles including the standard flat file format, and can include or exclude specific annotation sections via option flags.

## Pitfalls

- **Invalid Input File**: Providing a file that is not in valid ASN.1 format (e.g., FASTA, EMBL, or already in GenBank format) will cause parsing errors and the tool will fail to produce output, returning an obscure error message.

- **Wrong Accession Specified**: Using an incorrect or non-existent accession number with the `-seq` flag results in an empty output file with no indication of which record was requested, wasting analysis time.

- **Missing Input File**: Running asn2gb without specifying an input file (or using `-` when not reading from stdin) produces no output and may display a minimal error or hang waiting for input.

- **Large File Handling**: Converting very large ASN.1 files (containing thousands of sequences) without specifying output redirection can flood the terminal, making it difficult to capture or scroll through the output.

- **Incompatible Sequence Types**: Attempting to convert protein sequences using nucleotide-specific parsing options may result in malformed GenBank output or silent truncation of feature data.

## Examples

### Convert a simple ASN.1 file to GenBank format
**Args:** `input.asn > output.gb`
**Explanation:** This basic command reads the ASN.1 formatted sequence data from input.asn and writes the converted GenBank flat file format to output.gb using shell redirection.

### Extract a specific sequence by accession number
**Args:** `-seq AC12345 input.asn > output.gb`
**Explanation:** The `-seq` flag selects only the sequence with accession AC12345 from the input file, ignoring all other records and converting just the requested sequence to GenBank format.

### Convert with verbose debugging output
**Args:** `-v input.asn`
**Explanation:** The `-v` flag enables verbose mode, displaying processing progress and diagnostic information to stderr while still outputting the GenBank data to stdout.

### Specify an output filename directly
**Args:** `-out result.gb input.asn`
**Explanation:** The `-out` flag explicitly directs the GenBank output to result.gb instead of stdout, which is useful in scripting pipelines where shell redirection is less convenient.

### Convert using stdin input stream
**Args:**