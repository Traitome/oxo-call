---
name: blasr
category: alignment
description: A high-performance read mapping tool designed for aligning single-molecule real-time (SMRT) sequencing reads to a reference genome, optimized for long noisy reads from Pacific Biosciences platforms.
tags: alignment, long-reads, pacbio, smrt, genomics, read-mapping, bioinformatics
author: AI-generated
source_url: https://github.com/PacificBiosciences/blasr
---

## Concepts

- **Input data model**: blasr accepts query reads in FASTQ, FASTA, SAM, or BAM format and aligns them against a reference genome in FASTA format. The tool treats each query read independently, making it suitable for parallel execution across multiple cores.
- **Output formats**: Alignments can be output in several formats including SAM (standard alignment format), M4 (machine-friendly minimal format), M5 (MD5-based format), and XML. The SAM output format is recommended for compatibility with downstream tools like variant callers.
- **Scoring and alignment behavior**: blasr uses a match/mismatch scoring scheme with user-configurable penalties for insertions, deletions, and substitutions. The `-minScore` parameter ensures only alignments meeting a minimum quality threshold are reported, filtering out spurious mappings of low confidence.
- **Best n mapping**: The `-bestn` flag controls how many candidate alignments are reported per read. Setting this higher increases sensitivity but also computational cost; a value of 1 returns only the top alignment.
- **Threading and performance**: Multi-threading is controlled via the `-n` or `-numThreads` parameter, allowing parallel processing of multiple reads simultaneously for faster throughput on large datasets.

## Pitfalls

- **Using default parameters for short reads**: blasr was specifically designed for long reads (typically >1 kb) and performs poorly with short Illumina-style reads. The alignment algorithm assumes long read context; using default parameters with short reads produces many false positive alignments or fails to find valid mappings.
- **Forgetting to index the reference genome**: If the reference is passed directly without pre-indexing, blasr runs extremely slowly. The companion tool `blasr-build` must be used to create a suffix array index (`.sa` file) before running alignments; omitting this step causes runtime to scale linearly with reference size.
- **Setting minScore too low**: A `-minScore` value that is too permissive includes alignments with many errors, producing inaccurate results. Conversely, setting it too high causes valid alignments to be discarded, especially for noisy reads with lower overall sequence quality.
- **Ignoring read orientation**: SMRT reads can be generated in both forward and reverse complement orientations relative to the reference. By default, blasr searches both senses, but downstream tools may expect specific strand orientation; failing to account for this causes incorrect variant calls or downstream analysis failures.
- **Outputting non-SAM formats for incompatible tools**: Tools like GATK and freebayes expect SAM/BAM input. If alignments are output in M4 or XML format, additional conversion steps are required, adding complexity to the workflow and risking information loss during format changes.

## Examples

### Map a single FASTQ read file to a reference using default settings
**Args:** reads.fq -reference ref.fa -out aligned.sam -sam
**Explanation:** This basic command aligns all reads in FASTQ format to the reference genome and outputs results in SAM format, which is compatible with most downstream bioinformatics tools.

### Create a reference index before alignment
**Args:** ref.fa ref.sa
**Explanation:** The `blasr-build` companion binary creates a suffix array index file (`.sa`) from the reference FASTA, dramatically speeding up subsequent alignment runs.

### Align reads with a minimum alignment score threshold
**Args:** reads.fq -reference ref.fa -out aligned.sam -sam -minScore 200
**Explanation:** Setting `-minScore` to 200 filters out low-quality alignments with many mismatches or indels, keeping only mappings with sufficient similarity to be considered reliable.

### Report the top 3 alignments per read
**Args:** reads.fq -reference ref.fa -out aligned.sam -sam -bestn 3
**Explanation:** Using `-bestn 3` reports up to three candidate alignments per read, useful for applications requiring alternate mapping locations or for assessing alignment ambiguity.

### Use multiple threads for faster processing
**Args:** reads.fq -reference ref.fa -out aligned.sam -sam -n 8
**Explanation:** The `-n 8` parameter enables parallel processing across 8 threads, significantly reducing total runtime for large read datasets on multi-core systems.

### Output alignments in M4 format for machine parsing
**Args:** reads.fq -reference ref.fa -out aligned.m4 -m 4
**Explanation:** The M4 format provides a compact, machine-parseable output ideal for custom parsing scripts or integration into automatedpipelines without dealing with SAM headers.

### Align reads with increased gap penalty for longer gaps
**Args:** reads.fq -reference ref.fa -out aligned.sam -sam -penalties -indel 30
**Explanation:** Increasing the gap penalty to 30 causes the aligner to favor alignments with fewer and shorter indels, which can improve alignment precision for certain analysis types where insertions/deletions are less expected.

### Use FASTQ input with quality scores preserved in output
**Args:** reads.fq -reference ref.fa -out aligned.sam -sam -clip
**Explanation:** The `-clip` flag ensures that quality scores from the input FASTQ are preserved in the SAM output, allowing downstream tools to leverage per-base quality information for variant calling or quality control.