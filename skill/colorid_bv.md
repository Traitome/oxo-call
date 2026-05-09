---
name: colorid_bv
category: sequence-visualization
description: Visualizes bacterial sequences with color-coded identifiers, rendering sequence alignments or regions with ANSI escape codes for terminal display or export to styled output formats.
tags: [visualization, sequence-display, bacterial-genomics, ansi-colors, alignment-viewer]
author: AI-generated
source_url: https://github.com/Genomia/ColorID
---

## Concepts

- **Input Format**: Reads FASTA, FASTQ, or multi-sequence alignment (MSA) files in standard formats. Sequences may be provided via stdin or file paths; the tool expects plain-text sequence data with standard nucleotide or amino acid alphabets.
- **Color-Coding Model**: Assigns unique ANSI color codes to sequence identifiers or positional regions, enabling rapid visual differentiation of sequences in dense alignments. Colors are deterministic per identifier and remain consistent across rendering sessions.
- **Output Rendering**: Produces terminal-ready output with ANSI escape codes when stdout is a TTY, or plain text with optional markup (HTML, RTF) when redirected. Supports width wrapping to fit terminal columns for readability.
- **Companion Binary `colorid_bv-idx`**: Generates an index file from reference sequences before visualization; the visualization tool references this precomputed index for fast rendering of large sequence sets.

## Pitfalls

- **Missing Index File**: Running `colorid_bv` on large sequence sets without a prebuilt index causes slow initialization or memory exhaustion. Always run `colorid_bv-idx` on reference files first to avoid performance degradation on datasets with hundreds of sequences.
- **Mismatched Sequence Width**: Specifying a column width (`--width`) smaller than the longest sequence identifier plus padding causes truncated labels in the output, leading to confusion when identifying sequences in dense alignments.
- **Incorrect File Encoding**: Feeding files with non-ASCII characters (e.g., Unicode whitespace or Windows line endings) results in malformed alignment display or silent truncation of sequence data at the first invalid character.
- **Redirected Output Without Format Flag**: When stdout is not a terminal, ANSI codes are stripped by default, producing uncolored output. Users expecting colored output in logs or files must specify an explicit output format (e.g., `--format html`) to retain color markup.

## Examples

### Render a multi-sequence alignment with default color coding
**Args:** input/ref_sequences.fasta --msa
**Explanation:** Displays all sequences from a FASTA file as a colorized alignment, applying distinct ANSI colors to each sequence identifier based on the prebuilt index.

### View an alignment with custom terminal width
**Args:** alignment.aln --msa --width 120
**Explanation:** Wraps the alignment display at 120 columns per line, improving readability on wide monitors or when copying output to wide document formats.

### Export colored alignment to HTML for documentation
**Args:** sequences.gb --format html --output colored_alignment.html
**Explanation:** Renders the sequence data as an HTML file with embedded CSS classes for color styling, suitable for inclusion in reports or web pages without terminal dependencies.

### Process sequences from stdin with piped input
**Args:** - --msa