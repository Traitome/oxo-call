---
name: camlhmp
category: metagenomics
description: A tool for analyzing marker gene sequences from metagenomic samples, typically used for taxonomic profiling of microbiomes.
tags: [metagenomics, 16s, marker-gene, microbiome, taxonomic-profiling]
author: AI-generated
source_url: https://github.com/metagenomics/camlhmp
---

## Concepts

- **Input formats**: camlhmp accepts FASTA, FASTQ, and SAM/BAM files containing marker gene sequences (16S rRNA, 18S rRNA, ITS) from metagenomic samples.
- **Reference databases**: The tool uses built-in reference databases (Greengenes, SILVA, UNITE) for taxonomic assignment; custom databases can be provided via the `--db` flag.
- **Output formats**: Results are produced as TSV tables with taxonomic assignments, abundance counts, and confidence scores per sample; BIOM format for Qiime2 compatibility is supported.
- **Abundance estimation**: camlhmp computes relative abundance using read counts normalized by marker gene copy number and sequence length.
- **Paired-end support**: Mate pairs are merged automatically before classification when the `--pe` flag is specified.

## Pitfalls

- **Reference mismatch**: Using a reference database that doesn't cover the marker gene in your samples leads to many unassigned reads and incomplete profiles.
- **Duplicate sequences**: Not deduplicating input sequences before running camlhmp inflates abundance estimates and biases diversity metrics.
- **Wrong reference direction**: Sequences oriented opposite to the reference database result in failed alignments or misclassifications; use `--reverse-complement` if needed.
- **Insufficient sampling depth**: Low read counts per sample produce unreliable diversity estimates and wide confidence intervals.

## Examples

### Classify 16S sequences from a single FASTQ file

**Args:** `input.fq --db greengenes --output results.tsv`

**Explanation:** Runs taxonomic classification on input sequences using the Greengenes reference database and writes output to a TSV file.

### Analyze multiple samples from a directory

**Args:** `samples/ --db silva --output_dir results/ --format tsv`

**Explanation:** Processes all sequence files in the directory against the SILVA database and writes separate result files for each sample.

### Generate BIOM table forQiime2

**Args:** `input.fasta --db unite --output table.biom --biom`

**Explanation:** Produces a BIOM-formatted abundance table compatible with Qiime2 for downstream diversity analysis.

### Use custom reference database

**Args:** `input.fq --db custom_ref.fa --taxonomy custom_tax.txt --output custom_results.tsv`

**Explanation:** Performs classification using a user-provided reference sequences and corresponding taxonomy file.

### Merge paired-end reads before classification

**Args:** `R1.fq R2.fq --db greengenes --pe --output merged_results.tsv`

**Explanation:** Merges mate pairs using overlap detection, then classifies the assembled sequences for improved accuracy.

### Adjust minimum identity threshold

**Args:** `input.fq --db greengenes --min-id 0.97 --output results.tsv`

**Explanation:** Sets a higher identity threshold (97%) to be more stringent in taxonomic assignments, reducing false positives.

### Output confidence scores

**Args:** `input.fq --db silva --output results.tsv --scores --format extended`

**Explanation:** Includes confidence scores in the output showing the certainty of each taxonomic assignment.