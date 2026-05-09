---
name: abritamr
category: variant_detection
description: A command-line tool for detecting and characterizing antimicrobial resistance (AMR) genes in bacterial genome sequences. Supports multiple database formats, allele calling, and species-specific analysis.
tags:
  - antimicrobial-resistance
  - amr
  - variant-calling
  - bacterial-genomics
  - resistance-genes
  - allele-detection
author: AI-generated
source_url: https://github.com/ncbi-amr/abritamr
---

## Concepts

- **Input formats**: abritamr accepts FASTA files (.fasta, .fa) containing nucleotide sequences, and FASTQ files (.fastq, .fq) for raw reads. Multiple input files can be provided as a comma-separated list or directory glob.
- **Database model**: The tool uses a local database of known AMR alleles organized by gene name and variant. Each allele has a reference sequence and metadata including drug class, mechanism (e.g., enzymatic inactivation, target modification), and species of origin.
- **Output formats**: Results are produced in structured formats including TSV (default), JSON, and CSV. The output contains columns for gene name, allele variant, coverage, identity, drug class, and clinical significance.
- **Allele calling**: The tool performs fuzzy matching against reference alleles using coverage and identity thresholds. A gene is called present when coverage exceeds 90% and identity exceeds 80% by default.
- **Species filtering**: Analysis can be restricted to specific bacterial species using the `--species` flag, which applies species-specific cutoffs and filters out irrelevant matches.

## Pitfalls

- **Low coverage threshold**: Setting coverage below 80% can cause false positive AMR gene calls, particularly in fragmented assemblies where only partial genes are recovered. This leads to overestimation of resistance profiles in clinical reports.
- **Missing database updates**: Using an outdated AMR database results in failure to detect newly emerged resistance variants. The tool will report "None found" even when resistant strains are present, causing incorrect susceptibility interpretations.
- **Conflicting sequence headers**: Input FASTA headers containing special characters (colons, pipes, spaces) cause parsing errors. The tool may skip sequences or produce malformed output without clear error messages.
- **Ignoring partial matches**: Treating all matches equally regardless of coverage/identity depth can misclassify partial gene fragments as full resistance genes, leading to incorrect antimicrobial prescribing decisions.
- **Omitting species context**: Running analysis without species filtering produces many false positives from species-specific pseudogenes that are not functional in the target organism.

## Examples

### Detect AMR genes in a bacterial assembly

**Args:** `--input assembly.fasta --output results.tsv`

**Explanation:** Reads a FASTA assembly file, queries the AMR database, and writes detected resistance genes in TSV format with coverage and identity scores.

### Force species-specific analysis for Escherichia coli

**Args:** `--input isolates/ --species "Escherichia coli" --db custom_amr.fasta --output ecoli_amr.tsv`

**Explanation:** Restricts analysis to E. coli-relevant AMR alleles, using a custom database, and outputs results filtered to only genes applicable in this species.

### Adjust detection thresholds for noisy assemblies

**Args:** `--input contigs.fasta --min-coverage 50 --min-identity 75 --output lenient_calls.tsv`

**Explanation:** Lowers coverage and identity thresholds to account for fragmented or low-quality assemblies, capturing partial but potentially relevant resistance fragments.

### Export results in JSON format for downstream pipelines

**Args:** `--input genes.fasta --output json --format json --pretty`

**Explanation:** Produces JSON output enabling programmatic parsing by automated analysis pipelines and integration with laboratory information systems.

### Run with verbose logging for troubleshooting

**Args:** `--input sample.fa --output verbose.tsv --log-level debug --threads 8`

**Explanation:** Enables debug-level logging and uses 8 threads to trace database matching steps and identify why expected genes are not being detected.

### Query specific drug class only

**Args:** `--input genome.fa --output carbapenem.tsv --drug-class carbapenem`

**Explanation:** Filters output to only carbapenem resistance genes (e.g., KPC, NDM, VIM, IMP), useful for targeted surveillance of last-resort antibiotic failures.

### Build custom database from GenBank files

**Args:** `build --input gb_files/ --output my_amr_db.fasta --type nucleotide`

**Explanation:** Uses the companion binary to construct a custom AMR database from GenBank files, enabling detection of institution-specific or emerging resistance variants.