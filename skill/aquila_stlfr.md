---
name: aquila_stlfr
category: Genomics / Genome Assembly
description: Assembler for Single-Telomere-Length Fragment (STL-FR) reads in long-read sequencing data. Processes and reconstructs telomeric regions from PacBio or Oxford Nanopore data to generate telomere-to-telomere assemblies with high accuracy.
tags:
  - genomics
  - genome-assembly
  - telomere
  - long-reads
  - nanopore
  - pacbio
  - stl-fr
  - bioinformatics
  - structural-variation
author: AI-generated
source_url: https://github.com/aquila-assembly/aquila_stlfr
---

## Concepts

- **STL-FR Read Input**: The tool accepts raw long-read sequencing data in FASTQ or BAM format from PacBio SMRT or Oxford Nanopore platforms. STL-FR (Single-Telomere-Length Fragment Read) are specialized long reads that span entire telomeric regions, enabling complete reconstruction of chromosome ends.

- **Telomere Reconstruction Mode**: aquila_stlfr operates in two modes—`assembly` for de novo reconstruction of telomeric sequences from STL-FR reads, and `polish` for refinement of existing telomere assemblies using the same input reads. The mode is specified via the `--mode` flag with values `assembly` or `polish`.

- **Output Formats**: Generated outputs include FASTA files for assembled telomeric contigs, a summary JSON report containing coverage statistics and repeat annotations, and optional SAM alignments for downstream analysis. The output directory is specified with `--outdir` and defaults to `./aquila_output`.

- **Reference-Guided Assembly**: When a reference genome is provided via `--reference`, aquila_stlfr performs guided assembly to identify misassemblies and validate telomere-call metrics against known telomeric repeat单元 units. This improves accuracy in regions with complex repetitive structures.

- **Read Filtering Parameters**: Quality filtering is controlled by `--min-qv` (minimum quality value, default 20), `--min-length` (minimum read length in base pairs, default 5000), and `--min-coverage` (minimum coverage depth, default 3). These parameters significantly impact assembly continuity and base accuracy.

## Pitfalls

- **Using Incompatible Read Types**: Passing non-STL-FR reads (standard long reads that don't span full telomeric regions) will produce fragmented or no assemblies. STL-FR reads must be specifically extracted or generated from raw data using appropriate preprocessing tools, otherwise the assembler fails to find bridging reads across repeat boundaries.

- **Ignoring Read Length Thresholds**: Setting `--min-length` below 5000 when working with Nanopore data typically yields assemblies with excessive gaps and misjoins, because short reads cannot span the full telomeric repeat arrays (which extend 5-20 kb in humans). The resulting contigs will be fragmented and useless for T2T completion.

- **Mismatched Reference Sequences**: Providing a reference genome from a diverged species when using `--reference` causes incorrect read-to-reference mappings and generates false structural variation calls. The tool attempts to force alignment even with high divergence, leading to inflated variant lists in the output report.

- **Insufficient Coverage Depth**: Running assembly with `--min-coverage 1` produces assemblies with no consensus callable at many positions, resulting in ambiguous base calls marked as 'N' throughout输出的FASTA文件。 The assembly will be unusable for downstream analysis without manual curation.

- **Overwriting Output Directories**: Specifying an existing `--outdir` without the `--force` flag causes the tool to exit without processing, but many workflows fail to check for this and appear to hang or misreport. Subsequent downstream steps may then read stale partial results from previous runs.

## Examples

### Assemble telomeric regions from Nanopore STL-FR reads
**Args:** `--mode assembly --input nanopore_stlfr.fastq --outdir ./telomere_assembly --threads 16`
**Explanation:** Runs de novo assembly of telomeric sequences from Nanopore-derived STL-FR reads, using 16 threads for parallel processing and writing results to the specified output directory.

### Polish an existing telomere assembly with higher quality reads
**Args:** `--mode polish --input high_qv_stlfr.fastq --reference ./draft_telomere.fasta --outdir ./polished_output --min-qv 25`
**Explanation:** Refines an existing draft assembly using high quality value (QV≥25) STL-FR reads to improve base accuracy in the telomeric consensus sequence.

### Specify minimum read length filter for PacBio data
**Args:** `--mode assembly --input pacbio_stlfr.bam --outdir ./pacbio_assembly --min-length 10000`
**Explanation:** Applies a stricter minimum length filter of 10 kb to PacBio SMRT reads, ensuring only full-length telomeric spanning reads contribute to the assembly.

### Generate alignment file for downstream structural variation analysis
**Args:** `--mode assembly --input stlfr.fastq --outdir ./assembly --reference hg38.fa --output-alignments`
**Explanation:** Produces a SAM-formatted alignment file alongside the assembly, enabling direct comparison of assembled telomeric contigs against the reference genome for variant calling.

### Force overwrite existing output directory with new results
**Args:** `--mode assembly --input reads.fastq --outdir ./results --force --min-coverage 5`
**Explanation:** Overwrites any existing files in the output directory and requires at least 5× coverage depth across assembled regions to call consensus bases, discarding low-coverage regions.