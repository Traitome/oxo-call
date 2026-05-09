---
name: ampligone
category: genomics
description: A command-line tool for amplicon read processing, filtering, and variant calling from targeted sequencing experiments. Ampligone takes demultiplexed FASTQ files and outputs quality-filtered reads, OTU/ASV tables, and variant call formats suitable for downstream microbial or genetic diversity analysis.
tags:
  - amplicon-sequencing
  - read-filtering
  - variant-calling
  - fastq-processing
  - microbial-diversity
  - targeted-sequencing
author: AI-Generated
source_url: https://github.com/oxo-tools/ampligone
---

## Concepts

- **Paired-end read overlap and merging**: Ampligone expects paired-end FASTQ files (R1/R2) by default. Reads are merged using an overlap detection algorithm (minimum overlap: 12 bp, mismatch tolerance: 15% of overlap length). Merged reads shorter than 100 bp or with uncalled bases ('N') exceeding 5% are flagged for removal. Always inspect the `.log` file for merge success rates—if merge rates fall below 60%, check whether your amplicon length is shorter than the combined read length, which can cause spurious overlaps.

- **Quality filtering thresholds**: Reads are trimmed using a sliding-window approach (window size: 5 bp, quality threshold: Q20). After trimming, reads with final lengths below the `--min-length` value or average quality below `--min-qual` are discarded. The default min-length is 100 bp. For454-style or long-read amplicon data, you must increase `--min-length` appropriately, otherwise short high-quality reads will be erroneously dropped and reduce your effective read count.

- **Output formats and OTU/ASV generation**: Ampligone can produce three output formats: (1) a dereplicated FASTA file (`--output-fasta`) for OTU clustering tools like VSEARCH; (2) an ASV table in BIOM format (`--biom-output`) for qiime2-compatible diversity analysis; (3) a variant call table (`.vcf`) for haplotyping single nucleotide variants. The `--cluster-identity` flag (default: 0.97) controls the similarity threshold for dereplication before OTU picking, so ensure this matches your study's taxonomic resolution—using 0.97 for genus-level or 0.99 for species-level assignments.

- **Multiplexed sample handling**: If your reads still contain barcode sequences (from Illumina's dual-indexing or custom inline barcodes), use `--demux` with a manifest file specifying barcode sequences per sample. Ampligone does not auto-detect barcode orientation; ensure your manifest uses the correct strand orientation (`F` for forward-only, `R` for reverse-complement) or demultiplexing will assign reads to the wrong sample entirely, corrupting downstream diversity estimates.

## Pitfalls

- **Mismatched read orientation causing low merge rates**: If your amplicon is in the reverse-complement orientation relative to the sequencing primer, reads will not overlap correctly. Check your library prep protocol—ampligone assumes R1 reads start at the forward primer. If merge rates are unexpectedly low and your primer is on the reverse strand, use `--revcomp` to flip the R2 orientation before merging, otherwise you'll lose 30–70% of your reads.

- **Ignoring the --max-homopolymer flag for homopolymer-rich regions**: Amplicons from organisms like Pseudomonas or Streptomyces contain homopolymer tracts that cause indel errors in Illumina data. By default, ampligone sets `--max-homopolymer: 6`, but if your target gene has longer homopolymers (common in 16S or rpoB), reads with these tracts will be incorrectly soft-clipped. Increase this threshold or use `--skip-homopolymer-filter` to avoid systematic read loss in affected taxa.

- **Mixing single-end and paired-end inputs without specifying --single-end**: Providing a single FASTQ file when the workflow expects paired reads causes ampligone to either hang or produce empty output with an uninformative error ("EOF reached"). Always specify `--single-end` explicitly when processing solo reads, and note that merging QC will be skipped—the quality trim step will still apply, but over sequences without mate-pair validation.

- **Forgetting to index output directories**: Ampligone does not auto-create output directories. If you specify `--output-dir results/` and the directory does not exist, the tool will exit with a cryptic I/O error. Always pre-create output directories or use `mkdir -p` before running, especially in automated pipelines or Snakemake workflows.

- **Over-filtering with strict quality thresholds reduces effective depth**: Setting `--min-qual 30` and `--min-length 200` simultaneously can discard 40–60% of reads in low-quality or short-amplicon datasets. While stricter thresholds improve downstream taxonomic accuracy, they also reduce your read count per sample, potentially causing false absences in rare OTUs. Validate filtering thresholds on a subset of your data first using `--dry-run` before running the full dataset.

## Examples

### Filtering paired-end FASTQ files with default quality thresholds

**Args:** `--R1 sample_R1.fastq.gz --R2 sample_R2.fastq.gz --output-dir filtered/ --min-qual 25 --min-length 120`
**Explanation:** Runs quality-based trimming and length filtering on paired reads, discarding reads with average quality below Q25 or final length below 120 bp, suitable for standard 16S amplicon data with moderate base quality.

### Merging paired-end reads and exporting an ASV table in BIOM format

**Args:** `--R1 R1.fastq.gz --R2 R2.fastq.gz --merge --biom-output asv_table.biom --min-overlap 15 --max-mismatch 0.12`
**Explanation:** Merges overlapping read pairs and outputs a BIOM-format abundance table for direct import into qiime2, with strict overlap parameters to minimize chimeric calls in conserved regions.

### Demultiplexing inline-barcoded reads using a manifest file

**Args:** `--demux manifest.txt --R1 raw_R1.fastq.gz --R2 raw_R2.fastq.gz --output-dir demuxed/`
**Explanation:** Uses a manifest file containing barcode sequences to separate multiplexed reads into per-sample FASTQ files before filtering, required when samples were pooled with custom barcodes not handled by Illumina's index read.

### Dereplicating reads and clustering into OTUs with custom identity threshold

**Args:** `--R1 filtered_R1.fastq.gz --R2 filtered_R2.fastq.gz --cluster --cluster-identity 0.97 --output-fasta otus.fasta --derep-min 2`
**Explanation:** Dereplicates filtered reads, clusters at 97% identity, and outputs a FASTA file of representative OTU sequences with a minimum abundance of 2 reads, appropriate for genus-level diversity analysis.

### Generating a VCF file of SNP variants from haploid amplicon data

**Args:** `--R1 sample.fastq.gz --single-end --variant-call snps.vcf --min-qual 30 --min-allele-freq 0.05 --haploid`
**Explanation:** Processes single-end reads from haploid organisms (e.g., bacterial loci) and calls SNPs above 5% allele frequency into VCF format, suitable for detecting low-frequency drug-resistance mutations in clonal populations.

---