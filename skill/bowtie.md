---
name: bowtie
category: sequence_alignment
description: Ultrafast short-read aligner for DNA sequences using the Burrows-Wheeler Transform (BWT) and FM-index for memory-efficient alignment of short sequencing reads to a reference genome.
tags: [alignment, genomics, short-reads, BWT, SAM, Illumina, DNA]
author: AI-generated
source_url: https://bowtie-bio.github.io/bowtie/
---

## Concepts

- Bowtie uses a pre-built BWT index created by the companion tool `bowtie-build` to enable memory-efficient searches; the index must be built once before running alignments.
- Input reads must be in FASTQ or FASTA format, and output is written in SAM (Sequence Alignment/Map) format by default, enabling compatibility with downstream tools like SAMtools and Picard.
- The `-v` flag controls the maximum number of mismatches allowed (0-3), while `-k` specifies how many distinct alignments to report per read when multiple mapping positions exist.
- Bowtie supports both single-end and paired-end modes; in paired-end mode, `-1` and `-2` specify the forward and reverse read files, and `-X` sets the maximum insert size.
- The alignment algorithm prioritizes speed and memory efficiency by scanning reads against the compressed BWT index, making it suitable for large genomes like the human genome with limited RAM.

## Pitfalls

- Running `bowtie` without first building an index using `bowtie-build` will fail with an error stating the index file cannot be found; always create the index beforehand.
- Specifying `-v` with a value higher than 3 is not allowed, as bowtie only supports up to 3 mismatches; using larger values causes the tool to abort.
- Using inconsistent read files in paired-end mode—where read pairs are not in the same order or have different lengths—produces incorrect pair estimates or orphaned alignments.
- Not setting the `-m` flag when reads map to multiple locations causes bowtie to suppress all alignments for that read by default, potentially losing valid multi-mapping reads.
- Forgetting to specify the correct quality encoding (Phred+33 vs Phred+64) leads to misaligned reads, as the default auto-detection may be incorrect for older FASTQ files.

## Examples

### Align single-end reads to a reference index
**Args:** -x hg19_idx -U reads.fq -S output.sam
**Explanation:** Aligns single-end FASTQ reads against the pre-built human genome index named `hg19_idx` and outputs results in SAM format.

### Align paired-end reads with a maximum insert size
**Args:** -x hg19_idx -1 read1.fq -2 read2.fq -X 500 -S paired_output.sam
**Explanation:** Aligns paired-end reads where the maximum distance between read pairs (insert size) is set to 500bp, appropriate for Illumina paired-end libraries.

### Allow 2 mismatches per read alignment
**Args:** -x hg19_idx -U reads.fq -v 2 -S output.sam
**Explanation:** Permits up to 2 mismatches between the read and reference, increasing sensitivity but reducing alignment speed.

### Report up to 10 multiple alignments per read
**Args:** -x hg19_idx -U reads.fq -k 10 --best --strata -S output.sam
**Explanation:** Reports up to 10 distinct genomic locations for each read, with alignments sorted by best score and then stratum.

### Suppress unaligned reads from output
**Args:** -x hg19_idx -U reads.fq --un unaligned.fq -S output.sam
**Explanation:** Writes all unaligned reads to `unaligned.fq` for separate analysis while keeping only mapped reads in the SAM output.

### Use a specific quality encoding for older FASTQ files
**Args:** -x hg19_idx -U reads.fq --phred64 -S output.sam
**Explanation:** Forces bowtie to interpret quality scores as Phred+64 encoding (used by older Illumina pipelines), preventing misaligned reads.

### Run alignment with 8 threads for faster processing
**Args:** -x hg19_idx -U reads.fq -S output.sam -p 8
**Explanation:** Uses 8 parallel threads to accelerate alignment, significantly reducing runtime on multi-core systems.