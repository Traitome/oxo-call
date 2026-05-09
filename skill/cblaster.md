---
name: cblaster
category: Comparative Genomics
description: A tool for detecting clusters of homologous genes across multiple fungal and oomycete genomes. cblaster identifies gene clusters based on sequence homology and physical proximity, enabling detection of horizontal gene transfer events, lineage-specific expansions, and biosynthetic gene clusters.
tags: [gene-clustering, comparative-genomics, homology-search, fungi, oomycetes, blast, horizontal-gene-transfer]
author: AI-generated
source_url: https://github.com/quadram-institute/cblaster
---

## Concepts

- **Data Model**: cblaster takes query sequences (FASTA format) and searches them against a database of genomes. Genomes can be provided as GFF3, BED, or GenBank files containing gene annotations. The tool identifies clusters where multiple homologous genes are located within a specified physical distance (controlled by `--cluster-distance`).
- **I/O Formats**: Input requires query sequences in FASTA format and genome databases in GFF3, BED, or GenBank format. Output can be generated as JSON (machine-readable), tabular text, or HTML with visualization. The `--format` flag controls output type.
- **Key Behaviors**: cblaster uses BLAST+ for homology detection, then groups hits that cluster together based on genomic proximity. The `--cluster-distance` parameter (default 15000 bp) defines the maximum intergenic distance allowed between genes in the same cluster. Filtering by `--identity` (default 30%) and `--coverage` (default 50%) refines homology stringency.
- **Subcommands**: The tool offers search (primary homology search), plot (visualization), extract (sequence retrieval), and merge (combining results) subcommands. Each operates independently with its own flag set.

## Pitfalls

- **Missing Genome Database**: Running cblaster without defining the genome database (`--database` or `--organisms`) results in an error. You must specify at least one valid genome source containing gene annotations in GFF3, BED, or GenBank format.
- **Inappropriate Cluster Distance**: Setting `--cluster-distance` too small (e.g., 1000 bp) may split biologically relevant clusters, while overly large values (e.g., 500000 bp) merge distinct genomic regions into false positives. The default 15000 bp works for most fungal genomes but may need adjustment for species with larger intergenic regions.
- **Low Identity Threshold**: Using `--identity` below 25% risks detecting spurious homologies unrelated to genuine gene family relationships, generating false clusters. This is especially problematic in species with high sequence divergence or ancient duplication events.
- **No Hits Returned**: If no hits are found, verify that query sequences are in correct FASTA format, evalue threshold is appropriate (`--evalue` default 0.001), and target genomes actually contain homologous genes. Check input file paths and genome annotation formats.
- **Memory with Large Queries**: Processing thousands of query sequences against many genomes simultaneously can exhaust RAM. For large analyses, split queries into batches or reduce the number of target organisms.

## Examples

### Search for gene clusters using a query FASTA file
**Args:** search --query genes.fa --organisms fungi_db/ --output results.json
**Explanation:** This performs a homology search of sequences in genes.fa against the fungi_db genome database, saving results to results.json for downstream analysis.

### Search with strict identity and coverage thresholds
**Args:** search --query query.fa --database genome.gff --identity 70 --coverage 80 --evalue 1e-10 --output strict_hits.tsv
**Explanation:** This applies stringent filtering requiring 70% sequence identity and 80% coverage with a stricter evalue of 1e-10, filtering out weak or partial matches.

### Adjust cluster distance for large genomes
**Args:** search --query my_genes.fa --organisms large_genomes/ --cluster-distance 50000 --output wide_clusters.json
**Explanation:** This increases the maximum intergenic distance to 50000 bp to account for larger genomes where genes may be more spread out within functional clusters.

### Extract protein sequences from identified clusters
**Args:** extract --input clusters.json --output_dir extracted_seqs/ --format fasta
**Explanation:** This retrieves the protein sequences associated with clusters defined in clusters.json and writes them to the specified output directory in FASTA format.

### Generate HTML visualization of cluster results
**Args:** plot --input results.json --output clusters.html --title "Gene Family Analysis"
**Explanation:** This creates an interactive HTML visualization showing the genomic distribution and relationships of identified gene clusters.

### Merge results from multiple searches
**Args:** merge --input results1.json results2.json --output combined.json --format json
**Explanation:** This combines two separate cblaster result files into a single JSON file for unified analysis and visualization.

### Search with tabular output for easy parsing
**Args:** search --query targets.fa --organisms db/ --output table.tsv --format table
**Explanation:** This outputs results in a tab-separated table format ideal for parsing by scripts or importing into spreadsheets for manual review.

### Search against specific organisms in the database
**Args:** search --query biosynthetic_genes.fa --organisms aspergillus --output aspergillus_clusters.json
**Explanation:** This restricts the search to only Aspergillus species in the database, useful when analyzing gene families within a specific genus.