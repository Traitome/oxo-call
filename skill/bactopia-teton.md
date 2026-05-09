---
name: bactopia-teton
category: amr-detection
description: A Bactopia tool for detecting tetracycline antimicrobial resistance genes in bacterial protein sequences. It identifies tetracycline resistance determinants by searching query sequences against a curated tetracycline resistance gene database.
tags: [tetracycline, amr, antimicrobial-resistance, resistance-genes, bacteria, protein-search]
author: AI-generated
source_url: https://github.com/bactopia/bactopia
---

## Concepts

- **Input Format**: bactopia-teton accepts protein sequence files in ASSP (Annotated Staphylococcus aureus Protein) format or standard FASTA format containing putative protein translations from bacterial genomes.
- **Database Dependency**: The tool relies on a pre-built tetracycline resistance gene database created by the companion binary `bactopia-teton-build`. This database must exist before running analysis; without it, the tool will fail to find matching resistance determinants.
- **Output Files**: The tool produces a JSON report containing detected tetracycline resistance genes, their accession identifiers, sequence coverage, and amino acid identity matches. Results are organized by sample and include both gene name and coverage statistics.
- **Workflow Integration**: bactopia-teton is designed to operate within the broader Bactopia AMR pipeline, receiving input from upstream gene prediction tools (like Prokka) and passing results to downstream aggregation workflows.

## Pitfalls

- **Missing Database**: Running bactopia-teton without first running `bactopia-teton-build` to create the index will result in errors indicating the database cannot be found. Always execute the build command before analysis.
- **Incorrect Sequence Type**: Supplying nucleotide sequences instead of protein translations will produce zero results or false negatives, as the tool performs protein-to-protein alignment against the resistance gene database.
- **Duplicate Sample Names**: Using identical sample identifiers across multiple input files causes output file collisions, potentially overwriting results or mixing data between samples.
- **Insufficient Query Length**: Very short protein sequences (fewer than 10 amino acids) may fail to align reliably or produce spurious matches due to insufficient alignment length for statistical confidence.

## Examples

### Running basic tetracycline resistance detection
**Args:** --query sample_proteins.fasta --sample SAMPLE_001
**Explanation:** Executes tetracycline resistance gene detection on the provided protein sequences, using the default database and generating results saved with the sample identifier SAMPLE_001.

### Using a custom resistance gene database
**Args:** --query sample_proteins.fasta --db /path/to/custom_teton_db --sample MY_ISOLATE
**Explanation:** Runs detection against a user-built tetracycline database located at the specified path rather than the default bundled database.

### Specifying a custom output directory
**Args:** --query sample_proteins.fasta --sample ISOLATE_A --outdir ./amr_results
**Explanation:** Directs all output files to be written in the ./amr_results directory instead of the current working directory.

### Running with verbose logging for debugging
**Args:** --query sample_proteins.fasta --sample DEBUG_SAMPLE --verbose
**Explanation:** Enables detailed loggingoutput showing alignment scores, database matching statistics, and step-by-step progress for troubleshooting issues.

### Combining multiple samples in batch processing
**Args:** --querydir ./protein_files --sample_list samples.txt --outdir ./batch_results
**Explanation:** Processes multiple protein sequence files from a directory using a sample name mapping list, outputting all results to a shared directory for high-throughput analysis.