---
name: alevin-fry
category: Single-Cell RNA-Seq Quantification
description: A companion post-processing tool for alevic that filters, collates, and aggregates single-cell RNA-seq RAD quantification files into usable feature-barcode count matrices. Operates on alevic-output directories to generate filtered counts, resolve ambiguity, and produce output in matrix market (MTX), H5AD, or HDF5 formats.
tags: [scRNA-seq, single-cell, alevic, quantification, UMI, barcode-filtering, RAD-format, h5ad]
author: AI-generated
source_url: https://alevin-bioinfo.readthedocs.io/en/latest/alevin-fry.html
---

## Concepts

- alevin-fry operates on alevic-output directories containing RAD (Row Adaptive Data) files; it does not perform sequence alignment or UMI counting itself — those steps belong to the upstream alevic pipeline.
- The canonical workflow order is: generate-permit-list → collate → (optional) aggregate → (optional) prune. Skipping or reordering these steps will produce corrupt or missing output matrices.
- Permit lists define valid cell barcodes (from a known whitelist) or permit-type tags (CB=cell barcode, CR=cellular read); mismatching the permit list with the index chemistry used during alevic alignment is a common source of zero-row output.
- Output formats are controlled by `--matrix-format` (mtx, h5ad) and `--resolution` (cr-like, raw, spliced, ambiguous). Each combination yields a different feature-barcode matrix interpretation; `cr-like` is the standard UMI-filtered result.
- The `aggregate` subcommand merges multiple sample directories into a unified matrix using shared barcode metadata, enabling multi-sample differential expression downstream.

## Pitfalls

- Using the `--sketch` flag with large datasets without sufficient RAM causes silent memory exhaustion; the resulting MTX header lines may be truncated, making downstream tools like Scanpy throw cryptic H5AD parse errors.
- Mismatching the `--permit-list` file with the index chemistry (e.g., 3prime vs. 5prime droplet RNA) causes all barcodes to be rejected, yielding a matrix with zero rows and columns — this is easily mistaken for a failed sequencing run.
- Running `collate` before `generate-permit-list` completes or pointing `--input-dir` to a directory that is not a valid alevic output causes silent type-mismatch errors that produce a 0-byte output directory.
- Specifying `--resolution ambiguous` without a subsequently paired `prune` step creates a matrix containing ambiguous-assignment rows that inflate doublet detection rates in tools like Souporcell.
- Passing a permit list that contains non-unique barcodes (duplicate entries) to `generate-permit-list` silently discards duplicates beyond the first occurrence, reducing the expected cell count without raising an error.

## Examples

### Generate a filtered permit list from a 10x Chromium barcode whitelist
**Args:** `generate-permit-list --permit-list t2g.tsv --feature-technology 10x --input-dir alevic_output --output-dir fry_output`
**Explanation:** The permit list is generated from a tag-to-gene mapping file paired with the 10x droplet feature technology, which restricts valid cellular barcodes to those present in the whitelist, ensuring only real cells survive subsequent collate steps.

### Collate alevic RAD output into a UMI-filtered count matrix in MTX format
**Args:** `collate --input-dir alevic_output --output-dir fry_mtx --resolution cr-like --matrix-format mtx`
**Explanation:** Collating at `cr-like` resolution removes UMI-collapsed reads that failed barcode or annotation checks, and the MTX format creates three files (matrix.mtx, genes.tsv, barcodes.tsv) suitable for import into Scanpy or Seurat.

### Generate a count matrix in H5AD format for direct Scanpy import
**Args:** `collate --input-dir alevic_output --output-dir fry_h5ad --resolution cr-like --matrix-format h5ad`
**Explanation:** H5AD output bundles the sparse count matrix, gene list, and barcode metadata into a single HDF5 file that Scanpy loads with `sc.read_h5ad()` without requiring separate file management.

### Aggregate multiple alevic sample directories into a unified multi-sample matrix
**Args:** `aggregate --input-dir sample1_fry:sample2_fry:sample3_fry --output-dir aggregated --matrix-format mtx`
**Explanation:** The colon-delimited input paths cause alevin-fry to merge cell barcodes across all three samples using shared gene identifiers, creating a single matrix where rows are genes and columns span all samples with NA entries where a barcode was absent.

### Prune ambiguous UMI assignments from a pre-collated matrix
**Args:** `prune --input-dir fry_collated --output-dir fry_pruned --resolution ambiguous`
**Explanation:** The `prune` subcommand removes rows assigned to multiple features (e.g., reads overlapping two transcripts), reducing false positive doublet calls in downstream clustering tools that rely on clean, unambiguous count matrices.

### Generate a permit list using a custom barcode whitelist for Drop-seq data
**Args:** `generate-permit-list --permit-list dropseq_barcodes.txt --feature-technology dropseq --input-dir alevic_dropseq_output --output-dir fry_dropseq`
**Explanation:** Drop-seq uses a custom bead structure rather than the 10x whitelist format, so specifying the `dropseq` feature technology ensures the permit list parser correctly handles the barcode length and checksum fields native to Drop-seq bead design.

### Collate with raw resolution to retain all UMI counts including ambiguous ones
**Args:** `collate --input-dir alevic_output --output-dir fry_raw --resolution raw --matrix-format mtx`
**Explanation:** Using `raw` resolution bypasses UMI-filtering entirely, preserving all collapsed reads regardless of barcode quality or feature assignment ambiguity, which is useful for debugging alevic parameter choices before committing to filtered outputs.

### Generate an H5AD matrix and immediately prune for splicing-aware analysis
**Args:** `collate --input-dir alevic_output --output-dir fry_spliced --resolution spliced --matrix-format h5ad`
**Explanation:** The `spliced` resolution includes only reads mapped to the splicedTranscript index in alevic and outputs H5AD directly, producing a matrix where rows represent only mature mRNA counts suitable for RNA velocity or trajectory analysis pipelines.