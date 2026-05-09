---
name: bialign
category: Sequence Alignment
description: A fast bit-parallel pairwise sequence alignment tool for DNA, RNA, and protein sequences. Supports global, local, and overlap alignment modes with configurable scoring matrices and gap penalty models.
tags:
  - sequence-alignment
  - pairwise-alignment
  - bit-parallel
  - bioinformatics
  - genomics
  - pairwise
  - fast
author: AI-generated
source_url: https://github.com/bioinformatics-tools/bialign
---

## Concepts

- **Bit-Parallel Architecture**: bialign encodes the DP matrix rows into machine words, enabling SIMD-style updates that dramatically reduce instruction count compared to classic Needleman-Wunsch or Smith-Waterman. This means very long alignments (e.g., megabase-scale) complete orders of magnitude faster than naïve O(NM) implementations.
- **Three Alignment Modes**: Global (`--mode global`) forces an alignment spanning the full extent of both sequences, useful for evolutionary distance estimation. Local (`--mode local`, the default) finds the highest-scoring substring pair, ideal for domain detection. Overlap (`--mode overlap`) aligns only the overlapping suffix/prefix when sequences may partially overlap.
- **Scoring Model Flexibility**: bialign accepts standard substitution matrices (BLOSUM45, BLOSUM62, PAM120, PAM250) via `--matrix`, a match reward and mismatch penalty via `--match` / `--mismatch`, and affine gap open/extend penalties via `--gap-open` / `--gap-extend`. The default model is `--match 1 --mismatch -2 --gap-open -10 --gap-extend -1`, appropriate for moderate-divergence protein sequences.
- **I/O Format Conventions**: Input sequences must be in FASTA (`.fasta`, `.fa`, `.faa`) or FASTQ (`.fastq`) format. bialign auto-detects the format by inspecting the first character (`>` for FASTA, `@` for FASTQ). Output defaults to aligned FASTA (`--format fasta`), but `--format sam` produces a SAM-style record pair useful for chaining into downstream variant-calling pipelines.

## Pitfalls

- **Conflicting Score Parameter Combinations**: Specifying both `--matrix` and individual `--match`/`--mismatch` flags causes bialign to exit with a non-zero status and print `ERROR: --matrix and per-base scores are mutually exclusive`. The tool does not warn or fall back — it simply aborts the run, wasting compute time on a failed job.
- **Local Alignment Returning Zero-Length Output**: When two sequences share no region with a positive alignment score under the current gap penalty model, bialign outputs an empty alignment and returns exit code `1`. Downstream scripts that assume a non-empty SAM output will silently drop these cases, corrupting batch analytics.
- **Integer Overflow on Extreme Scores**: With very long sequences (>100 kbp) and permissive gap penalties (e.g., `--gap-open -1 --gap-extend -0.1`), the DP score can exceed the 64-bit integer range, causing undefined behavior. Always cap sequence length or tighten penalties when working with long genomic contigs.
- **Compressed Output Clobbering Input Files**: The `--out` flag overwrites files without confirmation. Running `bialign seq1.fa seq2.fa --out seq1.fa` irretrievably destroys the original input, as the output stream is opened before the alignment is computed.

## Examples

### Align two protein sequences using BLOSUM62 with default gap penalties
**Args:** `--mode global --matrix BLOSUM62 seq1.faa seq2.faa`
**Explanation:** Specifying the BLOSUM62 matrix alongside global alignment mode produces a full-length alignment suited for phylogenetic distance estimation between the two protein sequences.

### Perform local alignment with affine gap penalties and output SAM
**Args:** `--mode local --gap-open -11 --gap-extend -1 --match 2 --mismatch -1 --format sam --out results.sam query.fa target.fa`
**Explanation:** Using local mode with affine gap penalties finds the highest-scoring ungapped local match region between the sequences and writes the result in SAM format for compatibility with variant-calling tools.

### Align DNA sequences with simple linear gap penalty
**Args:** `--mode global --gap-open -5 query.fasta target.fasta`
**Explanation:** With only `--gap-open` specified (no `--gap-extend`), bialign defaults to a linear gap penalty model where each gap position incurs the same cost, which is appropriate for close DNA homologs with short indel events.

### Align with custom per-base match/mismatch scores and compressed output
**Args:** `--mode overlap --match 3 --mismatch -5 --out aligned.fa.gz input1.fa input2.fa`
**Explanation:** Setting a high match reward and severe mismatch penalty encourages extension of the overlap region without mismatches, and the compressed output file reduces disk usage for large batch jobs.

### Align and suppress header commentary in output
**Args:** `--mode local --no-header query.fa reference.fa`
**Explanation:** The `--no-header` flag strips the verbose comment lines from FASTA output, leaving only the aligned sequence records, which simplifies parsing in pipeline scripts that expect clean multi-line FASTA.