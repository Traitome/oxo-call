---
name: dorado
category: long-read
description: "Oxford Nanopore's official high-performance basecaller, demultiplexer, and duplex caller"
tags: [nanopore, basecalling, long-read, ont, gpu, demultiplexing, duplex, modbase, pod5, fast5, methylation]
author: oxo-call built-in
source_url: "https://github.com/nanoporetech/dorado"
---

## Concepts

- Dorado is Oxford Nanopore's official basecaller and replaces Guppy; it supports GPU and CPU basecalling.
- Main subcommands: dorado basecaller (basecall POD5/FAST5), dorado demux (demultiplex), dorado trim (adapter trim), dorado duplex, dorado aligner.
- Use 'dorado download --model <model>' to download basecalling models; models: fast, hac (high-accuracy), sup (super-accuracy).
- Basecalling: dorado basecaller <model_dir> <pod5_dir> outputs uBAM to stdout.
- Use --emit-fastq for FASTQ output instead of uBAM; --emit-sam for SAM output.
- Modified base (methylation) calling: add --modified-bases 5mCG or --modified-bases-models path to basecaller.
- Duplex calling (highest accuracy for paired reads): dorado duplex <model> <pod5_dir>.
- Dorado outputs to stdout — pipe to samtools or redirect to file.
- --resume-from resumes interrupted basecalling from a BAM file; output must use different filename.
- --min-qscore filters reads by mean quality score; use --min-qscore 10 for high-quality reads.
- --reference enables real-time alignment during basecalling using minimap2.
- --batchsize controls memory usage and throughput; larger batch sizes improve GPU utilization.
- POD5 format is preferred over FAST5 for performance; convert FAST5 to POD5 with pod5 tools.

## Pitfalls

- Dorado requires GPU for practical performance (slow on CPU); use -x cuda:0 to specify GPU device.
- POD5 format is preferred over FAST5 for Dorado; use pod5 tools to convert FAST5 to POD5 if needed.
- The model path must be to the downloaded model directory — download with 'dorado download --model hac'.
- Without --emit-fastq, Dorado outputs uBAM (unaligned BAM) — convert with samtools to FASTQ if needed.
- For duplex calling, both simplex and duplex reads are output — filter with --dx 1 tag in samtools.
- Dorado basecalling quality depends on the model: sup > hac > fast in accuracy but also in speed requirements.
- --resume-from requires different output filename than the resumed file; same filename loses existing basecalls.
- --batchsize and --chunksize affect memory usage; reducing chunksize is NOT recommended as it changes results.
- Demultiplexing should be done BEFORE trimming adapters/primers; trimming first may remove barcode flanking regions.
- Exit code 137 indicates OOM kill; reduce --batchsize or use smaller model.
- --reference alignment during basecalling requires sufficient CPU; CPU may become bottleneck on high-GPU systems.

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

### resume interrupted basecalling from existing BAM
**Args:** `basecaller hac pod5_files/ --resume-from incomplete.bam --device cuda:0 > complete.bam`
**Explanation:** --resume-from continues from incomplete.bam; complete.bam will contain all reads (use different filename)

### basecall with quality score filtering
**Args:** `basecaller sup pod5_files/ --min-qscore 10 --device cuda:0 > high_qual.bam`
**Explanation:** --min-qscore 10 filters out reads with mean Q-score < 10; reduces output to high-quality reads only

### basecall with real-time alignment to reference
**Args:** `basecaller hac pod5_files/ --reference genome.fa --device cuda:0 > aligned.bam`
**Explanation:** --reference enables alignment during basecalling using minimap2; outputs aligned BAM directly

### trim adapters from existing basecalled reads
**Args:** `trim calls.bam --sequencing-kit SQK-LSK114 --emit-fastq > trimmed.fastq`
**Explanation:** dorado trim removes adapters/primers post-basecalling; --sequencing-kit specifies kit for adapter sequences

### align existing basecalls to reference
**Args:** `aligner genome.fa calls.bam > aligned.bam`
**Explanation:** dorado aligner aligns basecalled reads to reference; uses minimap2 internally

### basecall with custom batch size for memory control
**Args:** `basecaller sup pod5_files/ --batchsize 64 --device cuda:0 > calls.bam`
**Explanation:** --batchsize 64 reduces memory usage; increase for better throughput on high-memory GPUs

### basecall with recursive directory scanning
**Args:** `basecaller hac data/ --recursive --device cuda:0 > calls.bam`
**Explanation:** --recursive finds all POD5/FAST5 files in data/ and subdirectories

### demultiplex without trimming barcodes
**Args:** `demux --kit-name SQK-NBD114-24 --no-trim --output-dir demux/ reads.bam`
**Explanation:** --no-trim preserves barcode sequences; use when you need to re-demultiplex later
