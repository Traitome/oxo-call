---
name: alpaqa-bio
category: Sequence Optimization / Bioinformatics
description: A bioinformatics tool for sequence optimization, alignment refinement, and biological sequence analysis. alpaqa-bio provides functions for optimizing biological sequences against target properties, performing iterative refinement, and generating variant collections for downstream analysis.
tags:
  - sequence-optimization
  - bioinformatics
  - sequence-analysis
  - variant-generation
  - biological-sequences
author: AI-generated
source_url: https://github.com/bioinformatics-tools/alpaqa-bio
---

## Concepts

- **Input Formats**: alpaqa-bio accepts FASTA (.fasta, .fa), FASTQ (.fastq, .fq), and plain text (.txt) formats for sequence input. Multi-sequence files are processed sequentially, with each sequence treated as an independent optimization target.

- **Output Formats**: Results are produced in optimized FASTA format by default, with optional JSON export for programmatic downstream processing. The tool maintains sequence headers from input to output for traceability.

- **Optimization Modes**: The tool supports three primary modes: (1) global optimization for maximum fitness, (2) local refinement for minor adjustments, and (3) diversity-focused variant generation. Mode selection significantly impacts runtime and result quality.

- **Scoring Functions**: Built-in scoring functions include GC-content matching, codon usage optimization, and thermodynamic stability estimation. Custom scoring scripts can be integrated via the `--score-script` flag.

## Pitfalls

- **Insufficient Sequence Length**: Running optimization on sequences shorter than 10bp produces unreliable results and may cause convergence to local optima. Always ensure input sequences meet minimum length requirements for the selected optimization mode.

- **Mismatched Scoring Function**: Using a GC-content scoring function for protein sequence optimization produces meaningless results. Select scoring functions that match your input sequence type (nucleotide vs. protein).

- **Resource Exhaustion with Large Inputs**: Processing files with >10,000 sequences without adjusting `--max-iterations` and memory limits causes tool failure. Use batch processing with the `--batch-size` flag for large inputs.

- **Output Overwriting**: Default behavior overwrites output files without confirmation. Always specify a unique output filename or enable the `--no-clobber` flag to prevent accidental data loss.

## Examples

### Optimize a DNA sequence for maximum GC content
**Args:** `input.fasta --mode global --score-func gc-content --output optimized.fasta`
**Explanation:** This runs global optimization on input sequences using GC content as the scoring function, producing sequences with optimally balanced GC content saved to the output file.

### Refine a protein sequence locally for improved thermodynamic stability
**Args:** `protein.fasta --mode local --score-func thermodynamic --max-iterations 500`
**Explanation:** This performs local refinement on protein sequences using thermodynamic stability scoring, limiting optimization to 500 iterations for fine-tuned adjustments.

### Generate 100 diverse variants from a template sequence
**Args:** `template.fasta --mode diversity --num-variants 100 --output variants.fasta`
**Explanation:** This generates 100 diverse sequence variants from the input template, useful for downstream mutational analysis and evolutionary studies.

### Process multiple sequences with batch optimization
**Args:** `batch_input.fasta --mode global --batch-size 50 --output batch_out.fasta`
**Args:** `--score-func codon-usage`
**Explanation:** This processes input sequences in batches of 50, optimizing each for codon usage, which improves expression potential in heterologous systems.

### Export results in JSON format for downstream analysis
**Args:** `input.fasta --mode local --output results.json --format json`
**Explanation:** This exports optimization results as JSON, preserving detailed scoring information and metadata suitable for programmatic downstream pipelines.