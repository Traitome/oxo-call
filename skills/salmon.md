---
name: salmon
category: rna-seq
description: Ultrafast quasi-mapping RNA-seq quantification at the transcript level
tags: [rna-seq, quantification, transcript, expression, quasi-mapping, tpm, counts]
author: oxo-call built-in
source_url: "https://salmon.readthedocs.io/"
---

## Concepts

- Salmon quantifies transcript-level expression using quasi-mapping (alignment-free); requires a transcriptome FASTA (not genome FASTA).
- Two-step workflow: (1) salmon index -t transcriptome.fa -i index_dir; (2) salmon quant -i index_dir -l A -1 R1 -2 R2 -o outdir.
- Use -l A for automatic library type detection (detects strandedness); explicit options: IU (unstranded PE), ISR (stranded reverse PE), ISF (stranded forward PE), U (unstranded SE), SF (stranded forward SE), SR (stranded reverse SE).
- Salmon outputs quant.sf in the output directory with columns: Name, Length, EffectiveLength, TPM, NumReads.
- Use --gcBias for GC bias correction (recommended for most datasets); --seqBias for sequence-specific bias correction.
- For decoy-aware indexing (more accurate), include genome as decoy: salmon index -t gentrome.fa -d decoys.txt -i index/
- Selective-alignment is the default mapping mode in recent Salmon versions (replaces --validateMappings); it provides higher accuracy than traditional quasi-mapping.
- tximport/tximeta R packages are the standard way to import Salmon output into DESeq2/edgeR.
- Use -k to set k-mer length for indexing (default 31); shorter k-mers increase sensitivity for noisy reads but may reduce specificity.
- Salmon can also quantify from existing BAM/SAM alignments using -a/--alignments flag (alignment-based mode).
- The --gencode flag splits transcript names at the first '|' for GENCODE-formatted FASTA files.

## Pitfalls

- CRITICAL: Salmon ARGS must start with a subcommand (index, quant, quantmerge, swim) — never with flags like -t, -i, -l. The subcommand ALWAYS comes first.
- Salmon requires transcriptome (cDNA) FASTA, NOT genome FASTA — using the genome will produce wrong results.
- Library type (-l) is critical — wrong strandedness gives dramatically wrong counts; use -l A to auto-detect.
- Without --gcBias, GC-biased data can produce systematically skewed quantification.
- Salmon output is per-transcript — use tximport to aggregate to gene level for gene-level DE analysis.
- The index directory must be created fresh — do not reuse a Bowtie2 or STAR index directory.
- For paired-end, -1 must be R1 (read 1) and -2 must be R2 (read 2) — swapping them causes errors.
- --validateMappings is deprecated in recent versions; selective-alignment is now the default and provides better accuracy.
- alevin subcommand has been removed in v1.10+; use alevin-fry for single-cell analysis instead.
- When using decoy-aware indexing, the decoys.txt file must contain chromosome names exactly as they appear in the genome FASTA headers.

## Examples

### build a Salmon transcriptome index
**Args:** `index -t transcriptome.fa -i salmon_index --threads 8`
**Explanation:** -t transcriptome FASTA (cDNA); -i index output directory; --threads for faster indexing

### quantify paired-end RNA-seq reads with automatic library type detection
**Args:** `quant -i salmon_index -l A -1 R1.fastq.gz -2 R2.fastq.gz -p 8 --gcBias --validateMappings -o sample_quant`
**Explanation:** -l A auto-detects strandedness; --gcBias corrects for GC content bias; --validateMappings increases accuracy

### quantify single-end RNA-seq reads
**Args:** `quant -i salmon_index -l A -r reads.fastq.gz -p 8 --gcBias -o sample_quant`
**Explanation:** -r for single-end reads; -l A auto-detects orientation; --gcBias recommended

### build decoy-aware salmon index for more accurate quantification
**Args:** `index -t gentrome.fa -d decoys.txt -i salmon_index_decoy --threads 8`
**Explanation:** gentrome.fa = cat transcriptome.fa genome.fa; decoys.txt = list of genome chromosome names; more accurate

### quantify bulk RNA-seq with strand-specific reverse library
**Args:** `quant -i salmon_index -l ISR -1 R1.fastq.gz -2 R2.fastq.gz -p 8 --gcBias --seqBias -o sample_quant`
**Explanation:** ISR = inward, stranded, reverse (dUTP libraries); --seqBias for sequence-specific bias correction; selective-alignment is now default

### quantify from existing BAM alignments
**Args:** `quant -i salmon_index -l A -a aligned.bam -p 8 --gcBias -o sample_quant`
**Explanation:** -a specifies input BAM/SAM alignments instead of raw reads; useful when re-using existing alignments from STAR/HISAT2

### merge multiple quantification results
**Args:** `quantmerge --quants sample1_quant sample2_quant sample3_quant -o merged_quant.sf`
**Explanation:** combines quant.sf files from multiple samples into a single file; useful for creating count matrices for downstream analysis

### build index with custom k-mer length for short reads
**Args:** `index -t transcriptome.fa -i salmon_index_k23 -k 23 --threads 8`
**Explanation:** -k 23 sets shorter k-mer length for short or noisy reads (e.g., 50bp); increases sensitivity but may reduce specificity compared to default k=31
