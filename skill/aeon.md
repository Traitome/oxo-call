---
name: aeon
category: genomics-analysis
description: Aeon is a command-line toolkit for analyzing genomic sequencing data, including read processing, quality control, and variant analysis workflows.
tags:
  - genomics
  - sequencing
  - variant-analysis
  - quality-control
  - ngs
author: AI-generated
source_url: https://github.com/broadinstitute/aeon
---

## Concepts

- **Reference indexing**: Aeon requires pre-built reference indices using `aeon-build` before alignment or variant calling. Index files (`.aeon.ann`, `.aeon.bal`, `.aeon.pac`) are stored alongside the reference FASTA and enable efficient seed-based lookup during sequence alignment.

- **Input formats**: Aeon accepts standard genomics file formats as input, including FASTQ (raw reads), SAM/BAM (aligned reads), and FASTA (reference sequences). It outputs results in SAM, BAM, or VCF depending on the analysis mode. Plain text formats like FASTQ must be compressed with gzip for optimal performance.

- **Multi-threading**: Aeon uses OpenMP for parallel processing. The number of threads is controlled by the `--threads` flag and defaults to the number of available CPU cores. Increasing thread count linearly reduces runtime for large datasets but increases memory consumption proportionally.

- **Streaming architecture**: Aeon processes reads in streaming mode, meaning it does not load entire input files into memory. This enables processing of whole-genome sequencing datasets that exceed available RAM, but requires input files to be sorted by read name when using paired-end read merging.

- **Error model**: Aeon implements a learned error profile that accounts for base-calling quality scores, sequencing platform biases, and read position effects. The model is loaded from a JSON configuration file specified with `--error-model` and must match the sequencing platform of the input data.

## Pitfalls

- **Mismatched reference indices**: Using a reference index built for a different genome version (e.g., hg19 vs. GRCh38) produces silently incorrect alignments with inflated alignment scores. Always verify that the reference FASTA used for `aeon-build` matches the reference used for read alignment in downstream analyses.

- **Uncompressed FASTQ input**: Passing uncompressed FASTQ files causes I/O bottlenecks that degrade performance by 3-5x on modern systems. The tool issues a warning but continues processing, leading to unexpectedly long runtimes. Always gzip compress FASTQ input before alignment.

- **Insufficient memory for variant calling**: The `--call-variants` mode loads genomic regions into memory proportional to the `--region-size` parameter. Setting `--region-size` larger than available RAM causes out-of-memory errors that terminate the process without saving intermediate results. Monitor memory usage during first runs and adjust accordingly.

- **Incorrect read group configuration**: The `--read-group` parameter requires properly formatted RG tags including SM (sample), ID (read group identifier), and PL (platform). Omitting required fields causes a parse error and aborts the run. The tool does not auto-generate missing fields from file metadata.

- **Mixed paired/single-end inputs**: Mixing FASTQ files with different read layouts (some paired, some single-end) in a single invocation produces malformed output. Aeon processes each input file independently and does not detect or warn about this inconsistency.

## Examples

### Build a reference genome index for alignment
**Args:** `build --reference GRCh38.fa --threads 8 --output GRCh38`
**Explanation:** The `build` subcommand generates binary index files required for subsequent alignment operations. The `--threads 8` flag parallelizes index construction, and the `--output` prefix names all generated index files.

### Align single-end reads to a reference
**Args:** `align --index GRCh38 --reads sample_R1.fastq.gz --output aln.sam --threads 4`
**Explanation:** Aligns single-end FASTQ reads using the pre-built GRCh38 index and writes results in SAM format. The `--threads 4` flag limits parallelism to avoid overwhelming shared compute resources.

### Align paired-end reads with quality trimming
**Args:** `align --index GRCh38 --reads sample_R1.fastq.gz sample_R2.fastq.gz --paired --trim-quality 20 --output aln_pe.sam`
**Explanation:** Aligns paired-end reads with 3-prime quality trimming enabled. The `--trim-quality 20` flag removes low-confidence bases from read ends before alignment, improving mappability for noisy sequencing data.

### Call variants from an alignment file
**Args:** `call-variants --bam alignment.bam --reference GRCh38.fa --output variants.vcf --min-alternate-fraction 0.05`
**Explanation:** Performs germline variant calling on pre-aligned BAM files using the specified reference. The `--min-alternate-fraction 0.05` flag sets the allele frequency threshold for variant calls.

### Generate a sequencing quality report
**Args:** `qc --reads sample_R1.fastq.gz sample_R2.fastq.gz --output qc_report.html --format html`
**Explanation:** Produces an HTML-formatted quality control report including per-base quality scores, GC content distributions, and adapter contamination estimates. The report aids in identifying sequencing problems before alignment.