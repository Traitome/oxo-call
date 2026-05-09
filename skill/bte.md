---
name: bte
category: sequence-alignment
description: A fast Burrows-Wheeler transform-based aligner for mapping short sequencing reads to a reference genome. Optimized for Illumina data with support for paired-end reads, multiple hits reporting, and SNP-prone region detection.
tags: [reads, alignment, genomics, NGS, BWT, short-reads, DNA-seq]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/bte
---

## Concepts

- **Input Format**: Accepts FASTQ format for query reads (single or paired-end) and FASTA format for the reference genome. The reference must be pre-indexed using the companion `bte-build` binary before alignment.
- **Output Formats**: Supports SAM (default), BAM (with `-b` flag), and CRAM (with `--cram` flag). SAM output includes mandatory fields: QNAME, FLAG, RNAME, POS, MAPQ, CIGAR, RNEXT, PNEXT, TLEN, SEQ, QUAL.
- **Alignment Algorithm**: Uses a modified BWT-backtracking algorithm with configurable mismatch tolerance (`-v` flag sets maximum mismatches, default 0) and insertion/deletion gap opening penalties (`--gap-open` and `--gap-ext`).
- **Index Building**: The companion `bte-build` creates the FM-index from the reference FASTA. Memory usage scales approximately 2.5× the reference genome size. Use `-c` for color-space indexing and `--offrate` to control index offset rate.

## Pitfalls

- **Running without pre-built index**: Attempting alignment with `bte` using an unindexed reference produces a crash with a generic "index not found" error. Always run `bte-build reference.fasta index_name` before alignment.
- **Mismatched read group headers**: Failing to declare read group information (`@RG`) in paired-end alignments results in invalid SAM files that Picard ValidateSamFile rejects, causing downstream processing failures.
- **Excessive memory for large references**: Using default parameters with reference genomes >4 billion base pairs can cause out-of-memory errors. Reduce `-max-stack` or use chunked processing with `--split` for whole-genome alignment.
- **Incorrect paired-end fragment length**: Not specifying expected insert size (`-X` flag) when processing paired-end data leads to incorrect pairing decisions and potentially 10-30% reduced mapping quality.
- **Using obsolete index versions**: Reusing indices built with older `bte-build` versions after tool updates produces silent errors where reads map incorrectly. Rebuild indices after each tool version upgrade.

## Examples

### Build FM-index from a bacterial reference genome
**Args:** references/ecoli_k12.fasta -p ecoli_k12
**Explanation:** The `-p` flag specifies the base name for the resulting index files (ecoli_k12.*.bt2), which are required for subsequent alignment runs.

### Align single-end reads to the reference
**Args:** -x ecoli_k12 reads.fq -S align.sam
**Args:** -x ecoli_k12 -1 read1.fq -2 read2.fq -S paired.sam
**Explanation:** The `-x` flag specifies the index prefix, and the `-S` flag sets the output SAM filename. For paired-end data, both `-1` and `-2` are required.

### Allow up to 2 mismatches per read
**Args:** -x ecoli_k12 reads.fq -v 2 -S align_m2.sam
**Args:** -x ecoli_k12 reads.fq -v 1 -a -S align_a.sam
**Explanation:** The `-v` parameter sets maximum mismatches (0-3 range typical). Adding `-a` outputs all valid alignments rather than just the best one.

### Output in BAM format with multiple threads
**Args:** -x ecoli_k12 reads.fq -b -@ 8 -S align.bam
**Args:** -x ecoli_k12 reads.fq --cram -o output.cram
**Args:** -x ecoli_k12 reads.fq -f -o output.sam
**Explanation:** The `-b` flag outputs BAM (compressed binary), `--cram` outputs CRAM format, and `-f` forces overwrite without prompting. Use `-@` for threaded output compression.

### Specify paired-end fragment size range
**Args:** -x ecoli_k12 -1 read1.fq -2 read2.fq -X 300:500 -S paired.sam
**Explanation:** The `-X` flag sets the expected inner fragment size range (minimum:maximum). Reads with outer distances outside this range are flagged as discordant.

### Map color-space reads from SOLiD data
**Args:** -x ecoli_k12_col colorspace.fq -S colorspace_align.sam --col
**Explanation:** The `--col` flag indicates color-space input encoding. Requires a color-space index built with `bte-build -c` during indexing.

### Output only properly paired reads with mapping quality ≥30
**Args:** -x ecoli_k12 -1 read1.fq -2 read2.fq -f -q 30 -F 2 -S highqual_paired.sam
**Explanation:** The `-q` flag filters reads below minimum mapping quality, and `-F 2` (SAM flag 2) excludes unmapped reads. Combining these outputs high-confidence properly paired alignments.