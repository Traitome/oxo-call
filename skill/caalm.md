---
name: caalm
category: Sequence Analysis / Consensus Assembly
description: Consensus and Alignment Tool for Large-scale Mapping - generates refined consensus sequences from BAM/SAM alignments with support for indel realignment, base quality recalibration, and multi-sample comparative analysis.
tags: [consensus, alignment, assembly, polishing, variant-calling, bam, sam]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/caalm
---

## Concepts

- **Input Format**: caalm accepts SAM (`.sam`) and BAM (`.bam`) alignment files as primary input. The tool automatically detects file format from the extension. For reference-based consensus, a FASTA reference sequence is required via the `--reference` flag.
- **Output Modes**: The tool produces consensus sequences in FASTA format by default. With `--pileup-out`, it generates per-position coverage and base frequency statistics. The `--vcf-out` flag emits variant calls when `--sample-vcf` mode is enabled.
- **Base Quality Recalibration**: The `--recalibrate` flag applies BQSR (Base Quality Score Recalibration) using known variant sites (VCF format). Without a known-sites VCF, recalibration uses only sequencing run statistics embedded in the alignment tags.
- **Indel Realignment**: The `--realign-indels` flag performs local realignment around indel positions using a banded Smith-Waterman algorithm. This modifies the alignment coordinates in the output BAM and may change consensus base calls at indel boundaries.
- **Multi-sample Mode**: When multiple BAM files are provided, `--multi-sample` computes sample-specific consensus and generates a combined pileup table with per-sample allele frequencies. Sample names are extracted from SM tags in read group headers.

## Pitfalls

- **Missing Read Groups**: Providing BAM files without SM or LB read group tags causes caalm to abort with "Read group missing required tag" error. Always annotate reads with read groups using `@RG` lines in the SAM header or `@PG` program records.
- **Mismatched Reference**: If the reference sequence does not match the alignment, consensus generation silently produces incorrect bases at mismatched positions. Verify reference compatibility with `samtools faidx` and `picard ValidateSamFile` before running caalm.
- **Uncompressed Output BAM**: Specifying `--output-bam` without `--compression-level` produces an uncompressed BAM that is 3-5x larger than necessary. Specify `--compression-level 5` or higher for typical workflows.
- **Memory Exhaustion on Large Files**: Genome-wide consensus on files >50GB with `--pooled-consensus` loads all pileup data into memory. Use `--chunk-size` to process genome regions in 10Mb segments to avoid OOM kills.
- **Incorrect Thread Count**: The `--threads` flag may be ignored if the compiled binary lacks threading support. Check availability with `caalm --version`; if threading is disabled, parallel processing via `parallel` or `GNU parallel` is required.

## Examples

### Generate consensus from single BAM file with default settings
**Args:** `alignments.bam --reference ref.fasta --output consensus.fasta`
**Explanation:** Produces a FASTA consensus sequence by majority-vote base calling at each reference position using all reads aligned to that position.

### Generate consensus with indel realignment enabled
**Args:** `sample.bam --reference grch38.fasta --realign-indels --output polished.fasta`
**Explanation:** Performs local realignment around indel positions before consensus calling, producing more accurate indel calls in the output sequence.

### Enable base quality recalibration with known variants
**Args:** `runs123.bam --recalibrate --known-sites dbsnp138.vcf.gz --reference hs37d5.fasta --output recal_consensus.fa`
**Explanation:** Recalibrates base quality scores using provided known variant sites, then generates a consensus sequence reflecting corrected quality information.

### Produce per-position pileup statistics for a genomic region
**Args:** `experiment.bam --pileup-out --region chr1:1000000-2000000 --reference ref.fa --output pileup.txt`
**Explanation:** Extracts reads overlapping the specified genomic interval and writes coverage depth and base composition per position for downstream visualization or QC.

### Multi-sample consensus with combined allele frequency table
**Args:** `cohort/*.bam --multi-sample --reference genome.fa --output cohort_results/`
**Explanation:** Processes all BAM files in the cohort directory, generates per-sample consensus sequences, and writes a combined table with per-sample allele frequencies at each variant site.

### Chunked processing to avoid memory exhaustion
**Args:** `large_alignments.bam --reference ref.fasta --chunk-size 10000000 --output chunks/`
**Explanation:** Splits the reference into 10Mb windows and processes each independently, writing consensus chunks to separate FASTA files in the output directory.

### Output compressed BAM with realignment
**Args:** `input.bam --realign-indels --output-bam --compression-level 9 --reference ref.fasta --output realigned.bam`
**Explanation:** Performs indel realignment and writes the modified alignment to a maximally compressed BAM file, suitable for archiving or downstream variant calling.

### Generate VCF of variant calls from consensus comparison
**Args:** `tumor.bam normal.bam --sample-vcf --reference ref.fa --min-af 0.05 --output variants.vcf`
**Explanation:** Compares tumor and normal consensus sequences and emits a VCF file containing positions where allele frequency differs between samples by at least 5%.

### Filter consensus to high-confidence bases only
**Args:** `align.bam --reference ref.fa --min-coverage 20 --min-baseq 30 --output highconf.fasta`
**Explanation:** Calls consensus only where coverage is at least 20 reads and base quality phred score is 30 or higher; positions failing thresholds are masked with `N`.

### Parallel processing across multiple threads
**Args:** `sample.bam --reference hg38.fasta --threads 8 --output cons.fa`
**Explanation:** Distributes consensus computation across 8 parallel threads, reducing wall-clock time on multi-core systems.