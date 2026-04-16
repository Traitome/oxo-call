---
name: busco
category: assembly
description: Benchmarking Universal Single-Copy Orthologs — genome and proteome completeness assessment using OrthoDB lineages
tags: [assembly, completeness, annotation, ortholog, genome, quality-assessment, eukaryote, bacteria, miniprot, odb12]
author: oxo-call built-in
source_url: "https://busco.ezlab.org/"
---

## Concepts

- BUSCO assesses genome/proteome completeness by searching for universal single-copy ortholog genes from OrthoDB.
- Use -i for input file; -o for output name; -l for lineage dataset (e.g., bacteria_odb12, eukaryota_odb12, vertebrata_odb12).
- Use -m to specify mode: genome (for genome assembly), proteins (for proteome), transcriptome (for transcriptome).
- Download lineage datasets with 'busco --download lineage_dataset' or use --auto-lineage for automatic selection.
- BUSCO 6 defaults to odb12 datasets (OrthoDB v12) — odb10 is legacy but still supported via --datasets_version odb10.
- Results summary: Complete (C), Complete Single-copy (S), Complete Duplicated (D), Fragmented (F), Missing (M).
- High-quality genome: >90% complete single-copy BUSCOs (C:>90% [S:>90%, D:<5%], F:<5%, M:<5%).
- Use -c N for multi-threading; --offline for offline mode with pre-downloaded datasets.
- Gene predictors for eukaryotes: --augustus (classic), --miniprot (fast, default in v6), --metaeuk (alternative).
- --long enables Augustus self-training optimization (slower but better for non-model organisms).
- --plot generates a summary plot from multiple BUSCO runs in a directory.
- --restart continues a previously interrupted run from the last checkpoint.

## Pitfalls

- busco has NO subcommands. ARGS starts directly with flags (e.g., -i, -o, -l, -m, -c). Do NOT put a subcommand like 'assess' or 'run' before flags.
- Choosing the wrong lineage (-l) gives misleading completeness scores — use the most specific clade available.
- For eukaryotic genomes, a gene predictor must be specified: --augustus, --miniprot, or --metaeuk.
- BUSCO 6 defaults to odb12 datasets; odb10 is deprecated — use --datasets_version odb10 only for backward compatibility.
- For large genomes, BUSCO is slow in genome mode — use proteins mode if a proteome is already available.
- The lineage dataset must be downloaded before offline use — internet access is needed for first-time use.
- The output name (-o) should not include path separators — use --out_path for output directory.
- --long significantly increases runtime but improves accuracy for non-model organisms.

## Examples

### assess completeness of a bacterial genome assembly
**Args:** `-i genome_assembly.fasta -o busco_bacteria -l bacteria_odb12 -m genome -c 8`
**Explanation:** -l bacteria_odb12 bacterial lineage dataset (odb12 is default); -m genome mode; -c 8 threads; output in busco_bacteria/

### assess completeness of a eukaryotic genome assembly with miniprot
**Args:** `-i eukaryote_assembly.fasta -o busco_euk -l eukaryota_odb12 -m genome --miniprot -c 16`
**Explanation:** --miniprot uses Miniprot gene predictor (fast, recommended for v6); eukaryota_odb12 is the broadest eukaryote dataset

### assess eukaryotic genome with Augustus and self-training
**Args:** `-i genome.fasta -o busco_augustus -l vertebrata_odb12 -m genome --augustus --long -c 16`
**Explanation:** --augustus uses Augustus gene predictor; --long enables self-training for non-model organisms (slower but more accurate)

### assess proteome completeness from predicted proteins
**Args:** `-i proteins.faa -o busco_proteome -l fungi_odb12 -m proteins -c 8`
**Explanation:** -m proteins for proteome mode; faster than genome mode; -l fungi_odb12 for fungal proteins

### assess transcriptome completeness
**Args:** `-i transcriptome.fasta -o busco_transcriptome -l vertebrata_odb12 -m transcriptome -c 8`
**Explanation:** -m transcriptome mode; -l vertebrata_odb12 for vertebrate transcriptome completeness

### run BUSCO with automatic lineage detection
**Args:** `-i genome.fasta -o busco_autolineage -m genome --auto-lineage -c 16`
**Explanation:** --auto-lineage automatically selects best lineage; useful when unsure of phylogenetic placement

### run BUSCO with Metaeuk gene predictor
**Args:** `-i genome.fasta -o busco_metaeuk -l insecta_odb12 -m genome --metaeuk -c 16`
**Explanation:** --metaeuk uses Metaeuk gene predictor; alternative to Augustus and Miniprot

### generate summary plot from multiple BUSCO runs
**Args:** `--plot /path/to/busco_results/ --plot_percentages`
**Explanation:** --plot generates summary bar chart from all short_summary.txt files in directory; --plot_percentages shows percentages instead of counts

### continue an interrupted BUSCO run
**Args:** `--restart -i genome.fasta -o busco_run -l bacteria_odb12 -m genome -c 8`
**Explanation:** --restart continues from last checkpoint; useful for long-running analyses that were interrupted

### list all available BUSCO datasets
**Args:** `--list-datasets`
**Explanation:** prints all available lineage datasets (archaea, bacteria, eukaryota, viruses); use before --download

### download a specific lineage dataset for offline use
**Args:** `--download vertebrata_odb12`
**Explanation:** downloads lineage dataset to local cache; enables --offline mode for future runs
