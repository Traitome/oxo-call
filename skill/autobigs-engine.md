---
name: autobigs-engine
category: Genomics / Sequence Analysis
description: A bioinformatics engine for automated identification, extraction, and analysis of biologically significant genomic regions (BIGS) from DNA/RNA sequencing data. Supports batch processing of FASTA/FASTQ files, annotation, and export to standard genomic formats.
tags: [genomics, sequence-analysis, BIGS, annotation, bioinformatics]
author: AI-generated
source_url: https://github.com/oxo-call/autobigs-engine
---

## Concepts

- **BIGS Identification**: The tool identifies biologically significant genomic segments by scanning input sequences against built-in motif databases and custom pattern definitions, outputting genomic coordinates in BED/GFF format.
- **Input Formats**: Accepts single or multi-entry FASTA files for reference sequences, FASTQ files for read analysis, and FASTQ-to-FASTA converted reads; supports gzipped (.gz) and uncompressed inputs via standard input redirection.
- **Output Formats**: Generates results in BED (default), GFF3, and CSV formats; the --outfmt flag controls output format, while --out prefix sets the base filename for multi-file exports.
- **Scoring System**: Each identified BIGS region receives a confidence score (0-100) based on motif match strength, flanking sequence conservation, and taxonomic conservation; regions below the --threshold score are excluded from output.
- **Batch Processing**: The --batch and --batch-list flags enable processing of multiple sample directories in parallel, with automatic sample naming from directory basenames.

## Pitfalls

- **Missing Sequence Index**: Running autobigs-engine without generating a sequence index using companion binary 'autobigs-engine-index' causes the tool to re-scan sequences for every query, increasing runtime by 10-50x on large genomes.
- **Threshold Too High**: Setting --threshold above 85 excludes biologically valid but weakly conserved regions, potentially missing regulatory elements or species-specific adaptations present in your input data.
- **Format Mismatch**: Specifying --outfmt gff3 when writing to stdout without redirecting to a file may produce malformed output in some pipelines because GFF3 requires tab-delimited structure that differs from BED.
- **Memory Limits on Large Genomes**: Genomes larger than 4GB require --memory-limit increase (default 8GB); otherwise, the tool crashes with segmentation fault during index loading.
- **Case Sensitivity in Patterns**: Custom motif patterns are case-sensitive by default; using mixed-case patterns like 'ATGCat' will not match lowercase sequences in your input, leading to missing detections.

## Examples

### Identify BIGS regions in a bacterial genome FASTA file
**Args:** --input genome.fasta --output results --threshold 70
**Explanation:** Scans the bacterial genome for all biologically significant regions with confidence score ≥70, saving results to results.bed in BED format.

### Analyze FASTQ reads for BIGS motifs and export CSV
**Args:** --input reads.fastq --outfmt csv --min-length 100 --output motifs
**Explanation:** Analyzes FASTQ reads for BIGS motifs, exports findings to motifs.csv with read names, coordinates, and scores for downstream RNA-seq analysis.

### Process batch of viral genomes with custom motif database
**Args:** --batch /data/viral-genomes/ --motif-db custom_motifs.db --threshold 60 --outfmt gff3
**Explanation:** Processes all FASTA files in the /data/viral-genomes/ directory against a custom motif database, outputs GFF3 annotations for each virus.

### Extract specific BIGS region by genomic coordinates
**Args:** --extract chr1:100000-200000 --reference ref.fa --output region_extract --format fasta
**Explanation:** Extracts the defined genomic region from the reference sequence and saves as a FASTA file for downstream primer design.

### Run with parallel processing on multi-core system
**Args:** --input large_genome.fa --threads 16 --output parallel_results --buffer-size 2G
**Explanation:** Utilizes 16 CPU threads and 2GB buffer for efficient processing of a large genome file, reducing wall-clock time on multi-core workstations.

### Disable score filtering to get all detections
**Args:** --input sequence.fa --threshold 0 --output all_detections --outfmt bed
**Explanation:** Disables confidence threshold filtering to report every detected motif regardless of score, useful for exploratory analysis.

### Custom scoring weights for regulatory elements
**Args:** --input enhancer_regions.fa --scoring-weights motif=0.7 conservation=0.2 flank=0.1 --output weighted
**Explanation:** Applies custom scoring weights prioritizing motif match strength over evolutionary conservation for regulatory element detection.