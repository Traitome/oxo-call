---
name: biopet-basecounter
category: Sequence Analysis
description: A bioinformatics tool for counting nucleotide bases (A, T, G, C) in FASTQ or FASTA sequence files. Generates base composition statistics useful for quality control and downstream analysis.
tags: [nucleotide-counting, fastq, fasta, quality-control, base-composition, biopet]
author: AI-generated
source_url: https://github.com/biopet/biopet
---

## Concepts

- **Input formats**: Accepts standard FASTQ (.fq/.fastq) and FASTA (.fa/.fasta) files as input, either as single files or multiple files specified with glob patterns.
- **Output structure**: Produces a tabular output with columns for base counts (A, T, G, N), percentages, and optionally per-position statistics when used with positional flags.
- **Key flags**: The `-o`/`--output` flag specifies the output file path, `-s`/`--sample` adds a sample name column for multi-sample runs, and `-p`/`--position` enables per-position base counting.
- **Paired-end handling**: When processing paired-end FASTQ files, use the `--paired` flag to count each read pair as a single entry rather than counting R1 and R2 separately.

## Pitfalls

- **Missing input file**: Forgetting to specify an input file will cause the tool to hang waiting for stdin, which may appear as the tool has frozen rather than throwing an error.
- **Incorrect file encoding**: Using compressed files without the `-z` flag (if supported) will produce incorrect counts or parsing errors, as the tool expects plain text FASTQ/FASTA format.
- **Output permission errors**: Writing to a directory without write permissions will silently fail or create an empty output file; always verify write permissions before running.
- **Memory with large files**: Processing very large FASTQ files without the `--buffer-size` flag may cause memory issues; use streaming mode for files larger than available RAM.

## Examples

### Count bases in a single FASTQ file

**Args:** `input.fq -o bases.txt`
**Explanation:** Reads the FASTQ file and outputs nucleotide counts to the specified output file.

### Count bases with sample name labeling

**Args:** `sample1.fq -s sample1 -o sample1_bases.txt`
**Explanation:** Adds the sample name column to the output for easier tracking when combining multiple samples.

### Process multiple FASTQ files at once

**Args:** `*.fq -o all_samples.txt`
**Explanation:** Glob pattern expands to process all FASTQ files in the current directory and merge counts into single output.

### Enable per-position base counting

**Args:** `input.fq -p -o positional.txt`
**Explanation:** Generates base composition for each position in the reads, useful for detecting positional bias in sequencing.

### Use paired-end mode for PE sequencing

**Args:** `read1.fq read2.fq --paired -o paired_counts.txt`
**Explanation:** Processes paired-end files together, counting each read pair as one unit rather than two separate reads.