---
name: abnumber
category: Sequence Processing / Antibody Analysis
description: A command-line tool for renumbering antibody amino acid sequences using standardized schemes (Kabat, Chothia, IMGT, Martin, enhanced Chothia). Handles both Heavy (VH) and Light (VL/Kappa/Lambda) chains. Input/output in FASTA, CSV/TSV, or JSON.
tags:
  - antibody
  - renumbering
  - immunology
  - sequence alignment
  - Kabat
  - IMGT
author: AI-generated
source_url: https://github.com/alkc/abnumber
---

## Concepts

- **Numbering schemes**: abnumber maps antibody CDR and framework regions to canonical positions. Kabat, Chothia, IMGT, Martin, and enhanced Chothia each define different insertion locations for CDR positions, so the same raw sequence may yield different residue labels across schemes.
- **Chain type detection**: The tool auto-detects Heavy (VH), Kappa Light (VK), or Lambda Light (VL) chains from the sequence itself or from input sequence headers, but can be overridden with `--chain` or `-t`.
- **Input formats**: FASTA (with sequence ID in the header line, e.g., `>VH|Kappa|IMGT` to set all three properties at once), CSV/TSV (with a `sequence` column), and JSON (dict mapping ID to sequence string). Multi-sequence inputs are processed in a single batch.
- **Output formats**: Results are written in the chosen format (FASTA, CSV/TSV, JSON). When writing to stdout in CSV/TSV mode, the output includes header labels for all renumbered positions and their corresponding residues.
- **Position semantics**: Numbered positions use the scheme's numbering label (e.g., `32` or `32A`, `105`). Gaps in the alignment (e.g., deleted residues not present in the input) are output as `-` in FASTA or empty strings in tabular formats.

## Pitfalls

- **Wrong scheme or chain type**: Supplying `--scheme imgt` when the input uses Kabat numbering conventions produces structurally incorrect position labels. This breaks downstream CDR analysis, grafting, or machine learning features that depend on consistent positions.
- **Invalid characters in input sequences**: Non-standard amino acid codes (e.g., `X`, `B`, `Z`, U, O, J in FASTA) cause silent skipping or errors depending on the input parser. Sequences containing stop codons (`*`) may be rejected or truncated. Always clean input to the 20 standard amino acid codes before piping to abnumber.
- **Auto-detection failure on ambiguous headers**: If a FASTA header is just `>seq1` with no chain prefix and the sequence is short, auto-detection may guess Light chain when it is actually Heavy, or vice versa, silently producing wrong numbering. Always explicitly set `--chain` when the chain type is known.
- **Mismatched input format flag**: Using `--format fasta` with a CSV input file (or vice versa) causes a parse error or empty output. The format flag must match the actual file contents, not the desired output.
- **Insertions handled differently by schemes**: A position labeled `33A` in Kabat may not exist in IMGT, which uses a different insertion labeling system. Mixing scheme labels in downstream analysis without normalization leads to position misalignment in multi-scheme comparisons.

## Examples

### Renumber a single Heavy chain sequence in FASTA format using the Kabat scheme
**Args:** `renumber -s kabat -t heavy -i seq.fasta`
**Explanation:** The `-s kabat` flag selects the Kabat numbering scheme, `-t heavy` declares the chain type, and `-i seq.fasta` provides the input FASTA file containing one or more VH sequences.

### Renumber Light chain sequences from CSV input with explicit scheme specification
**Args:** `renumber -s chothia -t kappa --format csv -i sequences.csv -o numbered.csv`
**Explanation:** Explicitly setting `-s chothia` and `-t kappa` overrides auto-detection. The `--format csv` flag tells the parser to read the tabular input file, and `-o numbered.csv` redirects the renumbered output to a file rather than stdout.

### Batch-renumber multiple sequences with enhanced Chothia scheme and JSON I/O
**Args:** `renumber -s echothia -t light -f json -i input.json -o output.json`
**Explanation:** The `-s echothia` flag selects enhanced Chothia numbering, `-t light` covers both Kappa and Lambda chains, and `-f json` configures JSON input and output for programmatic downstream processing.

### Renumber and print results to stdout in FASTA format with auto chain detection
**Args:** `renumber -s imgt -i mixed.fasta --format fasta`
**Explanation:** When `-t` is omitted and input is FASTA with embedded scheme/chain prefixes in headers (e.g., `>seq1|VH|IMGT`), auto-detection sets both scheme and chain type. Output is written to stdout for piping into subsequent tools.

### Renumber a single sequence piped from stdin using IMGT scheme and Kappa chain
**Args:** `renumber -s imgt -t kappa -i -`
**Explanation:** Passing `-i -` tells abnumber to read from stdin, which allows shell command chaining. `-t kappa` ensures the chain type is fixed even if stdin lacks header metadata, preventing misclassification of the piped sequence.