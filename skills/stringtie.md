---
name: stringtie
category: rna-seq
description: Efficient and accurate RNA-seq transcript assembly and quantification from RNA-seq alignments
tags: [rna-seq, assembly, transcript, expression, quantification, gtf, isoform]
author: oxo-call built-in
source_url: "https://ccb.jhu.edu/software/stringtie/"
---

## Concepts

- StringTie assembles transcripts from sorted BAM files (HISAT2/STAR output); the standard pipeline is HISAT2 --dta → StringTie.
- Use -G to provide a reference GTF annotation; StringTie will both assemble and quantify known transcripts.
- StringTie outputs a GTF file per sample and (with -e) expression estimates (TPM, FPKM) for known transcripts.
- The --merge step combines per-sample GTFs into a unified transcript catalog before re-running StringTie for count extraction.
- Use -e (estimate-only mode) with merged GTF for count extraction used in DESeq2/edgeR via prepDE.py3.
- Strandedness: --rf for reverse-strand (dUTP/TruSeq), --fr for forward-strand; default is unstranded.
- prepDE.py3 (bundled with StringTie) converts the -e output directory to count matrices for downstream DE analysis.
- StringTie supports long-read RNA-seq data with -L flag for PacBio/ONT alignments.
- Use --mix for hybrid short-read and long-read assembly (provide short reads first, long reads second).
- -B flag outputs Ballgown table files for differential expression analysis with Ballgown R package.
- --conservative flag enables conservative assembly with stricter parameters (-t -c 1.5 -f 0.05).
- -A outputs gene abundance estimates to a separate file for quick expression overview.

## Pitfalls

- StringTie has NO traditional subcommands. For normal assembly/quantification, ARGS starts directly with flags (e.g., -G, -o, -p, --rf). For the merge step, ARGS starts with --merge. Do NOT put a subcommand like 'assemble' or 'quantify' before flags.
- Input BAM must be sorted by coordinates; provide HISAT2 output sorted with samtools sort.
- HISAT2 must be run with --dta flag before StringTie — without it, transcript assembly quality is reduced.
- Without -G, StringTie performs de novo assembly only — use -G to also quantify known transcripts.
- The --merge step must use ALL per-sample GTFs, not just one — incomplete merge degrades results.
- The -e flag in re-quantification step requires the same merged GTF used for merging — not the original annotation.
- StringTie outputs FPKM and TPM but not raw counts; use prepDE.py3 to extract count matrices for DESeq2.
- -L for long reads changes coverage thresholds (-s 1.5 -g 0); do not mix with short-read parameters.
- --mix requires exactly two BAM files: short reads first, long reads second.
- Minimum transcript length (-m) defaults to 200bp; lower for small RNA analysis with -m 50.
- -B (Ballgown output) requires -G reference annotation to generate complete tables.

## Examples

### assemble transcripts from HISAT2-aligned RNA-seq BAM with reference annotation
**Args:** `-G genes.gtf -o sample1.gtf -p 8 --rf sample1_sorted.bam`
**Explanation:** -G reference GTF; -o output GTF; -p threads; --rf for reverse-strand dUTP library

### merge per-sample StringTie GTFs into unified transcript catalog
**Args:** `--merge -G genes.gtf -o merged.gtf sample1.gtf sample2.gtf sample3.gtf`
**Explanation:** --merge combines transcripts from all samples; -G guides merging with known annotation

### re-quantify known and assembled transcripts using merged annotation (for count extraction)
**Args:** `-e -B -G merged.gtf -o sample1_re/sample1.gtf -p 8 --rf sample1_sorted.bam`
**Explanation:** -e estimate-only mode; -B outputs Ballgown tables; required for prepDE.py3 count matrix extraction

### assemble and quantify without reference annotation (novel transcript discovery)
**Args:** `-o novel_transcripts.gtf -p 8 --rf sample1_sorted.bam`
**Explanation:** without -G, StringTie performs de novo assembly; discovers novel transcripts and isoforms

### extract count matrix from StringTie -e output for DESeq2 with prepDE.py3
**Args:** `-i sample_list.txt -g gene_count_matrix.csv -e transcript_count_matrix.csv`
**Explanation:** prepDE.py3 script; sample_list.txt contains sample_name → path/sample.gtf; -g/-e output matrices

### assemble transcripts from long-read RNA-seq data
**Args:** `-L -G genes.gtf -o long_read.gtf -p 8 sample_lr_sorted.bam`
**Explanation:** -L enables long-read mode (PacBio/ONT); adjusts coverage thresholds for long-read characteristics

### hybrid assembly with short and long reads
**Args:** `-G genes.gtf -o hybrid.gtf -p 8 --mix short_reads.bam long_reads.bam`
**Explanation:** --mix combines short-read precision with long-read completeness; short reads BAM first, long reads BAM second

### conservative transcript assembly for high-confidence results
**Args:** `--conservative -G genes.gtf -o conservative.gtf -p 8 sample_sorted.bam`
**Explanation:** --conservative applies stricter filters (-t -c 1.5 -f 0.05); produces fewer but higher-confidence transcripts

### output gene abundances to separate file
**Args:** `-G genes.gtf -o sample.gtf -A gene_abundance.txt -p 8 sample_sorted.bam`
**Explanation:** -A outputs gene-level abundance estimates to a separate file; useful for quick expression overview without parsing GTF

### assemble with minimum coverage thresholds for low-expressed genes
**Args:** `-G genes.gtf -o sample.gtf -c 0.5 -s 2.0 -p 8 sample_sorted.bam`
**Explanation:** -c 0.5 lowers multi-exon transcript coverage threshold; -s 2.0 lowers single-exon threshold; captures more low-expressed transcripts
