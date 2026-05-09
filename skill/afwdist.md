---
name: afwdist
category: Sequence Analysis
description: Computes pairwise alignment-free distances between biological sequences using k-mer based statistical models. Supports multiple distance metrics and outputs distance matrices for downstream phylogenetic or clustering analysis.
tags:
  - alignment-free
  - k-mer
  - distance-matrix
  - sequence-similarity
  - phylogenetics
author: AI-generated
source_url: https://github.com/bioinline/afwdist
---

## Concepts

- **K-mer counting as the basis for distance estimation**: afwdist builds k-mer frequency profiles from each input sequence and computes a distance metric between profiles. Larger k values capture longer shared motifs but require more sequence data to estimate reliably; k=3 or k=4 are standard for nucleotide sequences.
- **Multiple distance metrics**: The tool supports Jensen-Shannon divergence (default), Euclidean, Bray-Curtis, and Cosine distance. Jensen-Shannon is recommended for its symmetry and bounded range (0 to 1), where 0 indicates identical k-mer profiles and 1 indicates maximally dissimilar profiles.
- **Output format controls**: Distance matrices are written in Phylip format by default, which is compatible with most phylogenetic inference tools (e.g., RAxML, FastTree). CSV and JSON output formats are available for integration into pipelines or downstream scripting.
- **Sequence input requirements**: Input must be a multi-FASTA file. Each sequence must have a unique header line starting with `>`. Empty sequences or sequences shorter than the specified k-mer size are silently skipped, which can reduce the number of taxa in the output matrix.
- **Memory and runtime scaling**: Runtime scales approximately quadratically with the number of input sequences (pairwise comparison) and linearly with sequence length. For large sets (>500 sequences), consider using the `--threads` flag to enable parallelized distance computation.

## Pitfalls

- **K-mer size exceeding sequence length**: If any sequence in the input is shorter than the k-mer size, that sequence contributes only a single k-mer observation, severely distorting its distance profile. Always verify that the minimum sequence length is at least 5 times the k-mer size for reliable estimates.
- **Output file overwriting without warning**: The `--output` flag silently overwrites any existing file at the specified path. Use shell redirection (`>`) or explicitly check for file existence before running, because there is no confirmation prompt or backup mechanism.
- **Inconsistent sequence labels across runs**: If the input FASTA contains duplicate sequence headers, afwdist does not raise an error; it processes both sequences independently, which can cause silent downstream errors in phylogenetic software that expects unique labels.
- **Missing distance values when sequences are skipped**: Skipped sequences (too short or malformed) are omitted from the output matrix without a warning line in the log. This creates an asymmetric effective taxon count that is easy to miss, leading to incorrect assumptions about which taxa were compared.
- **Ignoring output precision settings**: Default output writes distances rounded to 6 decimal places. For highly similar sequences, this may truncate meaningful differences, leading to false zero distances in the matrix. Use `--precision 10` when analyzing closely related strains.

## Examples

### Compute a distance matrix from a FASTA file using default settings

**Args:** `sequences.fasta -o distances.phylip`
**Explanation:** This reads all sequences from the input FASTA, builds k-mer frequency profiles using the default k-mer size of 3, computes Jensen-Shannon distances between all pairs, and writes the resulting distance matrix in Phylip format to the output file.

### Calculate distances with a custom k-mer size of 4 and cosine metric

**Args:** `genomes.fasta --kmer 4 --metric cosine -o cosine_distances.phylip`
**Explanation:** Setting `--kmer 4` increases the k-mer length to 4, capturing slightly longer shared motifs, while `--metric cosine` switches to cosine similarity-based distance, which is less sensitive to differences in overall k-mer count magnitude.

### Output results in CSV format for a downstream R pipeline

**Args:** `sequences.fasta --metric euclidean --format csv -o distances.csv`
**Explanation:** Specifying `--format csv` writes a comma-separated file where each row and column corresponds to a sequence identifier, with pairwise Euclidean distances as cell values, making it straightforward to import directly into R using `read.csv()`.

### Run with 8 threads on a large dataset to speed up computation

**Args:** `large_dataset.fasta --threads 8 -o large_distances.phylip`
**Explanation:** The `--threads 8` flag parallelizes the pairwise distance calculations across 8 CPU cores, which can reduce wall-clock time substantially for large inputs, though the effective parallelism is capped by the number of sequence pairs.

### Set high precision for analyzing closely related viral strains

**Args:** `viral_seqs.fasta --kmer 3 --metric jsd --precision 10 -o viral_distances.phylip`
**Explanation:** Using `--precision 10` ensures that very small differences between highly similar viral sequences are preserved in the output, which is critical when differentiating strains that may only differ by one or two k-mer observations.