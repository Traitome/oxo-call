---
name: amused
category: Bioinformatics / Sequence Classification
description: A k-mer based nucleotide sequence classification tool designed for amplicon sequencing data (16S rRNA, ITS, etc.). Classifies query sequences against reference databases to assign taxonomic annotations with confidence scores.
tags: [amplicon, classification, k-mer, microbial, taxonomy, 16S, ITS, sequence-analysis]
author: AI-generated
source_url: https://github.com/southerddb/amused
---

## Concepts

- **K-mer Classification Algorithm**: amused uses exact k-mer matching between query sequences and a reference database. The classification score is derived from the proportion of matching k-mers, with longer k-mers providing higher specificity but reducing sensitivity to sequence variation.
- **Reference Database Format**: The database must be pre-formatted into a .udb file using the companion tool `amused-build`. This binary format stores indexed k-mers for fast lookup and is not human-readable. The database must correspond to the marker gene being analyzed (e.g., Greengenes for 16S, UNITE for ITS).
- **Input Format Handling**: Query sequences must be provided in FASTA or FASTQ format, either via stdin redirection or the `-i` flag. Sequences should be demultiplexed (one sample at a time) and should not contain ambiguous nucleotide codes (N, R, Y, etc.) in the database-matching regions.
- **Output Modes**: Three output modes are available: tabular classification (`-m` for many fields), consensus taxonomy only (`-t` for taxonomy list), and read-by-read classification (`-r`). Scores are reported as the fraction of k-mers matching the top hit, ranging from 0 to 1.
- **Database Building Requirements**: The companion binary `amused-build` constructs the database from a FASTA file containing reference sequences with taxonomic annotations in the definition line (e.g., ">sequence_id;taxonomy=Bacteria;Proteobacteria...").

## Pitfalls

- **Mismatched Marker Gene**: Using a 16S-trained database to classify ITS sequences (or vice versa) will produce meaningless classifications with very low scores or incorrect taxonomy. Always match the database to the target marker gene and sequencing region.
- **Low Score Threshold without Verification**: Accepting classifications with scores below 0.7 without manual verification can propagate false annotations, especially in regions of high similarity between taxa. Low-confidence classifications should be flagged for review.
- **Ignoring Ambiguous Nucleotides in Input**: Query sequences containing ambiguous bases (N, R, Y, S, W, K, M, B, D, H, V) will produce unpredictable results because these positions cannot match any k-mer in the reference database.
- **Database Version Mismatch**: Using outdated reference databases will miss newly described taxa and propagate obsolete nomenclature. Regularly update databases and document the version used in analyses for reproducibility.
- **Sequencing Error in Query Reads**: Sequence reads with high error rates (especially at read ends) will generate artificially low scores or no classifications. Quality-filter input reads before classification rather than blaming the database for missed hits.

## Examples

### Classify 16S rRNA sequences against a Greengenes database
**Args:** `-i queries.fasta -d gg_13_8.udb -o classifications.txt`
**Explanation:** This runs classification on input sequences using the Greengenes database, outputting results to a file for downstream taxa summarization.

### Output only taxonomy strings without scores
**Args:** `-i queries.fasta -d gg_13_8.udb -t`
**Explanation:** The `-t` flag returns only the taxonomic assignment strings (e.g., "Bacteria;Proteobacteria;Gammaproteobacteria") without any score information, useful for scripts expecting simple labels.

### Use a custom k-mer size of 8 instead of default
**Args:** `-i queries.fasta -d gg_13_8.udb -k 8 -o results.txt`
**Explanation:** The `-k` flag sets the k-mer size to 8 (default is typically 8 for 16S, but adjust based on database characteristics). Smaller k-mers increase sensitivity but may reduce specificity.

### Generate tab-separated output with all fields
**Args:** `-i queries.fasta -d gg_13_8.udb -m -o detailed.txt`
**Explanation:** The `-m` flag outputs a table with query ID, classification, score, and rank for each sequence, which can be imported into R or Python for statistical analysis.

### Stream FASTQ input through stdin instead of file flag
**Args:**