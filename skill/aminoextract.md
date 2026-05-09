---
name: aminoextract
category: Bioinformatics / Protein Sequence Analysis
description: A command-line tool for extracting amino acid sequences from protein databases, genome annotations, or nucleotide sequences with translation. Supports FASTA, GenBank, and GTF input formats, and can filter sequences based on length, taxonomy, or feature annotations.
tags:
  - amino acids
  - protein sequences
  - sequence extraction
  - fasta
  - bioinformatics
  - genomics
author: AI-generated
source_url: https://github.com/bioinformatics-tools/aminoextract
---

## Concepts

- **Input Formats**: aminoextract accepts nucleotide sequences (DNA/RNA) that are automatically translated into amino acid sequences using the specified genetic code, as well as direct protein sequences in FASTA format or feature annotations in GFF3/GTF format.
- **Sequence Filtering**: Sequences can be filtered by minimum/maximum length (`--min-length`, `--max-length`), by taxonomic ID (`--taxid`), or by annotation attributes such as gene name or protein product description using `--include` and `--exclude` patterns.
- **Output Formats**: The tool outputs results in FASTA format by default, with options for sequential plain text (`--format text`) or tab-delimited tables with sequence metadata (`--format table`), enabling easy downstream analysis in R or Python.
- **Genetic Code Selection**: When translating nucleotide input, the tool supports standard genetic codes (NCBI translation table 1 for the standard code, table 4 for the Mold/Protozoan code, etc.) via the `--genetic-code` parameter; default is the standard bacterial code.
- **Batch Processing**: Multiple input files can be processed at once using wildcard expansion or by specifying a file list with `--input-list`, with sequences aggregated into a single output unless `--split` is used to create separate output files.

## Pitfalls

- **Genetic Code Mismatch**: Specifying the wrong genetic code when translating nucleotide sequences results in incorrect amino acid sequences; for example, using the standard code (1) for mitochondrial genomes instead of the vertebrate mitochondrial code (2) produces frameshift-like errors in annotation.
- **Off-by-One Frame Errors**: Using `--frame +1` or `--frame +2` inappropriately (e.g., specifying the wrong reading frame) yields sequences that do not correspond to the actual protein coding regions, leading to false or truncated protein predictions.
- **Memory Overflow with Large Inputs**: Processing very large input files (e.g., whole-genome annotation files with millions of features) without the `--chunk-size` parameter can exhaust system memory, causing the tool to crash or terminate unexpectedly.
- **Ambiguous Taxonomic Filtering**: Using partial taxon names with `--taxid` that match multiple taxa (e.g., "Escherichia" matching both E. coli and E. fergusonii) can lead to inclusion or exclusion of unintended sequences, compromising analysis precision.
- **Output File Overwriting**: By default, aminoextract overwrites existing output files without confirmation; failing to specify a unique output name or using `--append` when appropriate results in unintended data loss.

## Examples

### Extract all protein sequences from a FASTA file
**Args:** `--input proteins.fasta --output extracted_proteins.fasta`
**Explanation:** Reads the input FASTA file containing amino acid sequences and writes all sequences to the specified output file without any filtering.

### Translate nucleotide sequences and output amino acids
**Args:** `--input genome_cds.fa --translate --genetic-code 1 --output translated_proteins.fasta`
**Explanation:** Reads nucleotide CDS sequences, translates them using the standard genetic code (NCBI table 1), and outputs the resulting protein sequences in FASTA format.

### Filter sequences by minimum length
**Args:** `--input all_proteins.fasta --min-length 50 --output long_proteins.fasta`
**Explanation:** Extracts only amino acid sequences that are at least 50 residues long, excluding shorter fragments that may represent partial proteins or annotation artifacts.

### Extract sequences matching a specific gene name pattern
**Args:** `--input annotated_proteins.gff --include "ATP synthase" --output atp_sequences.fasta`
**Explanation:** Uses the annotation file to find features with "ATP synthase" in their product description and extracts the corresponding amino acid sequences.

### Extract sequences from a specific taxonomic ID
**Args:** `--input mixed_species.fasta --taxid 9606 --output human_proteins.fasta`
**Explanation:** Filters the input sequences to only include those belonging to Homo sapiens (taxonomic ID 9606), useful for species-specific analyses.

### Output sequences in tab-delimited table format
**Args:** `--input proteins.fasta --format table --output metadata.tsv`
**Explanation:** Exports sequences along with their metadata (header, length, composition) in a tab-separated format ideal for import into spreadsheet applications or R data frames.

### Process multiple input files in batch
**Args:** --input "species_*.fasta" --output all_combined.fasta --append
**Explanation:** Uses shell-style wildcard to process all FASTA files matching the pattern and appends the results to a single combined output file.

### Use alternative reading frame for translation
**Args:** `--input genome.fa --translate --frame +2 --genetic-code 1 --output frame2_proteins.fasta`
**Explanation:** Translates the input nucleotide sequences using the +2 reading frame instead of the default +1, useful for analyzing alternative ORFs or verifying coding regions.