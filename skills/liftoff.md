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
**Explanation:** positional args are target then reference FASTA; -g is source annotation; -o output GFF3; -u lists unmapped features

### lift annotations and copy multi-copy gene families
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -copies -u unmapped.txt`
**Explanation:** -copies searches for additional copies of each gene in the target beyond the best single hit

### lift annotations between closely related species with lower identity threshold
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -s 0.85 -a 0.85 -u unmapped.txt`
**Explanation:** -s 0.85 sets minimum sequence identity to 85%; -a 0.85 sets minimum alignment coverage — useful for cross-species lift

### speed up repeated runs using a pre-built gffutils database
**Args:** `target.fasta reference.fasta -db reference.db -o lifted.gff3 -u unmapped.txt`
**Explanation:** -db uses a pre-built gffutils database instead of re-parsing the GFF3 each run; build with gffutils first

### lift annotations and write output to a specific directory with minimap2 intermediates
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -dir scratch_dir/ -p 16 -u unmapped.txt`
**Explanation:** -dir puts minimap2 intermediate files in scratch_dir/; -p 16 uses 16 parallel threads

### lift only specific feature types (e.g., just genes)
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -f gene -u unmapped.txt`
**Explanation:** -f gene lifts only features of type gene and their children; reduces output size for specific use cases

### polish lifted annotations to fix CDS issues
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted_polished.gff3 -polish -u unmapped.txt`
**Explanation:** -polish re-aligns exons to fix start/stop codon issues and inframe stops; creates {output}.gff and {output}_polished.gff

### annotate CDS status in output
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -cds -u unmapped.txt`
**Explanation:** -cds adds status tags (partial, missing_start, missing_stop, inframe_stop) to CDS features in output GFF3

### lift annotations with chromosome mapping for large genomes
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -chroms chroms.txt -unplaced unplaced.txt -u unmapped.txt`
**Explanation:** -chroms maps chr1->chr1, chr2->chr2 etc.; -unplaced handles scaffold genes after chromosomes; improves accuracy for complex genomes

### find gene copies with relaxed identity threshold
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -copies -sc 0.95 -s 0.9 -u unmapped.txt`
**Explanation:** -sc 0.95 for copy detection (must be > -s); finds recent duplicates; -s 0.9 for initial mapping

### lift with custom minimap2 options for divergent genomes
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -mm2_options="-a --end-bonus 5 --eqx -N 100 -p 0.3" -u unmapped.txt`
**Explanation:** custom minimap2 parameters; -N 100 increases secondary alignments, -p 0.3 lowers alignment score ratio for divergent genomes

### exclude partial mappings from main output
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -s 0.9 -a 0.9 -exclude_partial -u unmapped.txt`
**Explanation:** -exclude_partial writes mappings below -s/-a thresholds to unmapped file; main output contains only high-quality mappings

### lift with flanking sequence for improved alignment
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -flank 0.1 -u unmapped.txt`
**Explanation:** -flank 0.1 adds 10% flanking sequence to each gene; helps when gene structure differs between assemblies

### lift annotations with inferred gene and transcript features
**Args:** `target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -infer_genes -infer_transcripts -u unmapped.txt`
**Explanation:** -infer_genes and -infer_transcripts create parent features if GFF only has exon/CDS; useful for incomplete annotations
