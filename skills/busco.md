---
name: busco
category: assembly
description: Benchmarking Universal Single-Copy Orthologs — genome and proteome completeness assessment
tags: [assembly, completeness, annotation, ortholog, genome, quality-assessment, eukaryote, bacteria]
author: oxo-call built-in
source_url: "https://busco.ezlab.org/"
---

## Concepts

- BUSCO assesses genome/proteome completeness by searching for universal single-copy ortholog genes from OrthoDB.
- Use -i for input file; -o for output name; -l for lineage dataset (e.g., bacteria_odb10, eukaryota_odb10, vertebrata_odb10).
- Use -m to specify mode: genome (for genome assembly), proteins (for proteome), transcriptome (for transcriptome).
- Download lineage datasets with 'busco --download lineage_dataset' or use --auto-lineage for automatic selection.
- Results summary: Complete (C), Complete Single-copy (S), Complete Duplicated (D), Fragmented (F), Missing (M).
- High-quality genome: >90% complete single-copy BUSCOs (C:>90% [S:>90%, D:<5%], F:<5%, M:<5%).
- Use -c N for multi-threading; --offline for offline mode with pre-downloaded datasets.
- Use --augustus for eukaryotic gene prediction support (requires Augustus installed).

## Pitfalls

- Choosing the wrong lineage (-l) gives misleading completeness scores — use the most specific clade available.
- BUSCO requires Augustus for genome mode with eukaryotes — ensure Augustus is installed and configured.
- For large genomes, BUSCO is slow in genome mode — use proteins mode if a proteome is already available.
- The lineage dataset must be downloaded before offline use — internet access is needed for first-time use.
- The output name (-o) should not include path separators — use --out_path for output directory.
- BUSCO genome mode uses BLAST for initial mapping — ensure BLAST is in PATH.

## Examples

### assess completeness of a bacterial genome assembly
**Args:** `-i genome_assembly.fasta -o busco_bacteria -l bacteria_odb10 -m genome -c 8`
**Explanation:** -l bacteria_odb10 bacterial lineage dataset; -m genome mode; -c 8 threads; output in busco_bacteria/

### assess completeness of a eukaryotic genome assembly
**Args:** `-i eukaryote_assembly.fasta -o busco_euk -l eukaryota_odb10 -m genome -c 16`
**Explanation:** eukaryota_odb10 is the broadest eukaryote dataset; use vertebrata_odb10 or insecta_odb10 for more specific

### assess proteome completeness from predicted proteins
**Args:** `-i proteins.faa -o busco_proteome -l fungi_odb10 -m proteins -c 8`
**Explanation:** -m proteins for proteome mode; faster than genome mode; -l fungi_odb10 for fungal proteins

### assess transcriptome completeness
**Args:** `-i transcriptome.fasta -o busco_transcriptome -l vertebrata_odb10 -m transcriptome -c 8`
**Explanation:** -m transcriptome mode; -l vertebrata_odb10 for vertebrate transcriptome completeness

### run BUSCO with automatic lineage detection
**Args:** `-i genome.fasta -o busco_autolineage -m genome --auto-lineage -c 16`
**Explanation:** --auto-lineage automatically selects best lineage; useful when unsure of phylogenetic placement
