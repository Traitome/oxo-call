---
name: amptk
category: Bioinformatics Tools → Amplicon Sequencing
description: A toolkit for processing Illumina amplicon sequencing data, including demultiplexing, quality filtering, OTU clustering, and taxonomic assignment for metabarcoding applications such as fungal ITS and bacterial 16S analysis.
tags: amplicon, metabarcoding, ITS, 16S, demultiplex, clustering, FASTQ, Illumina, OTU, ASV
author: AI-generated
source_url: https://github.com/ampvis/amptk
---

## Concepts

- **Subcommand Structure**: amptk uses a subcommand-based interface where the primary `amptk` command invokes specific workflows via subcommands (e.g., `amptk illumina` for demultiplexing, `amptk cluster` for OTU clustering, `amptk filter` for quality filtering). The subcommand must be specified after `amptk`, not as a flag.
- **Paired-End Read Merging with `-p`**: For overlapping paired-end amplicon reads (typical of ITS or short marker genes), the `-p` flag in `amptk illumina` instructs the tool to merge forward and reverse reads using USEARCH/VSEARCH algorithms before downstream processing. This is essential for full-length amplicon reconstruction.
- **Input Requirements: Sample Mapping File**: All amptk subcommands require a mapping file (specified with `-i`) in CSV format that maps sample names to their corresponding index barcodes. Without this file, demultiplexing cannot assign reads to samples. The mapping file should have columns: `SampleID`, `ForwardPrimer`, `ReversePrimer`, `Barcode`, `FwdSeq`, `RevSeq`.
- **Clustering Methods**: `amptk cluster` supports three clustering strategies: `--dereplicate` for 100% identity clustering (unique sequences), `--de_novo` for de novo clustering at user-defined similarity (default 97%), and `--closed` for closed-reference clustering against a database. The choice impactsOTU resolution and biological interpretation.
- **Output Formats**: amptk produces standardized output files including: clustering results (FASTA), OTU table (CSV), and optional taxonomic assignments. The output files use the prefix specified by `-o` and always include biom-format OTU tables compatible with QIIME and Phyloseq for downstream diversity analysis.

## Pitfalls

- **Omitting the `-p` Flag for Overlapping Reads**: Running `amptk illumina` without `-p` on paired-end data where reads overlap (e.g., ITS1F/ITS4 amplicons) results in unmerged read pairs being treated as independent sequences, artificially inflating sequence counts and producing chimeric OTUs.
- **Using Default Quality Trimming Without Adjustment**: The default `-q` (quality threshold) and `-l` (minimum length) settings may be inappropriate for different amplicon lengths or sequencing chemistries. Using default 250 bp settings on 300 bp amplicons without adjustment loses valid reads, while overly permissive settings retain low-quality sequences that increase taxonomic misassignment.
- **Ignoring Primer Sequences inFASTQ**: Amptk expects demultiplexed FASTQ files with primer sequences still attached. If primer sequences have been previously stripped, the `-f` and `-r` flags for primer sequences may cause filtering failures or empty outputs. Ensure primer handling is consistent across the pipeline.
- **Confusing Cluster Methods Leading to Incompatible Outputs**: Using `--de_novo` clustering creates de novo OTUs that cannot be classified using closed-reference databases, while `--closed` requires a reference database in the correct format. Mixing methods without understanding their compatibility results in missing taxonomic assignments or empty OTU tables.
- **Specifying Incorrect File Paths for Input Files**: Amptk requires explicit paths to the forward reads (`-f`), reverse reads (`-r`), and mapping file (`-i`). Relative paths or incorrect filenames result in immediate tool failure with opaque error messages about missing files, causing workflow interruption.

## Examples

### Demultiplex paired-end Illumina amplicon FASTQ files
**Args:** `illumina -f reads_R1.fastq.gz -r reads_R2.fastq.gz -i sample_mapping.csv -o project_name -p`
**Explanation:** This subcommand demultiplexes the input FASTQ files using the barcode mapping, merges overlapping paired-end reads with `-p`, and outputs demultiplexed files with the prefix `project_name` for downstream processing.

### Cluster demultiplexed sequences into OTUs using de novo clustering
**Args:** `cluster -i project_name.demux.fasta -o project_name --de_novo --similarity 0.97`
**Explanation:** This performs de novo OTU clustering at 97% similarity on the demultiplexed FASTA file, creating OTUs without requiring a reference database, suitable for discovering novel diversity in unexplored environments.

### Filter sequences by quality and length thresholds
**Args:** `filter -i project_name.cluster.otu.fa -o project_name_filtered -q 30 -l 100`
**Explanation:** This applies a minimum quality score of 30 and minimum sequence length of 100 bp to remove low-quality or truncated sequences from the OTU FASTA file before taxonomic assignment or downstream analysis.

### Dereplicate sequences to obtain unique sequences (100% clustering)
**Args:** `cluster -i project_name.demux.fasta -o project_name_uniques --dereplicate`
**Explanation:** This collapses sequences to unique representative sequences at 100% identity, which is useful for generating a sequence inventory or for ASV approaches that require exact sequence variants rather than clustered OTUs.

### Assign taxonomy using a reference database
**Args:** `cluster -i project_name_filtered.otu.fa -o project_name_tax --closed -d UNITEITS_v123.fasta`
**Explanation:** This performs closed-reference clustering against the UNITE ITS database to assign taxonomic labels to OTUs based on sequence similarity to known reference sequences, enabling ecological interpretation.

### Extract a subset of samples from an OTU table
**Args:** `subset -i project_name.otu_table.csv -o project_subset -s sample1,sample2,sample3`
**Explanation:** This extracts only the specified samples from the OTU table, useful for comparing specific treatment groups or removing control samples before downstream statistical analysis.