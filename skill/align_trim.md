---
name: align_trim
category: Sequence Alignment Processing
description: A bioinformatics tool for trimming aligned sequence data, removing low-quality or poorly aligned regions from nucleotide or protein alignments in FASTA/GenBank formats. Supports manual coordinate-based trimming, automated quality threshold trimming, and end-trimming operations.
tags:
  - alignment
  - sequence processing
  - trimming
  - bioinformatics
  - fasta
author: AI-generated
source_url: https://github.com/bioinformatics-tools/align_trim
---

## Concepts

- **Input Alignment Formats**: The tool accepts aligned FASTA (FA), GenBank (GB), or NCBI-style alignment files where sequences are pre-aligned to a reference. Unaligned sequences will be rejected with an error unless `--allow-unaligned` is specified.
- **Output Writing**: Trimmed alignments are written to stdout by default or to the file specified by `-o/--output`. The output format mirrors the input format; switching formats requires explicit format flags (e.g., `--format fasta`).
- **Coordinate System**: Positions are 1-based inclusive for both start and end coordinates, matching standard bioinformatics conventions. The tool accepts zero-based coordinates only if the `--zero-based` flag is provided—a common source of off-by-one errors.
- **Trimming Modes**: The tool supports three primary modes: coordinate-based (`--from X --to Y`), quality-based (`--min-quality N`), and automated end-trimming (`--trim-ends`). These modes can be combined; coordinate filtering is applied first, then quality filtering, then end-trimming.

## Pitfalls

- **Specifying wrong coordinate direction**: Users may inadvertently specify a start coordinate greater than the end coordinate (e.g., `--from 500 --to 100`). The tool treats this as a range swap error and exits with code 1 unless `--auto-fix` is enabled, producing empty or corrupted output.
- **Using integer quality scores with float thresholds**: When working with PHRED-style quality scores, passing a float (e.g., `--min-quality 20.5`) causes silent truncation to integer (20), potentially allowing lower-quality bases than intended. Always use integer values for quality thresholds.
- **Forgetting `--ref-name` when trimming multi-reference alignments**: In files containing multiple reference sequences, omitting `--ref-name` causes the tool to apply trimming coordinates to the first sequence only, leading to unintended truncation of other sequences in the alignment.
- **Output file overwrites without warning**: By default, the tool overwrites existing output files silently. Running `align_trim -o existing.fasta` will permanently destroy the original file unless `--backup` is used to preserve a copy.

## Examples

### Trim alignment to positions 50-200
**Args:** `-i alignment.fasta --from 50 --to 200`
**Explanation:** Extracts the subsequence spanning columns 50 through 200 inclusive from every sequence in the alignment file, preserving the original FASTA format.

### Trim leading and trailing gaps automatically
**Args:** `--input aligned.fasta --trim-gaps --output trimmed.fasta`
**Explanation:** Removes columns from the ends of the alignment that consist entirely of gap characters (- or .), producing a cleaner alignment without disrupting sequence positions.

### Process protein alignment with minimum conservation filter
**Args:** `--input protein_align.fa -o conserved.fa --min-conservation 0.75`
**Explanation:** Removes alignment columns where the fraction of non-gap residues falls below 75%, useful for preparing phylogenetically informative alignments.

### Convert FASTA alignment toGenBank format
**Args:** `--input alignment.fa --format gb --output align.gb`
**Explanation:** Converts the alignment from FASTA-like format to GenBank flatfile format while preserving all sequence data and annotations.

### Trim specific reference in multi-ref alignment file
**Args:** `-i multi_ref.aln --ref-name NM_001 --from 10 --to 500`
**Explanation:** Applies trimming only to the sequence or feature named "NM_001", leaving all other sequences unchanged within the same file.

### Preview trimming results without writing output
**Args:** `-i test.aln --from 1 --to 100 --dry-run`
**Explanation:** Runs the trimming operation in simulation mode, outputting the result to stdout for inspection but not modifying any files, useful for testing parameters before batch processing.

### Trim alignment and generate statistics report
**Args:** `-i alignment.fasta --from 25 --to 400 --stats stats.json`
**Explanation:** Performs the coordinate trim and additionally writes a JSON file containing column-wise conservation scores, base composition, and gap frequency for downstream analysis.