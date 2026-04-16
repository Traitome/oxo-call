---
name: agat
category: annotation
description: Another GTF/GFF Analysis Toolkit — comprehensive suite for GFF/GTF file manipulation, format conversion, statistics, and quality checking
tags: [gff, gtf, annotation, gene-model, format-conversion, statistics, genome, gxf, fix, filter]
author: oxo-call built-in
source_url: "https://github.com/NBISweden/AGAT"
---

## Concepts

- AGAT provides 80+ tools for GFF3/GTF manipulation, statistics, format conversion, filtering, and quality checking. All tools are invoked as standalone commands: `agat_<prefix>_<name>` (e.g., `agat_sp_statistics`, `agat_convert_sp_gff2gtf`).
- Tool prefixes indicate processing mode: `agat_sp_` (smart parsing) loads the full file into memory, checks completeness, fixes errors, and infers missing features. `agat_sq_` (sequential) processes line-by-line with minimal memory — faster but no completeness checks or fixes. Always prefer `agat_sp_` tools unless memory is a constraint.
- Common input flag: `--gff` or `-f` or `-i` for input GFF/GTF file. Common output flag: `-o` or `--output` for output file (defaults to STDOUT if omitted).
- `agat_convert_sp_gxf2gxf` is the core fix/standardize tool: it removes duplicates, fixes duplicated IDs, adds missing ID/Parent attributes, adds missing features (e.g., exons from CDS, UTRs from CDS+exon), and sorts the output. Other `agat_sp_` tools run this parser automatically, so you do NOT need to run `agat_convert_sp_gxf2gxf` before other `agat_sp_` tools.
- AGAT handles non-standard GFF3/GTF files and can repair malformed annotations that other tools reject.
- Configuration: `agat config --expose` creates an `agat_config.yaml` in the working directory to customize output format (GFF3 vs GTF), verbosity, etc. AGAT uses the config from the working directory when present.
- Feature levels: `agat levels --expose` creates a `feature_levels.yaml` defining feature type hierarchy and relationships.
- AGAT output defaults to GFF3; change to GTF via the config file or `--gtf` flag where supported.

## Pitfalls

- In AGAT v1.7.0+, the installed command names do NOT have the `.pl` suffix. Use `agat_convert_sp_gff2gtf` (not `agat_convert_sp_gff2gtf.pl`). The `.pl` names appear in help output and older documentation but are not available as executables in conda/pixi installations.
- AGAT tools are standalone commands (not subcommands). ARGS starts with the tool name (e.g., `agat_sp_statistics --gff file.gff`). Do NOT prefix with `agat` — each tool is invoked directly.
- AGAT may auto-fix GFF3 errors silently — check output for unexpected changes, especially when processing well-formed files.
- Coordinate systems: GFF3 is 1-based closed [start, end]; BED is 0-based half-open [start, end). After converting GFF→BED with `agat_convert_sp_gff2bed`, coordinates are automatically adjusted.
- Very large GFF3 files may be slow with `agat_sp_` tools (full memory load) — use `agat_sq_` tools for simple operations on huge files, but note they skip completeness checks.
- `agat_sp_statistics` counts isoform lengths cumulatively by default, which can make total mRNA length exceed genome size. It automatically computes statistics twice when isoforms are present (with and without keeping longest isoform).
- Input compressed files: AGAT accepts `.gz` compressed GFF/GTF files directly (detected by `.gz` extension).

## Examples

### convert GFF3 to GTF format
**Args:** `agat_convert_sp_gff2gtf --gff annotation.gff3 -o annotation.gtf`
**Explanation:** --gff specifies input GFF3; -o writes GTF output; handles feature hierarchy automatically; 7 GTF types available (1, 2, 2.1, 2.2, 2.5, 3, relax)

### get comprehensive annotation statistics
**Args:** `agat_sp_statistics --gff annotation.gff3 -o statistics_report.txt`
**Explanation:** outputs gene count, mRNA count, exon statistics, intron size distribution; when isoforms are present, computes statistics twice (with all isoforms and with longest isoform only)

### filter genes by minimum length
**Args:** `agat_sp_filter_gene_by_length --gff annotation.gff3 --size 300 --test ">=" -o filtered.gff3`
**Explanation:** --size 300 sets the length threshold; --test ">=" keeps genes ≥300 bp; creates two output files (pass and fail)

### fix and standardize a malformed GFF3 file
**Args:** `agat_convert_sp_gxf2gxf --gff malformed.gff3 -o fixed.gff3`
**Explanation:** repairs common GFF3 errors: removes duplicates, fixes duplicated IDs, adds missing ID/Parent attributes, adds missing features (exon from CDS, UTR from CDS+exon), sorts output

### extract CDS sequences from a genome using GFF annotations
**Args:** `agat_sp_extract_sequences --gff annotation.gff3 -f genome.fa -t cds -o cds_sequences.fa`
**Explanation:** -f provides the reference genome FASTA; -t cds extracts CDS features; handles split features (e.g., CDS across multiple exons) by merging chunks

### keep only the longest isoform per gene
**Args:** `agat_sp_keep_longest_isoform --gff annotation.gff3 -o longest_isoforms.gff3`
**Explanation:** for each locus, keeps the isoform with the longest CDS (or longest concatenated exons if no CDS); reduces annotation complexity

### merge multiple GFF3 annotation files
**Args:** `agat_sp_merge_annotations --gff annot1.gff3 --gff annot2.gff3 -o merged.gff3`
**Explanation:** --gff can be specified multiple times or point to a directory; the AGAT parser handles duplicated names and fixes oddities

### manage and standardize feature IDs
**Args:** `agat_sp_manage_IDs --gff annotation.gff3 --prefix gene -o re_ided.gff3`
**Explanation:** --prefix sets the ID prefix; IDs are reformatted as prefix.letterCode.number; useful for ensuring unique, consistent identifiers

### convert GFF3 to BED format with coordinate adjustment
**Args:** `agat_convert_sp_gff2bed --gff annotation.gff3 -o annotation.bed`
**Explanation:** converts GFF3 (1-based closed) to BED (0-based half-open) with automatic coordinate adjustment

### expose and modify AGAT configuration
**Args:** `config --expose`
**Explanation:** creates agat_config.yaml in the working directory; modify to change output format (GFF3/GTF), verbosity, and other defaults; AGAT auto-detects config in the working directory

### split a large GFF3 file into smaller files
**Args:** `agat_sq_split --gff large_annotation.gff3 -i 500 -o split_output`
**Explanation:** -i 500 sets 500 genes per file; uses sequential processing (low memory); output files are named split_output_1.gff, split_output_2.gff, etc.

### add intron features to a GFF3 file that only has exons
**Args:** `agat_sp_add_introns --gff annotation.gff3 -o with_introns.gff3`
**Explanation:** creates intron features between consecutive exons within each transcript; required by some downstream tools (e.g., rMATS)
