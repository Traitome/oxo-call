---
name: chewbbaca
category: Bioinformatics - Bacterial Typing
description: Tool for bacterial core genome MLST (cgMLST) analysis, enabling allele calling against schemas, schema creation and evaluation, and clustering for outbreak investigation.
tags: mlst, bacteria, genotyping, allele-calling, cgmlst, sequence-typing, bioinformatics
author: AI-generated
source_url: https://github.com/B-UMMI/chewBBACS
---

ChewBBACA is a Python-based bioinformatics tool for bacterial subspecies classification using Multilocus Sequence Typing (MLST) and core genome MLST (cgMLST). It performs allele calling by comparing assembled genomes against a defined schema of locus alleles, enabling high-resolution typing for molecular epidemiology and outbreak investigation.

## Concepts

- **Schema Structure**: A chewBBACA schema is a directory containing one FASTA file per locus, where each entry represents a known allele sequence with unique numeric or alphanumeric identifiers (e.g., `aroE_1`, `aroE_2`). The schema defines the gene targets used for typing and must contain allele calls for all loci to generate a complete sequence type.
- **Input Format**: ChewBBACA requires assembled genome FASTA files (contigs), not raw sequencing reads or quality-trimmed reads. Each input file should contain the complete or near-complete genome assembly, as partial assemblies may result in missing allele calls and incomplete profiles.
- **Allele Calling Algorithm**: The tool uses either BLAST or VSEARCH to align input genome sequences against schema alleles, selecting the best match based on sequence identity and coverage thresholds (default: 100% identity and 100% coverage for exact matches). Configured thresholds determine whether calls are exact, approximate, or uncertain.
- **Output Formats**: Results are produced as tab-delimited text files containing per-genome allele calls, sequence types (STs), and cgMLST allele distances. The `--export-alleles` flag exports called alleles as FASTA files for downstream analysis or schema expansion.
- **Schema Evaluation**: The `schema_eval` subcommand assesses schema quality by analyzing allele frequency distributions, detecting pseudogenes, and identifying loci with low discrimination power. This helps optimize schemas for specific bacterial populations.

## Pitfalls

- **Using Raw Sequencing Reads**: Feeding raw Illumina or Nanopore reads instead of assembled contigs will cause complete failure or meaningless results, as chewBBACA performs alignment against allele sequences and cannot assemble reads internally.
- **Mismatched Schema and Species**: Using a schema developed for one bacterial species (e.g., *Listeria monocytogenes*) against genomes from a different species (e.g., *Escherichia coli*) yields no valid allele calls or predominantly novel/unknown calls due to lack of sequence homology.
- **Loose Identity Thresholds**: Setting `--identity` below 0.95 or `--coverage` below 0.90 may incorrectly assign alleles from divergent homologs, producing misleading sequence types and false clustering relationships in outbreak analysis.
- **Insufficient Allele Representation**: Schemas with few alleles per locus (fewer than 5-10) provide poor discrimination and may group unrelated isolates as identical, reducing the resolution needed for epidemiological conclusions.
- **Inconsistent Schema Versions**: Running analyses with different schema versions across batches produces incomparable sequence types; schemas must remain fixed or explicitly versioned for reproducible longitudinal studies.

## Examples

### Perform allele calling on a single genome against an existing schema
**Args:** `allele_call -i genome.fasta -o results_output/ --schema schema_directory/`
**Explanation:** Aligns the assembled genome against all locus alleles in the schema and outputs tab-delimited allele calls for each gene, enabling sequence type assignment.

### Call alleles on multiple genomes in batch mode
**Args:** `allele_call -i genomes_directory/ -o batch_results/ --schema cgmlst_schema/ --cd --threads 8`
**Expalanation:** Processes all FASTA files in the input directory in parallel using 8 threads, producing per-genome allele tables and a consolidated matrix for comparative analysis.

### Evaluate the quality and discrimination power of a schema
**Args:** `schema_eval -sf schema_directory/ -i training_genomes/ -o eval_report.tsv`
**Explanation:** Analyzes how well alleles in the schema discriminate among the training genomes, identifying loci with low variation or potential issues like pseudogenes.

### Create a new schema from a set of reference genomes
**Args:** `create_schema -i reference_genomes/ -o new_schema/ --max-loci 50 --min-records 3 -- Representation 0.95`
**Explanation:** Generates a schema with up to 50 loci, requiring each allele to be present in at least 3 genomes and achieving 95% representation across input genomes.

### Export called alleles to FASTA files for further analysis
**Args:** `allele_call -i genome.fasta -o results/ --schema schema/ --export-alleles alleles_output/`
**Explanation:** Outputs both the allele call table and FASTA files containing the specific allele sequences called for each locus, useful for phylogenetic reconstruction.

### Generate a consensus sequence type from replicated isolates
**Args:** `consensus -i replicates_directory/ -o consensus_output/ --schema schema/ --method majority`
**Explanation:** Combines allele calls from multiple replicates of the same isolate and applies majority voting to resolve discrepant calls into a single consensus profile.

### Compute pairwise genetic distances between isolates
**Args:** `distances -i results_directory/ -o distance_matrix.tsv --schema schema/ --outfmt tsv`
**Explanation:** Calculates the number of allelic differences across all shared loci between all pairwise combinations of called genomes, producing a distance matrix for clustering.

### Filter results to exact-only calls and export as table
**Args:** `allele_call -i genome.fasta -o results/ --schema schema/ --export-table table.tsv --exact-only`
**Explanation:** Restricts output to alleles matching at 100% identity and coverage, producing a clean table with confirmed allele calls suitable for reporting.