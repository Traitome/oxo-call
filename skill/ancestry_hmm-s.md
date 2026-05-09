---
name: ancestry_hmm-s
category: sequence-analysis
description: Hidden Markov model-based tool for phylogenetic ancestry inference and sequence classification. Analyzes DNA or protein sequences against reference HMM profiles to determine evolutionary relationships and ancestral state probabilities.
tags: [hmm, phylogenetics, sequence-analysis, ancestry, evolution, classification, bioinformatics]
author: AI-generated
source_url: https://github.com/hmmerr/ancestry-hmm
---

## Concepts

- **HMM Profile Input**: The tool accepts reference hidden Markov model profiles (typically in HMMER3 format .hmm) that represent evolutionary conserved sequence families or protein domains. Profiles define position-specific scoring matrices and insertion/deletion penalties.
- **Sequence Input Formats**: Supports multiple sequence input formats including FASTA, EMBL, GenBank, and unaligned sequence lists. Sequences are parsed and converted to emission symbols matching the model's alphabet (protein or nucleotide).
- **Output Modes**: Provides three distinct output modes—tabular summary (-t), full alignment (-a), and JSON structured results (-j). The tabular output includes E-value, score, bias, and coverage metrics for each query-target comparison.
- **Score Thresholds**: Uses log-odds scores converted from raw HMM scores. The tool applies conditional nucleotide or amino-acid emission probabilities against background null model distributions to compute statistical significance.

## Pitfalls

- **Mismatched Alphabet**: Running nucleotide sequences against a protein HMM profile (or vice versa) produces zero hits because emission symbols cannot match the profile's score matrix. Always verify the sequence alphabet matches the profile type before execution.
- **E-value Confusion**: An E-value of 0.0 does not mean "perfect match"—it indicates the result is so significant it rounds to zero under the chosen threshold. Use raw score and bit-score alongside E-value to interpret true alignment quality.
- **Missing Null Model**: Omitting the --null flag when analyzing sequences from biased composition datasets (e.g., extreme GC content in bacterial genomes) leads to false positive hits. The default null model assumes uniform residue distribution.
- **Memory Limits**: Large HMM profiles (thousands of states) combined with millions of sequences can exceed available RAM. Process large datasets in discrete chunks or reduce --maxfreetristates to limit memory allocation.

## Examples

### Identify ancestry of a single protein sequence

**Args:** -t protein.hmm query_sequence.fasta --domE 0.001
**Explanation:** Searches query_sequence.fasta against the protein.hmm profile and reports only domain matches with E-value better than 0.001 in tabular format.

### Batch process multiple sequences with custom output

**Args:** -a my_families.hmm dataset/*.fasta -o batch_results.out
**Explanation:** Processes all FASTA files in dataset/ directory against my_families.hmm and writes full alignments to batch_results.out.

### Filter results by bit-score threshold

**Args:** -t reference.hmm queries.fa --cut_ta --score 25.0
**Explanation:** Runs search with trusted cutoff applied and only reports hits scoring at least 25.0 bits, ensuring high-confidence matches.

### Generate JSON output for downstream parsing

**Args:** -j annotation.hmm sequences.fasta --jsonPretty -o parsed.json
**Explanation:** Outputs machine-readable JSON format with pretty-printing enabled for integration into automated pipelines.

### Control search sensitivity with E-value

**Args:** -t conserved_domains.hmm input.fa -E 10.0
**Explanation:** Sets a lenient E-value threshold of 10.0 to capture weak but potentially biologically relevant hits for exploratory analysis.

### Specify alternative null model file

**Args:** -t bacterial_markers.hmm genome_seqs.fasta --null bg_freq.txt
**Explanation:** Uses custom background frequency file bg_freq.txt instead of the default uniform null model to account for nucleotide bias.