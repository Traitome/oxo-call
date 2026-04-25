---
name: liftoff
category: annotation
description: Accurate transfer of genome annotations between assemblies of the same or related species
tags: [annotation, liftover, gff, genes, genome, comparative-genomics, transfer]
author: oxo-call built-in
source_url: "https://github.com/agshumate/Liftoff"
---

## Concepts
- Liftoff maps annotations (GFF3/GTF) from a reference assembly to a target assembly using minimap2 alignments.
- Gene models are lifted by aligning each annotated sequence individually, then finding the best placement in the target genome.
- Multi-copy genes are handled with --copies; Liftoff reports unmapped features in a separate file (unmapped_features.txt by default).
- The -g flag specifies the source annotation GFF3/GTF file; -db can specify a pre-built gffutils database for faster repeated runs.
- Liftoff works best when source and target are the same species or closely related; divergent sequences lower mapping accuracy.
- -infer_transcripts and -infer_genes can fill in missing transcript/gene records when the source annotation lacks them.
- -polish re-aligns exons to fix start/stop codon issues and inframe stop codons; increases accuracy but runtime.
- -cds annotates CDS status (partial, missing start/stop, inframe stop codon) in the output GFF3.
- -chroms TXT maps corresponding chromosomes between reference and target; format: ref_chr,target_chr per line.
- -unplaced TXT handles unplaced contigs after chromosome mapping; useful for mapping genes from scaffolds.
- Alignment thresholds: -s (sequence identity, default 0.5), -a (coverage, default 0.5); adjust for cross-species.
- -mm2_options customizes minimap2 parameters; default is "-a --end-bonus 5 --eqx -N 50 -p 0.5".
- -d (distance scaling factor, default 2.0) controls how far apart alignment nodes can be to be connected.
- -flank (0.0-1.0) adds flanking sequence as fraction of gene length; helps when gene structure differs.
- -exclude_partial writes partial mappings below thresholds to unmapped file instead of main output.

## Pitfalls
- Not providing the source genome FASTA (-s or positional) causes Liftoff to fail; both reference FASTA and target FASTA are required.
- GFF3 files with non-standard feature types may not lift correctly; check that parent-child relationships use standard SO terms.
- Liftoff outputs GFF3 by default; use -f or check the output for coverage_tag to assess per-feature lift quality.
- Very fragmented target assemblies cause many features to be unmapped; contig N50 should be larger than the largest gene.
- Running without -dir leaves minimap2 index files in the working directory; use -dir to specify a scratch directory.
- Lifted annotations may have sequence_ID mismatches if target contig names differ; check that FASTA headers match expected names.
- -polish significantly increases runtime; only use when CDS accuracy is critical.
- -copies with -sc (copy sequence identity) must be > -s (mapping identity); default -sc=1.0 may miss recent duplicates.
- -chroms file format: one line per chromosome pair "ref_chr,target_chr"; no header, comma-separated.
- -unplaced should contain reference contig names only; Liftoff finds best target location automatically.
- Default -s 0.5 and -a 0.5 are permissive; increase for same-species (0.95+) or decrease for cross-species (0.7-0.8).
- -infer_genes and -infer_transcripts assume standard exon/CDS structure; may not work with all annotation formats.
- KeyError for scaffold names: ensure all sequence IDs in GFF3 exist in reference FASTA headers.

## Examples

### lift annotations from reference GFF3 to a new assembly
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -g reference.gff3 source annotation; -o lifted.gff3 output GFF3; -u unmapped.txt unmapped features

### lift annotations and copy multi-copy gene families
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -copies -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -g reference.gff3 source annotation; -o lifted.gff3 output GFF3; -copies search for gene copies; -u unmapped.txt unmapped features

### lift annotations between closely related species with lower identity threshold
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -s 0.85 -a 0.85 -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -g reference.gff3 source annotation; -o lifted.gff3 output GFF3; -s 0.85 sequence identity threshold; -a 0.85 coverage threshold; -u unmapped.txt unmapped features

### speed up repeated runs using a pre-built gffutils database
**Args:** `target.fasta reference.fasta -db reference.db -o lifted.gff3 -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -db reference.db gffutils database; -o lifted.gff3 output GFF3; -u unmapped.txt unmapped features

### lift annotations and write output to a specific directory with minimap2 intermediates
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -dir scratch_dir/ -p 16 -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -g reference.gff3 source annotation; -o lifted.gff3 output GFF3; -dir scratch_dir/ intermediate files directory; -p 16 threads; -u unmapped.txt unmapped features

### lift only specific feature types (e.g., just genes)
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -f gene -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -g reference.gff3 source annotation; -o lifted.gff3 output GFF3; -f gene filter feature type; -u unmapped.txt unmapped features

### polish lifted annotations to fix CDS issues
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted_polished.gff3 -polish -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -g reference.gff3 source annotation; -o lifted_polished.gff3 output GFF3; -polish re-align exons; -u unmapped.txt unmapped features

### annotate CDS status in output
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -cds -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -g reference.gff3 source annotation; -o lifted.gff3 output GFF3; -cds annotate CDS status; -u unmapped.txt unmapped features

### lift annotations with chromosome mapping for large genomes
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -chroms chroms.txt -unplaced unplaced.txt -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -g reference.gff3 source annotation; -o lifted.gff3 output GFF3; -chroms chroms.txt chromosome mapping; -unplaced unplaced.txt unplaced contigs; -u unmapped.txt unmapped features

### find gene copies with relaxed identity threshold
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -copies -sc 0.95 -s 0.9 -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -g reference.gff3 source annotation; -o lifted.gff3 output GFF3; -copies search for copies; -sc 0.95 copy identity threshold; -s 0.9 mapping identity threshold; -u unmapped.txt unmapped features

### lift with custom minimap2 options for divergent genomes
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -mm2_options="-a --end-bonus 5 --eqx -N 100 -p 0.3" -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -g reference.gff3 source annotation; -o lifted.gff3 output GFF3; -mm2_options="-a --end-bonus 5 --eqx -N 100 -p 0.3" minimap2 parameters; -u unmapped.txt unmapped features

### exclude partial mappings from main output
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -s 0.9 -a 0.9 -exclude_partial -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -g reference.gff3 source annotation; -o lifted.gff3 output GFF3; -s 0.9 identity threshold; -a 0.9 coverage threshold; -exclude_partial exclude partial mappings; -u unmapped.txt unmapped features

### lift with flanking sequence for improved alignment
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -flank 0.1 -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -g reference.gff3 source annotation; -o lifted.gff3 output GFF3; -flank 0.1 add 10% flanking sequence; -u unmapped.txt unmapped features

### lift annotations with inferred gene and transcript features
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -infer_genes -infer_transcripts -u unmapped.txt`
**Explanation:** liftoff command; target.fasta target assembly; reference.fasta reference assembly; -g reference.gff3 source annotation; -o lifted.gff3 output GFF3; -infer_genes create gene features; -infer_transcripts create transcript features; -u unmapped.txt unmapped features
