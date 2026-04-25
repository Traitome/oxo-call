---
name: prokka
category: metagenomics
description: Rapid prokaryotic genome annotation pipeline for bacteria, archaea, and viruses
tags: [annotation, genome, prokaryote, bacteria, gff, genbank, metagenomics, assembly]
author: oxo-call built-in
source_url: "https://github.com/tseemann/prokka"
---

## Concepts
- Prokka annotates assembled prokaryotic genomes; input is a nucleotide FASTA (contigs/scaffolds).
- Use --outdir to specify output directory and --prefix for output file prefix.
- Prokka outputs: GFF3, GenBank, FASTA (proteins and CDS), TSV table, and text statistics.
- Use --kingdom to specify organism type: Bacteria (default), Archaea, Mitochondria, Viruses.
- Use --genus and --species for taxonomy-specific database searches (improves annotation).
- Use --cpus N for parallel processing; --gram to specify Gram stain for signal peptide prediction.
- For metagenome-assembled genomes (MAGs), use --metagenome flag for more permissive annotation.
- Use --proteins to add custom protein FASTA database for annotation (e.g., organism-specific proteins).
- --usegenus uses genus-specific BLAST databases when combined with --genus.
- --gcode sets genetic code/translation table (default 11 for bacteria).
- --evalue sets BLAST e-value cutoff (default 1e-09); lower is more stringent.
- --coverage sets minimum coverage on query protein (default 80%).
- --rfam enables searching for ncRNAs with Infernal+Rfam (slow but thorough).
- --fast mode uses only basic BLASTP databases for faster annotation.
- --compliant enforces GenBank/ENA/DDBJ compliance standards.
- --addgenes adds 'gene' features for each 'CDS' feature.

## Pitfalls
- prokka has NO subcommands. ARGS starts directly with flags (e.g., --outdir, --kingdom, --cpus). Do NOT put a subcommand like 'annotate' or 'run' before flags. The input FASTA file is the last positional argument.
- Prokka requires contigs to be in multi-FASTA format — single sequences must be properly formatted.
- --outdir must be a new directory — Prokka will not overwrite existing output without --force.
- For repeat-heavy assemblies or metagenomes, use --metagenome to avoid missing fragmented features.
- Prokka uses a heuristic database search; custom --proteins databases greatly improve accuracy for specific taxa.
- Sequence IDs in the FASTA are used as scaffold names — ensure they are ≤20 characters for GenBank compatibility.
- --locustag sets the locus tag prefix for gene IDs; use a unique prefix per genome for multi-genome studies.
- --usegenus requires --genus to be specified; check available databases with --listdb.
- --rfam is very slow; only use when ncRNA annotation is critical.
- --fast mode skips many databases; use only for quick preliminary annotation.
- --compliant requires --centre to be set for GenBank submission compatibility.
- Lower --evalue increases stringency but may miss distant homologs.

## Examples

### annotate a bacterial genome assembly
**Args:** `--kingdom Bacteria --genus Escherichia --species coli --strain K12 --cpus 8 --outdir prokka_output --prefix ecoli_K12 assembly.fasta`
**Explanation:** prokka command; --kingdom Bacteria organism type; --genus Escherichia --species coli --strain K12 improve annotation; --cpus 8 threads; --outdir prokka_output output directory; --prefix ecoli_K12 output prefix; assembly.fasta input FASTA

### annotate a metagenome-assembled genome (MAG)
**Args:** `--metagenome --cpus 8 --outdir mag_annotation --prefix bin001 bin001_contigs.fasta`
**Explanation:** prokka command; --metagenome mode for MAGs; --cpus 8 threads; --outdir mag_annotation output directory; --prefix bin001 output prefix; bin001_contigs.fasta input FASTA; increases sensitivity for fragmented genomes

### annotate archaea genome
**Args:** `--kingdom Archaea --cpus 8 --outdir archaea_output --prefix archaea_sample archaea_assembly.fasta`
**Explanation:** prokka command; --kingdom Archaea switches annotation databases; --cpus 8 threads; --outdir archaea_output output directory; --prefix archaea_sample output prefix; archaea_assembly.fasta input FASTA

