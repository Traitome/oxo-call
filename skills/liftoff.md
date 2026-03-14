---
name: liftoff
category: genome-annotation
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

## Pitfalls

- Not providing the source genome FASTA (-s or positional) causes Liftoff to fail; both reference FASTA and target FASTA are required.
- GFF3 files with non-standard feature types may not lift correctly; check that parent-child relationships use standard SO terms.
- Liftoff outputs GFF3 by default; use -f or check the output for coverage_tag to assess per-feature lift quality.
- Very fragmented target assemblies cause many features to be unmapped; contig N50 should be larger than the largest gene.
- Running without -dir leaves minimap2 index files in the working directory; use -dir to specify a scratch directory.
- Lifted annotations may have sequence_ID mismatches if target contig names differ; check that FASTA headers match expected names.

## Examples

### lift annotations from reference GFF3 to a new assembly
**Args:** `liftoff target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -u unmapped.txt`
**Explanation:** positional args are target then reference FASTA; -g is source annotation; -o output GFF3; -u lists unmapped features

### lift annotations and copy multi-copy gene families
**Args:** `liftoff target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -copies -u unmapped.txt`
**Explanation:** -copies searches for additional copies of each gene in the target beyond the best single hit

### lift annotations between closely related species with lower identity threshold
**Args:** `liftoff target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -s 0.85 -a 0.85 -u unmapped.txt`
**Explanation:** -s 0.85 sets minimum sequence identity to 85%; -a 0.85 sets minimum alignment coverage — useful for cross-species lift

### speed up repeated runs using a pre-built gffutils database
**Args:** `liftoff target.fasta reference.fasta -db reference.db -o lifted.gff3 -u unmapped.txt`
**Explanation:** -db uses a pre-built gffutils database instead of re-parsing the GFF3 each run; build with gffutils first

### lift annotations and write output to a specific directory with minimap2 intermediates
**Args:** `liftoff target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -dir scratch_dir/ -p 16 -u unmapped.txt`
**Explanation:** -dir puts minimap2 intermediate files in scratch_dir/; -p 16 uses 16 parallel threads

### lift only specific feature types (e.g., just genes)
**Args:** `liftoff target.fasta reference.fasta -g reference.gff3 -o lifted.gff3 -f gene -u unmapped.txt`
**Explanation:** -f gene lifts only features of type gene and their children; reduces output size for specific use cases
