---
name: prodigal
category: annotation
description: Fast prokaryotic gene prediction tool for bacteria and archaea genomes and metagenomes
tags: [annotation, gene-prediction, prokaryote, bacteria, archaea, metagenomics, orf]
author: oxo-call built-in
source_url: "https://github.com/hyattpd/Prodigal"
---

## Concepts
- Prodigal predicts protein-coding genes in prokaryotic (bacterial/archaeal) genomes and metagenomes.
- Two modes: single genome mode (default, trains on input genome) and metagenomic mode (-p meta).
- Use -i for input FASTA; -a for protein output FASTA; -d for nucleotide CDS FASTA; -f for output format.
- Output formats: -f gbk (GenBank, default), -f gff (GFF3), -f sco (summary).
- Use -p meta for metagenomic sequences (fragmented/short contigs) or unknown organisms.
- Prodigal is fast and accurate for prokaryotes; not suitable for eukaryotes (use Augustus or BRAKER).
- The -g flag specifies genetic code: -g 4 for Mycoplasma/Spiroplasma (TGA = Trp); default is standard code 11.
- -c (closed ends) prevents genes from running off contig edges; produces only complete genes.
- -m treats runs of N as masked sequence; prevents gene predictions across assembly gaps.
- -n bypasses Shine-Dalgarno trainer and forces full motif scan; for unusual genomes.
- -s outputs all potential genes with scores; useful for evaluating prediction confidence.
- -t writes or reads a training file; speeds up repeated analysis of similar genomes.

## Pitfalls
- Prodigal only works for prokaryotes — do NOT use for eukaryotic gene prediction.
- Without -p meta, Prodigal trains on the input sequence — short/fragmented sequences need -p meta mode.
- The default output (-o) is GenBank format; specify -f gff3 for GFF3 format.
- Prodigal does not predict tRNAs or rRNAs — use Aragorn and Barrnap for those.
- Very short sequences (<20 kb) may not provide enough training data for single mode — use -p meta.
- The protein file (-a) header format includes gene coordinates and strand — useful for downstream analysis.
- -c (closed ends) may miss true genes at contig boundaries; use only when complete genes are required.
- -m is essential for assemblies with gaps (N's); prevents prediction errors across scaffold gaps.
- -n is slower but more accurate for genomes with unusual Shine-Dalgarno sequences.
- Partial genes (01, 10, 11 in output) indicate incomplete predictions at contig edges.
- Training files (-t) save time when analyzing multiple similar genomes (e.g., strains).

## Examples

### predict genes in a bacterial genome and output protein and GFF files
**Args:** `-i genome.fasta -a proteins.faa -d cds.fna -f gff -o gene_predictions.gff`
**Explanation:** -i input genome; -a protein FASTA output; -d CDS nucleotide FASTA; -f gff format; -o GFF output

### predict genes in metagenomic contigs
**Args:** `-i metagenomic_contigs.fasta -a meta_proteins.faa -d meta_cds.fna -f gff -o meta_genes.gff -p meta`
**Explanation:** -p meta mode for metagenomes or mixed/unknown organisms; handles fragmented sequences

### predict genes with non-standard genetic code (Mycoplasma)
**Args:** `-i mycoplasma_genome.fasta -a mycoplasma_proteins.faa -f gff -o mycoplasma_genes.gff -g 4`
**Explanation:** -g 4 specifies genetic code 4 (TGA codes for Trp instead of stop); for Mycoplasma/Spiroplasma

### predict genes and output in GenBank format for import into annotation tools
**Args:** `-i assembly.fasta -a proteins.faa -f gbk -o predictions.gbk`
**Explanation:** -f gbk GenBank format; compatible with many genome browsers and annotation tools

### predict only complete genes (closed ends)
**Args:** `-i genome.fasta -a proteins.faa -f gff -o genes.gff -c`
**Explanation:** -c prevents genes from running off contig edges; outputs only complete genes

### mask assembly gaps during gene prediction
**Args:** `-i genome.fasta -a proteins.faa -f gff -o genes.gff -m`
**Explanation:** -m treats N's as masked; prevents gene predictions across assembly gaps

### bypass Shine-Dalgarno trainer for unusual genomes
**Args:** `-i genome.fasta -a proteins.faa -f gff -o genes.gff -n`
**Explanation:** -n forces full motif scan; for genomes with atypical ribosome binding sites

### output all potential genes with scores
**Args:** `-i genome.fasta -a proteins.faa -f gff -o genes.gff -s all_genes.txt`
**Explanation:** -s outputs all potential genes with scores; evaluate prediction confidence

### train on one genome and apply to another
**Args:** `-i genome1.fasta -t genome1.trn` then `-i genome2.fasta -t genome1.trn -a proteins.faa -f gff -o genes.gff`
**Explanation:** -t writes training file from genome1; reuse with genome2 for consistent parameters

### quiet mode for batch processing
**Args:** `-i genome.fasta -a proteins.faa -f gff -o genes.gff -q`
**Explanation:** -q suppresses stderr output; useful for batch processing many genomes

### predict genes from stdin
**Args:** `cat genome.fasta | prodigal -a proteins.faa -f gff -o genes.gff`
**Explanation:** reads from stdin when -i is omitted; useful for piped workflows
