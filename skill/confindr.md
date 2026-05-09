---
name: confindr
category: assembly_qc
description: Detects contamination in microbial genome assemblies by comparing contigs against a database of known contaminant sequences including vectors, adapters, and common laboratory/organismal pathogens.
tags: [contamination, assembly_qc, microbial_genomics, fasta, quality_control, bioinformatics]
author: AI-generated
source_url: https://github.com/ctgakram/confindr
---

## Concepts

- **Input Format**: Confindr accepts FASTA format genome assemblies as input. Each contig in the assembly is independently analyzed for contamination signatures. The input can be a single genome or multiple genomes in one file.
- **Contaminant Database**: The tool uses a built-in database of common contaminants including cloning vectors (e.g., pBR322, pUC derivatives), adapter sequences, primer sequences, and known contaminant organisms (E. coli K-12, PhiX174, etc.). This database is automatically loaded unless a custom database is specified.
- **Output Reporting**: Results are written to an output directory containing files that list contaminated contigs, their coordinates, and the likely source of contamination (vector name, organism, or sequence type). Output formats include tabular text files and optional JSON.
- **Taxonomic Assignment**: When contamination is detected, confindr attempts to identify the taxonomic source (genus/species level when possible), allowing researchers to determine whether contamination originated from lab操作, other organisms, or sequencing artifacts.
- **Detection Sensitivity**: The tool compares sequence segments against the contaminant database using local alignment. Users can adjust parameters to balance sensitivity between detecting low-level contamination and avoiding false positives from homologous native sequence.

## Pitfalls

- **Ignoring Low-Copy Contamination**: Failing to check assemblies with default settings may miss subtle contamination present as short contigs or regions with high sequence divergence from the database. Low contamination levels can still confound downstream analyses like pangenome studies or variant calling.
- **Not Validating Results by Blast**: Confindr provides initial contamination flags but does not replace manual verification. Without running BLAST or mapping confirmatory analyses, false positives from regions sharing homology with contaminants (e.g., conserved housekeeping genes) can lead to incorrectly discarding valid assembly contigs.
- **Using Outdated Databases**: The built-in contaminant database may not include newer vectors or emerging contaminants common in sequencing facilities. Relying solely on the default database risks missing recently introduced实验室 contaminants that have become prevalent in modern sequencing workflows.
- **Assuming Zero Contamination Means Clean**: An assembly passing without flagged contamination does not guarantee it is free of all contaminants. Confindr cannot detect novel contaminants not in its database, organisms with high sequence divergence, or contamination from sources with no available reference sequence.
- **Overlooking Single-Contig Assemblies**: For draft assemblies with very few contigs, contamination of a single contig represents a significant fraction of the assembly. Treating results equally regardless of assembly completeness can mask serious contamination issues in nearly complete genomes.

## Examples

### Detect contamination in a bacterial genome assembly

**Args:** `-i assembly.fasta -o contamination_results`

**Explanation:** This runs confindr on a FASTA assembly file and writes results to the specified output directory, producing reports listing any contaminated contigs identified against the built-in database.

### Use a custom contaminant database

**Args:** `-i genome.fasta -o results -db custom_contaminants.fasta`

**Explanation:** Specifying a custom database allows detection of facility-specific vectors, primers, or other sequences not included in the default confindr database, improving detection accuracy for known local contaminants.

### Run with verbose output for troubleshooting

**Args:** `-i input.fasta -o output -v`

**Explanation:** Verbose mode prints additional diagnostic information during execution, helping identify where failures occur if the run terminates unexpectedly or produces unexpected results.

### Adjust minimum alignment length for detection

**Args:** `-i assembly.fasta -o results -ml 50`

**Explanation:** Setting minimum alignment length to 50 base pairs increases specificity for longer contaminant fragments while reducing false positives from short spurious matches, useful for assemblies with many short contigs.

### Output results in JSON format for programmatic parsing

**Args:** `-i genome.fasta -o results --json`

**Explanation:** JSON output provides structured data suitable for integration into bioinformatics pipelines or automated downstream analysis, facilitating batch processing of multiple genome assemblies.

### Specify a custom output filename prefix

**Args:** `-i assembly.fasta -o results -p my_sample`

**Explanation:** Using a custom prefix instead of the default output naming allows organizing results from multiple samples in a common directory without filename conflicts, streamlining high-throughput workflows.