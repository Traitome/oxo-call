---
name: compleasm
category: Genome Assembly QC
description: Finds complete universal single-copy orthologs (USCOs) in genome assemblies to assess assembly quality. Compleasm identifies near-complete sets of conserved eukaryotic genes and reports their presence and completeness, providing metrics for assembly scaffold/contig N50, genome size estimation, and gene space coverage.
tags: [genome-assembly, orthologs, quality-control, busco, gene-completeness, assembly-qc, scaffolding]
author: AI-generated
source_url: https://github.com/amphioxys/compleasm
---

## Concepts

- Compleasm searches for universal single-copy orthologs (USCOs) from selectable lineage databases (e.g., arthropoda_odb10, fungi_odb10, eukaryota_odb10) in genome assemblies using local alignment with apanasin, producing completeness percentages similar to BUSCO but with faster local-only mode.
- Input requires a genome assembly in FASTA format (either nucleotide or protein) and a specified lineage database; the tool automatically detects input type and uses appropriate search mode (DNA or protein).
- Output includes multiple files: `full_table.tsv` with per-gene coordinates and scores, `short_summary.txt` with percentage metrics, `missing_list.tsv` with absent orthologs, and `hmmout.txt` with HMMER search results.
- The tool supports parallelization via `-t/--threads` to accelerate searches on multi-core systems, and supports `-m/--mode` choices like `genome`, `proteins`, or `transcripts` to match the input assembly type.
- Downloadable lineage databases are managed via `compleasm download` and stored in the user-configured database directory; always verify the selected lineage matches your organism's taxonomic group for meaningful results.

## Pitfalls

- Using a lineage database unrelated to your organism (e.g., running arthropoda on a fungal genome) produces meaningless completeness scores below 5% even for high-quality assemblies, wasting computation and leading to incorrect quality assessments.
- Providing a low-quality assembly with many gaps or ambiguous bases will yield low completeness percentages and inflate the "fragmented" count, creating false impressions of incomplete gene space rather than true biological absence.
- Ignoring the `missing_list.tsv` output hides critical information about which specific orthologs failed to align; without examining this, you cannot determine whether gaps in gene space are biological or assembly artifacts.
- Running without specifying `-t/--threads` defaults to single-threaded execution, making searches on large eukaryotic genomes extremely slow; on assemblies >100 Mb, this can take hours instead of minutes.
- Specifying protein input with nucleotide flags or vice versa degrades search sensitivity or causes failure; the tool attempts auto-detection but explicit mismatches between format and mode flags cause unexpected behavior.

## Examples

### Assess arthropod genome assembly quality using arthropoda lineage
**Args:** `-i mygenome.fa -o output_dir -l arthropoda_odb10 -t 8`
**Explanation:** Runs compleasm with the arthropoda lineage database on a nucleotide FASTA assembly, using 8 threads for parallel execution, producing completeness percentages for 99 arthropod-specific USCOs.

### Run protein-mode analysis on a protein FASTA file
**Args:** `-i myprotein.fa -o output_dir -l eukaryota_odb10 -m proteins -t 4`
**Explanation:** Performs protein-mode search directly against protein sequences rather than translating, which is faster and more accurate when protein assemblies are already available.

### Download a specific fungal lineage database
**Args:** `download -l fungi_odb10`
**Explanation:** Downloads and caches the Fungal OrthoDB v10 lineage database for subsequent compleasm runs, storing it in the configured database directory.

### View short summary without parsing full table
**Args:** `-i assembly.fa -o results -l metazoa_odb10`
**Explanation:** Produces a `short_summary.txt` file containing the main completeness metrics (C:%, F:%, M:%) without generating the detailed full_table for quick quality glance.

### Analyze transcriptome assembly for gene completeness
**Args:** `-i transcriptome.fa -o txp_results -l eukaryotic_odb10 -m transcripts -t 16`
**Explanation:** Runs compleasm in transcript mode on a transcriptome assembly, which treats introns as absent and evaluates only expressed gene content, using 16 threads for faster processing of long transcript sequences.