---
name: askocli
category: Codon Usage Analysis
description: A command-line tool for analyzing synonymous codon usage, codon usage bias, and genetic code patterns in DNA or protein-coding sequences. Supports ENC calculations, RSCU analysis, and CAI computations for evolutionary and biotechnological studies.
tags: [codon-usage, synonymous-codon, genetic-code, ENC, RSCU, CAI, nucleotide-analysis, bioinformatics]
author: AI-Generated
source_url: https://github.com/eynorey/askocli
---

## Concepts

- **Sequence Input Formats**: askocli accepts FASTA (.fasta, .fa, .fna) and GenBank flatfile (.gb) formats. Nucleotide sequences must be complete coding sequences (CDS) with valid start (ATG) and stop codons; partial or non-coding sequences will produce misleading bias metrics.
- **Codon Usage Indices**: The tool calculates Effective Number of Codons (ENC), Relative Synonymous Codon Usage (RSCU), Codon Adaptation Index (CAI), and GC content. ENC values range from 20 (extreme bias) to 61 (no bias); RSCU values of 1.0 indicate equal usage of synonymous codons.
- **Reference Sets**: CAI calculations require a reference set of highly expressed genes (provided via `--reference` flag). Without an appropriate reference, CAI values are meaningless; for microbial genomes, a ribosomal protein gene set is recommended.
- **Output Modes**: Results are exported as tab-delimited text (default) or CSV (via `--output-format csv`). Large multi-sequence files should use `--batch` mode to avoid memory exhaustion; each sequence is processed independently.
- **Translation Table**: The tool defaults to the Standard Genetic Code (translation table 1). For mitochondrial, bacterial, or archaeal genomes, specify `--genetic-code` with the appropriate NCBI translation table number (e.g., 11 for bacterial).

## Pitfalls

- **Missing Stop Codon**: Sequences lacking an in-frame stop codon are processed but generate an "incomplete CDS" warning. Using such sequences for ENC calculations violates the assumptions of codon usage models and produces inaccurate bias estimates.
- **Wrong Reference Set for CAI**: Supplying a poorly curated reference gene set (e.g., all annotated genes rather than highly expressed ones) deflates CAI discriminatory power. The resulting CAI values will cluster near 1.0, eliminating the index's ability to distinguish expression levels.
- **Ignoring Genetic Code Flag**: For organisms with alternative genetic codes (e.g., Tetrahymena using translation table 6), forgetting `--genetic-code` causes systematic misassignment of amino acids to codons, corrupting all downstream RSCU and ENC values.
- **Sequence Polarity Confusion**: Providing sequences in reverse complement orientation without using `--reverse-complement` produces inverted codon usage patterns; this is a common error when analyzing the lagging strand, which exhibits biased codon usage by design.

## Examples

### Calculate ENC and GC content for a single FASTA sequence
**Args:** `NC_000913.fna --enc --gc-content --format fasta`
**Explanation:** Computes the Effective Number of Codons and GC fraction for all complete CDS in the E. coli genome FASTA file, outputting tabular results to stdout.

### Compute RSCU values for multiple genes with CSV output
**Args:** `genes_collection.fa --rscu --output-format csv --output rscu_results.csv`
**Explanation:** Calculates Relative Synonymous Codon Usage for each gene in the batch file and exports results as comma-separated values for spreadsheet import.

### Analyze codon usage with a custom reference set for CAI
**Args:** `output_seqs.fasta --cai --reference ribosomal_proteins.fasta --genetic-code 11`
**Explanation:** Estimates Codon Adaptation Index using ribosomal protein genes as the reference for Bacillus subtilis (genetic code 11), suitable for predicting heterologous expression levels.

### Generate ENC vs. GC third position plot data
**Args:** `genome_cds.gb --enc-plot --gc-silent --output enc_gc_data.txt`
**Explanation:** Outputs ENC values and GC content at third positions for plotting an ENC vs. GC curve, used to detect selection pressure vs. mutation bias in codon usage.

### Compare codon bias between two genomic samples
**Args:** `sampleA_cds.fa sampleB_cds.fa --enc --rscu --paired-output comparison_results.tsv`
**Explanation:** Performs side-by-side ENC and RSCU analysis on two sequence sets, outputting aligned results for direct comparative statistics in downstream scripts.

### Process a multi-record GenBank file with batch mode
**Args:** `plasmid_collection.gb --batch --enc --rscu --gc-content --threads 4`
**Explanation:** Processes each CDS entry in the GenBank file using 4 parallel threads, suitable for high-throughput analysis of vector or viral genomes with many independent genes.