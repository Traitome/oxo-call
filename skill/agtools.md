---
name: agtools
category: bioinformatics/genome-annotation
description: A suite of command-line tools for manipulating, converting, and analyzing genome annotations in GFF3, BED, GTF, and other common formats. Includes utilities for extracting sequences, filtering features, computing statistics, and format conversion.
tags: [genome-annotation, GFF3, BED, GTF, sequence-extraction, bioinformatics]
author: AI-generated
source_url: https://github.com/AGTools-project/agtools
---

## Concepts

- agtools operates through subcommands (e.g., `extract`, `filter`, `stats`, `bedgen`) that perform specific operations on genome annotation files (typically GFF3, BED, or GTF format).
- Input files can be specified via stdin using `-` or as positional arguments, allowing pipeline integration with other bioinformatics tools like `bgzip`, `tabix`, and sequence aligners.
- The `extract` subcommand retrieves nucleotide or protein sequences from FASTA files using genomic coordinates or feature IDs from annotation files, supporting both BED interval and GFF3 feature-based extraction.
- Filtering and statistics subcommands accept feature type filters (e.g., `--type mRNA`, `--type gene`) and attribute filters (e.g., `--attribute gene_id=.*`) for refined analysis.
- Output formats are controlled via flags like `-o` for file output or can be piped to stdout; format conversion between BED, GFF3, and GTF is supported through specialized subcommands.

## Pitfalls

- Specifying incorrect feature types (e.g., using `--type exon` when the GFF3 uses lowercase `exon`) results in zero features being processed, silently producing empty output files.
- Forgetting to index input BED or GFF3 files with `bgzip` and `tabix` causes significant performance degradation on large genomes, and some subcommands may fail entirely on unindexed files over 100MB.
- Mixing coordinate systems (1-based GFF3 start positions vs 0-based BED start positions) when creating custom interval files leads to off-by-one errors in sequence extraction and downstream analysis.
- Using overlapping feature sets without understanding agtools' default behavior (which processes each feature independently) can produce duplicate sequences in output.
- Not specifying the correct genome assembly via `--genome` or `--assembly` when extracting sequences results in coordinates being interpreted incorrectly, producing sequences from the wrong chromosomal regions.

## Examples

### Extract sequences using a BED file
**Args:** `extract input.bed reference.fasta -o extracted_seqs.fasta`
**Explanation:** Retrieves FASTA sequences from reference.fasta for each interval defined in input.bed, outputting to extracted_seqs.fasta.

### Filter GFF3 by feature type and output statistics
**Args:** `filter input.gff3 --type gene --attribute product=hydrolase | stats --count`
**Explanation:** Filters GFF3 features to retain only gene features with product attribute containing "hydrolase", then pipes to statistics to count matching features.

### Convert GFF3 to BED format
**Args:** `bedgen input.gff3 -o genes.bed`
**Explanation:** Converts gene features from GFF3 format to BED format, writing 0-based coordinate BED records to genes.bed for downstream tools like UCSC utilities.

### Count features by type from indexed GFF3
**Args:** `count input.gff3.gz --group-by type`
**Explanation:** Counts features grouped by their feature type column from a tabix-indexed GFF3 file, producing a tab-separated summary of feature type frequencies.

### Extract protein sequences using gene IDs
**Args:** `extract --features gene_ids.txt genome.fasta --fasta-output protein_seqs.fasta --translation-table 1`
**Explanation:** Extracts translated protein sequences for a list of gene IDs from a genomic FASTA file using the standard genetic code (table 1), outputting protein sequences in FASTA format.