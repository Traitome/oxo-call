---
name: bactopia
category: Bioinformatics Workflows / Bacterial Genomics
description: A flexible and scalable bacterial whole genome sequencing (WGS) analysis workflow built on Nextflow. Bactopia automates read QC, assembly, annotation, antimicrobial resistance detection, MLST, and phylogenetic analysis.
tags: bacterial-genomics, wgs, nextflow, assembly, annotation, antimicrobial-resistance, mlst, phylogenetics, snp-analysis
author: AI-generated
source_url: https://bactopia.github.io/
---

## Concepts

- **Input Format**: Bactopia accepts Illumina paired-end or single-end FASTQ files organized in a directory structure. Paired-end reads require distinct R1 and R2 files with standard naming conventions (e.g., `sample_R1_001.fastq.gz` and `sample_R2_001.fastq.gz`).
- **Output Artifacts**: The workflow produces multiple output files including assembled contigs (FASTA), annotated genomes (GBK/JSON), detailed reports (JSON/HTML), antimicrobial resistance predictions, MLST sequence types, and species calls.
- **Dataset System**: Bactopia includes a companion dataset system (`bactopia-build`) that allows users to create custom datasets for reference-based SNP calling, gene detection databases, and species-specific parameters.
- **Execution Backends**: The workflow supports multiple execution platforms including local, Slurm, SGE, PBS, and containerized execution via Singularity or Docker.
- **Configuration**: All pipeline parameters are controlled via `--param` files in JSON or YAML format, making reproducible research straightforward.

## Pitfalls

- **Insufficient Disk Space**: Assembly files and intermediate results can consume tens of gigabytes per sample. Failing to pre-allocate adequate storage leads to workflow termination mid-execution.
- **Memory and Time Limits**: Large bacterial genomes (>6 Mb) or high-depth datasets require substantial RAM. Underestimating resource requirements causes the workflow to crash during memory-intensive steps like assembly or annotation.
- **File Naming Issues**: Bactopia requires strict file naming conventions. Non-standard filenames or inconsistent naming across samples cause the sample sheet parser to fail and samples to be skipped silently.
- **Incompatible Read Types**: Mixing single-end and paired-end reads in a single run without proper configuration leads to incorrect processing. The `--prefer-shortest` parameter exists for specific scenarios but may produce unexpected results.
- **Missing Dependencies**: Some Bactopia modules require external tools like ABRicate, Prokka, or Snippy. Failing to install these dependencies or dataset files results in incomplete analysis outputs.

## Examples

### Running Basic Whole Genome Sequencing Analysis
**Args:** `--wfs wf-annex` `--samples /path/to/sample_directory`
**Explanation:** Runs the annexation workflow, which performs QC, assembly, annotation, and basic trait detection on all FASTQ files found in the specified directory.

### Running with Custom Parameters
**Args:** `--wfs wf-full` `--param custom_params.yaml` `--outdir /results/output`
**Explanation:** Executes the full workflow using a custom parameter file to override default settings, with results written to the specified output directory instead of the default location.

### Using an Existing Dataset
**Args:** `--datasets /path/to/bactopia-datasets` `--species_list staph aureus`
**Explanation:** Uses a pre-built dataset for Staphylococcus aureus, enabling species-specific SNP calling, AMR gene detection, and accurate MLST typing for that organism.

### Specifying a Sample Sheet
**Args:** `--sample_sheet /path/to/samples.csv` `--fastq_dir /path/to/fastq_files`
**Explanation:** Uses a CSV-formatted sample sheet to explicitly map sample names to their corresponding FASTQ files, useful when filenames don't follow the default naming pattern.

### Executing on an HPC with Slurm
**Args:** `-profile slurm` `-with-singularity` `--outdir /scratch/results`
**Explanation:** Launches the workflow on a Slurm-managed HPC cluster using Singularity containers for environment isolation, writing outputs to the specified scratch directory.

### Running Without Gene Detection
**Args:** `--wfs wf-ont` `--skip_abricate` `--skip_mlst`
**Explanation:** Runs a specialized ONT (Oxford Nanopore) workflow but skips AMR gene detection and MLST, useful when only assembly and annotation are required.

### Using Multiple Datasets
**Args:** `--datasets /path/to/dataset1:/path/to/dataset2` `--species_list "staph aureus:salmonella"`
**Explanation:** Combines multiple custom datasets for different organisms, enabling parallel analysis when processing samples from multiple bacterial species.