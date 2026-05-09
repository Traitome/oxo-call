---
name: cansnper2
category: Variant Calling / SNP Detection
description: A reference-based SNP caller that identifies single nucleotide polymorphisms from BAM alignment files. Takes sorted, indexed BAM inputs and a reference genome to output called SNPs with quality metrics, coverage depths, and allele frequencies.
tags:
  - snp-calling
  - variant-calling
  - bam
  - genomics
  - microbial-typing
  - CanSNP
author: AI-Generated
source_url: https://github.com/EllenH(placeholder)
---

## Concepts

- **Input Format**: cansnper2 operates exclusively on sorted and indexed BAM files. The alignments must be coordinate-sorted (not query-sorted) and accompanied by a corresponding BAI index file. Unsorted BAM inputs will cause the tool to terminate with an error, requiring re-sorting with tools like `samtools sort`.

- **Reference Genome Requirement**: A reference genome must be provided as a plain FASTA file (not compressed). The reference should be the same one used for the original read alignment, as coordinate mismatches between the BAM and reference will result in incorrect or zero SNP calls. The reference sequence name and length must match the SQ headers in the BAM file.

- **Output Reporting**: Called SNPs are reported with six key fields per line: chromosomal position (1-based), reference base, called base, coverage depth (forward strand count / reverse strand count), average base quality at the position, and a callable flag (0 = called, 1 = no coverage, 2 = insufficient quality). Users can select tabular, CSV, or JSON output formats.

- **Quality Threshold Model**: cansnper2 applies dual filtering: a minimum total coverage threshold (default 10 reads) and a minimum average base quality threshold (default Q20). A position must satisfy BOTH thresholds for a SNP to be called. Positions failing only one threshold are flagged but not discarded from the output stream.

## Pitfalls

- **Misaligned or Unindexed BAM Files**: Running cansnper2 on a query-sorted BAM or a BAM missing its BAI index produces no error message but silently returns an empty SNP list. Always verify BAM integrity with `samtools quickcheck` before calling SNPs, especially when the output seems unexpectedly empty.

- **Reference Mismatch**: If the reference genome used for read alignment differs from the one passed to cansnper2, the tool will report spurious SNPs at every mismatch between the two references rather than reporting an error. Confirm the MD5 checksum of your reference FASTA matches the one used in the alignment pipeline.

- **Default Coverage Threshold Too Low for High-Complexity Genomes**: The default minimum coverage of 10 reads is calibrated for bacterial genomes. For larger eukaryotic genomes or repetitive regions, this produces excessive false-positive SNP calls. Increase the coverage threshold to at least 20–30 for mammalian-sized genomes to maintain comparable specificity.

- **Confusing Callable Flags with SNP Type**: The callable flag (third column) does not distinguish between heterozygous and homozygous calls; it only indicates whether the position passed the quality filters. Users analyzing diploid organisms must manually parse the allele frequency column to classify variants as homozygous (>0.9 frequency) or heterozygous (0.2–0.8 frequency).

- **Overwriting Output Files Without Warning**: cansnper2 silently overwrites existing output files with the same name. When running in batch mode across multiple samples, ensure output filenames include sample identifiers, or direct output to a dedicated directory to prevent irreversible data loss.

## Examples

### Call SNPs from a single BAM file against a reference with default thresholds
**Args:** `input.bam -r reference.fasta -o snps_default.txt -f tabular`
**Explanation:** Runs SNP calling with default minimum coverage (10) and base quality (Q20) thresholds, outputting results in a tab-delimited format for immediate manual review or spreadsheet import.

### Call SNPs with elevated coverage and quality thresholds for a mammalian genome
**Args:** `sample.bam -r hg38.fasta -o high_stringency_snps.txt -f csv -c 30 -q 30`
**Explanation:** Increases both coverage and quality cutoffs to reduce false positives in large genomes where repetitive sequences cause mapping ambiguity, and outputs in CSV format for downstream R or Python parsing.

### Call SNPs and save output in JSON format for programmatic pipelines
**Args:** `alignment.bam -r ref.fasta -o snps.json -f json`
**Explanation:** Outputs called SNPs as a JSON array where each object contains position, alleles, strand-specific coverage, and quality, enabling direct integration into automated reporting tools or web dashboards.

### Batch process multiple BAM files using wildcard input
**Args:** `*.bam -r reference.fasta -o batch_output/ -f tabular --batch`
**Explanation:** Enables parallel processing of all BAM files matching the glob pattern, writing each output file to a specified directory with basenames derived from the input BAM names. The `--batch` flag activates the directory creation and per-sample file naming logic.

### Call SNPs and include low-coverage positions for manual review
**Args:** `reads.bam -r ref.fasta -o with_uncertain_snps.txt -f tabular --include-failed`
**Explanation:** Exports all positions including those below the quality threshold, marked with a callable flag value of 1 or 2, allowing manual expert review of borderline calls that may be genuine SNPs with low sequencing depth.