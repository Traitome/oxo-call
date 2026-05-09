---
name: abricate
category: antimicrobial_resistance_screening
description: Screen nucleotide contigs against curated databases of antimicrobial resistance (AMR) genes and virulence factors to identify potential resistance profiles or virulence determinants in bacterial genomes.
tags: amr, antimicrobial_resistance, virulence, card, vfdb, resfinder, gene_screening, genome_analysis
author: AI-generated
source_url: https://github.com/tseemann/abricate
---

## Concepts

- **Database Selection**: ABRicate supports multiple curated databases including CARD (Comprehensive Antibiotic Resistance Database), VFDB (Virulence Factor Database), NCBI, ResFinder, and MEGARes. Each database serves a different purpose—CARD for AMR, VFDB for virulence factors—so selecting the appropriate database is critical for meaningful results.
- **Identity and Coverage Thresholds**: The tool matches sequences usingPercent Identity and coverage calculations. The default thresholds are 80% identity and 80% coverage, but these can be adjusted via `--minid` and `--mincov` flags. Lowering thresholds increases sensitivity but may introduce false positives, especially for fragmented assemblies.
- **Input/Output Formats**: ABRicate accepts FASTA or multi-FASTA input files (either plain or gzipped). Output formats include tab-delimited (default), CSV (`--csv`), JSON (`--json`), and a tabular format (`--tab`) showing gene, database, coverage, identity, and accession information.
- **Batch Processing**: Multiple input files can be processed in a single run by providing them as space-separated arguments or by using a filename pattern. Results are concatenated in the output if `--summary` is specified, facilitating comparison across samples.
- **Database Management**: Use `abricate-check` to verify installed databases, `abricate-get` to download additional databases, and `abricate-build` to create custom databases from user-provided FASTA files with gene annotations.

## Pitfalls

- **Using Default Thresholds on Poor-Quality Assemblies**: Fragmented assemblies with many small contigs may generate false positive matches because low coverage regions can meet the default 80% thresholds. This leads to inflated resistance gene counts and misinterpretation of AMR potential.
- **Neglecting Database Updates**: Databases like CARD and VFDB are regularly updated with new resistance genes and alleles. Running ABRicate with outdated databases without checking (`abricate-check`) may miss newly emergent resistance mechanisms or virulence factors.
- **Ignoring Strand Orientation**: ABRicate reports matches on both forward and reverse strands but does not flip reverse complement matches to the canonical gene orientation by default. This can cause confusion when comparing gene lists across samples if strand information is not considered.
- **Misinterpreting Partial Matches as Full Genes**: A match meeting the minimum threshold does not guarantee a functional gene product. Partial genes, truncated open reading frames, or pseudogenes may be reported as hits, potentially overestimating the organism's resistance phenotype.
- **Inconsistent Database Naming Across Analyses**: Different databases use different gene nomenclature (e.g., `blaTEM-1` in CARD vs. `TEM-1` in ResFinder). Mixing database results without accounting for nomenclature differences can lead to duplicate counting or missing relevant genes in comparative analyses.

## Examples

### Screen a genome assembly against the CARD database for AMR genes
**Args:** `--db card assembly contigs.fasta`
**Explanation:** Runs the assembly against the CARD database, the most comprehensive collection of antimicrobial resistance gene sequences and mutations, to identify known resistance determinants.

### Screen a genome against VFDB for virulence factors
**Args:** `--db vfdb sample contigs.fasta`
**Explanation:** Checks the input contigs against the Virulence Factor Database to identify genes encoding toxins, adhesins, secretion systems, and other virulence-associated proteins.

### Adjust minimum identity to 90% to reduce false positives
**Args:** `--minid 0.90 genome.fasta`
**Explanation:** Requires a 90% sequence identity match, stricter than the default 80%, which reduces spurious hits from low-identity or fragmented matches while still capturing closely related alleles.

### Output results in CSV format for spreadsheet analysis
**Args:** `--csv --db card sample.fasta`
**Explanation:** Generates comma-separated output that can be directly imported into Excel or R for downstream statistical analysis or visualization.

### Check which databases are currently installed
**Args:** `--check`
**Explanation:** Lists all available ABRicate databases along with their version information, allowing verification of database integrity before running analyses.

### Generate a summary report across multiple genomes
**Args:** --summary --db card sample1.fasta sample2.fasta sample3.fasta`
**Explanation:** Produces a concatenated tabular summary showing gene presence/absence and coverage for each genome, facilitating rapid comparative analysis across multiple isolates.

### Use a custom minimum coverage of 95% for high-confidence matches
**Args:** `--mincov 0.95 --db resfinder contig.fasta`
**Explanation:** Requires 95% alignment coverage, ensuring that the matched region spans nearly the entire reference gene, which is important for detecting complete resistance gene cassettes.

### Output results in JSON format for programmatic parsing
**Args:** `--json --db card assembly.fa`
**Explanation:** Provides machine-readable JSON output suitable for integration into automated pipelines or for parsing by scripts that require structured data.