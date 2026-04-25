---
name: cutadapt
category: qc
description: Finds and removes adapter sequences, primers, poly-A tails and other unwanted sequences from sequencing reads
tags: [trimming, adapter, quality-control, fastq, ngs, illumina, single-cell, linked-adapters, demultiplexing]
author: oxo-call built-in
source_url: "https://cutadapt.readthedocs.io/"
---

## Concepts

- cutadapt removes known adapter sequences; use -a for 3' adapter (read 1), -A for 3' adapter of read 2 in paired-end mode.
- For paired-end reads, use -o for output read 1 and -p for output read 2; both input files come as positional args at the end.
- Quality trimming is done with -q: -q 20 trims bases below Phred 20 from the 3' end; -q 20,20 trims both ends.
- Use --minimum-length (-m) to discard reads shorter than N bases after trimming.
- Use -j N for multi-core processing (0 = auto-detect all cores); gzipped output by specifying .gz output filenames.
- Common adapter sequences: Illumina universal: AGATCGGAAGAGC; Nextera: CTGTCTCTTATA; polyA: AAAAAAAAAAAAAAA.
- Use --discard-trimmed to remove any read where an adapter was found (useful for amplicon decontamination).
- Linked adapters (-a FWD...REV) remove both 5' and 3' adapters in one operation; use ... notation.
- --nextseq-trim performs NextSeq-specific quality trimming for dark cycles (high-quality G bases).
- --action controls what happens when adapter is found: trim (default), retain, mask (N), lowercase, crop, none.
- --rc / --revcomp checks both read and reverse complement for adapter matches.
- Anchoring (^ for 5', $ for 3') forces adapter to be at read ends only.
- -n / --times removes up to N adapters from each read (default 1).

## Pitfalls

- cutadapt has NO subcommands. ARGS starts directly with flags (e.g., -a, -A, -o, -p, -q) or with the adapter specification. Do NOT put a subcommand like 'trim' or 'remove' before flags.
- For paired-end, omitting -p (second output) processes only read 1 — both outputs must be specified for proper PE handling.
- Adapter sequences are case-insensitive but IUPAC ambiguity codes (N, R, Y) are supported in adapter sequences.
- Without --minimum-length, very short reads after trimming (1-2 bp) can be passed through causing issues downstream.
- cutadapt does NOT auto-detect adapters — you must specify the adapter sequence explicitly (unlike fastp).
- For linked adapters (5'+3'), use -a FRONT...BACK notation rather than specifying separately.
- The -u option unconditionally removes N bases from the 5' (-u N) or 3' end (-u -N) without adapter matching.
- --overlap (-O) default is 3; increase for more stringent matching to avoid random matches.
- --error-rate (-e) default is 0.1 (10%); adjust for more/less tolerance.
- --no-indels prevents indels in alignments (only mismatches), useful for amplicon data.
- --pair-filter controls which read must match filter criteria: any (default), both, first.

## Examples

### remove Illumina TruSeq adapters from paired-end reads
**Args:** `-a AGATCGGAAGAGCACACGTCTGAACTCCAGTCA -A AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT -o R1_trimmed.fastq.gz -p R2_trimmed.fastq.gz R1.fastq.gz R2.fastq.gz`
**Explanation:** cutadapt command; -a/-A specify 3' adapters for R1/R2; -o/-p specify output files; input files are positional arguments at the end

### trim adapters and quality-filter, discarding short reads
**Args:** `-a AGATCGGAAGAGC -A AGATCGGAAGAGC -q 20 --minimum-length 36 -j 8 -o R1_trimmed.fastq.gz -p R2_trimmed.fastq.gz R1.fastq.gz R2.fastq.gz`
**Explanation:** cutadapt command; -a/-A adapters for R1/R2; -q 20 quality trimming; --minimum-length 36 discards short reads; -j 8 uses 8 cores; -o/-p outputs; input files positional

### remove polyA tail from single-end RNA-seq reads
**Args:** `-a A{20} -q 20 --minimum-length 30 -j 4 -o trimmed.fastq.gz reads.fastq.gz`
**Explanation:** cutadapt command; -a A{20} matches polyA stretch of 20+ As; -q 20 quality trims; --minimum-length 30 discards short; -j 4 cores; -o output; useful for 3'-seq

### trim Nextera transposase adapters from paired-end ATAC-seq data
**Args:** `-a CTGTCTCTTATA -A CTGTCTCTTATA -q 20 --minimum-length 20 -j 8 -o R1_trimmed.fastq.gz -p R2_trimmed.fastq.gz R1.fastq.gz R2.fastq.gz`
**Explanation:** cutadapt command; -a/-A Nextera adapters; -q 20 quality trimming; --minimum-length 20 allows short ATAC fragments; -j 8 cores; -o/-p outputs; inputs positional

### remove 5' primer from single-end amplicon reads
**Args:** `-g ACACTGACGACATGGTTCTACA --discard-untrimmed -o trimmed.fastq.gz reads.fastq.gz`
**Explanation:** cutadapt command; -g specifies 5' adapter/primer; --discard-untrimmed removes reads without primer; -o output file; input positional; amplicon decontamination

### use linked adapters for amplicon with both 5' and 3' primers
**Args:** `-a ^FWDPRIMER...RCREVPRIMER -A ^REVPRIMER...RCFWDPRIMER --discard-untrimmed -o out1.fastq.gz -p out2.fastq.gz in1.fastq.gz in2.fastq.gz`
**Explanation:** cutadapt command; -a/-A linked adapters; ^ anchors to 5' end; ... notation links 5' and 3' adapters; --discard-untrimmed removes non-matching; -o/-p outputs

### NextSeq-specific quality trimming
**Args:** `-a AGATCGGAAGAGC --nextseq-trim 20 -o trimmed.fastq.gz reads.fastq.gz`
**Explanation:** cutadapt command; -a adapter sequence; --nextseq-trim 20 handles dark cycles (high-quality G bases) for NextSeq/NOVA-seq; -o output; input positional

### mask adapters with N instead of trimming
**Args:** `-a AGATCGGAAGAGC --action mask -o masked.fastq.gz reads.fastq.gz`
**Explanation:** cutadapt command; -a adapter; --action mask replaces adapter with N characters; -o output; input positional; preserves read length

### remove multiple adapters with multiple rounds
**Args:** `-g ^TTAAGGCC -g ^AAGCTTA -a TACGGACT -n 2 -o output.fastq input.fastq`
**Explanation:** cutadapt command; -g multiple 5' adapters; -a 3' adapter; -n 2 runs two rounds of adapter removal; -o output; input positional

### check reverse complement for adapter matches
**Args:** `-a AGATCGGAAGAGC --rc -o trimmed.fastq.gz reads.fastq.gz`
**Explanation:** cutadapt command; -a adapter; --rc checks both read and reverse complement; -o output; input positional; outputs RC if match found on RC

### demultiplex using inline barcodes
**Args:** `-g file:barcodes.fasta -o {name}.fastq.gz input.fastq.gz`
**Explanation:** cutadapt command; -g file:FASTA reads adapter sequences from file; -o {name} creates separate files per barcode; input positional

### anchored 5' adapter for strict primer matching
**Args:** `-g ^ACACTGACGACATGGTTCTACA -o trimmed.fastq.gz reads.fastq.gz`
**Explanation:** cutadapt command; -g 5' adapter; ^ anchors to 5' end for strict matching at start; -o output; input positional; prevents internal matches
