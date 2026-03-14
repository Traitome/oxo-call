---
name: meme
category: motif-analysis
description: MEME Suite — motif discovery and analysis; meme, fimo, tomtom, mast, ame for finding transcription factor binding sites and sequence motifs
tags: [meme, fimo, tomtom, motif, transcription-factor, chip-seq, atac-seq, sequence-motif, pwm]
author: oxo-call built-in
source_url: "https://meme-suite.org/meme/doc/overview.html"
---

## Concepts
- MEME Suite includes: `meme` (de novo motif discovery), `fimo` (motif scanning), `tomtom` (motif comparison), `mast` (motif alignment), `ame` (enrichment analysis), `streme` (short motif discovery), `centrimo` (central enrichment).
- MEME is installed in `$MEME` or typically `/usr/local/meme/`; binaries are in `$MEME/bin/`; motif databases in `$MEME/share/meme/db/motif_databases/` (or `$MEMEDB`).
- Input for `meme`: a FASTA file of sequences (e.g. ChIP-seq peaks, promoters); output written to a directory (`-oc output_dir`).
- **MEME occurrence models**: `-mod zoops` (Zero Or One Per Sequence — default), `-mod oops` (Exactly One Per Sequence), `-mod anr` (Any Number of Repetitions).
- `fimo` scans a FASTA sequence file against a motif database or MEME output; requires a motif file in MEME format and a sequence file.
- FIMO `--thresh` sets the p-value threshold (default 1e-4); lower values are more stringent.
- Motif databases bundled with MEME Suite: JASPAR, HOCOMOCO, TRANSFAC, ENCODE, CIS-BP; access via path in `$MEME/share/meme/db/motif_databases/`.
- `-dna`, `-rna`, `-protein` flags specify the alphabet; `-dna` is the default for genomic sequences.
- `tomtom` compares query motifs (from MEME output) against a reference database to identify known TF matches.
- MEME Suite outputs HTML reports (`meme.html`, `fimo.html`) and XML/TSV files for downstream parsing.
- Streme is faster than MEME for large input sets and finds shorter motifs; use it when MEME is too slow.

## Pitfalls
- DANGER: `meme` uses only the first `-maxsites` sequences if the input is large; for ChIP-seq datasets, pre-filter to the top 500–1000 peaks by signal strength.
- Large `-nmotifs` values greatly increase runtime; start with `-nmotifs 5` and increase only if needed.
- Not masking repeats before running MEME leads to repetitive-element motifs dominating results; always run RepeatMasker or `bedtools maskfasta` first.
- FIMO without `--max-strand` reports hits on both strands; for strand-specific analyses, filter the output by strand column.
- The `--oc` (output to clean directory) and `-o` (fail if directory exists) flags behave differently; use `--oc` to overwrite existing results.
- `meme` requires at least 2 sequences in the input FASTA; too few sequences or too-short sequences (< motif width) cause failures.
- On HPC, MEME's built-in parallelism uses `-p` (MPI processes); ensure MPI is loaded if using `-p > 1`.

## Examples

### discover de novo motifs in ChIP-seq peak sequences
**Args:** `-dna -mod zoops -nmotifs 10 -minw 6 -maxw 20 -oc meme_output peaks.fasta`
**Explanation:** -dna for DNA sequences; -mod zoops allows zero or one motif per sequence; -nmotifs 10 finds up to 10 motifs; -minw/-maxw set motif width range

### scan sequences for known TF binding motifs with FIMO
**Args:** `fimo --thresh 1e-4 --oc fimo_output $MEME/share/meme/db/motif_databases/JASPAR/JASPAR2022_CORE_vertebrates_non-redundant_v2.meme peaks.fasta`
**Explanation:** scans peaks.fasta for JASPAR vertebrate motifs; --thresh filters by p-value; output in fimo_output/fimo.tsv

### compare discovered motifs against a known database with TOMTOM
**Args:** `tomtom -oc tomtom_output meme_output/meme.xml $MEME/share/meme/db/motif_databases/JASPAR/JASPAR2022_CORE_vertebrates_non-redundant_v2.meme`
**Explanation:** matches MEME-discovered motifs against JASPAR; output ranks known TFs by similarity score

### test motif enrichment in a foreground vs background with AME
**Args:** `ame --oc ame_output --control shuffled_bg.fasta peaks.fasta $MEME/share/meme/db/motif_databases/HOCOMOCO/HOCOMOCOv11_core_HUMAN_mono_meme_format.meme`
**Explanation:** AME (Analysis of Motif Enrichment) tests which motifs from the database are enriched in peaks.fasta compared to shuffled_bg.fasta

### run STREME for fast short motif discovery
**Args:** `streme --oc streme_output --dna --p peaks.fasta --n shuffled.fasta`
**Explanation:** streme is faster than meme for large datasets; --p is the foreground set; --n is the control/background; finds short enriched motifs

### extract sequences for peak regions using bedtools first
**Args:** `bedtools getfasta -fi genome.fa -bed peaks.bed -fo peaks.fasta`
**Explanation:** prerequisite step before MEME; extracts FASTA sequences for peak coordinates; ensure genome.fa and peaks.bed use the same chromosome names

### run MEME with reverse complement consideration
**Args:** `-dna -revcomp -mod zoops -nmotifs 5 -oc meme_rc peaks.fasta`
**Explanation:** -revcomp considers both strands for motif discovery; essential for TF binding site discovery where binding can occur on either strand
