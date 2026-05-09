---
name: a5-miseq
category: Genome Assembly
description: A5-miseq is a pipeline for microbial genome assembly from Illumina paired-end sequencing data. It performs integrated quality control, read trimming, error correction, assembly, and coverage calculation for bacterial genomes.
tags: genomics, assembly, bacteria, illumina, fastq, contigs, microbial
author: AI-generated
source_url: https://www.mgc.ai/
---

## Concepts

- **Input format:** A5-miseq accepts paired-end FASTQ files from Illumina sequencing. Reads should be in standard FASTQ format with quality scores (Phred+33 or Phred+64 encoding). The pipeline expects forward (R1) and reverse (R2) read files as separate inputs.
- **Data processing workflow:** The pipeline performs read quality filtering, adapter trimming, error correction using a Bayesian approach, de Bruijn graph assembly, and post-assembly coverage calculation. All steps run sequentially without intermediate file management.
- **Output artifacts:** Results include assembled contigs in FASTA format (`contigs.fa`), a coverage histogram, read statistics summary, and an HTML report showing assembly metrics such as N50, total assembled bases, and estimated coverage depth.
- **Paired-read pairing logic:** A5-miseq uses insert size estimates from the data to resolve read relationships during assembly. Standard Illumina library preparation insert sizes (200-800 bp) work directly; custom insert sizes may require specification via parameters.

## Pitfalls

- **Using unpaired or single-end reads:** A5-miseq is optimized specifically for paired-end Illumina data. Providing single-end reads will cause the pipeline to fail or produce poor-quality assemblies without notification.
- **Ignoring low-quality input data:** Providing FASTQ files with average Phred quality scores below 20 leads to fragmented assemblies with many small contigs. Pre-processing with tools like Trimmomatic improves results.
- **Specifying incorrect read orientation:** Mixed-orientation read files (forward-reverse reversed) cause the assembly algorithm to misplace read connections, resulting in chimeric contigs and inflated assembly sizes.
- **Naming output files with special characters:** Using spaces, hyphens, or special characters in output directory paths breaks downstream file handling and prevents HTML report generation.

## Examples

### Assemble paired-end reads from a bacterial genome
**Args:** --left sample_R1.fastq --right sample_R2.fastq --output assembly_result
**Explanation:** Runs the full A5-miseq pipeline with default parameters on paired-end Illumina reads, producing assembled contigs in the specified output directory.

### Specify a minimum read quality threshold
**Args:** --left sample_R1.fastq --right sample_R2.fastq --output assembly_result --QUALCUT 25
**Explanation:** Filters reads with Phred quality scores below 25 before assembly, improving assembly quality for datasets with moderate per-base quality.

### Set a custom minimum contig length cutoff
**Args:** --left sample_R1.fastq --right sample_R2.fastq --output assembly_result --MINCONTIGLEN 200
**Explanation:** Removes contigs shorter than 200 bp from the final assembly, useful when targeting chromosome-scale assembly from high-coverage data.

### Assemble with custom insert size
**Args:** --left sample_R1.fastq --right sample_R2.fastq --output assembly_result --INSMEAN 500
**Explanation:** Specifies a 500 bp mean insert size for the library, helping the assembler correctly pair overlapping read connections.

### Run without coverage calculation
**Args:** --left sample_R1.fastq --right sample_R2.fastq --output assembly_result --NOCOV
**Explanation:** Skips post-assembly coverage calculation to speed up runtime when only contig sequences are needed.