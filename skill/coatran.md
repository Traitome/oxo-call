---
name: coatran
category: sequencing/qc
description: Convert between common DNA/RNA sequencing formats and filter reads by quality, length, and adapters. Supports FASTA, FASTQ, SAM, BAM, and their compressed variants with automatic format detection.
tags:
  - fastq
  - fasta
  - sam
  - bam
  - quality-filter
  - adapter-trim
  - format-conversion
  - sequencing
  - read-filtering
author: AI-generated
source_url: https://github.com/galaxyproject/galaxy献
---

## Concepts

- **Format auto-detection**: coatran automatically detects the input format (FASTA, FASTQ, SAM, BAM) based on file magic bytes and extension, so `--input-format` is optional unless disambiguation is needed. Output format is inferred from the `--output` extension unless `--output-format` is explicitly given.
- **Paired-end and single-end modes**: In paired-end mode use `--input1` + `--input2` or `--input-interleaved` for a single interleaved file. The tool maintains sync between read pairs during filtering; if one read is discarded, its mate is also discarded by default to preserve proper pairing.
- **Quality encoding matters**: Sanger (Phred+33), Illumina 1.8+ (Phred+33), and Illumina 1.5 (Phred+64) encodings differ. The `--quality-encoding` flag must match the input or results will be interpreted as garbage. Use `--auto-detect-encoding` to let coatran guess the encoding on the first 1000 reads.
- **Filtering pipeline order**: Trimming is applied before length-based and quality-based filtering. Adapter sequences are stripped with `--strip-adapter` before quality cutoff. This ordering ensures trimmed bases contribute to quality metrics for downstream filters.

## Pitfalls

- **Omitting `--output` causes data overwrite**: Without an explicit `--output` flag, coatran writes to stdout if the format supports it. If stdout is redirected to the same file as input in a shell pipeline, the input is silently truncated before it is read.
- **Mismatched pair order silently discards mates**: When specifying `--input1` and `--input2`, both files must be sorted identically. If reads appear in different order between the two files, coatran may pair them incorrectly or discard mates, producing an invalid paired-end output file with no warning.
- **Ignoring exit codes from filtering**: Non-zero exit codes (e.g., exit 2) signal that reads were completely filtered out, leaving the output file empty. Scripts that do not check `coatran`'s exit code will propagate empty files downstream, corrupting later analyses.
- **Using `--quality-encoding illumina` on Sanger data**: Illumina 1.8+ files use Sanger encoding (Phred+33). Specifying `--quality-encoding illumina` on Sanger-encoded FASTQ causes all quality scores to be read 31 points too low, triggering excessive read discarding and unreliable filtering results.
- **Forgetting `--compatibility-mode` for legacy tools**: Some older aligners require specific SAM header lines or flag values. Without `--compatibility-mode`, coatran produces SAM 1.6 output which older tools like `MAQ` reject with a parse error.

## Examples

### Convert a gzip-compressed FASTQ to uncompressed BAM
**Args:** `convert --input reads.fastq.gz --output reads.bam`
**Explanation:** coatran detects the gzip compression from the `.gz` extension and the FASTQ format from content, then outputs BAM. The uncompressed BAM is written to `reads.bam`.

### Filter paired-end FASTQ files, keeping only high-quality reads above length 50
**Args:** `filter --input1 R1.fastq --input2 R2.fastq --min-length 50 --quality-cutoff 20 --output R1_filtered.fastq --output2 R2_filtered.fastq`
**Explanation:** Both read files are filtered in synchronized paired-end mode; any read pair where either read is shorter than 50 bases or has any base below Phred 20 is discarded, and matching output files are written.

### Trim adapter sequences from an interleaved paired-end FASTQ and convert to Sanger FASTQ
**Args:** `trim --input-interleaved pe.fastq --strip-adapter AGATCGGAAGAGC --output trimmed.fastq --output-format fastq`
**Explanation:** The interleaved paired-end file is processed read-by-read, removing the Illumina TruSeq adapter (AGATCGGAAGAGC) from either end of each read, then written as a standard FASTQ.

### Convert a Sanger FASTQ to Illumina 1.5 encoded FASTQ for a legacy sequencer
**Args:** `convert --input sample.fastq --quality-encoding sanger --target-quality-encoding illumina15 --output sample_illumina15.fastq`
**Explanation:** The source qualities are interpreted as Sanger (Phred+33), converted to Illumina 1.5 (Phred+64) encoding, and written to a new file compatible with older Illumina pipeline software.

### Generate a per-base quality summary report for a large BAM file
**Args:** `stats --input alignments.bam --report-quality-histogram --report-per-base --output qc_report.txt`
**Explanation:** coatran reads the BAM as-is (converting quality on the fly), computes a histogram of quality scores and per-position quality distributions, and writes the report to the specified text file.

### Convert and re-encode a BAM file to SAM with custom sort order
**Args:** `convert --input alignments.bam --output sorted.sam --output-format sam --sort-order queryname`
**Explanation:** The BAM is decompressed and converted to SAM format, with reads sorted by query name rather than the default coordinate order, enabling compatibility with queryname-aware downstream tools.

### Batch-convert multiple FASTQ files using a prefix and output directory
**Args:** `convert --input sample1.fastq sample2.fastq --output-dir ./converted --output-prefix trimmed_`
**Explanation:** Both input files are converted individually, prefixed with `trimmed_`, and written to the `./converted` directory, preserving original filenames with the new prefix prepended.

### Filter reads by a custom sequence pattern, discarding matching reads
**Args:** `filter --input reads.fastq --discard-containing NNNNN --quality-cutoff 10 --output clean.fastq`
**Explanation:** Any read containing five or more consecutive N bases or with any base quality below Phred 10 is discarded. The remaining high-confidence reads are written to `clean.fastq`.

### Interleave two single-end FASTQ files into one paired-end interleaved file
**Args:** `convert --input1 R1.fastq --input2 R2.fastq --output interleaved.fastq --output-mode interleaved`
**Explanation:** Two coordinate-matched single-end files are merged into a single interleaved paired-end FASTQ file, where each record contains one read from each input file alternating by line.