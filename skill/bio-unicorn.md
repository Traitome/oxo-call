---
name: bio-unicorn
category: sequence-analysis
description: A versatile bioinformatics tool for read alignment, variant detection, and sequence manipulation. Supports FASTQ, FASTA, BAM, VCF, and custom formats. Designed for high-throughput genomic data processing with parallel execution capabilities.
tags:
  - sequencing
  - alignment
  - variant-calling
  - genomics
  - read-processing
author: AI-generated
source_url: https://github.com/bio-unicorn/docs
---

## Concepts

- **Input formats**: bio-unicorn accepts FASTQ, FASTA, BAM, and VCF files as primary inputs. Paired-end reads should be specified with two separate files using `--read1` and `--read2` flags, while single-end reads use `--input`. The tool auto-detects compression (gz, bz2) based on file extensions.
- **Output modes**: Results are emitted in configurable formats via `--output-format` (bam, vcf, bed, json). Default output is BAM for alignments and VCF for variants. Use `--output-prefix` to specify base names; the tool appends appropriate extensions automatically.
- **Alignment model**: Uses a hybrid seed-and-extend algorithm with configurable seed length (`--seed-length`, default 20). Mitochondrial and nuclear genomes require separate index files. The tool maintains separate的家系 (family) indices for rapid switching between reference assemblies.
- **Parallel execution**:bio-unicorn spawns worker threads based on `--threads` (defaults to CPU count - 1). Large files benefit from chunked processing via `--chunk-size`, which splits input into manageable segments for memory-efficient handling.

## Pitfalls

- **Mismatched reference indices**: Providing an index built from a different genome version than your input reads causes silent misalignments. Always rebuild indices when updating reference assemblies; verify with `--check-index` before processing.
- **Ignoring read orientation for paired-end data**: Specifying `--fr` (forward-reverse) or `--rf` (reverse-forward) incorrectly leads to false variant calls. Confirm library prep strandedness from your sequencing facility; default `--fr` assumes standard Illumina paired-end protocol.
- **Memory exhaustion with large files**: Setting `--threads` too high combined with `--chunk-size` exceeding available RAM triggers OOM kills. Monitor memory with `htop`; reduce thread count or chunk size when usage exceeds 80% available memory.
- **Overwriting output without confirmation**: Running with `--output-prefix` pointing to existing files silently overwrites them. Use `--no-overwrite` flag to enable safe mode; the tool then aborts if output files exist.

## Examples

### Align single-end reads to a reference genome

**Args:** `--reference hg38.fa --input reads.fq.gz --output-prefix aligned --output-format bam`

**Explanation:** Aligns single-end FASTQ reads to the hg38 reference genome and outputs aligned reads in BAM format with the prefix "aligned.bam".

### Perform paired-end alignment with specific orientation

**Args:** `--reference chr22.fa --read1 R1.fq.gz --read2 R2.fq.gz --orientation fr --output-prefix sample1 --threads 8`

**Explanation:** Aligns paired-end reads to chromosome 22 using forward-reverse orientation with 8 worker threads for parallel processing.

### Generate variant calls from aligned BAM

**Args:** `--input aligned.bam --variant-calling --reference hg38.fa --min-depth 10 --output-prefix variants`

**Explanation:** Calls variants from a previously aligned BAM file, requiring minimum 10x coverage depth, outputting VCF with "variants.vcf".

### Index a reference genome for repeated use

**Args:** index --reference GRCh38.fa --index-name GRCh38 --kmer-size 15`

**Explanation:** Creates a searchable index from the GRCh38 reference using 15-mer seeds for faster subsequent alignments.

### Convert BAM to FASTQ for downstream tools

**Args:** `--input aligned.bam --convert-to fastq --output-prefix extracted --read1-only`

**Explanation:** Extracts read sequences from a BAM file into FASTQ format, outputting only first-in-pair reads with prefix "extracted_R1.fq".

### Filter variants by quality and depth

**Args:** `--input raw.vcf --filter-quality 30 --filter-depth 20 --output-prefix filtered --output-format vcf`

**Explanation:** Filters input VCF to retain variants with minimum GQ score 30 and read depth 20, outputting filtered results to "filtered.vcf".

### Run alignment with custom seed parameters

**Args:** `--reference mm10.fa --input reads.fq --seed-length 18 --max-mismatches 2 --output-prefix aligned_seed18`

**Explanation:** Aligns using shorter 18bp seeds with tolerance for 2 mismatches, trading speed for sensitivity on divergent reads.