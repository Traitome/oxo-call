---
name: bbmap
category: sequence-alignment
description: BBMap is a fast, memory-efficient short-read aligner using the Burrows-Wheeler Transform to map sequencing reads to a reference genome or set of references, producing output in SAM, BAM, or other formats.
tags: [read-mapping, short-reads, bwt, fastq, sam-bam]
author: AI-generated
source_url: https://sourceforge.net/projects/bbmap/
---

## Concepts

- BBMap uses a **kmer-based seed-and-extend alignment algorithm** built on the Burrows-Wheeler Transform (BWT). During mapping, it seeds hits using minimizers and then extends alignments with an approximate Smith-Waterman algorithm, making it both fast for reads with many exact matches and sensitive for divergent sequences.
- The tool accepts **paired-end (PE) and single-end (SE) reads** from multiple I/O formats: FASTQ, FASTA, QSEQ, SAM, BAM, and their gzip/bzip2-compressed equivalents. For paired reads, you must specify read layout using `interleaved`, `in`, and `in2` parameters, or use the `singleton` flag to handle singletons rescued from PE libraries.
- BBMap outputs results by default to **stdout in SAM format**, which can be redirected to a file. Alternatively, use the `out=` parameter to write directly to a file, or omit it to let the shell handle redirection. Output can also be piped directly into companion tools like `sort`, `pileup`, or `callvariants.sh`.
- **Ambiguous base resolution** (`ambig=`) controls how BBMap handles reads mapping equally to multiple loci. Options include `random` (assign to one randomly), `all` (output one alignment per locus), `toss` (discard the read), and `unique` (assign only if one best hit exists). Misconfiguring this is a common source of false uniqueness in downstream analysis.
- **Quality trimming and filtering** are applied pre-mapping via `qtrim`, `trimq`, `minq`, and `maq` parameters. BBMap uses the **Phred+33 or Phred+64** quality encoding scheme automatically detected from the input, but you can override it with the `qin` parameter (33, 64, or auto).

## Pitfalls

- Specifying `in=` with a single file that contains paired-end reads but omitting `interleaved=true` or `in2=` causes BBMap to treat the file as single-end, resulting in **every read being analyzed as a singleton** and losing the pairing information needed for correct library-size estimation and discordant read detection.
- Setting `k=1` or a very small kmer size dramatically **increases memory usage and runtime** because the BWT index produces an enormous number of seed hits per read, causing the extend phase to dominate runtime and potentially run out of memory on large genomes.
- Using `qin=64` for modern Illumina data (which uses Phred+33 encoding since Illumina 1.8+) causes all quality scores to be **interpreted as 31 points lower than intended**, resulting in aggressive trimming or discard of high-quality bases, severely reducing mapping efficiency.
- Redirecting SAM output with `> output.sam` while BBMap also writes stats to **stderr** causes interleaved progress messages and metrics to contaminate the SAM file, corrupting downstream tools like `samtools view` or `GATK`.
- Setting `ambiguous=all` without understanding downstream tools' expectations produces **multiple alignments per read**, which causes `samtools sort` to fail or duplicate reads in variant-calling pipelines, and inflates the total number of mapped reads beyond the actual read count.

## Examples

### Map single-end FASTQ reads to a reference genome indexed on the fly

**Args:** `ref=hg38.fa in=sample_R1.fastq.gz out=mapped_SE.sam k=13 minid=0.9`
**Explanation:** BBMap builds its BWT index in memory (or on disk with `usejni`) and maps the FASTQ reads with a minimum identity of 90%, using a kmer size of 13 for seeding, which is a standard default for short Illumina reads.

### Map paired-end interleaved reads from a single FASTQ file

**Args:** `ref=ref.fa in=pe_interleaved.fastq.gz interleaved=true out=pe_mapped.sam pairedonly=t`
**Explanation:** The `interleaved=true` flag tells BBMap the file contains read1/read2 pairs in alternating records, and `pairedonly=t` discards any unpaired reads so the output contains only properly paired alignments.

### Map paired-end reads from two separate files (read1 and read2)

**Args:** `ref=ecoli.fasta in1=R1.fastq.gz in2=R2.fastq.gz out=pe_mapped.sam maxindel=100`
**Explanation:** `in1` and `in2` specify the forward and reverse read files, and `maxindel=100` limits insertion/deletion events to 100 bp to reduce spurious alignments in bacterial genomes where large indels are less biologically common.

### Align reads with aggressive quality trimming and write statistics to a file

**Args:** `ref=genome.fa in=reads.fastq.gz out=trimmed_mapped.sam qtrim=rl trimq=10 maq=10 stats=alignment_stats.txt`
**Explanation:** `qtrim=rl` trims low-quality bases from both ends, `trimq=10` and `maq=10` apply a Phred-10 quality threshold, and `stats=` writes mapping statistics to a tab-delimited file for quality assessment of the mapping run.

### Map reads and save unmapped reads to a separate file for downstream recovery

**Args:** `ref=assembly.fa in=reads.fastq.gz out=mapped.sam outu=unmapped.fastq.gz`
**Explanation:** The `out=` parameter writes all alignments to the SAM file, while `outu=` saves reads that failed to map to a FASTQ file, enabling you to investigate why certain reads did not align or attempt de novo assembly on the unmapped fraction.