---
name: busco
category: genome_quality_assessment
description: Assess genome assembly completeness and gene annotation quality using universal single-copy orthologs. BUSCO searches for highly conserved single-copy genes from selected lineage datasets to provide a quantitative measure of assembly and annotation completeness.
tags: [busco, genomics, assembly_quality, orthologs, benchmarking, genome_assessment]
author: AI-generated
source_url: https://busco.ezlab.org/
---

## Concepts

- BUSCO evaluates genome assemblies by searching for universal single-copy orthologs from a lineage-specific database. The lineage datasets (e.g., `eukaryota`, `bacteria`, `fungi`, `vertebrata`) contain hundreds of orthologs assumed to be single-copy in the target clade; their presence/completeness indicates assembly quality.
- The tool supports three input modes: `genome` (raw assembly contigs/scaffolds), `transcriptome` (predicted protein or nucleotide sequences from assembly), and `proteins` (annotated protein FASTA). Selecting the wrong mode produces meaningless results.
- BUSCO internally uses HMMER for sequence search and can invoke AUGUSTUS for ab initio gene prediction when assessing genome assemblies. Proper AUGUSTUS species configuration is critical for accurate gene prediction in genome mode.
- Output classification groups hits into: Complete and single-copy (C), Complete and duplicated (D), Fragmented (F), and Missing (M). The primary quality metric is the percentage of complete single-copy orthologs.
- Lineage datasets are hosted remotely and must be downloaded or specified by name (e.g., ` vertebrata_odb10`). Use `--download` with a dataset name to fetch required files.

## Pitfalls

- Specifying an incorrect or mismatched lineage dataset leads to unreliable assessments. For example, using a bacterial lineage dataset on eukaryotic genome data will search for bacterial orthologs that are absent, producing artificially high missing scores.
- Running BUSCO in genome mode without proper AUGUSTUS configuration causes the run to fail or hang. The AUGUSTUS species parameter (via `--augustus_species`) must match a trained species model available in the AUGUSTUS config directory.
- Using insufficient CPU threads (default is 1) makes large genome analyses extremely slow. For assemblies larger than 100 Mb, always specify `-c` with a higher thread count to parallelize HMMER searches.
- Attempting to resume an interrupted run without the `-f` flag fails because BUSCO refuses to overwrite existing output directories, leaving the analysis incomplete.
- Providing a transcriptome as input when expecting genome FASTA (or vice versa) produces zero hits or crashes, because the search algorithm expects specific sequence types for each mode.

## Examples

### Assess a eukaryotic genome assembly for completeness
**Args:** `-i assembly.fasta -o busco_output -m genome -l eukaryota_odb10 -c 8`
**Explanation:** This runs BUSCO in genome mode on a eukaryotic assembly, using the eukaryota lineage dataset and 8 CPU cores for faster HMMER searches.

### Run BUSCO on an annotated protein set
**Args:** `-i proteins.fasta -o busco_proteins -m proteins -l fungi_odb10 -c 4`
**Explanation:** Directly assesses an annotated protein FASTA file using the fungi lineage dataset, skipping gene prediction and evaluating protein presence.

### Download a lineage dataset before running analysis
**Args:** `--download archaea_odb10`
**Explanation:** Downloads the archaea ortholog dataset to the local BUSCO data folder so it can be used in subsequent runs without remote fetching.

### Resume an interrupted BUSCO run with forced restart
**Args:** `-i assembly.fasta -o busco_output -m genome -l eukaryota_odb10 -c 8 -f`
**Explanation:** The `-f` flag forces overwriting of the existing output directory, allowing a failed or interrupted run to restart from the beginning.

### Run quietly with minimal console output
**Args:** `-i assembly.fasta -o busco_output -m genome -l eukaryota_odb10 -c 8 -q`
**Explanation:** The `-q` flag suppresses most console messages, useful when running in batch scripts or redirecting logs.

### Specify a custom AUGUSTUS species for gene prediction
**Args:** `-i assembly.fasta -o busco_output -m genome -l eukaryota_odb10 --augustus_species human -c 8`
**Explanation:** Explicitly sets the AUGUSTUS species parameter to `human`, required for gene prediction when analyzing mammalian assemblies in genome mode.

### Assess a transcriptome assembly from raw reads
**Args:** `-i transcripts.fasta -o busco_transcriptome -m transcriptome -l eukaryota_odb10 -c 8`
**Explanation:** Runs in transcriptome mode, which first translates contigs into six-frame translations before searching for ortholog matches.