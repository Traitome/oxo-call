---
name: blaze2
category: Sequence Alignment & Mapping
description: A high-performance tool for aligning and mapping biological sequences against reference databases, supporting fast both pairwise and multiple sequence alignment with configurable scoring schemes.
tags: [sequence-alignment, genomics, bioinformatics, read-mapping, fastq, fasta]
author: AI-generated
source_url: https://github.com/lmduong/blaze2
---

## Concepts

- **Data model**: blaze2 aligns query sequences (reads or contigs) against a reference database (genome, transcriptome, or protein database) using an efficient seed-and-extend algorithm. Query input must be in FASTA or FASTQ format; reference sequences can be provided pre-indexed via `blaze2-build` or on-the-fly from a FASTA file.
- **Input/Output formats**: blaze2 accepts raw or compressed (`.gz`) FASTA/FASTQ files as queries and as references. Output alignment format is configurable: default is a tab-delimited text format with fields for query ID, subject ID, alignment coordinates, strand, CIGAR string, and alignment score. An optional JSON output mode (`--outfmt json`) provides machine-parseable results for downstream pipelines.
- **Indexing step**: Before aligning against large reference sequences, `blaze2-build` constructs aFM-index or hash-based index to accelerate seeding. This step is required once per reference and must be re-run if the reference file changes, otherwise alignment will silently produce incorrect results.
- **Scoring schemes**: blaze2 supports multiple nucleotide (simple match/mismatch) and amino-acid (BLOSUM/PAM substitution matrix) scoring modes via `--scoring`. Default is +1 match / -2 mismatch for nucleotides. Choosing the wrong scoring scheme for the data type (e.g., using amino-acid scoring on nucleotide reads) degrades alignment sensitivity and accuracy.
- **Paired-end read handling**: When two FASTQ files are supplied as query input (via `--read1` and `--read2`), blaze2 performs mate-pair rescue and fragment-length estimation. Specifying only one file in a paired-end experiment causes all read pairs to be processed independently, breaking mate-pair scoring.

## Pitfalls

- **Forgetting to index the reference**: Running blaze2 directly on a large unindexed reference FASTA without prior `blaze2-build` is extremely slow or may silently fall back to a brute-force mode, yielding sub-optimal alignments. Always run `blaze2-build reference.fasta` before aligning against genomes larger than 1 Mb.
- **Mismatched read encoding**: blaze2 strictly requires Sanger/Illumina 1.8+ quality encoding in FASTQ input. Solexa/Illumina 1.3-format files will be parsed incorrectly, causing read sequences to be silently garbled and alignment to fail or produce nonsensical CIGAR strings.
- **Specifying only one file of a paired-end pair**: Providing `--read1 reads_1.fastq.gz` without `--read2 reads_2.fastq.gz` discards mate-pair rescue, so the `-X` fragment-length and `-I` insert-size options become ineffective and all read pairs are aligned independently without proper pairing score bonuses.
- **Incompatible scoring mode**: Using `--scoring blosum62` on nucleotide FASTQ reads causes incorrect alignments because the matrix assumes amino-acid substitution patterns. The tool may still produce output but with severely reduced sensitivity, as many true matches are penalized incorrectly.
- **Compressed output without correct extension**: Passing `--outfile results.gz` does not automatically write gzip-compressed output; the flag `--compress` must be added. Without it, a `.gz` extension in the filename merely renames the output file without compressing it, and downstream tools expecting gzip may fail or consume the uncompressed file.

## Examples

### Align single-end FASTQ reads against an indexed reference
**Args:** `--read1 sample.fastq.gz --ref-index refgenome.blaze2 reference.fasta --outfile alignments.txt`
**Explanation:** This aligns all reads from `sample.fastq.gz` against the pre-built FM-index `refgenome.blaze2`, writing tab-delimited results to `alignments.txt`.

### Build an FM-index from a reference FASTA for faster subsequent alignments
**Args:** `--threads 8 --kmer-size 15 reference_genome.fasta --index output_base`
**Explanation:** `blaze2-build` constructs a 15-kmer FM-index of `reference_genome.fasta` using 8 threads, producing `output_base` as the index base name for use with `--ref-index` in alignment commands.

### Align paired-end reads with mate-pair rescue and insert size constraints
**Args:** `--read1 R1.fastq.gz --read2 R2.fastq.gz --ref-index ref.blaze2 --outfile paired_alignments.txt -X 500 -I 350 --min-insert 200 --max-insert 600`
**Explanation:** Both read files are supplied so blaze2 can rescue mate pairs, enforce a fragment length range of 200–600 bp, and apply pairing score bonuses in the output.

### Align FASTA contigs with amino-acid scoring against a protein database
**Args:** `--read1 protein_contigs.fasta --ref-index proteins.blaze2 --outfile protein_alignments.txt --scoring blosum62 --seqtype protein --outfmt json`
**Explanation:** With `--seqtype protein`, blaze2 switches to amino-acid mode and uses BLOSUM62 scoring to align `protein_contigs.fasta` against the protein FM-index, outputting machine-parseable JSON results.

### Limit output to high-scoring alignments only
**Args:** `--read1 query.fastq.gz --ref-index ref.blaze2 --outfile high_score.txt --min-score 50 --max-alignments 1 --outfmt tabular`
**Explanation:** The `--min-score 50` flag discards all alignments with a score below 50, and `--max-alignments 1` ensures only the top-scoring alignment per query is reported, reducing output file size for strict reporting pipelines.