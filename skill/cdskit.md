---
name: cdskit
category: Genomics / Sequence Manipulation
description: A bioinformatics toolkit for extracting, converting, and manipulating Coding Sequences (CDS) from genomic annotations. Provides utilities for extracting CDS regions, converting between common biological formats, filtering sequences by length or annotation criteria, and generating sequence statistics.
tags: [cds, coding-sequence, genomics, sequence-extraction, gff, bed, fasta, bioinformatics]
author: AI-generated
source_url: https://github.com/...
---

## Concepts

- **CDS Extraction Workflow**: cdskit reads genome annotations (GFF3, BED) paired with genomic sequences (FASTA) to extract Coding Sequence regions based on feature coordinates, outputting sequences in user-specified formats.
- **Multi-format I/O**: The tool supports input formats (GFF3, BED, GTF) and output formats (FASTA, BED, CSV), enabling integration with downstream pipelines like multiple sequence alignment or phylogenetics.
- **Strand-awareness**: Extracted CDS sequences retain strand orientation; use the `--strand` flag to control whether to return the original orientation or the reverse-complemented transcript.
- **Annotation Filtering**: Can filter by gene ID, transcript ID, biotype, or sequence length thresholds, allowing selective extraction of specific CDS subsets (e.g., protein-coding only,排除 pseudogenes).
- **Companion Binaries**: The cdskit suite includes `cdskit-build` for creating indexed databases, `cdskit-stats` for generating summary statistics, and `cdskit-filter` for batch filtering operations.

## Pitfalls

- **Mismatched Reference Genomes**: Using a FASTA file that does not match the coordinate system in the annotation file will produce sequences with wrong bases or Ns instead of stop codons. Always verify the genome assembly version matches between files.
- **Incorrect Feature Types**: In GFF3 files, CDS features are often nested within exon and transcript parent features. If specifying `--feature-type gene` instead of `--feature-type CDS`, you will extract genomic DNA rather than the translated sequence.
- **Duplicate Entry Oversight**: Duplicate gene IDs in the annotation file without disambiguation produce only the first match. Use `--filter-duplicates first` or provide a transcript-to-gene mapping to avoid silent data loss.
- **Frame Shifts from Incomplete CDS**: Partial CDS at sequence ends (incomplete annotations) produce frameshifted translations. The tool flags these but may not exclude them by default; check for `--skip-incomplete` in your workflow.
- **Memory Usage on Large Genomes**: Extracting CDS from chromosome-scale annotations without streaming (`--stream`) loads entire datasets into memory, causing performance issues on systems with limited RAM.

## Examples

### Extract all CDS sequences from a genome annotation
**Args:** extract --gff annotations.gff3 --fasta genome.fa --output cds_sequences.fasta
**Explanation:** This extracts all CDS features from the GFF3 annotation using genomic coordinates, outputting protein-coding sequences in FASTA format for downstream analysis.

### Extract CDS in reverse-complemented orientation
**Args:** extract --gff annotations.gff3 --fasta genome.fa --strand reverse --output cds_rev.fasta
**Explanation:** Returns the reverse complement of each CDS sequence, matching the orientation found in transcript annotations for 5' to 3' alignment workflows.

### Filter and extract only long CDS (>300bp)
**Args:** filter --gff annotations.gff3 --min-length 300 | extract --fasta genome.fa --stdin --output long_cds.fasta
**Explanation:** First filters annotation for CDS features meeting length criteria, then pipes the filtered features to extract corresponding sequences.

### Generate statistics summary for extracted CDS
**Args:** stats --fasta cds_sequences.fasta --output cds_summary.tsv
**Explanation:** Generates summary statistics including sequence count, total length, GC content, N content, and nucleotide composition for quality control.

### Build indexed database for fast repeated queries
**Args:** build --gff annotations.gff3 --fasta genome.fa --db cdskit.db --threads 8
**Explanation:** Creates an indexed database combining annotation and sequence data, enabling sub-second retrieval for subsequent queries with `--db` flag.

### Extract specific transcripts by ID list
**Args:** extract --gff annotations.gff3 --fasta genome.fa --ids transcript_ids.txt --output selected_cds.fasta
**Explanation:** Reads a text file containing transcript IDs (one per line) and extracts only matching CDS sequences, useful for targeted gene sets.

### Convert GFF3 annotation to BED format
**Args:** convert --input annotations.gff3 --format bed --output annotations.bed
**Explanation:** Converts CDS feature coordinates from GFF3 to BED format, enabling compatibility with tools like UCSC Genome Browser orBEDTools.