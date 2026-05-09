---
name: asciigenome
category: sequence visualization
description: A terminal-based genome browser that visualizes DNA, RNA, and protein sequences in ASCII art format. Provides interactive navigation, sequence searching, and alignment viewing capabilities within a command-line interface.
tags: [genome-browser, terminal Visualization, ascii, sequence-analysis, interactive]
author: AI-generated
source_url: https://github.com/asciigenome/asciigenome
---

## Concepts

- **Primary Input Format**: asciigenome accepts FASTA and FASTQ files containing DNA, RNA, or protein sequences. Multi-sequence files (multi-FASTA) are supported, with each record treated as a separate sequence or combined based on the `--combine` flag.
- **Sequence Data Model**: Sequences are stored internally as character arrays with positional indices. The tool maintains a cursor position for navigation and supports both linear (single-contig) and circular (plasmid) sequence representations depending on the `--circular` flag.
- **Output Modes**: Three primary visualization modes exist: (1) `--view` displays raw sequence with index markers, (2) `--feature` shows annotated features (genes, motifs) in ASCII art blocks, and (3) `--align` renders pairwise or multiple sequence alignments in text format.
- **Interactive Navigation**: Within interactive mode, keyboard commands (j/k for up/down, h/l for left/right, / for search, n for next match) enable traversal. The viewport can be resized with `--width` and `--height` parameters controlling characters displayed per line and number of visible lines respectively.

## Pitfalls

- **Specifying Incorrect Index Range**: Using `--range` with start greater than end (e.g., `--range 100:50`) silently produces empty output rather than an error, leading to missed visualization of the intended region.
- **Memory Usage with Large Files**: Loading entire multi-GB FASTA files into memory without the `--stream` flag causes excessive memory consumption and potential system slowdown; asciigenome defaults to streaming for files exceeding 100MB.
- **Feature Parsing Dependencies**: The `--feature` visualization mode requires a separate annotation file in GFF3 or BED format. Running without providing `--annotation` results in no features displayed without warning, only a sequence-only view.
- **Encoding Mismatch**: Sequence files with non-ASCII characters (UTF-8 BOM, Windows line endings) cause parsing errors. Using `--input-encoding utf-8` resolves most encoding-related failures but must be explicitly specified.
- **Circular Mode Misunderstanding**: Enabling `--circular` on linear sequences (chromosomes) still wraps at the specified length but may produce visually confusing outputs; this mode is intended specifically for plasmids and circular contigs.

## Examples

### Display a simple DNA sequence from FASTA file
**Args:** `--view --file sequences.fasta --seq-id seq1`
**Explanation:** Opens the specified sequence in view mode, displaying the DNA sequence with positional indices. The cursor starts at position 0 (the beginning of seq1).

### View a specific region of a sequence
**Args:** `--view --file genome.fasta --seq-id chr1 --range 1000:2000 --width 80`
**Explanation:** Displays positions 1000-2000 of chr1 with 80 characters per line, enabling focused examination of a particular genomic region without loading the entire sequence.

### Visualize annotated features on a sequence
**Args:** `--feature --file genome.fasta --seq-id contig_A --annotation features.gff3 --show-label`
**Explanation:** Renders the sequence with annotated features from the GFF3 file overlaid in ASCII block characters, showing feature labels for easy identification of genes and regulatory elements.

### Search for a motif sequence
**Args:** `--search --file reads.fasta --seq-id read_42 --pattern TATAA --highlight`
**Explanation:** Locates all instances of the TATAA motif and highlights them with inverse video characters in the output, aiding in promoter or TATA-box identification.

### Display circular plasmid with feature annotation
**Args:** `--circular --feature --file plasmid.fasta --seq-id pBR322 --annotation plasmid_annot.gff --range 0:5000`
**Explanation:** Visualizes the circular plasmid in feature mode, wrapping the display from end to beginning seamlessly, showing annotated features in ASCII format.

### Filter and view only reads containing N bases
**Args:** `--view --file reads.fasta --filter N --min-quality 20`
**Explanation:** Filters input reads to display only those containing ambiguous bases (N) and minimum quality score of 20, useful for quality control and identifying problematic sequences.

### Export alignment visualization to file
**Args:** `--align --file aln.fasta --output alignment_view.txt --format text`
**Explanation:** Generates a text-based alignment visualization of the aligned sequences and writes to the specified output file for downstream use or sharing.

### Stream large genome file without loading fully
**Args:** `--stream --view --file large_genome.fa --seq-id chr22 --range 0:10000`
**Explanation:** Uses streaming mode to access only the needed region without loading the entire multi-GB file into memory, essential for working on systems with limited RAM.

### Interactive browsing session
**Args:** `--interactive --file genome.fasta --seq-id scaffold1 --width 120 --height 30`
**Explanation:** Launches an interactive terminal session with specified viewport dimensions, enabling keyboard-driven navigation through the sequence with real-time rendering.

### Combine multi-FASTA records for viewing
**Args:** `--view --file combined.fa --combine --width 100`
**Explanation:** Concatenates all sequences in the multi-FASTA file into a single continuous view with 100 characters per line, useful for quick scanning of multiple short sequences.