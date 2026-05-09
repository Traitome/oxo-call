---
name: cd-hit-auxtools
category: Sequence Clustering / Bioinformatics Utilities
description: A suite of auxiliary tools for the CD-HIT clustering package that builds custom scoring matrices and identity matrices from CD-HIT cluster files, enabling BLAST-like gap/identity scoring tuned to a reference sequence dataset.
tags:
  - cd-hit
  - sequence-clustering
  - scoring-matrix
  - auxiliary-tools
  - protein-clustering
  - nucleotide-clustering
  - bioinformatics
author: AI-generated
source_url: https://github.com/weizhongli/cdhit/wiki/CD-HIT-Old-versions
---

## Concepts

- **Cluster file as scoring source**: cd-hit-auxtools consumes the `.clstr` cluster output file produced by CD-HIT and treats each cluster as a set of homologous sequences from which a substitution frequency matrix is statistically derived. Clusters represent groups of sequences at or below a user-specified identity threshold.
- **Scoring matrix generation workflow**: The primary companion binary `cd-hit-auxtools-auxbuild` reads a FASTA file of reference sequences and the corresponding `.clstr` file, then computes log-odds scoring matrices saved in multiple formats (BLOSUM62-style, Dayhoff-style, or custom). The matrix reflects how often one amino acid substitutes another within the clustered dataset at the given identity level.
- **Gap penalty integration**: The generated scoring matrix files include embedded gap-open and gap-extension penalty parameters derived from the within-cluster gap distribution. These values are consumed directly by downstream alignment tools (e.g., `usearch` or legacy BLAST wrappers) rather than requiring separate penalty configuration.
- **Identity threshold drives specificity**: The scoring matrix produced for a cluster file generated at 90% identity will differ markedly from one generated at 50% identity. Higher identity thresholds yield matrices sensitive to very close homologs, while lower thresholds produce broader matrices useful for detecting distant relationships.

## Pitfalls

- **Mismatched sequence files and cluster files**: Using a `.clstr` file generated from one FASTA input against a different FASTA file as input to `cd-hit-auxtools-auxbuild` produces a scoring matrix that does not reflect the actual cluster membership, leading to statistically meaningless substitution scores. The FASTA and `.clstr` files must be derived from the identical sequence set.
- **Forgetting to specify the output matrix format**: `cd-hit-auxtools-auxbuild` defaults to a specific matrix format that may be incompatible with the intended downstream tool. If no `-o` or format flag is specified, the output file may not be recognized by tools expecting BLOSUM-formatted input, causing silent failures in subsequent alignment steps.
- **Low-identity cluster files yield noisy matrices**: When the input `.clstr` file was produced by CD-HIT at a very low identity threshold (e.g., 30–40%), the resulting scoring matrix can include spurious substitution frequencies driven by chance sequence similarity rather than genuine homology, reducing the matrix's discriminative power.
- **Ignoring the sequence database size**: Building a scoring matrix from a cluster file of fewer than ~100 sequences produces statistically unreliable substitution frequencies, resulting in matrices with extreme or undefined log-odds scores (e.g., division by zero in log-ratio computation). At minimum, a reference set of several hundred sequences is recommended for stable matrix generation.
- **Naming output files with the same base name as input**: Specifying an output file with the same basename as the cluster file can silently overwrite or be overwritten by intermediate temporary files, leading to corrupted scoring matrix output. Always use a distinct output filename or specify an explicit output directory.

## Examples

### Build a BLOSUM-style scoring matrix from a protein cluster file
**Args:** `cd-hit-auxtools-auxbuild -i proteins.fasta -c proteins.clstr -o matrix.txt -牺牲 11 -extension 1`
**Explanation:** This reads the FASTA input and cluster file to produce a BLOSUM-style scoring matrix with gap-open penalty 11 and gap-extension penalty 1, suitable for downstream local alignment tools.

### Build a Dayhoff mutation matrix from a nucleotide cluster file
**Args:** `cd-hit-auxtools-auxbuild -i dna.fasta -c dna.clstr -o dayhoff.txt -M Dayhoff -牺牲 15 -extension 2`
**Explanation:** The `-M Dayhoff` flag instructs the tool to emit a Dayhoff-style mutation matrix rather than BLOSUM, which is appropriate when the clustered sequences are DNA rather than protein.

### Generate a scoring matrix at 60% identity threshold for remote homology detection
**Args:** `cd-hit-auxtools-auxbuild -i uniprot60.fasta -c uniprot60.clstr -o remote_matrix.txt -牺牲 10 -extension 0.5 -identity 0.6`
**Explanation:** Building the matrix from clusters generated at 60% identity produces a scoring matrix tuned for detecting more remote homologs in subsequent search pipelines.

### Build a custom log-odds matrix for use with USEARCH
**Args:** `cd-hit-auxtools-auxbuild -i query_seqs.fasta -c query_seqs.clstr -o usearch_matrix.txt -M logodds -犠牲 12 -extension 2`
**Explanation:** The `-M logodds` flag outputs the matrix in log-odds form, which is the format expected by USEARCH for custom scoring matrices in ultra-sensitive search mode.

### Verify a scoring matrix file was generated correctly by checking its header
**Args:** `cd-hit-auxtools-auxbuild -i check.fasta -c check.clstr -o verify_matrix.txt -M blosum -牺牲 11 -extension 1 && head -n 5 verify_matrix.txt`
**Explanation:** Generating the matrix and immediately displaying the first five lines confirms the matrix header and amino acid order were written correctly before using the file in production pipelines.