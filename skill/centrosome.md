---
name: centrosome
category: GenomeAssembly
description: A Bionano Genomics tool for haplotype-resolved de novo assembly of long-read sequencing data, including optical mapping and single-molecule sequencing data. Supports multi-step workflows from read alignment through assembly refinement.
tags: [genomics, assembly, long-reads, optical-mapping, haplotype-resolution, bionano]
author: AI-generated
source_url: https://github.com/bionanogenomics/centrosome
---

## Concepts

- **Data Model**: Centrosome operates on position-sorted FASTA/FASTQ files and SAM/BAM alignment files, treating sequences as linear contigs with defined start/end coordinates. Each input file represents a collection of candidate molecules or assembled contigs.
- **I/O Formats**: Primary inputs are FASTA (reference sequences), FASTQ (read data), and SAM/BAM (alignment maps). Outputs include assembled FASTA contigs, VCF variant files, and JSON/CJSON assembly metrics. All coordinate systems are 1-based, inclusive.
- **Key Behaviors**: The tool uses a iterative refinement pipeline that progressively improves assembly quality by detecting and resolving conflicts between overlapping sequences. Memory usage scales with the total base-pair count of input molecules, not sequence count.
- **Companion Binaries**: The toolset includes `centrosome-build` for reference preparation and `centrosome-mapseq` for quality assessment. These are invoked as separate executables, not as subcommands within the main `centrosome` binary.

## Pitfalls

- **Mismatched reference version**: Using a reference genome version that diverges from the input reads leads to fragmented assemblies with numerous misjoins. Always verify reference annotation coordinates match your read data source organism and build.
- **Incorrect file sorting**: Feeding unsorted or incorrectly sorted input files causes the alignment engine to fail silently or produce empty output. Verify sort order using `samtools header` or dedicated validation utilities before running assembly.
- **Insufficient overlap parameters**: Setting minimum overlap too high discards valid assembly connections, resulting in fragmented output; setting it too low permits false joins between unrelated sequences. Test with multiple overlap values and evaluate N50 metrics.
- **Ignoring run-time warnings**: The tool outputs warnings for low-complexity regions and repetitive content that often correlate with assembly errors. Suppressing or ignoring these warnings leads to low-quality haplotypes in subsequent analysis steps.

## Examples

### Build a reference index from a FASTA genome file

**Args:** centrosome-build --reference assembly.fasta --outdir ref_index/
**Explanation:** This creates indexed lookup structures required for read alignment, enabling fast k-mer matching during the assembly phase.

### Assemble long reads using overlap-layout-consensus

**Args:** centrosome assemble --input reads.fasta --ref ref_index/ --min-overlap 1000 --threads 8 --output assembly.fasta
**Explanation:** This performs OLC assembly using the prepared reference index, requiring minimum 1kb overlap between reads to form contigs.

### Filter reads by minimum length before assembly

**Args:** centrosome filter --input reads.fastq --min-length 5000 --output filtered_reads.fastq
**Explanation:** This removes shorter than 5kb reads that contribute noise to assembly without adding meaningful structural information.

### Generate layout statistics for assembled contigs

**Args:** centrosome layout --assembly assembly.fasta --stats layout_stats.json
**Explanation:** This computes coverage metrics, N50 values, and contig length distributions, outputting them in JSON format for downstream analysis.

### Align molecules against assembled reference for validation

**Args:** centrosome align -- molecules.sam --reference ref_index/ --output alignment.bam
**Explanation:** This validates assembly quality by measuring alignment rate and coverage of the assembled contigs back to the reference index.