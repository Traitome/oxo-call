---
name: dorado
category: long-read
description: "Oxford Nanopore's official high-performance basecaller, demultiplexer, and duplex caller"
tags: [nanopore, basecalling, long-read, ont, gpu, demultiplexing, duplex, modbase]
author: oxo-call built-in
source_url: "https://github.com/nanoporetech/dorado"
---

## Concepts

- Dorado is Oxford Nanopore's official basecaller and replaces Guppy; it supports GPU and CPU basecalling.
- Main subcommands: dorado basecaller (basecall POD5/FAST5), dorado demux (demultiplex), dorado trim (adapter trim), dorado duplex.
- Use 'dorado download --model <model>' to download basecalling models; models: fast, hac (high-accuracy), sup (super-accuracy).
- Basecalling: dorado basecaller <model_dir> <pod5_dir> outputs uBAM to stdout.
- Use --emit-fastq for FASTQ output instead of uBAM; --emit-sam for SAM output.
- Modified base (methylation) calling: add --modified-bases 5mCG or --modified-bases-models path to basecaller.
- Duplex calling (highest accuracy for paired reads): dorado duplex <model> <pod5_dir>.
- Dorado outputs to stdout — pipe to samtools or redirect to file.

## Pitfalls

- Dorado requires GPU for practical performance (slow on CPU); use -x cuda:0 to specify GPU device.
- POD5 format is preferred over FAST5 for Dorado; use pod5 tools to convert FAST5 to POD5 if needed.
- The model path must be to the downloaded model directory — download with 'dorado download --model hac'.
- Without --emit-fastq, Dorado outputs uBAM (unaligned BAM) — convert with samtools to FASTQ if needed.
- For duplex calling, both simplex and duplex reads are output — filter with --dx 1 tag in samtools.
- Dorado basecalling quality depends on the model: sup > hac > fast in accuracy but also in speed requirements.

## Examples

### basecall ONT POD5 files with high-accuracy model on GPU
**Args:** `basecaller hac pod5_files/ --device cuda:0 > calls.bam`
**Explanation:** hac high-accuracy model; pod5_files/ directory of POD5 files; --device cuda:0 GPU; output uBAM

### basecall with super-accuracy model and modified base calling
**Args:** `basecaller sup pod5_files/ --modified-bases 5mCG_5hmCG --device cuda:0 > calls_mods.bam`
**Explanation:** sup model; --modified-bases detects 5mC and 5hmC methylation; requires compatible modified base model

### basecall and output FASTQ format
**Args:** `basecaller hac pod5_files/ --emit-fastq --device cuda:0 | gzip > basecalled_reads.fastq.gz`
**Explanation:** --emit-fastq outputs FASTQ instead of uBAM; pipe to gzip for compressed output

### demultiplex barcoded reads using Dorado
**Args:** `demux --kit-name SQK-NBD114-24 --output-dir demux_output/ --emit-fastq reads.bam`
**Explanation:** --kit-name specifies barcode kit; --emit-fastq outputs FASTQ files per barcode

### run duplex basecalling for highest accuracy paired reads
**Args:** `duplex sup pod5_files/ --device cuda:0 > duplex_calls.bam`
**Explanation:** duplex mode uses paired strands for highest accuracy; requires paired-strand pod5 files
