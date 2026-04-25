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

- Salmon ARGS must start with a subcommand (index, quant, quantmerge, swim) — never with flags like -t, -i, -l. The subcommand ALWAYS comes first.
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
**Explanation:** salmon index subcommand; -t transcriptome.fa transcriptome FASTA (cDNA); -i salmon_index index output directory; --threads 8 for faster indexing

### quantify paired-end RNA-seq reads with automatic library type detection
**Args:** `quant -i salmon_index -l A -1 R1.fastq.gz -2 R2.fastq.gz -p 8 --gcBias -o sample_quant`
**Explanation:** salmon quant subcommand; -i salmon_index index directory; -l A auto-detects strandedness; -1 R1.fastq.gz -2 R2.fastq.gz paired-end inputs; -p 8 threads; --gcBias corrects for GC content bias; -o sample_quant output directory; selective-alignment is default in recent versions

### quantify single-end RNA-seq reads
**Args:** `quant -i salmon_index -l A -r reads.fastq.gz -p 8 --gcBias -o sample_quant`
**Explanation:** salmon quant subcommand; -i salmon_index index directory; -l A auto-detects orientation; -r reads.fastq.gz single-end reads input; -p 8 threads; --gcBias recommended; -o sample_quant output directory

### build decoy-aware salmon index for more accurate quantification
**Args:** `index -t gentrome.fa -d decoys.txt -i salmon_index_decoy --threads 8`
**Explanation:** salmon index subcommand; -t gentrome.fa transcriptome+genome FASTA; -d decoys.txt list of genome chromosome names; -i salmon_index_decoy output directory; --threads 8; more accurate quantification

### quantify bulk RNA-seq with strand-specific reverse library
**Args:** `quant -i salmon_index -l ISR -1 R1.fastq.gz -2 R2.fastq.gz -p 8 --gcBias --seqBias -o sample_quant`
**Explanation:** salmon quant subcommand; -i salmon_index index; -l ISR inward stranded reverse for dUTP libraries; -1 R1.fastq.gz -2 R2.fastq.gz paired-end inputs; -p 8 threads; --gcBias for GC bias; --seqBias for sequence-specific bias; -o sample_quant output; selective-alignment is now default

### quantify from existing BAM alignments
**Args:** `quant -i salmon_index -l A -a aligned.bam -p 8 --gcBias -o sample_quant`
**Explanation:** salmon quant subcommand; -i salmon_index index directory; -l A auto-detect library; -a aligned.bam input BAM/SAM alignments; -p 8 threads; --gcBias bias correction; -o sample_quant output; useful when re-using existing alignments from STAR/HISAT2

### merge multiple quantification results
**Args:** `quantmerge --quants sample1_quant sample2_quant sample3_quant -o merged_quant.sf`
**Explanation:** salmon quantmerge subcommand; --quants sample1_quant sample2_quant sample3_quant input directories; -o merged_quant.sf output file; combines quant.sf files from multiple samples; useful for creating count matrices for downstream analysis

### build index with custom k-mer length for short reads
**Args:** `index -t transcriptome.fa -i salmon_index_k23 -k 23 --threads 8`
**Explanation:** salmon index subcommand; -t transcriptome.fa transcriptome FASTA; -i salmon_index_k23 output directory; -k 23 sets shorter k-mer length for short/noisy reads; --threads 8; increases sensitivity but may reduce specificity compared to default k=31

### quantify with GENCODE transcriptome and gene name splitting
**Args:** `quant -i salmon_index -l A -1 R1.fq.gz -2 R2.fq.gz -p 8 --gcBias --gencode -o gencode_quant`
**Explanation:** salmon quant subcommand; -i salmon_index index; -l A auto-detect; -1 R1.fq.gz -2 R2.fq.gz paired inputs; -p 8 threads; --gcBias bias correction; --gencode splits transcript names at first '|' for GENCODE-formatted FASTA; -o gencode_quant output; simplifies downstream gene-level aggregation

### quantify with multiple bias corrections
**Args:** `quant -i salmon_index -l A -1 R1.fq.gz -2 R2.fq.gz -p 8 --gcBias --seqBias --posBias -o bias_corrected_quant`
**Explanation:** salmon quant subcommand; -i salmon_index index; -l A auto-detect; -1 R1.fq.gz -2 R2.fq.gz paired inputs; -p 8 threads; --gcBias GC bias; --seqBias sequence bias; --posBias positional bias; -o bias_corrected_quant output; comprehensive bias correction for highest accuracy

### quantify with range factorization for memory efficiency
**Args:** `quant -i salmon_index -l A -1 R1.fq.gz -2 R2.fq.gz -p 8 --rangeFactorizationBins 4 -o efficient_quant`
**Explanation:** salmon quant subcommand; -i salmon_index index; -l A auto-detect; -1 R1.fq.gz -2 R2.fq.gz paired inputs; -p 8 threads; --rangeFactorizationBins 4 reduces memory usage; -o efficient_quant output; useful for large transcriptomes or memory-constrained systems

### quantify with transcriptome mapping output
**Args:** `quant -i salmon_index -l A -1 R1.fq.gz -2 R2.fq.gz -p 8 --writeMappings=mappings.sam -o with_mappings`
**Explanation:** salmon quant subcommand; -i salmon_index index; -l A auto-detect; -1 R1.fq.gz -2 R2.fq.gz paired inputs; -p 8 threads; --writeMappings=mappings.sam outputs SAM-format alignments to transcriptome; -o with_mappings output; useful for debugging or alternative quantification methods
