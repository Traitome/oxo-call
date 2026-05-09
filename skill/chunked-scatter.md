---
name: chunked-scatter
category: Distributed Genomics Processing
description: Splits large genomic datasets (BAM, VCF, FASTA, etc.) into independently processable chunks for parallelized scatter-gather workflows across multiple workers.
tags:
  - parallel-processing
  - genomics
  - chunking
  - scatter-gather
  - distributed
  - bam
  - vcf
  - fasta
  - data-splitting
author: AI-generated
source_url: https://github.com/上车-下车/chunked-scatter
---

## Concepts

- **Chunk boundary strategy**: By default, chunk boundaries are determined by genomic coordinate ranges (e.g., chromosomal regions), ensuring that genomic features like genes or variants are not arbitrarily split across chunk boundaries. Use the `--split-mode` flag to control whether boundaries are fixed-size (by base pairs) or feature-aware (by gene annotations).
- **Scatter output structure**: Each worker receives a manifest file (`scatter-manifest.json`) in the output directory that enumerates all chunk input paths and their corresponding output paths. This manifest drives the gather phase where results are merged back into a unified dataset.
- **Input format handling**: `chunked-scatter` auto-detects input file format (BAM, VCF, CRAM, FASTA, BED) based on file extension and magic bytes. Companion binaries like `chunked-scatter-build` must be run first to generate alignment-specific chunking metadata for indexed formats.
- **Worker communication**: Workers do not communicate with each other during the scatter phase; each operates on an independent chunk. The gather phase requires all workers to complete before merging can begin, and partial failures are reported via the manifest.
- **Reference-aware chunking**: When processing CRAM or aligned BAM files, `--reference` must be specified so that chunk boundaries can be validated against the reference sequence dictionary, preventing coordinate mismatches between chunks.

## Pitfalls

- **Chunk boundaries split genomic features**: When using `--split-mode fixed`, a variant or read that spans a chunk boundary will be assigned to only one chunk, causing underrepresentation in downstream analysis. Always prefer `--split-mode feature` for variant calling or alignment polishing tasks.
- **Missing `--reference` with CRAM input**: Supplying CRAM files without the `--reference` flag causes chunk boundaries to be resolved using the CRAM index alone, which can silently produce coordinate misalignment in the output chunks. Always pass the matching reference FASTA with `--reference`.
- **Output directory not empty**: Running `chunked-scatter` into a directory that already contains a `scatter-manifest.json` will silently overwrite the manifest, orphaning the previous chunk assignments and causing gather to fail or produce duplicated results.
- **Incorrect `--workers` count**: Setting `--workers` to a value larger than the number of genomic chromosomes produces empty chunks, wasting compute resources. The effective parallelism is bounded by the number of independently chunked regions.
- **Gather phase run before scatter completion**: Attempting to run `chunked-scatter-gather` while some workers are still processing causes partial merging and corrupts the final output. Always verify all chunks are complete via the manifest status field before invoking gather.

## Examples

### Split a BAM file into 8 chromosomal chunks for parallel alignment

**Args:** `input reads.bam --workers 8 --output ./chunks --split-mode feature`
**Explanation:** The tool splits reads.bam by chromosome (feature-aware mode) into 8 chunks written to ./chunks, producing a scatter-manifest.json that lists each chromosomal region independently.

### Chunk a VCF file using fixed base-pair intervals for parallel genotyping

**Args:** `input cohort.vcf.gz --output ./vcf-chunks --split-mode fixed --chunk-size 1000000 --workers 4`
**Explanation:** The VCF is divided into 1 Mb fixed-size intervals across 4 workers, enabling parallelized genotyping without regard to actual variant density.

### Process a CRAM file with an explicit reference

**Args:** `input alignments.cram --reference GRCh38.fa --output ./cram-chunks --workers 6`
**Explanation:** Supplying the reference FASTA ensures chunk boundaries are coordinate-verified against the CRAM sequence dictionary, preventing alignment drift during distributed processing.

### Initialize scatter metadata with the companion build binary before splitting

**Args:** `build --input sorted.bam --output ./metadata`
**Explanation:** The companion `chunked-scatter-build` binary generates alignment-specific chunking metadata (e.g., bin indices) that is required for accurate BAM scatter operations.

### Gather chunk outputs after parallel processing completes

**Args:** `gather --manifest ./chunks/scatter-manifest.json --output merged.bam`
**Explanation:** The gather binary reads the manifest, validates all chunks for completeness, and merges the per-chunk results into a single unified output file using the manifest's recorded output paths.

### Split a multi-sample VCF into per-sample chunks using a sample list

**Args:** `input multi-sample.vcf.gz --output ./sample-chunks --samples samples.txt --workers 3`
**Explanation:** The tool reads the sample list and isolates reads or variants belonging to the specified 3 samples into separate chunks, enabling parallelized per-sample joint calling.