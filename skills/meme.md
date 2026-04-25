---
name: meme
category: utilities
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
- `-pal` forces palindromic motif discovery for DNA-binding proteins that bind as dimers.
- `-objfun` specifies the objective function: classic, de (differential enrichment), se (significant enrichment), cd (central distance), ce (central enrichment).
- `-markov_order` sets the order of the background Markov model for calculating expected motif frequencies.

## Pitfalls
- `meme` uses only the first `-maxsites` sequences if the input is large; for ChIP-seq datasets, pre-filter to the top 500–1000 peaks by signal strength.
- Large `-nmotifs` values greatly increase runtime; start with `-nmotifs 5` and increase only if needed.
- Not masking repeats before running MEME leads to repetitive-element motifs dominating results; always run RepeatMasker or `bedtools maskfasta` first.
- FIMO without `--max-strand` reports hits on both strands; for strand-specific analyses, filter the output by strand column.
- The `--oc` (output to clean directory) and `-o` (fail if directory exists) flags behave differently; use `--oc` to overwrite existing results.
- `meme` requires at least 2 sequences in the input FASTA; too few sequences or too-short sequences (< motif width) cause failures.
- On HPC, MEME's built-in parallelism uses `-p` (MPI processes); ensure MPI is loaded if using `-p > 1`.
- `-searchsize` limits the portion of dataset used for motif search; increase for more thorough searches on large datasets.
- `-maxsize` limits the total dataset size; sequences exceeding this limit are skipped.
- `-neg` provides a negative/control dataset for differential enrichment objective functions (de, se).

## Examples

### discover de novo motifs in ChIP-seq peak sequences
**Args:** `-dna -mod zoops -nmotifs 10 -minw 6 -maxw 20 -oc meme_output peaks.fasta`
**Explanation:** meme command; -dna for DNA sequences; -mod zoops allows zero or one motif per sequence; -nmotifs 10 finds up to 10 motifs; -minw 6 -maxw 20 set motif width range; -oc meme_output output directory; peaks.fasta input FASTA

### scan sequences for known TF binding motifs with FIMO
**Args:** `fimo --thresh 1e-4 --oc fimo_output $MEME/share/meme/db/motif_databases/JASPAR/JASPAR2022_CORE_vertebrates_non-redundant_v2.meme peaks.fasta`
**Explanation:** fimo subcommand; --thresh 1e-4 p-value threshold; --oc fimo_output output directory; $MEME/share/.../JASPAR...meme motif database; peaks.fasta input FASTA

### compare discovered motifs against a known database with TOMTOM
**Args:** `tomtom -oc tomtom_output meme_output/meme.xml $MEME/share/meme/db/motif_databases/JASPAR/JASPAR2022_CORE_vertebrates_non-redundant_v2.meme`
**Explanation:** tomtom subcommand; -oc tomtom_output output directory; meme_output/meme.xml input MEME motifs; $MEME/share/.../JASPAR...meme motif database

### test motif enrichment in a foreground vs background with AME
**Args:** `ame --oc ame_output --control shuffled_bg.fasta peaks.fasta $MEME/share/meme/db/motif_databases/HOCOMOCO/HOCOMOCOv11_core_HUMAN_mono_meme_format.meme`
**Explanation:** ame subcommand; --oc ame_output output directory; --control shuffled_bg.fasta background sequences; peaks.fasta foreground sequences; $MEME/share/.../HOCOMOCO...meme motif database

### run STREME for fast short motif discovery
**Args:** `streme --oc streme_output --dna --p peaks.fasta --n shuffled.fasta`
**Explanation:** streme subcommand; --oc streme_output output directory; --dna alphabet; --p peaks.fasta foreground set; --n shuffled.fasta control/background

### extract sequences for peak regions using bedtools first
**Args:** `bedtools getfasta -fi genome.fa -bed peaks.bed -fo peaks.fasta`
**Explanation:** bedtools getfasta subcommand; -fi genome.fa reference FASTA; -bed peaks.bed input BED; -fo peaks.fasta output FASTA

### run MEME with reverse complement consideration
**Args:** `-dna -revcomp -mod zoops -nmotifs 5 -oc meme_rc peaks.fasta`
**Explanation:** meme command; -dna alphabet; -revcomp considers both strands; -mod zoops zero or one per sequence; -nmotifs 5 number of motifs; -oc meme_rc output directory; peaks.fasta input FASTA

### discover palindromic motifs for dimer-binding TFs
**Args:** `-dna -pal -mod zoops -nmotifs 5 -minw 10 -maxw 20 -oc meme_pal peaks.fasta`
**Explanation:** meme command; -dna alphabet; -pal forces palindromic motif discovery; -mod zoops model; -nmotifs 5 number of motifs; -minw 10 -maxw 20 motif width range; -oc meme_pal output directory; peaks.fasta input FASTA

### use differential enrichment objective function with control sequences
**Args:** `-dna -mod zoops -objfun de -neg control_peaks.fasta -nmotifs 5 -oc meme_de peaks.fasta`
**Explanation:** meme command; -dna alphabet; -mod zoops model; -objfun de differential enrichment objective; -neg control_peaks.fasta control sequences; -nmotifs 5 number of motifs; -oc meme_de output directory; peaks.fasta input FASTA

### run MEME with higher-order Markov background model
**Args:** `-dna -mod zoops -markov_order 3 -nmotifs 5 -oc meme_markov peaks.fasta`
**Explanation:** meme command; -dna alphabet; -mod zoops model; -markov_order 3 3rd-order Markov model for background; -nmotifs 5 number of motifs; -oc meme_markov output directory; peaks.fasta input FASTA

### limit search to top sequences for faster runtime
**Args:** `-dna -mod zoops -searchsize 100000 -nmotifs 5 -oc meme_fast peaks.fasta`
**Explanation:** meme command; -dna alphabet; -mod zoops model; -searchsize 100000 limits search to first 100kb; -nmotifs 5 number of motifs; -oc meme_fast output directory; peaks.fasta input FASTA

### run MEME with MPI parallelization on HPC
**Args:** `-dna -mod zoops -p 8 -nmotifs 5 -oc meme_mpi peaks.fasta`
**Explanation:** meme command; -dna alphabet; -mod zoops model; -p 8 MPI processes for parallel search; -nmotifs 5 number of motifs; -oc meme_mpi output directory; peaks.fasta input FASTA

### find motifs with exact number of sites per sequence
**Args:** `-dna -mod oops -nmotifs 3 -minw 8 -maxw 15 -oc meme_oops peaks.fasta`
**Explanation:** meme command; -dna alphabet; -mod oops exactly one motif per sequence; -nmotifs 3 number of motifs; -minw 8 -maxw 15 motif width range; -oc meme_oops output directory; peaks.fasta input FASTA
