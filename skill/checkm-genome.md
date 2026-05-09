---
name: checkm-genome
category: Genome Assembly Quality Assessment
description: Estimates genome completeness and contamination of microbial genome assemblies by identifying single-copy marker genes from lineage-specific marker sets. Part of the CheckM package, it provides statistics for bins, draft genomes, and metagenome-assembled genomes.
tags: [genome-quality, completeness, contamination, marker-genes, microbial-genomics, bins, checkm]
author: AI-generated
source_url: https://github.com/Ecogenomics/CheckM
---

## Concepts

- **Lineage-specific marker sets**: checkm-genome uses curated single-copy marker genes organized by taxonomic lineage (e.g., g__Proteobacteria) to estimate genome completeness. These markers are species-appropriate and provide more accurate quality estimates than universal marker sets alone.

- **Quality estimation model**: Completeness is calculated as the percentage of expected marker genes detected at sufficient coverage, while contamination is estimated by counting additional partial or divergent marker copies. An "uncertainty" value reflects ambiguity due to fragmented or split marker sequences.

- **File I/O formats**: Input accepts directories of genome files (default extension `.fna`) or single genome files. Output can be in tab-separated format (`-o 2`) or the default formatted tree view. The `-f` flag overwrites existing output files; without it, checkm-genome aborts if output already exists.

- **Bin scoring and identification**: The `-b` (bin_by_bin) flag restricts analysis to the most prevalent bin when a folder contains multiple bins, useful for MAG (metagenome-assembled genome) quality reporting. Single genomes bypass this behavior automatically.

- **Tree database dependency**: checkm-genome requires the CheckM reference tree database (`checkm data set` or bundled `CheckMData>`) containing lineage-specific marker gene profiles. Without this database, analysis fails with an "Unable to load NCBI taxonomy" error.

## Pitfalls

- **Missing or outdated CheckM database**: Running checkm-genome without the reference data causes errors like "Error: Unable to load NCBI taxonomy." Always run `checkm data set` before first use, or use the `--data_dir` flag pointing to an existing CheckMData directory.

- **Incorrect file extension filtering**: Using `-x fasta` when input files use `.fna` (or vice versa) results in zero genomes analyzed and no output. Verify actual file extensions in the input directory before specifying the extension filter.

- **Intermixing bins and draft genomes**: Placing both multi-sample bin folders and single genome files in the same input directory produces misleading per-sample statistics because checkm treats all files as belonging to one sample rather than separating them.

- **Conflicting output file overwrite**: Without the `-f` flag, checkm-genome silently skips analysis if the output file already exists, producing no error message but also no results. Always use `-f` when intentionally re-running analysis or verify output file absence beforehand.

- **Insufficient threads for large datasets**: The default single-threaded mode (`-t 1`) becomes prohibitively slow for directories containing hundreds of genomes. Use `-t 8` or higher to enable parallel processing, significantly reducing wall-clock time on multi-core systems.

## Examples

### Assess completeness and contamination of draft genomes in a folder

**Args:** `-q -x fna /path/to/genomes output.tsv`
**Explanation:** The `-q` flag suppresses progress messages, `-x fna` filters for FASTA nucleotide files, and the results are written to `output.tsv` in tab-separated format for downstream parsing.

### Generate a formatted tree view with default extensions

**Args:** `-i /path/to/genomes -o 1 > checkm_output.txt`
**Explanation:** The `-o 1` flag produces the default formatted tree view written to stdout, which is redirected to a file. This format is human-readable but not ideal for automated processing.

### Overwrite existing output and limit to specific bins per sample

**Args:** `-f -i /path/to/genomes -o 2 -b > output.tsv`
**Explanation:** The `-f` flag forces overwrite of existing files, `-o 2` produces tab-separated output, and `-b` restricts analysis to the most prevalent bin per sample, useful for MAG quality reporting workflows.

### Analyze with 16 threads for large genome collections

**Args:** `-t 16 -f -i /path/to/genomes -o 2 -x fna > output.tsv`
**Explanation:** The `-t 16` flag enables parallel processing across 16 CPU cores, `-f` permits file overwrite, and combined with `-o 2` and `-x fna`, produces tab-separated output for a directory of FASTA genome files.

### Run quiet mode with custom CheckM database path

**Args:** `-q -f --data_dir /custom/checkm_data -i /path/to/genomes -o 2 > output.tsv`
**Explanation:** The `--data_dir` flag specifies a non-default CheckM database location, `-q` suppresses screen output, and `-f` allows overwriting, while `-o 2` produces machine-readable tab-separated results.