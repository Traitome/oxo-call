---
name: any2fasta
category: Sequence Format Conversion
description: Converts various biological sequence file formats (GenBank, EMBL, GCG, PIR, FASTA, etc.) to FASTA format. Supports both nucleotide and protein sequences.
tags: [sequence-conversion, fasta, format-converter, bioinformatics, nucleotide, protein]
author: AI-generated
source_url: https://emboss.sourceforge.net/release/readme.EMBOSS-6.6.0/packages/EMBOSS-6.6.0/html/index.html
---

## Concepts

- **Multi-format input support**: any2fasta accepts numerous sequence formats including GenBank, EMBL, GCG, MSF, PIR, and FASTA. If the input format is ambiguous, use the `-sfmt` flag to explicitly specify the source format.
- **Molecule type detection**: By default, the tool auto-detects whether sequences are nucleotide or protein. Use `-protein` when processing protein sequences to ensure correct translation table handling.
- **FASTA output standardization**: The tool outputs sequences in standard FASTA format with '>' for the header line containing the sequence identifier, followed by the sequence data wrapped at 60 characters per line by default.
- **Companion binary**: any2fasta-build is a companion utility that indexes FASTA files into a BLAST database for faster sequence alignments.

## Pitfalls

- **Auto-detection failure on ambiguous files**: When input files contain format-specific features that don't clearly distinguish the type, auto-detection may fail and produce garbled output or errors.
- **Overwriting output without confirmation**: The `-outseq` flag will overwrite existing files without prompting, potentially losing previous data.
- **Wrong molecule type causing incorrect translation**: Specifying `-nucleotide` for protein sequences or vice versa can silently corrupt the data when using downstream tools that rely on the correct reading frame.
- **Memory exhaustion with extremely large files**: For multi-gigabyte sequence collections, insufficient RAM can cause the tool to fail mid-conversion, leaving partial output files.

## Examples

### Convert a GenBank file to FASTA format
**Args:** `-sequence input.gbk -outseq output.fasta -sfmt gcg`
**Explanation:** This explicitly specifies the input GenBank format and writes the converted sequences to a new FASTA file.

### Convert an EMBL nucleotide file to FASTA with uppercase output
**Args:** `-sequence input.embl -outseq nucleotide.fasta -sfmt embl -uppercase`
**Explanation:** This converts EMBL format while converting all sequence characters to uppercase for compatibility with case-sensitive alignment tools.

### Convert protein sequences from PIR format to FASTA
**Args:** `-sequence proteins.pir -outseq proteins.fasta -sfmt pir -protein`
**Explanation:** Explicitly specifying `-protein` ensures correct handling of amino acid sequences and prevents nucleotide-related parsing errors.

### Auto-detect format and convert multi-sequence file
**Args:** `-sequence multiinput.seq -outseq multioutput.fasta`
**Explanation:** When the input format has recognizable magic bytes or internal markers, the tool auto-detects the format without manual specification.

### Use companion binary to build a searchable database index
**Args:** `sequences.fasta -d blast -t "Reference sequences"`
**Explanation:** any2fasta-build indexes the FASTA file into a BLAST database using the specified database type and descriptive title for downstream alignments.