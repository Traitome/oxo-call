---
name: aviary
category: genomics/assembly
description: A bioinformatics toolkit for viral and bacterial genome assembly, variant calling benchmarking, and sample recovery. Provides modular workflows for assembling short and long reads, calling variants against a reference, and evaluating assembly or variant accuracy.
tags:
- assembly
- variant-calling
- benchmarking
- viral-genomics
- bacterial-genomics
- short-reads
- long-reads
- quality-metrics
author: AI-generated
source_url: https://github.com/Flinny/aviary
---

## Concepts

- Aviary organizes analysis into distinct modules: `assembly` (de novo assembly from reads), `variant_calling` (SNP/indel detection against a reference), `benchmark` (accuracy evaluation), and `recover` (species detection and binning).
- Input reads are accepted in FASTQ or FASTA format; references must be in FASTA format with corresponding GFF annotations for benchmarking.
- The tool uses a configuration file (YAML) to define sample sheets, reference paths, and pipeline parameters across multiple samples, enabling reproducible batch processing.
- Output includes assembly FASTA files, aligned BAM/VCF files, and JSON/CSV metric reports comparing called variants or assembled contigs to ground truth.
- Aviary supports both short-read (ILLUMINA) and long-read (ONT, PACBIO) technologies, with technology-specific default settings for read error profiles and coverage thresholds.

## Pitfalls

- Using a reference sequence that is too divergent from the sample reads (>5% nucleotide divergence) results in poor mapping rates and inflated false variant calls, degrading benchmark accuracy.
- Running benchmark without providing a matching GFF annotation file causes the tool to skip gene-level evaluation, leaving critical recall/precision metrics undefined for variant calling.
- Specifying incorrect library type flags (e.g., treating short-read data as long-read) triggers wrong error models in the variant caller, leading to systematic base-level false positives or missed indels.
- Overwriting output directories between runs without enabling the `--resume` flag loses intermediate results and forces full recalculation of completed stages.
- Mixing read files from different samples in a single sample sheet entry creates chimeric assemblies or false composite variants, corrupting downstream comparative analyses.

## Examples

### Benchmark a variant caller against a truth set
**Args:** `benchmark --referenceRefseq.fa --variantsTruth.vcf --samples my_samples.csv --outputdir ./bench_results`
**Explanation:** Compares called variants from the sample sheet against a curated truth VCF to generate precision, recall, and F1 scores for each sample.

### Assemble short reads de novo
**Args:** `assembly --samples reads.csv --outputdir ./assemblies --maxthreads 16`
**Explanation:** Performs de novo assembly on all samples in the CSV using multi-threaded execution, producing FASTA contigs in the output directory.

### Run variant calling on aligned reads
**Args:** `variant_calling --referenceRefseq.fa --samples aligned_samples.csv --outputdir ./vcfs --gff annotations.gff`
**Explanation:** Calls SNPs and indels for each aligned sample against the provided reference using the matching annotation file to restrict analysis to coding regions.

### Recover species from metagenomic reads
**Args:** `recover --inputDir ./metagenome_fastq --outputdir ./recovered --database custom_db.fa`
**Explanation:** Detects and bins species from metagenomic FASTQ files using a custom database, outputting binned sequences and abundance estimates.

### Resume an interrupted benchmark run
**Args:** `benchmark --samples my_samples.csv --outputdir ./bench_results --resume`
**Explanation:** Restarts a partial benchmark run by checking for existing intermediate files and continuing from the last completed stage, saving computational time.

### Evaluate long-read assembly quality
**Args:** `assembly --samples longreads.csv --outputdir ./ont_assemblies --longread --medaka`
**Explanation:** Uses ONT-specific settings and the Medaka polishers appropriate for long-read error profiles when assembling Oxford Nanopore data.

### Generate a consolidated metric report
**Args:** `benchmark --referenceRefseq.fa --variantsTruth.vcf --samples multi_sample.csv --outputdir ./report --json --reportName combined_metrics`
**Explanation:** Processes multiple samples and produces a single JSON-formatted metric report summarizing performance across the entire cohort.