### annotate genome with custom protein database for improved annotation
**Args:** `--kingdom Bacteria --proteins custom_proteins.faa --cpus 8 --outdir custom_annotation --prefix sample genome.fasta`
**Explanation:** prokka command; --kingdom Bacteria organism type; --proteins custom_proteins.faa adds custom protein FASTA; --cpus 8 threads; --outdir custom_annotation output directory; --prefix sample output prefix; genome.fasta input FASTA; takes priority over defaults

### annotate genome and add specific locus tag prefix
**Args:** `--kingdom Bacteria --locustag MYORG --cpus 8 --outdir annotated --prefix genome_v1 assembly.fasta`
**Explanation:** prokka command; --kingdom Bacteria organism type; --locustag MYORG sets locus tag prefix for gene names; --cpus 8 threads; --outdir annotated output directory; --prefix genome_v1 output prefix; assembly.fasta input FASTA; important for GenBank

### annotate with genus-specific database
**Args:** `--kingdom Bacteria --genus Escherichia --usegenus --cpus 8 --outdir annotation --prefix ecoli assembly.fasta`
**Explanation:** prokka command; --kingdom Bacteria organism type; --genus Escherichia genus name; --usegenus uses genus-specific BLAST database; --cpus 8 threads; --outdir annotation output directory; --prefix ecoli output prefix; assembly.fasta input FASTA; improved accuracy

### annotate with custom genetic code
**Args:** `--kingdom Bacteria --gcode 4 --cpus 8 --outdir annotation --prefix myco assembly.fasta`
**Explanation:** prokka command; --kingdom Bacteria organism type; --gcode 4 uses Mycoplasma genetic code; --cpus 8 threads; --outdir annotation output directory; --prefix myco output prefix; assembly.fasta input FASTA; check NCBI genetic code tables

### annotate with stringent e-value
**Args:** `--kingdom Bacteria --evalue 1e-15 --cpus 8 --outdir annotation --prefix strict assembly.fasta`
**Explanation:** prokka command; --kingdom Bacteria organism type; --evalue 1e-15 increases annotation stringency; --cpus 8 threads; --outdir annotation output directory; --prefix strict output prefix; assembly.fasta input FASTA; higher confidence but may miss distant homologs

### annotate with Rfam ncRNA search
**Args:** `--kingdom Bacteria --rfam --cpus 8 --outdir annotation --prefix with_ncrna assembly.fasta`
**Explanation:** prokka command; --kingdom Bacteria organism type; --rfam enables Infernal+Rfam search for non-coding RNAs; --cpus 8 threads; --outdir annotation output directory; --prefix with_ncrna output prefix; assembly.fasta input FASTA; slow but thorough

### fast annotation mode
**Args:** `--kingdom Bacteria --fast --cpus 8 --outdir annotation --prefix quick assembly.fasta`
**Explanation:** prokka command; --kingdom Bacteria organism type; --fast uses only basic BLASTP databases; --cpus 8 threads; --outdir annotation output directory; --prefix quick output prefix; assembly.fasta input FASTA; much faster but less comprehensive

### annotate with GenBank compliance
**Args:** `--kingdom Bacteria --compliant --centre MYLAB --addgenes --cpus 8 --outdir annotation --prefix compliant assembly.fasta`
**Explanation:** prokka command; --kingdom Bacteria organism type; --compliant enforces GenBank standards; --centre MYLAB required; --addgenes adds gene features; --cpus 8 threads; --outdir annotation output directory; --prefix compliant output prefix; assembly.fasta input FASTA

### annotate with HMM database
**Args:** `--kingdom Bacteria --hmms trusted.hmm --cpus 8 --outdir annotation --prefix hmm assembly.fasta`
**Explanation:** prokka command; --kingdom Bacteria organism type; --hmms trusted.hmm uses trusted HMM profiles; --cpus 8 threads; --outdir annotation output directory; --prefix hmm output prefix; assembly.fasta input FASTA; priority annotation
