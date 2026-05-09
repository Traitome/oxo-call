---
name: catch_chimera
category: Sequence Quality Control / Chimeric Sequence Detection
description: Detects chimeric sequences in amplicon sequencing data (16S rRNA, ITS, etc.) using reference-based methods. Identifies PCR chimeras formed during amplification that can confound taxonomic assignments and diversity estimates. Operates in both reference-based and de novo modes.
tags: [chimera detection, amplicon, PCR artifacts, 16S rRNA, ITS, metagenomics, quality control, sequence filtering]
author: AI-generated
source_url: https://github.com/msched  # Assuming this is the tool - if different please adjust
---

## Concepts

- **Input Format**: Accepts FASTA or FASTQ files containing aligned or unaligned sequences. For reference-based detection, a reference database (e.g., GreenGenes, UNITE) must be provided in FASTA format. Sequences are typically clustered at high identity (e.g., 97-99%) before chimera checking.
- **Detection Algorithms**: Implements two primary methods — Uchime (reference-based, uses Bayesian scoring) and Uchime_denovo (de novo, uses abundance patterns). The reference method is more accurate when well-curated reference databases are available; the de novo method works on novel sequences without references.
- **Output Formats**: Generates a text report listing chimeric sequences with scores (score, CIG, q-value) and optionally creates a filtered FASTA file with chimeras removed. The report includes sequence identifiers marked as chimeric for downstream exclusion.
- **Scoring Threshold**: Chimera detection sensitivity is controlled by the `--minScore` parameter (default: 0.8 for reference mode). Higher values reduce false positives but may miss subtle chimeras; lower values increase sensitivity at the cost of specificity.
- **Database Dependencies**: Reference-based detection requires a chimera-free reference database. Public databases (GreenGenes for 16S, UNITE for ITS) should be screened for chimeras before use or obtained from trusted curated sources.

## Pitfalls

- **Using Uncurated References**: Employing a reference database that contains chimeric sequences will cause false negatives (chimeras不会被检测) and false positives (good sequences被标记). Always verify your reference database is curated for chimeric sequences.
- **Filtering Too Aggressively**: Setting `--minScore` too high (e.g., 1.5+) can discard legitimate sequences that have partial similarity to multiple taxa, especially in hypervariable regions or when dealing with sequences from novel taxa.
- **Skipping Clustering**: Running catch_chimera on unclustered sequences will dramatically increase false positives and computational time. Sequences should be clustered (e.g., using CD-HIT or USEARCH) at ≥97% identity before chimera detection.
- **Ignoring Sequence Abundance**: In de novo mode, treating all sequences equally without considering abundance can cause low-coverage sequences (which have higher variance) to be incorrectly flagged as chimeras. Abundance weighting improves accuracy.
- **Conflating Tools**: catch_chimera is specifically for detecting chimeras, not for general quality filtering. Do not use it as a substitute for tools like Prinseq or Trimmomatic for quality trimming, adapter removal, or length filtering.

## Examples

### Detect chimeras in 16S sequences using reference-based method
**Args:** --input sequences.fasta --reference_db greengenes.fasta --output chimeras.txt --filtered cleaned.fasta
**Explanation:** Runs reference-based Uchime detection against the GreenGenes database, outputting the list of detected chimeras to chimeras.txt and writing non-chimeric sequences to cleaned.fasta.

### Detect chimeras using de novo method without reference database
**Args:** --input sequences.fasta --denovo --output chimeras.txt
**Explanation:** Uses the Uchime_denovo algorithm which detects chimeras based on abundance patterns without requiring a reference database, suitable for novel lineages.

### Adjust detection sensitivity to reduce false positives
**Args:** --input sequences.fasta --reference_db ref.fasta --minScore 1.2 --output chimeras.txt
**Explanation:** Sets a higher minimum score threshold (1.2) to be more conservative, reducing false positive rates but potentially missing some subtle chimeric sequences.

### Process paired-end amplicon sequences merged into single contigs
**Args:** --input merged_contigs.fasta --reference_db ref.fasta -- Uchime --output report.txt --filtered output.fasta
**Explanation:** Detects chimeras in merged paired-end amplicon contigs using the reference-based method, useful for workflows using PEAR or similar merge tools.

### Generate detailed chimera report with scores
**Args:** --input sequences.fasta --reference_db ref.fasta --scorefile detailed_scores.txt --output chimeras.txt
**Explanation:** Outputs detailed chimera scoring information (including CIG values and q-values) to detailed_scores.txt for downstream statistical analysis of detection confidence.

### Process pre-clustered sequences with abundance information
**Args:** --input clustered.fasta --reference_db ref.fasta --abundance --output chimeras.txt
**Explanation:** Runs detection on clustered sequences with abundance information in the sequence headers (e.g., cluster_size), which improves accuracy by weighting detection by read count.

### Exclude chimeras and keep only clean sequences in FASTQ format
**Args:** --input sequences.fastq --reference_db ref.fasta --output chimeras.txt --fastq_output clean.fastq
**Explanation:** Reads input FASTQ and outputs only clean (non-chimeric) sequences in FASTQ format, preserving quality scores for downstream processing.

### Run on ITS region sequences using UNITE reference database
**Args:** --input its_sequences.fasta --reference_db unite.fasta --output its_chimeras.txt --minScore 0.9
**Explanation:** Detects chimeras in ITS fungal amplicon sequences using the UNITE database with a slightly relaxed threshold (0.9), accounting for higher intra-species variability in ITS regions.