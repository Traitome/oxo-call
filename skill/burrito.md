---
name: burrito
category: functional-annotation
description: A command-line tool for automated functional annotation of protein sequences using HMMER profiles and BLAST homology searches.
tags:
  - protein-annotation
  - hmm
  - functional-domain
  - homology
  - biopython
author: AI-generated
source_url: https://github.com/fstrobeck/burrito
---

## Concepts

- Burrito annotates protein sequences by searching queries against HMM databases (such as Pfam) and BLAST databases to identify functional domains, protein families, and homology-based annotations.
- The tool accepts input in standard sequence formats (FASTA, GenBank, EMBL) and outputs annotation results in structured formats including JSON, TSV, and HTML reports.
- Annotation confidence is determined by E-value thresholds, bit scores, and coverage percentages; sequences failing quality thresholds are flagged with low-confidence scores for manual review.
- Burrito supports parallel processing via multiprocessing flags to accelerate large-scale annotation tasks across multiple CPU cores.
- The annotation pipeline includes validation steps that check for conflicting domain assignments and redundant hits, resolving overlaps by preferring higher-scoring matches.

## Pitfalls

- Specifying DNA sequences instead of translated protein sequences causes HMMER search failures because domain models are trained on amino acid alphabets, producing invalid E-values and zero significant hits.
- Using default E-value thresholds (e.g., 0.01) on small query sets results in excessive false-positive annotations; thresholds should be tightened to 1e-5 or stricter for genomes smaller than 100 proteins.
- Selecting an incompatible HMM database version for your organism (e.g., using a bacterial-only database for eukaryotic proteins) yields no significant matches and missing functional annotations.
- Omitting the `--translate` flag when input consists of coding DNA sequences causes burrito to treat nucleotides as protein characters, resulting in corrupted output and incorrect domain calls.
- Insufficient disk space for temporary files during parallel annotation jobs causes truncated output files and silent data loss when the pipeline cannot write intermediate results.

## Examples

### Annotating a protein FASTA file against Pfam HMMs
**Args:** `annotate --query proteins.fasta --db pfam --format json --evalue 1e-10 --output annotations.json`
**Explanation:** This searches protein sequences in FASTA format against the Pfam database using a stringent E-value threshold to identify conserved functional domains.

### Translating and annotating coding DNA sequences
**Args:** `annotate --query cds.fasta --translate --db pfam --format tabular --output cds_annotations.tsv`
**Explanation:** The `--translate` flag ensures DNA sequences are first translated into protein sequences before HMM profile searches, enabling annotation of genomic coding regions.

### Fetching sequences with annotation from a database
**Args:** `fetch --ids UP000005640 --type uniprot --format fasta --output human_proteome.fasta`
**Explanation:** This retrieves protein sequences from UniProt using accession identifiers and saves them in FASTA format for downstream annotation workflows.

### Filtering annotated sequences by confidence score
**Args:** `filter --input annotations.json --min-score 50 --max-evalue 1e-15 --output high_confidence.json`
**Explanation:** This retains only high-confidence annotations where the bit score exceeds 50 and the E-value is below 1e-15, removing uncertain or spurious domain assignments.

### Parallel annotation of multiple genomes
**Args:** `annotate --query proteomes/ --db cdd --format html --evalue 1e-5 --cpus 8 --output batch_results/`
**Explanation:** This processes all protein files in the proteomes directory using 8 CPU cores in parallel against the CDD database, generating HTML reports for each genome.