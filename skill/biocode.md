---
name: biocode
category: bioinformatics/sequence-analysis
description: Python library and CLI toolkit for biological sequence manipulation, annotation handling, and bioinformatics workflow automation. Provides tools for FASTA/FASTQ processing, sequence translation, k-mer counting, and GFF3 annotation management.
tags:
  - bioinformatics
  - sequence-analysis
  - fasta
  - fastq
  - gff3
  - genomics
  - python
  - ngs
author: AI-generated
source_url: https://github.com/jdidion/biocode
---

## Concepts

- Biocode automatically detects input format (FASTA vs FASTQ) based on file headers and sequence structure. Use explicit `--fasta` or `--fastq` flags when the automatic detection produces unexpected results, as mixed-format files will cause parsing failures.
- The `seq` subcommand provides sequence filtering, translation, and extraction operations. Sequences are identified by their FASTA defline ID (the string after the `>` symbol), which must be unique within a file for most operations to succeed correctly.
- Biocode uses 1-based inclusive coordinates for all sequence ranges, matching the biological convention used by GFF3 and other standard bioinformatics formats. Specifying coordinates in 0-based or exclusive format will produce off-by-one errors in extracted subsequences.
- K-mer counting and sequence similarity searches are performed using locality-sensitive hashing when input files exceed available RAM, which trades precision for memory efficiency. Results from large datasets should be interpreted as approximate rather than exact counts.
- The annotate subcommand handles GFF3 attribute parsing using semicolon-delimited key-value pairs and URL encoding for special characters, following the GFF3 specification exactly.

## Pitfalls

- Specifying invalid sequence IDs in filtering or extraction commands causes silent empty output rather than an error message, because biocode treats missing sequences as a valid (empty) result set. Always verify that at least one sequence was processed by checking the output file size or using `biocode seq count`.
- Attempting to translate sequences containing ambiguous nucleotide characters (N, R, Y, etc.) without the `--ambig` flag causes the translation to fail entirely for that sequence. Sequences with ambiguity codes must either be filtered out first or translated with ambiguous amino acid handling enabled.
- Memory exhaustion during k-mer counting operations on large FASTQ files (billions of reads) does not produce a graceful error; the process will be killed by the OS. Use the `--chunk-size` flag to limit memory usage at the cost of longer processing time.
- Mixing GFF3 and GenBank format annotations in the same workflow produces incompatible coordinate systems and attribute formats, leading to downstream parsing errors in tools that consume the output.
- Specifying strandedness incorrectly in annotation operations causes all features on the negative strand to be reported with reversed coordinates, which breaks genome browser visualization and cross-species comparison tools.

## Examples

### Extract subsequences from a specific genomic region
**Args:** `seq extract --input contigs.fasta --range chr1:1000000-2000000 --output region1.fasta`
**Explanation:** Extracts bases 1,000,000 through 2,000,000 from chromosome 1 using 1-based inclusive coordinates, writing only the overlapping region to a new FASTA file.

### Translate DNA sequences to protein with ambiguous codon handling
**Args:** `seq translate --input orfs.fasta --output proteins.faa --ambig --frame 1`
**Explanation:** Translates nucleotide sequences in reading frame 1 into amino acid sequences, preserving X for ambiguous codons that cannot be resolved to a single residue.

### Count 25-mers in a FASTQ file with memory-efficient streaming
**Args:** `seq kmer --input reads.fastq --k 25 --output kmers.txt --chunk-size 500`
**Explanation:** Counts all 25-nucleotide k-mers appearing in the FASTQ file, processing data in 500MB chunks to avoid memory exhaustion on large NGS datasets.

### Filter sequences by length and GC content
**Args:** `seq filter --input mixed.fasta --min-length 100 --min-gc 0.40 --max-gc 0.65 --output filtered.fasta`
**Explanation:** Retains only sequences that are at least 100 bases long with GC content between 40% and 65%, removing short fragments and extreme-composition sequences.

### Convert between FASTA and FASTQ format with quality encoding
**Args:** `seq convert --input sequences.sanger --format fastq --quality-offset 33 --output sequences.fastq`
**Explanation:** Converts Sanger-quality sequences to standard FASTQ format, explicitly specifying the Phred+33 quality score offset required for modern aligners.

### Extract gene annotations from a GFF3 file by feature type
**Args:** `annotate filter --input annotations.gff3 --type gene --output genes.gff3`
**Explanation:** Extracts only gene features from a GFF3 file, removing CDS, exon, and other sub-feature annotations to produce a simplified annotation file.