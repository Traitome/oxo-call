---
name: architeuthis
category: genome-assembly
description: Long-read sequence assembler and error corrector optimized for high-AT marine genomes. Handles PacBio HiFi and Oxford Nanopore reads with built-in repeat detection and scaffolding.
tags: [long-reads, assembly, de-novo, marine-genomics, PacBio, Nanopore]
author: AI-generated
source_url: https://github.com/oxo-bio/architeuthis
---

## Concepts

- architeuthis processes raw long-read input in BAM or FASTQ/FAST5 format and produces polished assembly contigs, intermediate scaffold graphs in GFA format, and comprehensive quality metrics in JSON
- Memory requirements scale with estimated genome size; for AT-rich marine genomes (60–70% AT content), allocating 1 GB RAM per 100 Mbp of estimated haploid genome size prevents graph construction failures
- The assembler builds a sparse de Bruijn graph internally using k-mers extracted from raw reads; k-mer size directly affects repeat resolution—larger k-mers improve repeat discrimination but require higher coverage depth to maintain graph connectivity
- Built-in repeat detection annotates telomeric, centromeric, and rDNA repeat classes during assembly; these annotations are written as BED annotations alongside the primary FASTA output
- The scaffolding module integratesHi-C chromatin contact maps when supplied via the `--hic` flag, producing chromosome-level scaffolds with orientation flags

## Pitfalls

- Specifying the wrong read type with `--reads` leads to internal parameter mismatches because architeuthis applies PacBio HiFi-specific error models differently from Oxford Nanopore basecalling profiles, resulting in degraded consensus quality
- Setting `--genome-size` significantly below the actual genome size causes premature graph truncation and fragmented assemblies with collapsed repetitive regions; always use a kmer-based pre-estimation run before assembly
- Choosing a k-mer size smaller than 21 (`-k 21`) reduces repeat resolution in centromeric satellite arrays, producing assemblies with mis-assembled repeat copies and inflated total length
- Omitting `--min-read-length` on datasets with high subread percentages results in excessive computational overhead from fragments that cannot contribute meaningfully to graph connectivity
- Skipping the `--qc-only` flag when assessing read quality before full assembly wastes compute—always run quality control first on new datasets to verify N50 and read accuracy distributions

## Examples

### Assemble a PacBio HiFi dataset with default parameters
**Args:** `--reads hifi_reads.bam --genome-size 1.5G --output assembly_run1`
**Explanation:** This runs the complete assembly pipeline using the default k-mer size and repeat detection thresholds appropriate for a mid-sized marine genome assembled from HiFi reads.

### Assemble Nanopore reads with Hi-C integration for chromosome-level scaffolding
**Args:** `--reads nanopore_q20.fastq.gz --hic hic_contacts.pairs.gz --genome-size 2.1G -k 31 --output chrom_scaffold`
**Explanation:** The combination of Nanopore reads with Hi-C contact maps enables the scaffolding module to orient and order contigs into chromosome-scale scaffolds with predicted centromere positions.

### Perform quality control assessment before running full assembly
**Args:** `--reads dataset.fastq.gz --qc-only --output qc_report`
**Explanation:** This runs only the quality control module, computing N50, read accuracy distribution, and coverage estimates without initiating the memory-intensive graph construction phase.

### Assemble with reduced k-mer size for low-coverage datasets
**Args:** `--reads lowcov_reads.bam --genome-size 800M -k 21 --min-coverage 15 --output lowcov_assembly`
**Explanation:** A smaller k-mer size compensates for reduced graph connectivity in low-coverage datasets by ensuring sufficient k-mer frequency even across sparse repeat regions.

### Resume a partially failed assembly from checkpoint files
**Args:** `--resume --checkpoint architeuthis_ckpt.bin --output resumed_assembly`
**Explanation:** The resume flag instructs the assembler to reload the binary checkpoint file and continue graph construction from the last successfully completed iteration rather than restarting from scratch.