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
**Explanation:** stringtie command; -G genes.gtf reference GTF annotation; -o sample1.gtf output GTF; -p 8 threads; --rf reverse-strand dUTP library; sample1_sorted.bam input BAM

### merge per-sample StringTie GTFs into unified transcript catalog
**Args:** `--merge -G genes.gtf -o merged.gtf sample1.gtf sample2.gtf sample3.gtf`
**Explanation:** stringtie --merge mode; -G genes.gtf guides merging with known annotation; -o merged.gtf output merged GTF; sample1.gtf sample2.gtf sample3.gtf input per-sample GTFs; combines transcripts from all samples

### re-quantify known and assembled transcripts using merged annotation (for count extraction)
**Args:** `-e -B -G merged.gtf -o sample1_re/sample1.gtf -p 8 --rf sample1_sorted.bam`
**Explanation:** stringtie command; -e estimate-only mode; -B outputs Ballgown tables; -G merged.gtf merged annotation GTF; -o sample1_re/sample1.gtf output GTF; -p 8 threads; --rf reverse-strand library; sample1_sorted.bam input BAM; required for prepDE.py3 count matrix extraction

### assemble and quantify without reference annotation (novel transcript discovery)
**Args:** `-o novel_transcripts.gtf -p 8 --rf sample1_sorted.bam`
**Explanation:** stringtie command; -o novel_transcripts.gtf output GTF; -p 8 threads; --rf reverse-strand library; sample1_sorted.bam input BAM; without -G performs de novo assembly; discovers novel transcripts and isoforms

### extract count matrix from StringTie -e output for DESeq2 with prepDE.py3
**Args:** `-i sample_list.txt -g gene_count_matrix.csv -e transcript_count_matrix.csv`
**Explanation:** prepDE.py3 companion script; -i sample_list.txt input file with sample_name → path/sample.gtf; -g gene_count_matrix.csv gene count matrix output; -e transcript_count_matrix.csv transcript count matrix output

### assemble transcripts from long-read RNA-seq data
**Args:** `-L -G genes.gtf -o long_read.gtf -p 8 sample_lr_sorted.bam`
**Explanation:** stringtie command; -L long-read mode (PacBio/ONT); -G genes.gtf reference GTF; -o long_read.gtf output GTF; -p 8 threads; sample_lr_sorted.bam input BAM; adjusts coverage thresholds for long-read characteristics

### hybrid assembly with short and long reads
**Args:** `-G genes.gtf -o hybrid.gtf -p 8 --mix short_reads.bam long_reads.bam`
**Explanation:** stringtie command; -G genes.gtf reference GTF; -o hybrid.gtf output GTF; -p 8 threads; --mix hybrid assembly mode; short_reads.bam short reads BAM first; long_reads.bam long reads BAM second; combines short-read precision with long-read completeness

### conservative transcript assembly for high-confidence results
**Args:** `--conservative -G genes.gtf -o conservative.gtf -p 8 sample_sorted.bam`
**Explanation:** stringtie command; --conservative applies stricter filters (-t -c 1.5 -f 0.05); -G genes.gtf reference GTF; -o conservative.gtf output GTF; -p 8 threads; sample_sorted.bam input BAM; produces fewer but higher-confidence transcripts

### output gene abundances to separate file
**Args:** `-G genes.gtf -o sample.gtf -A gene_abundance.txt -p 8 sample_sorted.bam`
**Explanation:** stringtie command; -G genes.gtf reference GTF; -o sample.gtf output GTF; -A gene_abundance.txt gene-level abundance output file; -p 8 threads; sample_sorted.bam input BAM; useful for quick expression overview without parsing GTF

### assemble with minimum coverage thresholds for low-expressed genes
**Args:** `-G genes.gtf -o sample.gtf -c 0.5 -s 2.0 -p 8 sample_sorted.bam`
**Explanation:** stringtie command; -G genes.gtf reference GTF; -o sample.gtf output GTF; -c 0.5 multi-exon transcript coverage threshold; -s 2.0 single-exon transcript coverage threshold; -p 8 threads; sample_sorted.bam input BAM; captures more low-expressed transcripts
