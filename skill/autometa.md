---
name: autometa
category: genome-annotation
description: Autometa is a command-line pipeline for automated structural and functional annotation of microbial (bacterial and archaeal) genomes. It wraps third-party tools (Prodigal for gene prediction, HMMER for domain detection, DIAMOND for homology search) under a single batch-friendly interface, accepting assembled FASTA contigs and producing GFF3 and multi-FASTA outputs. Autometa manages its own SQLite-backed database-of-families (DMF) for curated protein families used in functional annotation.
tags: [bacterial-genome-annotation, gene-prediction, functional-annotation, gff3, hmm, prodigal, batch-processing, microbial-genomics]
author: AI-generated
source_url: https://github.com/NYCU-EDA/autometa
---

## Concepts

- **Input is assembled contigs in multi-FASTA format.** Autometa does not perform assembly or read QC; raw sequencing reads must be assembled externally (e.g., with SPAdes or Unicycler) before autometa can annotate ORFs. The input FASTA must have one header line per contig/scaffold starting with `>`.
- **Gene prediction is handled by Prodigal internally.** When autometa runs gene-calling, it invokes Prodigal with configurable training parameters. The predicted ORFs are written to a multi-FASTA protein file and a GFF3 feature file. You control the scoring cutoff via `--coding-score-cutoff`; raising this value reduces predicted genes but increases precision.
- **Functional annotation uses a DMF database of curated protein families.** The Database-of-Families (DMF) is a SQLite file populated by `autometa-build`; it stores HMM profiles and associated metadata. During annotation, HMMER scans ORF translations against these profiles. If the DMF is missing or stale, annotation falls back to DIAMOND BLASTP against a provided reference database or skips functional assignment entirely.
- **Batch mode processes multiple genomes in one command.** The `autometa batch` subcommand takes a tab-delimited samplesheet (`--input`) listing genome IDs, paths to their assembled FASTA files, and optionally taxonomy ranks. Each row is processed independently through the same workflow, making autometa-batch suitable for pangenome or survey projects.
- **Output formats: GFF3 for features, multi-FASTA for proteins, and JSON for provenance.** The annotation GFF3 file contains `gene`, `CDS`, and `mRNA` features with attributes such as `ID`, `Name`, and `product`. A companion multi-FASTA (`.faa`) holds the translated protein sequences in the same order as the GFF. A JSON log captures tool versions, runtimes, and parameters for reproducibility.

## Pitfalls

- **Running functional annotation without a populated DMF produces zero annotations.** If you invoke `autometa workflow` with `--dmf` pointing to a non-existent or empty SQLite database, the HMMER step completes silently with 0 hits and all ORFs remain uncharacterized. Always run `autometa-build update` before the first functional-annotation run.
- **Specifying the wrong Kingdom (bacteria vs. archaea) causes poorly trained gene models.** Prodigal uses different codon tables and translation tables for Archaea (option `--kingdom ar`) versus Bacteria (default). Annotating an archaeal genome with the bacterial kingdom flag yields incorrect start-codon selection and truncated proteins at the N-terminus.
- **Setting a coding-score-cutoff that is too high silently discards real genes.** Prodigal assigns each ORF a coding score; if this score is below `--coding-score-cutoff` the CDS is dropped entirely. A cutoff above the default (0.5) may eliminate 10–30 % of genuine short genes, particularly those near contig ends or in AT-rich genomes.
- **Batch mode samplesheets with DOS line endings cause silent row skips.** Autometa's tab-parser chokes on `\r\n` line endings, treating a row as zero-length and skipping it without a warning message. Pre-process samplesheets with `sed -i 's/\r//g' file.tsv` before passing `--input`.
- **Naming input contigs with characters that GFF3 reserves causes malformed output.** Contig headers containing spaces, pipe (`|`), or colon (`:`) are percent-encoded in the GFF3 `##sequence-region` directives, which some downstream tools reject. Sanitize contig names to simple alphanumeric and underscore characters before annotation.

## Examples

### Annotate a single bacterial genome with default settings
**Args:** workflow --genome mygenome.fna --output-dir annot_results --kingdom bacteria
**Explanation:** The `workflow` subcommand runs gene prediction and functional annotation in sequence. Without a DMF specified, autometa performs only structural annotation (Prodigal) and writes a GFF3 and protein FASTA to the output directory.

### Populate and verify the functional-annotation DMF database
**Args:** autometa-build update --dmf autometa.dmf --fams-dir ./hmm_families --overwrite
**Explanation:** The `autometa-build` companion binary populates the DMF from a directory of curated HMM families. Passing `--overwrite` rebuilds all entries; omitting it preserves pre-existing entries and only adds new families.

### Batch-annotate multiple genomes listed in a tab-delimited samplesheet
**Args:** batch --input genomes.tsv --output-dir batch_results --threads 8 --dmf autometa.dmf
**Explanation:** The tab-delimited samplesheet (default expected columns: `id`, `fasta`) is processed in parallel using 8 threads. The `--dmf` argument enables functional annotation for all genomes; omitting it restricts each run to gene calling only.

### Perform gene prediction only (skip HMMER-based functional annotation)
**Args:** workflow --genome contigs.fna --output-dir orf_results --workflow gene --no-functional --coding-score-cutoff 0.3
**Explanation:** Using `--workflow gene` limits execution to the Prodigal step. The `--no-functional` flag prevents autometa from invoking HMMER, and `--coding-score-cutoff 0.3` lowers the ORF retention threshold to capture shorter genes that would otherwise be discarded.

### Annotate an archaeal genome with stricter ORF quality control
**Args:** workflow --genome archaeon_contigs.fna --output-dir archaeon_annot --kingdom ar --coding-score-cutoff 0.7 --dmf archaea.dmf
**Explanation:** Setting `--kingdom ar` selects archaeal translation and codon tables in Prodigal. Raising `--coding-score-cutoff` to 0.7 discards low-confidence ORFs, which reduces spurious annotations in GC-homopolymer regions common in archaeal genomes.