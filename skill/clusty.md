---
name: clusty
category: Sequence Clustering / Bioinformatics
description: A tool for clustering biological sequences (DNA, RNA, or protein) based on sequence similarity. It groups similar sequences into clusters, typically retaining a representative sequence per cluster. Commonly used for dereplication, OTU picking in metagenomics, and reducing sequence redundancy.
tags: clustering, sequence-analysis, bioinformatics, fasta, fastq, dereplication, otu-picking
author: AI-generated
source_url: https://github.com/clusty/clusty
---

## Concepts

- **Input formats:** Clusty accepts FASTA and FASTQ files containing DNA, RNA, or protein sequences. Sequences must be linear (not circular) and contain standard IUPAC nucleotide or amino acid codes.
- **Clustering by similarity:** Sequences are grouped using an identity threshold (e.g., 97% identity means sequences with ≥97% identical positions are placed in the same cluster). The representative is typically the longest or first sequence in the cluster.
- **Output file types:** Clusty produces cluster mapping files (mapping sequence IDs to cluster IDs), representative sequence files (one sequence per cluster), and optionally abundance tables counting members per cluster.
- **Word size parameter:** Smaller word sizes increase sensitivity but slow performance; larger word sizes speed analysis but may miss distant matches. The default is typically 4-8 depending on sequence type.

## Pitfalls

- **Setting identity threshold too high:** Using 100% or near-100% identity on noisy data (like raw sequencing reads) will create thousands of singleton clusters with no meaningful grouping, defeating the purpose of clustering.
- **Mismatching sequence type:** Attempting to cluster nucleotide sequences using a protein-specific identity threshold produces nonsensical results because the scoring matrices differ (Nucleotides use match/mismatch; proteins use BLOSUM/PAM matrices).
- **Ignoring memory limits:** Clustering extremely large datasets (millions of sequences) without adjusting memory settings or using streaming options can cause out-of-memory crashes, especially with exact clustering algorithms.
- **Input file encoding issues:** Using gzip-compressed input without the `-z` flag, or providing files with Windows-style line endings (CRLF), causes parsing errors or silent failures where no sequences are processed.

## Examples

### Cluster protein sequences at 90% identity threshold
**Args:** `-i proteins.fasta -o clusters.txt -t 0.90 -T protein`
**Explanation:** This sets the identity threshold to 90% and specifies protein input, ensuring correct scoring matrices are applied for amino acid sequence comparison.

### Use smaller word size for sensitive clustering of similar sequences
**Args:** `-i reads.fasta -o clusters.txt -W 4`
**Explanation:** Reducing word size from default to 4 increases sensitivity for highly similar sequences, useful when clustering closely related variants that differ by few bases.

### Output only cluster representatives to a FASTA file
**Args:** `-i sequences.fasta -o clusters.txt --rep-fasta representatives.fasta -t 0.95`
**Explanation:** The `--rep-fasta` flag extracts one representative sequence per cluster (typically the longest) and writes them to a separate FASTA file for downstream analysis.

### Limit maximum cluster size to prevent large chimeras
**Args:** `-i sequences.fasta -o clusters.txt -t 0.97 --max-cluster-size 100`
**Explanation:** Setting a maximum cluster size prevents over-merging of distantly related sequences that could form chimeric clusters, common in metagenomic OTU picking.

### Enable verbose logging to troubleshoot clustering issues
**Args:** `-i sequences.fasta -o clusters.txt -t 0.90 -v -log clusty_log.txt`
**Explanation:** Verbose mode prints progress messages and timing information, which helps diagnose slow performance or unexpected behavior during large batch jobs.

### Cluster using 8 threads for faster processing
**Args:** `-i large_dataset.fasta -o clusters.txt -t 0.97 -threads 8`
**Explanation:** Multi-threaded execution significantly speeds up clustering of large FASTA files. Ensure the system has sufficient CPU cores before setting this value.

### Generate abundance table from paired FASTQ files
**Args:** `-i sample1.fasta -i sample2.fasta -o clusters.txt -t 0.97 -- abundance-table abundance.txt`
**Explanation:** When multiple input files are provided, clusty tracks which sequences came from which sample, enabling generation of a per-sample abundance matrix for comparative analysis.