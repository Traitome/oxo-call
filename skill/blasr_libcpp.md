---
name: blasr_libcpp
category: alignment/read-mapping
description: A high-performance read aligner for PacBio SMRT sequencing data that maps long reads to a reference genome with support for SAM, BAM, and MAF output formats. Designed for long-read genomics applications requiring sensitive alignment of noisy sequences.
tags: [bioinformatics, alignment, pacbio, smrt, long-reads, genomics, read-mapping]
author: AI-generated
source_url: https://github.com/PacificBiosciences/blasr
---

## Concepts

- **Input Read Formats**: blasr_libcpp accepts multiple PacBio read formats including FASTA, FASTQ, SAM, BAM, and H5 bas files. The tool automatically detects format from file extensions; use `--format` flag to explicitly specify when extension is ambiguous.

- **Alignment Algorithm**: Uses a modified suffix array (MSA) based algorithm optimized for long, error-prone reads typical of SMRT sequencing. The tool performs local alignment by default, computing optimal mapping for each read independently without requiring a seed-and-extend approach.

- **Output Format Options**: Supports SAM (default text), BAM (binary compressed), MAF (multiple alignment format), and pistol (PacBio-specific JSON format). Use `--out` flag to specify output filename; omit for stdout; use `--sam` flag for SAM header generation with reference information.

- **Scoring Parameters**: Default scoring uses match=1, mismatch=-2, gapOpen=-3, gapExtend=-1. These can be tuned using `--match`, `--mismatch`, `--gap`, and `--gapExtend` flags. For higher accuracy with noisy PacBio reads, consider reducing mismatch penalty relative to gap penalties.

## Pitfalls

- **Forgetting Reference Index**: Running blasr_libcpp without a pre-built reference index causes the tool to build one in-memory each run, significantly slowing repeated alignments. For multiple alignments against the same reference, first build an index using `blasr_libcpp-build` or the tool will auto-generate a temporary index file.

- **Mismatching Read and Reference Format Extensions**: Providing a FASTA reference with .fa extension while reads are in FASTQ may cause silent errors or incorrect quality handling. Always explicitly verify format with `--format` flags when working with non-standard extensions.

- **Excessive Thread Usage with I/O-Bound Tasks**: Over-parallelizing with `-nthreads` when reads are stored on network storage or spinning disks causes I/O contention, making alignments slower than with fewer threads. Start with 4-8 threads and adjust based on actual compute resources.

- **Ignoring Read Quality Scores**: When input FASTQ files contain quality scores but alignment treats them uniformly, sensitivity drops. Use `--quality` flag to enable quality-aware alignment scoring that penalizes mismatches in low-quality bases more heavily.

## Examples

### Align PacBio reads from a FASTA file to a reference genome
**Args:** `reads.fasta ref.fasta --out alignments.sam --nproc 8`
**Explanation:** Maps long reads from a FASTA file to a reference using 8 parallel threads, outputting results in SAM format for downstream variant calling or analysis tools.

### Map FASTQ reads with quality-aware scoring enabled
**Args:** `reads.fastq ref.fasta --out alignments.sam --quality`
**Explanation:** Enables quality-aware alignment that uses per-base quality scores to weight alignments, improving accuracy for noisy SMRT reads by penalizing mismatches in low-quality regions more heavily.

### Generate alignments in BAM format with sorted output
**Args:** `reads.fasta ref.fasta --out sorted.bam --bam --sort`
**Explanation:**Outputs alignments in binary BAM format rather than text SAM, with records sorted by genomic coordinate for efficient genome viewer visualization and indexing.

### Tune gap penalties for structural variation detection
**Args:** `reads.fasta ref.fasta --out alignments.sam --gapOpen -5 --gapExtend -2`
**Explanation:** Increases gap penalties to reduce alignment artifacts around insertions and deletions, useful when searching for structural variants where gap presence is biologically meaningful.

### Run alignment with custom scoring matrix for high-identity regions
**Args:** `reads.fasta ref.fasta --out alignments.sam --match 2 --mismatch -4`
**Explanation:** Increases match reward and mismatch penalty to favor high-identity alignments, reducing spurious alignments in repetitive or low-complexity regions of the genome.