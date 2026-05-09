---
name: codon-bias
category: sequence-analysis
description: Calculates codon usage bias indices and Effective Number of Codons (ENC) for DNA sequences, supporting evolutionary and expression analysis.
tags: [codon-bias, sequence-analysis, genomics, gene-expression]
author: AI-Generated
source_url: https://github.com/EMBO-OSS/codon-bias
---
## Concepts

- **Codon Bias Index Calculation**: The tool computes ENC (Effective Number of Codons) values ranging from 20 to 61, where values below 35 indicate strong codon bias and values approaching 61 indicate uniform codon usage typical of weakly expressed or AT-rich genes.

- **Sequence Input Formats**: Accepts FASTA, EMBL, and GenBank formats via the `-sequence` flag. Multi-sequence entries are processed individually, with one bias report per sequence written to the output file.

- **Codon Usage Table Dependency**: Custom codon frequency files can be supplied via `-cfile` to reference organism-specific usage patterns. When omitted, the tool defaults to the E. coli K-12 codon usage table embedded in the resource directory.

- **Output Modes**: Results are printed as human-readable text by default. The `-nohead` flag suppresses column headers, enabling programmatic parsing. The `-rscu` flag outputs Relative Synonymous Codon Usage values instead of ENC.

- **Auto Mode for Pipelines**: The `-auto` flag bypasses interactive prompts, making the tool suitable for integration into batch workflows where all required parameters must be specified on the command line.

## Pitfalls

- **Missing Codon Frequency File**: Providing a non-existent or malformed codon usage table path via `-cfile` causes the tool to exit with code 127 and produce no output, silently discarding the input sequence analysis.

- **Conflicting Output Flags**: Combining `-rscu` with `-outfile` results in the RSCU values being written to the specified file, but the ENC column is omitted from the report, potentially breaking downstream parsing scripts that expect both metrics.

- **Ignoring Sequence Type**: Passing protein sequences (translated genes) without the `-translate` flag causes the tool to misinterpret amino acid codons as nucleotide triplets, yielding nonsensical ENC values above 61 that appear valid but are biologically meaningless.

- **Batch Processing Order**: When processing large multi-sequence files with `-auto`, the tool processes sequences sequentially and reports exit code 0 even if individual sequences fail validation, requiring manual inspection of the output log to identify truncated entries.

- **Integer Overflow with Short Sequences**: Sequences shorter than 63 nucleotides (21 codons) trigger integer division errors in the variance calculation, outputting "NaN" for the standard deviation column while still printing the ENC value, which may cause downstream statistical tools to crash.

## Examples

### Calculate codon bias for a single gene sequence
**Args:** `-sequence-inputseq.fasta -outfile-output.txt -auto`
**Explanation:** Reads the single FASTA entry from `inputseq.fasta`, computes the ENC value, and writes the bias report to `output.txt` without prompting for confirmation.

### Generate RSCU values for a codon usage comparison
**Args:** `-sequence-cds_collection.fasta -rscu -outfile-rscu_matrix.txt -auto`
**Explanation:** Outputs Relative Synonymous Codon Usage values for all synonymous codon groups in the input CDS sequences, formatted as a tab-delimited matrix suitable for clustering analysis.

### Use a custom codon usage table for yeast analysis
**Args:** `-sequence-saccharomyces_cds.fasta -cfile-yeast_codons.txt -outfile-yeast_bias.txt -auto`
**Explanation:** References the provided yeast codon frequency file instead of the default bacterial table, ensuring accurate bias calculation for organisms with distinct tRNA gene copy numbers.

### Suppress headers for programmatic parsing
**Args:** `-sequence-multiple_genes.fasta -nohead -outfile-parseable.txt -auto`
**Explanation:** Outputs ENC results without column headers or formatting, creating a whitespace-delimited file where column positions directly correspond to sequence ID, ENC value, and standard deviation for automated extraction.

### Calculate bias from protein sequence (translated input)
**Args:** `-sequence-protein_seqs.fasta -translate -outfile-protein_bias.txt -auto`
**Explanation:** First translates the input protein sequences back to hypothetical DNA using the standard genetic code, then calculates codon bias indices on the reconstructed codons, producing estimates for protein-centric datasets.

### Batch process a directory with custom output naming
**Args:** `-sequence-dir/input_dir/ -cfile-ecoli_codons.txt -outfile-batch_results.txt -auto`
**Explanation:** Processes all valid sequence files in the specified directory, appending each result to `batch_results.txt`, with one ENC entry per sequence line and the input filename recorded in the first column.