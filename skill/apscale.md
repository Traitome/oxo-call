---
name: apscale
category: Sequence Analysis / Bioinformatics
description: A command-line tool for plasmid and sequence analysis, part of the ApE (A Plasmid Editor) ecosystem. Performs restriction site mapping, open reading frame (ORF) detection, translation, and sequence validation. Useful for high-throughput analysis of DNA constructs and cloning workflows.
tags: [plasmid, molecular-biology, restriction-enzyme, orf, translation, dna-analysis, cloning, bioinformatics]
author: AI-generated
source_url: https://github.com/bowser-lab/apscale
---

## Concepts

- **Input Format Handling**: apscale accepts multiple sequence file formats including FASTA (.fasta, .fa), GenBank (.gb, .gbk), and ACE format. Sequence files must be valid; malformed or ambiguous nucleotide characters (anything other than A, T, G, C, U, N) will cause parsing failures.
- **Restriction Analysis**: The tool identifies restriction enzyme cut sites within input sequences using a built-in database of common enzymes (e.g., EcoRI, BamHI, HindIII). Results include enzyme name, recognition site sequence, cut position(s), and fragment sizes produced by complete digestion.
- **ORF Detection and Translation**: apscale scans all six reading frames (three forward, three reverse) for open reading frames meeting a minimum length threshold (default ≥30 codons). Detected ORFs can be translated into amino acid sequences with configurable codon tables (default: standard genetic code).
- **Output Modes**: Results can be emitted in text format (human-readable reports), JSON (machine-parseable), or summary format. Multiple analyses can be combined in a single run by specifying multiple analysis modules.

## Pitfalls

- **Missing Input File**: Forgetting to specify an input file produces no error message but exits silently; always verify the file path exists before running. This leads to wasted analysis runs and missed data.
- **Incorrect Recognition Site Sequence**: Providing the wrong restriction enzyme name (e.g., "ECORI" instead of "EcoRI") silently skips that enzyme with no warning, resulting in incomplete restriction maps.
- **Unrecognized Nucleotide Characters**: Input sequences containing IUPAC ambiguous codes beyond standard A/T/G/C/N are treated as unknown, causing imprecise restriction mapping and fragment size calculations.
- **Frame-Shift in Manual Translation**: Manually specifying a reading frame that starts at base 2 or 3 causes the translated protein to be frame-shifted, producing incorrect amino acid sequences that do not match in-silico experiments.

## Examples

### Restriction site analysis for a plasmid sequence