---
name: bipode-httr
category: Sequence Processing / Pattern Mining
description: A command-line utility for detecting, decomposing, and transforming binary and biallelic sequence patterns across genomic datasets. Operates on FASTA, FASTQ, and tabular formats to extract positional k-mer motifs, evaluate nucleotide diversity, and export results in multiple formats (CSV, BED, JSON).
tags: [sequence-analysis, k-mer, binary-pattern, nucleotide-diversity, genomic-patterns, bioinformatics]
author: AI-generated
source_url: https://github.com/bipode-project/bipode-httr
---

## Concepts

- **Binary pattern encoding**: bipode-httr represents each nucleotide as a 2-bit binary code (A=00, C=01, G=10, T=11) and processes sliding windows of k-mer length to produce a binary motif signature. The `--klen` flag controls window size; valid ranges are 2–16. This encoding enables fast bitwise comparisons between sequences without string parsing overhead.
- **Multi-format I/O**: The tool accepts FASTA (`.fasta`, `.fa`, `.fna`), FASTQ (`.fastq`, `.fq`), and tabular (`.tsv`, `.csv`) input via stdin or explicit `--input` flag. Output formats are controlled by `--outfmt csv|bed|json|summary`; default is CSV with positional, motif, count, and frequency columns.
- **Reference-guided decomposition**: When a reference genome is supplied via `--reference`, bipode-httr computes per-position deviation from the reference allele, generating a difference profile. This is useful for identifying strain-specific SNPs or low-complexity regions where binary patterns diverge from the consensus.
- **Aggregate statistics and thresholding**: The `--min-count` and `--min-freq` flags filter low-abundance motifs from output. Motifs below the threshold are discarded before writing, reducing noise in downstream analysis. The `--summary` flag produces per-sequence statistics including entropy, GC bias, and total unique motif counts.

## Pitfalls

- **Using `--klen` values above 16**: k-mer lengths exceeding 16 produce memory usage proportional to 4^k, which can exhaust available RAM on typical workstations. The tool will attempt to allocate but may crash or be killed by the OS with no error message. Always cap `--klen` at 16 for standard use; larger values require specialized high-memory environments.
- **Feeding multi-line FASTA without proper sequence wrapping**: Some pipelines produce FASTA with very long headerless lines (no 80-character wrapping). bipode-httr's parser reads line-by-line and may silently concatenate adjacent sequence lines that belong to different entries if the `>` line is missing, producing garbage binary patterns with inflated k-mer counts.
- **Mismatched `--outfmt` and `--output` file extensions**: Specifying `--outfmt json` but writing to a `.csv` file extension via `--output` produces no warning — the tool writes JSON content into the file regardless of extension. Downstream parsers expecting CSV will fail silently or crash.
- **Omitting `--min-count` on large repeat-rich genomes**: Genomes with long homopolymer runs (e.g., bacterial poly-A tracts) generate dominant motifs that dominate output. Without `--min-count` filtering, the output CSV can grow to gigabyte scale, and downstream statistical summaries become uninterpretable due to a single motif overwhelming all frequency calculations.
- **Confusing `--reference` strand orientation**: The tool aligns input sequences against the reference in forward orientation only. Reverse-complement patterns are detected by the encoding but are not explicitly flagged in output. If you need strand-specific results, preprocess sequences with `seqtk seq -r` to reverse-complement before piping into bipode-httr, otherwise alleles on the reverse strand will be attributed to the forward reference position.

## Examples

### Detect all 6-mer binary motifs in a bacterial genome FASTA
**Args:** `--input genome.fasta --klen 6 --outfmt csv --output motifs.csv`
**Explanation:** Reads the FASTA file, encodes each nucleotide as 2-bit binary, slides a 6-mer window across every base, and writes a CSV with motif sequence, chromosomal position, count, and frequency.

### Find motifs that deviate from a reference human genome at population-level frequency
**Args:** `--input variants.tsv --reference hg38.fa --klen 4 --outfmt bed --min-freq 0.05 --summary`
**Explanation:** Loads the TSV of variant calls, compares each position to the reference, outputs BED-formatted deviation coordinates filtered to motifs present in at least 5% of samples, and prints a summary table to stderr.

### Extract high-frequency octamer patterns from a FASTQ file with read-level statistics
**Args:** `--input reads.fastq --klen 8 --outfmt json --min-count 50 --output high_freq_octamers.json`
**Explanation:** Parses FASTQ quality strings alongside sequences, computes 8-mer binary patterns only for bases with base quality ≥ Q20, retains motifs appearing at least 50 times, and exports per-read JSON with motif lists and entropy scores.

### Generate a difference profile between two closely related viral strains
**Args:** `strain_A.fasta strain_B.fasta --reference strain_A.fasta --klen 5 --outfmt summary`
**Explanation:** Computes binary motifs for both FASTAs side-by-side, subtracts the reference profile, and produces a summary report of positions where strain B differs from strain A with 5-mer context and log2 fold-change in motif frequency.

### Process a compressed FASTQ stream and write filtered binary patterns to JSON
**Args:** `--input