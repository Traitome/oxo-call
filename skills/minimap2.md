---
name: minimap2
category: alignment
description: Versatile sequence aligner for long reads, assembly-to-reference, and short reads
tags: [long-read, nanopore, pacbio, alignment, assembly, mapping, hifi]
author: oxo-call built-in
source_url: "https://github.com/lh3/minimap2"
---

## Concepts
- minimap2 uses presets (-x) to set alignment parameters; always use the right preset for your data type.
- Key presets: map-ont (Oxford Nanopore), map-pb (PacBio CLR), map-hifi (PacBio HiFi/CCS), sr (short reads), splice/splice:hq (RNA-seq), asm5/asm20 (assembly-to-assembly).
- minimap2 outputs PAF by default; use -a to output SAM (pipe to samtools); -c adds CIGAR to PAF.
- For RNA-seq long reads, use -x splice (nanopore cDNA) or -x splice:hq (PacBio Iso-Seq); these enable spliced alignment.
- Use -t N for multi-threading; the output is to stdout — always redirect or pipe.
- For all-vs-all overlap detection (de novo assembly), use -x ava-ont or -x ava-pb; output PAF to miniasm or hifiasm.
- Build a reusable index with 'minimap2 -d ref.mmi ref.fa'; then align with 'minimap2 -a ref.mmi reads.fa'. Saves time for repeated runs.
- Use --MD flag to add MD string to SAM — required by downstream tools like Medaka and Sniffles for correct operation.
- --cs outputs the cs tag for describing sequence differences in a concise way.
- --eqx writes =/X CIGAR operators instead of M; useful for distinguishing matches from mismatches.
- --secondary=no suppresses secondary alignments; useful for unique mapping analyses.
- --sam-hit-only outputs only mapped reads in SAM format.
- -Y uses soft clipping for supplementary alignments; recommended for SV calling.
- -k sets k-mer size (default 15); -w sets minimizer window size (default 10).
- -I splits index for large genomes; default 8G, increase for large references.

## Pitfalls
- minimap2 has NO subcommands. ARGS starts directly with flags (e.g., -ax, -d, -x). Do NOT put a subcommand like 'align' or 'map' before flags.
- Without -a, minimap2 outputs PAF format — most downstream tools expect SAM/BAM, so use -a.
- Using the wrong preset for your data type produces incorrect alignments — always match preset to data.
- minimap2 with -a outputs SAM to stdout — pipe to samtools: minimap2 -a -x preset ref.fa reads.fq | samtools sort -o out.bam.
- For human genome, minimap2 uses ~14 GB RAM for long-read alignment.
- The reference can be a FASTA file — minimap2 indexes it on the fly (no separate index step needed for single runs).
- A pre-built .mmi index is preset-specific — a map-ont index cannot be reused for map-hifi alignment.
- Secondary alignments are reported by default (-N 5) — use --secondary=no to suppress them for uniquely mapping analyses.
- --cs tag increases output size but is required by some downstream tools.
- -k and -w affect sensitivity and speed; smaller k increases sensitivity but slows down.
- Split index (-I) may reduce performance; use larger values for whole-genome alignment.
- Soft clipping (-Y) vs hard clipping affects SV detection; choose based on downstream tools.

## Examples

### align Oxford Nanopore reads to a reference genome
**Args:** `-ax map-ont reference.fa nanopore_reads.fastq.gz | samtools sort -o aligned_sorted.bam`
**Explanation:** -ax map-ont is the Nanopore preset; -a outputs SAM; output piped to samtools sort

### align PacBio HiFi (CCS) reads to a reference genome
**Args:** `-ax map-hifi reference.fa hifi_reads.fastq.gz | samtools sort -o hifi_aligned.bam`
**Explanation:** -a outputs SAM; -x map-hifi preset for PacBio HiFi/CCS reads with high accuracy

### align Nanopore cDNA reads for RNA-seq spliced alignment
**Args:** `-ax splice --junc-bed known_junctions.bed reference.fa rna_reads.fastq.gz | samtools sort -o rna_aligned.bam`
**Explanation:** -a outputs SAM; -x splice enables spliced alignment for RNA-seq; --junc-bed provides known splice junctions

### compare two genome assemblies (assembly vs reference)
**Args:** `-ax asm5 reference.fa assembly.fa | samtools sort -o assembly_vs_ref.bam`
**Explanation:** -a outputs SAM; -x asm5 for assemblies with ≤5% sequence divergence from reference

### map long reads and output in PAF format for structural variant analysis
**Args:** `-x map-ont -c reference.fa reads.fastq.gz > aligned.paf`
**Explanation:** -x map-ont Nanopore preset; -c includes CIGAR in PAF; PAF format is preferred by SV callers like Sniffles2

### compute all-vs-all overlaps for de novo ONT assembly
**Args:** `-x ava-ont reads.fastq.gz reads.fastq.gz | gzip > overlaps.paf.gz`
**Explanation:** -x ava-ont for all-vs-all ONT overlaps; used as input to miniasm; -x ava-pb for PacBio CLR

### build a reusable minimap2 index for repeated ONT alignments
**Args:** `-d reference_ont.mmi -x map-ont reference.fa`
**Explanation:** -d creates a .mmi index file tied to the map-ont preset; reuse with: minimap2 -a reference_ont.mmi reads.fq

### align with cs tag for variant calling
**Args:** `-ax map-hifi --cs reference.fa reads.fastq.gz | samtools sort -o aligned_cs.bam`
**Explanation:** -a outputs SAM; -x map-hifi HiFi preset; --cs outputs cs tag describing sequence differences; useful for variant calling

### align with =/X CIGAR operators
**Args:** `-ax map-ont --eqx reference.fa reads.fastq.gz | samtools sort -o aligned_eqx.bam`
**Explanation:** -a outputs SAM; -x map-ont Nanopore preset; --eqx writes =/X instead of M in CIGAR; distinguishes matches from mismatches

### align suppressing secondary alignments
**Args:** `-ax map-ont --secondary=no reference.fa reads.fastq.gz | samtools sort -o unique.bam`
**Explanation:** -a outputs SAM; -x map-ont Nanopore preset; --secondary=no outputs only primary alignments; useful for unique mapping

### align with soft clipping for SV calling
**Args:** `-ax map-ont -Y --MD reference.fa reads.fastq.gz | samtools sort -o sv_ready.bam`
**Explanation:** -a outputs SAM; -x map-ont Nanopore preset; -Y uses soft clipping for supplementary alignments; --MD adds MD tag for SV callers

### align short reads (Illumina)
**Args:** `-ax sr reference.fa reads_R1.fastq.gz reads_R2.fastq.gz | samtools sort -o short_aligned.bam`
**Explanation:** -a outputs SAM; -x sr preset for short reads; accepts paired-end files

### align with custom k-mer and window size
**Args:** `-ax map-ont -k 17 -w 10 reference.fa reads.fastq.gz | samtools sort -o custom_k.bam`
**Explanation:** -a outputs SAM; -x map-ont Nanopore preset; -k 17 increases k-mer size; -w 10 sets window size; tune for data characteristics

### output only mapped reads
**Args:** `-ax map-ont --sam-hit-only reference.fa reads.fastq.gz | samtools sort -o mapped_only.bam`
**Explanation:** -a outputs SAM; -x map-ont Nanopore preset; --sam-hit-only excludes unmapped reads from SAM output; reduces file size
