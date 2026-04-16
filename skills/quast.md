---
name: quast
category: assembly
description: Quality assessment tool for genome assemblies with reference-based and reference-free metrics
tags: [assembly, quality-assessment, genome, n50, contigs, benchmarking, metagenome]
author: oxo-call built-in
source_url: "https://quast.sourceforge.net/"
---

## Concepts
- QUAST assesses assembly quality with metrics: N50, N90, total length, number of contigs, misassemblies, and more.
- Use -r for reference genome (enables reference-based metrics); without -r, only reference-free metrics are computed.
- Use -g for gene annotation GFF/GTF (enables gene coverage metrics).
- Multiple assemblies can be compared in one run: quast assembly1.fasta assembly2.fasta -o quast_output/
- Use --threads N for parallel processing; --min-contig for minimum contig length threshold (default: 500 bp).
- metaQUAST (for metagenomes) automatically downloads reference genomes and aligns contigs.
- QUAST outputs HTML report, PDF, and TSV table in the output directory.
- Key metrics: N50 (contig length where 50% of assembly is in contigs ≥ this size), L50 (number of contigs ≥ N50).
- --k-mer-stats computes k-mer-based quality metrics; recommended for large genomes.
- --eukaryote and --fungus flags optimize gene prediction for eukaryotic/fungal genomes.
- --rna-finding predicts ribosomal RNA genes using Barrnap.
- --conserved-genes-finding counts conserved orthologs using BUSCO.
- --circos generates Circos visualization plots.
- --large uses optimal parameters for large genomes (mammalian-size).

## Pitfalls
- quast has NO subcommands. ARGS starts directly with input files or flags (e.g., -r, -o, --threads). Do NOT put a subcommand like 'assess' or 'analyze' before flags.
- Without -r reference, misassembly detection is not possible — always provide reference for isolate assemblies.
- The reference genome must match the genome build used for assembly for accurate misassembly detection.
- --min-contig default (500 bp) excludes short contigs from metrics — adjust based on your assembly.
- For fragmented assemblies, NG50 (based on genome size) is more informative than N50.
- metaQUAST requires internet access for downloading reference genomes automatically.
- QUAST HTML reports require a browser — download locally if running on a server.
- --k-mer-stats increases memory and time consumption; use with caution on large genomes.
- --eukaryote and --fungus are required for proper gene prediction in non-prokaryotic genomes.
- --conserved-genes-finding requires BUSCO installation; only works on Linux.
- --large flag is recommended for genomes >100 Mbp; automatically sets appropriate parameters.

## Examples

### assess assembly quality with reference genome
**Args:** `-r reference.fasta -g genes.gff assembly.fasta -o quast_output/ --threads 8`
**Explanation:** -r reference for comparison; -g gene annotation; output in quast_output/; generates HTML report

### compare multiple assemblies without reference genome
**Args:** `spades_assembly.fasta megahit_assembly.fasta flye_assembly.fasta -o assembly_comparison/ --threads 8`
**Explanation:** multiple assembly FASTAs compared side-by-side; --no-icarus to skip large interactive browser

### assess metagenome assembly quality with metaquast.py
**Args:** `metaquast.py -r reference1.fasta,reference2.fasta assembly.fasta -o metaquast_output/ --threads 16`
**Explanation:** metaquast.py for metagenome assemblies; -r multiple references; or use --auto-ref for automatic

### assess assembly with minimum contig length filter
**Args:** `-r reference.fasta assembly.fasta -o quast_out/ --min-contig 1000 --threads 8`
**Explanation:** --min-contig 1000 excludes contigs shorter than 1000 bp from all statistics

### assess large genome with optimized parameters
**Args:** `-r reference.fasta assembly.fasta -o quast_large/ --large --threads 16`
**Explanation:** --large uses optimal parameters for large genomes (>100 Mbp); sets -e -m 3000 -i 500 -x 7000

### compute k-mer based quality metrics
**Args:** `-r reference.fasta assembly.fasta -o quast_kmer/ --k-mer-stats --k-mer-size 101 --threads 8`
**Explanation:** --k-mer-stats computes k-mer-based metrics; --k-mer-size 101 sets k-mer size

### assess eukaryotic genome assembly
**Args:** `-r reference.fasta assembly.fasta -o quast_euk/ --eukaryote --gene-finding --threads 8`
**Explanation:** --eukaryote optimizes for eukaryotic gene prediction; --gene-finding uses GeneMark-ES

### assess fungal genome assembly
**Args:** `-r reference.fasta assembly.fasta -o quast_fungus/ --fungus --gene-finding --threads 8`
**Explanation:** --fungus optimizes for fungal gene prediction; combines --eukaryote with fungal-specific settings

### predict rRNA genes during assessment
**Args:** `-r reference.fasta assembly.fasta -o quast_rna/ --rna-finding --threads 8`
**Explanation:** --rna-finding predicts ribosomal RNA genes using Barrnap

### assess with BUSCO conserved genes
**Args:** `-r reference.fasta assembly.fasta -o quast_busco/ --conserved-genes-finding --threads 8`
**Explanation:** --conserved-genes-finding counts conserved orthologs using BUSCO; requires BUSCO installation

### generate Circos visualization
**Args:** `-r reference.fasta assembly.fasta -o quast_circos/ --circos --threads 8`
**Explanation:** --circos generates Circos plot for visualizing assembly against reference

### use custom labels for multiple assemblies
**Args:** `assembly1.fasta assembly2.fasta -o quast_compare/ -l "spades,megahit" --threads 8`
**Explanation:** -l assigns custom labels to assemblies in reports; comma-separated list matching input order
