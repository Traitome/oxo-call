---
name: clermontyping
category: Bioinformatics - Microbial Serotyping
description: In silico serotyping tool for Streptococcus pneumoniae and other pathogenic bacteria based on capsular polysaccharide synthesis (cps) locus sequence analysis. Identifies bacterial serotypes from genomic FASTA/FASTQ inputs using database-driven sequence homology matching.
tags: [serotyping, pneumococcus, streptococcus-pneumoniae, bacterial-typing, in-silico-typing, cps-locus, wgs, pathogen-identification]
author: AI-generated
source_url: https://github.com/pasteur这支/ClermonTyping
---

## Concepts

- **Input Formats**: Accepts FASTA (.fasta/.fa) and FASTQ (.fastq/.fq) genomic sequence files. The tool analyzes the complete or partial cps (capsular polysaccharide synthesis) locus to identify serotype-specific gene sequences and markers.

- **Serotype Database**: Uses a curated reference database of known serotype-specific sequences (Capsule synthesis genes like cpsA through cpsM). Each of the 90+ known S. pneumoniae serotypes has distinctive genetic signatures in the cps locus that clermontyping matches against.

- **Output Modes**: Produces results in multiple formats - plain text summary, tab-delimited table, or JSON. Output includes the predicted serotype, confidence score, and which specific genes/segments contributed to the identification.

- **Detection Threshold**: Uses a minimum sequence identity and coverage threshold (typically 90% identity over 80% coverage) to make a serotype call. Sequences below this threshold may be reported as "non-typeable" (NT) or unknown.

## Pitfalls

- **Incomplete Genome Assemblies**: Providing incomplete or highly fragmented genome assemblies can lead to false negative results because the cps locus may be split across multiple contigs, preventing proper detection of serotype-specific markers.

- **Database Version Mismatch**: Using an outdated database can cause misidentification of newly discovered serotypes (e.g., serotypes 6D, 11E, 23B) that were added in newer database versions. Always verify you have the latest database release.

- **Novel or Rare Serotypes**: The tool may incorrectly assign "non-typeable" or misidentify rare serotypes if the query sequence has insufficient homology to reference sequences. Always validate critical results with traditional PCR methods.

- **Mixed Infections**: Samples containing DNA from multiple serotypes can produce ambiguous or incorrect results, as the tool assumes a clonal infection. For epidemiological surveillance, ensure bacterial isolates are pure before sequencing.

- **Horizontal Gene Transfer**: Some serotype-specific genes can be transferred horizontally, leading to incorrect serotype assignments if the database doesn't account for recombinant variants or chimeric sequences.

## Examples

### Basic serotyping from a FASTA genome file

**Args:** -i SRR123456.fasta -o result.txt
**Explanation:** Runs serotype identification on the input genome FASTA file and writes results to result.txt. This is the most common usage for single genome analysis.

### Batch processing multiple genomes in a directory

**Args:** -i /path/to/genome_directory/ -o batch_results.txt --batch
**Explanation:** Processes all FASTA files in the specified directory and outputs a table of serotype results for each genome. Use --batch for high-throughput epidemiological studies.

### Output results in JSON format

**Args:** -i genome.fasta -o result.json --json
**Explanation:** Outputs serotype results in JSON format for easier parsing by downstream scripts or integration with reporting pipelines. JSON output includes confidence scores and matched gene details.

### Specify a custom database file

**Args:** -i genome.fasta -o result.txt -db custom_serotype_db.fasta
**Explanation:** Uses a custom database file instead of the default. Useful for analyzing non-pneumococcal species or adding local serotype variants to the reference set.

### Adjust minimum identity threshold for stricter matching

**Args:** -i genome.fasta -o result.txt --min-identity 95 --min-coverage 90
**Explanation:** Sets stricter matching thresholds (95% identity, 90% coverage) to reduce false positives at the cost of potentially increasing non-typeable results. Use for high-confidence applications.

### Generate verbose output with gene-level details

**Args:** -i genome.fasta -o verbose_result.txt --verbose
**Explanation:** Outputs detailed information about which specific cps genes were matched, their positions, and alignment scores. Essential for debugging ambiguous serotype calls.

### Export results in CSV format for spreadsheet analysis

**Args:** -i genome.fasta -o result.csv --csv
**Explanation:** Outputs serotype results in CSV format for easy import into Excel or statistical software. CSV includes sample name, serotype, and confidence columns.