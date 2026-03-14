---
name: hisat2
category: alignment
description: Graph-based alignment of short reads to a reference genome; successor to TopHat2 for RNA-seq alignment
tags: [alignment, rna-seq, splicing, ngs, illumina, spliced-alignment, transcriptome]
author: oxo-call built-in
source_url: "https://daehwankimlab.github.io/hisat2/"
---

## Concepts

- HISAT2 requires a genome index built with 'hisat2-build genome.fa index_prefix'; the index is a set of files with the given prefix.
- For RNA-seq, use the splice-site and exon-aware index (hisat2-build with --ss and --exon files from the GTF) for better splicing.
- HISAT2 outputs SAM to stdout by default — pipe to 'samtools view -b' or use -S/-o to write to a file.
- Use -p N for multi-threading; -x for index prefix; -1/-2 for paired-end reads; -U for single-end reads.
- The --dta flag (downstream transcriptome assembly) is recommended when using StringTie; --dta-cufflinks for Cufflinks.
- HISAT2 summary statistics are printed to stderr — capture with 2> align_summary.txt.
- Use --no-spliced-alignment for DNA alignment (genomic mode); default allows spliced alignments for RNA.

## Pitfalls

- HISAT2 outputs SAM to stdout — always pipe to samtools or redirect to a file to avoid filling your terminal.
- For RNA-seq downstream of StringTie, use --dta flag — without it, StringTie produces suboptimal transcripts.
- The -x index prefix must match the exact prefix used during hisat2-build (not the .fa file name).
- HISAT2 does not work well with very long reads (>300 bp) — use STAR or minimap2 for long reads.
- Paired-end read order matters: -1 must be R1, -2 must be R2; swapping them reduces mapping rate.
- Providing --rna-strandness (RF/FR) for strand-specific libraries improves quantification accuracy.

## Examples

### build a HISAT2 genome index from a reference FASTA
**Args:** `-p 8 genome.fa genome_index`
**Explanation:** builds index with prefix genome_index using 8 threads; creates genome_index.*.ht2 files

### align paired-end RNA-seq reads to the genome with 8 threads
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --dta -S aligned.sam`
**Explanation:** --dta optimizes for StringTie; -x index prefix; -1/-2 paired-end reads; -S output SAM file

### align paired-end RNA-seq reads and output sorted BAM directly
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --dta | samtools sort -@ 4 -o sorted.bam`
**Explanation:** pipe HISAT2 output to samtools sort to produce a sorted BAM without intermediate SAM file

### align strand-specific paired-end RNA-seq (reverse-strand library)
**Args:** `-p 8 -x genome_index -1 R1.fastq.gz -2 R2.fastq.gz --rna-strandness RF --dta -S aligned.sam`
**Explanation:** --rna-strandness RF for dUTP/reverse-strand libraries (most modern RNA-seq); use FR for forward-strand

### align single-end RNA-seq reads
**Args:** `-p 4 -x genome_index -U reads.fastq.gz --dta -S aligned.sam`
**Explanation:** -U for single-end reads; same index and --dta flag apply

### build splice-site aware index using GTF annotation for improved RNA-seq
**Args:** `-p 8 genome.fa genome_spliceaware_index --ss splice_sites.txt --exon exons.txt`
**Explanation:** --ss and --exon files extracted from GTF with hisat2_extract_splice_sites.py and hisat2_extract_exons.py
