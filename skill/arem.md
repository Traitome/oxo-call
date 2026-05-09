---
name: arem
category: sequence_analysis
description: A bioinformatics tool for analyzing sequencing reads and detecting repetitive or low-complexity regions in genomic data. Supports FASTQ and FASTA input formats, with output for read classification and quality metrics.
tags:
  - genomics
  - read_analysis
  - sequence_filtering
  - quality_control
  - repetitive_elements
author: AI-generated
source_url: https://github.com/arem/arem
---

## Concepts

- **Input formats**: arem accepts FASTQ and FASTA files as primary input, supporting both single-end and paired-end read data. Files can be provided via stdin using Unix pipes, enabling integration with standard bioinformatics pipelines.
- **Output modes**: The tool produces aligned read classifications, repeat masks, and summary statistics in TSV or JSON format. Results include read-level annotations and aggregate reports about low-complexity regions.
- **Quality filtering**: Default thresholds filter reads based on average quality scores and length requirements. Users can adjust minimum quality (Phred score), minimum read length, and repeat density limits through command-line arguments.
- **Companion binary**: The companion tool `arem-build` constructs index databases from reference sequences for subsequent read classification. Index files must be built once and can be reused across multiple analysis runs.

## Pitfalls

- **Mismatched index versions**: Using an index built with a different version of arem-build than the arem binary causes alignment failures or silently incorrect results. Always rebuild indices when upgrading arem.
- **Uncompressed input assumed**: Passing gzip-compressed FASTQ files without the `-z` flag causes parse errors. Either decompress files beforehand with `gunzip -c` or use the built-in compression flag.
- **Insufficient disk space for temporary files**: Large input files require substantial temporary directory space (controlled by `--tmp-dir`). Running out of disk space corrupts output and leaves partial files.
- **Ignoring read orientation for paired-end data**: For paired-end libraries, specifying incorrect read orientation (`--fr` vs `--rf`) leads to misclassification of valid read pairs as failures.

## Examples

### Classify reads from a FASTQ file against a reference index

**Args:** `input.fq -x reference_index -o results.tsv`
**Explanation:** This runs read classification using the pre-built reference index, outputting results to the specified TSV file.

### Filter low-quality reads from a FASTA input

**Args:** `input.fa -q 20 -m 50 -o filtered.fa`
**Explanation:** This filters reads retaining only those with minimum Phred quality 20 and minimum length 50 bases.

### Process paired-end reads with forward-reverse orientation

**Args:** `read1.fq read2.fq -x paired_index --fr -o paired_results.json`
**Explanation:** This processes paired-end data assuming forward-reverse orientation and outputs results in JSON format.

### Use gzip-compressed input without decompression

**Args:** `sample.fq.gz -x index -z -o output.tsv`
**Explanation:** This directly reads gzip-compressed FASTQ input using the `-z` flag, avoiding manual decompression.

### Specify custom temporary directory for large files

**Args:** `large_input.fq -x index --tmp-dir /scratch/user -o output.tsv`
**Explanation:** This directs temporary file creation to the specified scratch directory with adequate disk space.