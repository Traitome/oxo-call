---
name: bpp-core
category: bioinformatics/sequence-analysis
description: Core library providing fundamental bioinformatics data types, alphabet handling, and sequence utilities for the Bio++ C++ library suite. Supports DNA, RNA, and protein sequence operations with multiple file format I/O.
tags: [bio++, sequence, alphabet, bioinformatics, c++, sequence-analysis]
author: AI-generated
source_url: https://github.com/BioPP/bpp-core
---

## Concepts

- **Alphabet abstraction**: Bio++ treats biological alphabets (DNA, RNA, Protein) as first-class objects that enforce valid character sets. The alphabet determines which characters are legal in sequences and how sequence content is interpreted.
- **Sequence and Container hierarchy**: Sequences inherit from VectorSite objects and are organized into SequenceContainers and SiteContainers, enabling efficient bulk operations on aligned or unaligned sequence sets.
- **File format I/O**: bpp-core uses pluggable format readers/writers identified by format name strings (fasta, mase, clustal, stockholm). Format auto-detection occurs when unspecified, but explicit declaration prevents ambiguity errors.
- **Alphabet compatibility**: Sequences are bound to specific alphabets at creation; attempting operations between sequences of incompatible alphabets (e.g., computing distance between a DNA sequence and a Protein sequence) throws AlphabetException.

## Pitfalls

- **Omitting the alphabet specification**: Without `--alphabet` (or `-a`), the default alphabet is applied silently. Using DNA defaults on a protein sequence file causes character rejection and truncated output.
- **Assuming format auto-detection is always correct**: Binary format files or non-standard character distributions may be misidentified during auto-detection, leading to silently corrupted or empty sequence output.
- **Mismatched input/output formats**: Converting between formats that have incompatible alphabet requirements (e.g., a codon alphabet output to plain Fasta) drops or transforms characters in ways that are not reversible.
- **File path with special characters**: Spaces or unquoted shell metacharacters in input/output file paths cause I/O failures that are reported as generic "cannot open file" errors without context.
- **Ignoring alphabet case sensitivity**: By default, Bio++ alphabets are case-sensitive. Passing lowercase sequences to an uppercase-only alphabet parser results in all characters being treated as gaps or unknowns.

## Examples

### Display available sequence alphabets
**Args:** `alphalist`
**Explanation:** Lists all registered alphabet types (DNA, RNA, Protein, Codon, etc.) that can be used with other bpp-core utilities, helping identify correct alphabet names for subsequent operations.

### Convert a FASTA DNA file to Mase format
**Args:** `--inf=sequences.fasta --out=sequences.mase --format=fasta --alphabet=DNA`
**Explanation:** Explicitly specifies the input format and alphabet to ensure correct character parsing before writing the output in Mase format, avoiding auto-detection ambiguity.

### Extract sequences with specific characters from a protein file
**Args:** `--sequences=protein.fasta --alphabet=Protein --motif=WK --output=filtered.fasta --format=fasta`
**Explanation:** Filters the input sequence file to retain only sites containing the specified motif pattern, demonstrating motif-based sequence extraction capabilities.

### Display sequence statistics from a multi-fasta input
**Args:** `--sequences=reads.fasta --alphabet=DNA --stat --output=stats.txt`
**Explanation:** Computes and outputs per-sequence statistics (length, composition, unknown character count) to a file, useful for quality assessment before downstream analysis.

### Validate sequence alphabet compatibility
**Args:** `--sequences=test.fasta --alphabet=DNA --check --verbosity=2`
**Explanation:** Runs alphabet compliance checking with verbose output, reporting every invalid character and its position, enabling systematic identification of malformed input sequences.

### Convert between genetic code interpretations
**Args:** `--sequences=cds.fasta --alphabet=Codon --gencode=Standard --output=aa.fasta --format=fasta`
**Explanation:** Demonstrates codon alphabet handling where sequences are interpreted as codons and translated using the specified genetic code before output, showing alphabet-specific translation workflows.

### Batch process multiple sequence containers
**Args:** `--indir=./sequences --outdir=./output --format=fasta --alphabet=DNA --recursive`
**Explanation:** Recursively processes all sequence files in the input directory, applying consistent alphabet and format parameters, and writing transformed results to the corresponding output directory structure.