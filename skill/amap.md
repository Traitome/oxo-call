---
name: amap
category: Sequence Alignment / Protein Mapping
description: AMAP (Amino Acid MAPping) aligns amino acid query sequences against a reference database to identify homologous proteins and functional domains using efficient scoring algorithms.
tags:
  - protein-alignment
  - sequence-mapping
  - homology-detection
  - bioinformatics
  - amino-acid-sequences
author: AI-generated
source_url: https://github.com/sujae/AMAP
---

## Concepts

- **Sequence Input Format**: AMAP accepts amino acid sequences in standard FASTA format for both query inputs and database references. Each sequence entry must begin with a header line starting with '>' followed by one or more lines of single-letter amino acid codes.
- **Database Construction**: A reference database must be built using the companion `amap-build` binary before any mapping queries can be performed. The built database index contains precomputed scoring data for accelerated lookups during alignment.
- **Scoring Matrices**: AMAP uses amino acid substitution matrices (such as BLOSUM62 or PAM series) to evaluate residue matches and mismatches. The choice of scoring matrix significantly affects alignment sensitivity and specificity.
- **Output Format Types**: Alignment results can be generated in multiple formats including tabular, XML, and human-readable text. The `--outfmt` flag controls the output serialization, with tabular format being most suitable for downstream bioinformatic processing.
- **E-value Thresholding**: Statistical significance is assessed via E-value calculation, which estimates the expected number of false alignments at a given score threshold. Lower E-values indicate higher confidence alignments.

## Pitfalls

- **Building Database with Wrong File Type**: Using nucleotide sequences instead of amino acid sequences as input for `amap-build` will produce incorrect alignments because the scoring matrices are designed for amino acid residues only.
- **Mismatched Query-Database Formats**: Attempting to map a query sequence that contains invalid amino acid characters (such as 'U' for selenocysteine or 'O' for pyrrolysine without proper flag specification) causes the alignment to fail silently or produce truncated results.
- **Ignoring E-value Cutoff**: Running AMAP without specifying an E-value threshold with `--evalue` results in all alignments being reported, including spurious matches with high E-values that represent false positives.
- **Specifying Invalid Scoring Matrix**: Using a matrix name that does not exist in the AMAP matrix library causes the tool to terminate with an error, wasting computation time on failed jobs.
- **Overwriting Output Files**: Redirecting output to an existing file path does not prompt for confirmation; the file is silently overwritten, potentially losing previous analysis results.

## Examples

### Build a protein reference database from a FASTA file

**Args:** -i proteins.fasta -d amapdb
**Explanation:** This constructs a preindexed database from amino acid sequences in `proteins.fasta`, enabling rapid subsequent mapping queries against this reference set.

### Map a single query sequence with default settings

**Args:** -q query.fasta -d amapdb -o results.txt
**Explanation:** This aligns the amino acid sequence from `query.fasta` against the prebuilt database and writes all alignments meeting default thresholds to `results.txt`.

### Filter results by E-value threshold

**Args:** -q query.fasta -d amapdb -o highconf.txt --evalue 1e-10
**Explanation:** This restricts output to only those alignments with an expected false positive rate below one in ten billion, yielding high-confidence homologous matches.

### Output alignments in tabular format for scripting

**Args:** -q queries.fasta -d amapdb -o tabular_out.tsv --outfmt tab
**Explanation:** This generates a tab-delimited file where each row represents a separate alignment, facilitating automated parsing in downstream bioinformatic pipelines.

### Use BLOSUM80 scoring matrix for close homologs

**Args:** -q query.fasta -d amapdb -o strict_align.txt --matrix BLOSUM80 --evalue 0.001
**Explanation:** The BLOSUM80 matrix penalizes substitutions more strictly than BLOSUM62, making it suitable for detecting evolutionarily close protein homologs with conservative changes.

### Generate XML output for visualization tools

**Args:** -q query.fasta -d amapdb -o alignment.xml --outfmt xml
**Explanation:** XML format preserves detailed alignment metadata including positional scoring information that can be consumed by visualization applications or web interfaces.

### Search with multiple query sequences in batch

**Args:** -q batch_queries.fasta -d amapdb -o batch_results.txt --evalue 1e-5 --max_target_seqs 5
**Explanation:** This processes all sequences in `batch_queries.fasta`, reporting up to five best hits per query, enabling high-throughput screening of large sequence sets.