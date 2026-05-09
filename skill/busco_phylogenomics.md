---
name: busco_phylogenomics
category: Genome Assembly Assessment / Phylogenomics
description: Assess genome assembly and annotation completeness using single-copy orthologs (BUSCO) for phylogenomic analyses. This tool compares input sequences against curated universal ortholog datasets to evaluate genome quality and identify single-copy markers suitable for phylogenetic reconstruction.
tags: [busco, orthologs, genome-quality, phylogenomics, assembly-assessment, single-copy-genes, benchmarking]
author: AI-generated
source_url: https://busco.ezlab.org/
---

## Concepts

- **Input modes**: BUSCO supports multiple input modes including `genome` (raw genome assemblies), `proteins` (predicted protein sets), and `transdecoder` (predicted coding sequences from transcriptomes). Selecting the correct mode determines which analysis pipeline is applied.
- **Lineage datasets**: BUSCO provides pre-computed ortholog datasets for over 100 reference lineages (e.g., `vertebrata_odb10`, `fungi_odb10`, `bacteria_odb10`). The chosen lineage defines the expected single-copy orthologs used for benchmarking; using a too-distant lineage reduces sensitivity while using a too-specific lineage reduces coverage.
- **Output interpretation**: Results include Complete (C), Fragmented (F), and Missing (M) ortholog counts. The BUSCO percentage (C + F divided by total) indicates assembly completeness. For phylogenomics, high complete single-copy ortholog count without duplicates is critical for ortholog selection.
- **Batch processing for multiple genomes**: The `-l` flag can be combined with batch mode to process multiple genomes sequentially, generating comparative metrics suitable for phylogenetic sampling design.

## Pitfalls

- **Using wrong lineage dataset**: Selecting an inappropriate lineage (e.g., fungi lineage for plant data) causes false negatives because the tool searches for orthologs not expected in that clade. This reduces the identified single-copy gene set and may bias phylogenetic conclusions.
- **Insufficient CPU allocation**: Default single-threaded execution is prohibitively slow for large genomes. Under-resourcing CPU (`-c 1`) on large assemblies can cause runs to take days; allocate at least half the available cores for reasonable runtime.
- **Ignoring fragmented hits**: High fragmented counts combined with low complete counts often indicate incomplete genome annotation or assembly gaps. Treating these as acceptable without verification leads to poor phylogenetic markers with ambiguous orthology assignments.
- **Outdated BUSCO database**: Running without `--updated` or using cached outdated lineage databases misses newly identified core orthologs. This reduces the total candidate gene pool available for phylogenomics analyses.

## Examples

### Assess genome assembly completeness using vertebrate lineage
**Args:** -i assembly.fasta -o busco_output -m genome -l vertebrata_odb10 -c 8
**Explanation:** This runs BUSCO in genome mode on a vertebrate genome assembly, using the vertebrate lineage dataset and 8 CPU cores for parallel HMMER search, producing completeness statistics for phylogenomic quality assessment.

### Evaluate protein prediction from transcriptome assembly
**Args:** -i proteins.fasta -o proteome_busco -m proteins -l metazoa_odb10 -c 12
**Explanation:** This assesses predicted protein sequences (e.g., from Trinity or Oyster) against the Metazoa ortholog dataset, determining how many single-copy orthologs are represented in the predicted proteome.

### Run in offline mode without internet access
**Args:** -i genome.fasta -o offline_run -m genome -l bacteria_odb10 --offline -c 4
**Explanation:** This runs BUSCO without attempting to download or update lineage datasets, useful for secure or offline HPC environments where network access is restricted.

### Restart an interrupted run efficiently
**Args:** -i genome.fasta -o busco_restart -m genome -l insecta_odb10 -c 16 -r
**Explanation:** This restarts a previous incomplete run, skipping already completed ortholog searches and resuming only failed or pending batches, saving substantial computation time on large datasets.

### Generate short summary only for quick assessment
**Args:** -i genome.fasta -o quick_busco -m genome -l fungi_odb10 -c 8 --short
**Explanation:** This runs BUSCO with abbreviated output (summary only), reducing I/O overhead for rapid quality checks during iterative assembly improvement workflows where full detailed reports are not needed.