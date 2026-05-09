---
name: amiga
category: Genome Analysis
description: A bioinformatics tool for rapid alignment and analysis of genomic sequences. Supports FASTA and FASTQ input formats, performs local and global sequence alignment, and generates aligned output with quality scores.
tags: [sequence-analysis, alignment, genomics, fastq, fasta]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/amiga
---

## Concepts

- **Input Formats**: amiga accepts FASTA (`.fasta`, `.fa`) and FASTQ (`.fastq`, `.fq`) input files. Multi-sequence files are processed sequentially, with each sequence treated as an independent query.
- **Alignment Modes**: The tool supports local alignment (`--local`) and global alignment (`--global`) algorithms, determined by the `-m` flag. Local alignment allows partial sequence matches while global alignment requires full-length alignment.
- **Output Handling**: Results are written to stdout by default or to a specified output file via `-o`. Output format is either SAM-like text or a custom alignment format depending on the `--outfmt` selection.
- **Scoring Parameters**: Default scoring uses a match score of +2, mismatch penalty of -3, and gap penalty of -5. These can be customized using `--match`, `--mismatch`, and `--gap` flags respectively.
- **Threading**: Multi-threaded execution is supported via `-t` for parallel processing of multiple query sequences, significantly reducing runtime on multi-core systems.

## Pitfalls

- **Incorrect Input Encoding**: Supplying files with wrong file extensions or non-nucleotide characters causes immediate failure. The tool does not auto-detect format; use the `-f` flag explicitly.
- **Memory Exhaustion with Large Files**: Processing files larger than available RAM without specifying `--chunk-size` leads to out-of-memory errors. Always estimate memory requirements before running on whole-genome scale inputs.
- **Misaligned Parameters**: Using global alignment mode for highly divergent sequences produces poor alignments and may discard valid local matches. Always preview with a small subset before full runs.
- **Ignoring Quality Scores**: FASTQ files processed with default settings ignore quality information. Use `--quality-filter` to incorporate phred scores when alignments require quality-aware scoring.
- **Output File Overwrites**: The tool silently overwrites existing output files without warning. Use `-o` with a new filename or enable `--no-overwrite` to prevent data loss.

## Examples

### Align a single FASTQ sequence against a reference database
**Args:** `--query seq1.fq --reference ref.fasta --global -o aligned.sam`
**Explanation:** Performs global alignment of query sequence seq1.fq against reference database ref.fasta and writes SAM-formatted results to aligned.sam.

### Run local alignment with custom scoring
**Args:** `--query reads.fq --reference genome.fa --local --match 5 --mismatch -4 --gap -2 -o local_aln.txt`
**Explanation:** Uses custom scoring parameters (match +5, mismatch -4, gap -2) for local alignment, allowing partial matches with more lenient gap penalties.

### Process multiple sequences using multiple threads
**Args:** `--query all_reads.fastq --reference ref.fa --global -t 8 -o parallel_out.sam`
**Explanation:** Enables 8-thread parallel processing for faster alignment of multiple sequences against the reference, significantly reducing wall-clock time on multi-core machines.

### Filter alignments by minimum quality threshold
**Args:** `--query input.fq --reference ref.fa --global --quality-filter 20 -o filtered.sam`
**Explanation:** Only outputs alignments where the FASTQ phred quality score meets or exceeds 20, reducing false positives in downstream analysis.

### Specify chunk size for large file processing
**Args:** `--query large_dataset.fq --reference ref.fa --global --chunk-size 500MB -o large_output.sam`
**Explanation:** Processes input in 500 megabyte chunks to manage memory usage when working with datasets approaching or exceeding available RAM.

### Convert output to custom format
**Args:** `--query seq.fq --reference ref.fa --local --outfmt custom -o custom_aln.txt`
**Explanation:** Outputs alignment results in the custom text format instead of default SAM format, useful for integration with specific downstream tools.

### Preview alignment without writing full output
**Args:** `--query seq.fq --reference ref.fa --local --max-output 10`
**Explanation:** Limits output to the first 10 alignments for quick preview of alignment quality before running a full analysis, saving time during parameter optimization.