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

## Pitfalls

- Input BAM must be sorted by coordinates; provide HISAT2 output sorted with samtools sort.
- HISAT2 must be run with --dta flag before StringTie — without it, transcript assembly quality is reduced.
- Without -G, StringTie performs de novo assembly only — use -G to also quantify known transcripts.
- The --merge step must use ALL per-sample GTFs, not just one — incomplete merge degrades results.
- The -e flag in re-quantification step requires the same merged GTF used for merging — not the original annotation.
- StringTie outputs FPKM and TPM but not raw counts; use prepDE.py3 to extract count matrices for DESeq2.

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
