---
name: AMDirT
category: Ancient Metagenomics / Paleogenomics
description: A modular toolkit for the analysis and retrieval of ancient metagenomic data. AMDirT provides subcommands for building taxonomic reference databases, read counting, sequence extraction, diversity profiling, and quality-based read filtering from BAM or FASTQ inputs.
tags:
  - ancient-dna
  - metagenomics
  - paleogenomics
  - taxonomic-profiling
  - read-counting
  - bioinformatics
  - amdirtool
author: AI-generated
source_url: https://github.com/rleroy247/AMDirT
---

## Concepts

- **Reference database construction with `amdirt-build`**: The `build` subcommand constructs a local taxonomic reference database from one or more FASTA files. Headers must follow a specific naming convention (e.g., `>species_name|taxid|...`) so that reads can be correctly assigned to taxa during downstream counting and extraction steps.

- **Read quantification with `amdirt-count`**: The `count` subcommand tallies reads mapped to each taxonomic node from a sorted, indexed BAM file aligned against the reference database. Output is typically a tab-delimited count matrix where rows are samples and columns are taxa, suitable for downstream statistical analysis in R or Python.

- **Sample metadata format requirements**: AMDirT subcommands that operate on sample sets (e.g., count, extract) require a metadata file (CSV or TSV) in which each row represents a sample and must contain at minimum columns for the sample identifier and the absolute path to the corresponding BAM file. Missing or malformed metadata is a frequent cause of silent failures where no reads are counted.

- **Read extraction and filtering**: The `extract` subcommand retrieves sequence reads belonging to specified taxa from source BAM or FASTQ files and can apply a mapping-quality (`-mq`) threshold to retain only high-confidence alignments. Extracted reads are written to individual FASTA/FASTQ files per taxon, enabling focused downstream analyses such as damage profiling.

- **Diversity profiling**: The `diversity` subcommand computes alpha-diversity metrics (e.g., Shannon index, observed taxa) from the count matrix produced by `count`, optionally stratified by sample group or time period defined in the metadata file. This allows cross-sample comparisons of microbial community diversity in ancient specimens.

- **Output format conventions**: By default, AMDirT writes tab-separated output files without a trailing newline on the last line, which can cause parsing issues in some downstream tools. Most R parsing functions handle this gracefully, but command-line utilities like `awk` may behave unexpectedly.

## Pitfalls

- **Missing or misnamed BAM index files**: Running `count` or `extract` on a BAM file without a corresponding `.bai` index causes the subcommand to fail with a non-obvious error about missing read groups or a silent zero-count output. Always ensure a sorted `.bam` file is accompanied by its `.bai` file before invoking AMDirT subcommands.

- **Mismatched FASTA headers between database and BAM alignments**: If the reference database was built with one header format (e.g., `seqid|species`) but the alignments were produced against a different naming scheme, `count` will report zero counts for every taxon despite successful SAM parsing. Verify header consistency before running any quantification subcommand.

- **Insufficient strand-bias or damage filtering for ancient samples**: By default, `extract` does not apply post-mortem damage (PMD) filtering, which means modern contamination can be over-represented in extracted reads. Skipping damage-based quality thresholds leads to inflated diversity estimates and potentially misleading biological conclusions.

- **Incorrect metadata column names or missing sample paths**: AMDirT expects exact column name matches for the sample ID and the BAM file path (typically `sample_id` and `bam_path`). If the metadata file uses different column names or contains empty path cells, the subcommand either skips those samples entirely or aborts with an unhelpful column-not-found error.

- **Using unsorted BAM input for `count`**: The `count` subcommand requires a coordinate-sorted or queryname-sorted BAM file for efficient iteration. Feeding an unsorted BAM file may produce incorrect read counts due to duplicate or out-of-order records, and in some versions causes a crash mid-execution.

- **Mixing sample sets with inconsistent reference databases**: If you build a reference database from one set of FASTA files and later run `count` on samples aligned against a different reference, the count matrix will be sparse or all-zero. Each sample set must be re-aligned and re-counted against the same database version to ensure cross-sample comparability.

## Examples

### Build a taxonomic reference database from a set of FASTA files
**Args:** `build --ref genus_list.fasta --output db_taxonomy`
**Explanation:** The `build` subcommand parses headers from `genus_list.fasta` using the embedded taxonomic identifiers and creates an indexed reference database at `db_taxonomy` for use by all other AMDirT subcommands.

### Count reads per taxon for a single sample using the assembled database
**Args:** `count --bam sample1.sorted.bam --db db_taxonomy --out sample1_counts.tsv`
**Explanation:** The `count` subcommand iterates over alignments in the sorted BAM, assigns each read to the best-matching taxon in the database, and writes a tab-delimited count table to `sample1_counts.tsv`.

### Count reads across multiple samples defined in a metadata file
**Args:** `count --metadata samples_meta.tsv --db db_taxonomy --out count_matrix.tsv`
**Explanation:** When a metadata TSV is provided, `count` processes all samples listed in the file in batch mode, producing a unified count matrix where columns are taxa and rows are sample identifiers.

### Extract high-quality reads for a specific genus with mapping-quality threshold
**Args:** `extract --db db_taxonomy --bam sample1.sorted.bam --taxon "Staphylococcus" --mq 30 --out staph_reads.fasta`
**Explanation:** The `extract` subcommand retrieves all reads assigned to Staphylococcus with mapping quality ≥ 30 and writes them to a single FASTA file, excluding low-quality alignments that could introduce contamination bias.

### Compute Shannon diversity index across sample groups defined in metadata
**Args:** `diversity --counts count_matrix.tsv --meta samples_meta.tsv --group period --out diversity_results.tsv`
**Explanation:** The `diversity` subcommand calculates Shannon alpha-diversity per sample and optionally compares groups defined by the `period` column in the metadata file, producing summary statistics for paleoecological interpretation.