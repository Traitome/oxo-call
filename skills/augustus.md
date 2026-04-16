---
name: augustus
category: annotation
description: Gene prediction for eukaryotic genomes using Hidden Markov Models with species-specific training and evidence integration
tags: [annotation, gene-prediction, eukaryote, hmm, genome, gff, rna-seq, braker]
author: oxo-call built-in
source_url: "https://bioinf.uni-greifswald.de/augustus/"
---

## Concepts

- AUGUSTUS predicts eukaryotic genes using species-specific trained HMM parameters. Use `--species=help` to list all available trained species models.
- Use --species to specify a pre-trained species model (e.g., human, arabidopsis, zebrafish, fly, rice, maize, yeast).
- Use `--gff3=on` for GFF3 output (default is AUGUSTUS-specific format); `--outfile` for output file instead of stdout.
- Evidence integration: use `--hintsfile` with extrinsic hints (RNA-seq introns, EST alignments, protein matches) for better accuracy. Use `--extrinsicCfgFile` to set hint weights.
- Generate hints from RNA-seq BAM: `bam2hints --in=alignments.bam -o hints.gff` converts spliced alignments to intron hints for AUGUSTUS.
- Gene model types (`--genemodel`): partial (default, allows incomplete genes at boundaries), complete (only full genes), intronless (single-exon genes), atleastone, exactlyone, bacterium.
- Alternative transcripts: `--alternatives-from-evidence=true` reports alternative splicing suggested by hints; `--alternatives-from-sampling=true` reports suboptimal but plausible alternatives.
- UTR prediction: `--UTR=on` predicts untranslated regions (works only for species with UTR training data).
- Output sequences: `--protein=on` outputs protein sequences; `--codingseq=on` outputs CDS sequences alongside the GFF.
- Softmasking: `--softmasking=1` (default) treats lowercase regions as masked repeats, avoiding gene predictions in repetitive sequences.
- Protein profile extension: `--proteinprofile=filename` uses protein family profiles to improve prediction of specific gene families.
- For training on a new species, use BRAKER (automates AUGUSTUS training with RNA-seq or protein evidence) or `etraining` and the `autoAug.pl` pipeline.
- Comparative Gene Prediction (CGP) mode: use `--treefile` and `--alnfile` for multi-species comparative prediction when related genome alignments are available.
- Strand selection: `--strand=both` (default), `--strand=forward`, or `--strand=backward`.

## Pitfalls

- CRITICAL: AUGUSTUS has no subcommands. ARGS starts directly with flags like `--species`, `--gff3`. The input FASTA filename is the last positional argument.
- Using the wrong species model severely reduces accuracy — choose the closest related organism. Use `--species=help` to see all options.
- AUGUSTUS without RNA-seq hints may miss many genes in organisms with complex gene structures. Always provide hints when possible.
- The GFF3 output requires `--gff3=on` — default AUGUSTUS format is not standard GFF3 and is not compatible with most downstream tools.
- For large genomes, split into chromosomes/scaffolds and run in parallel for speed.
- AUGUSTUS is sensitive to repeat-masked input — run RepeatMasker before AUGUSTUS for best results. Use `--softmasking=1` with soft-masked FASTA.
- Training AUGUSTUS from scratch requires ~1000 manually curated gene structures. Use BRAKER to automate training with RNA-seq data.
- `--UTR=on` only works for species that have UTR parameters trained. Check species availability before relying on UTR prediction.
- `bam2hints` requires input BAM to be sorted by reference sequence name and then by position. Unsorted BAMs produce incorrect hints.
- `--noInFrameStop=true` filters out transcripts with in-frame stop codons — recommended for final high-quality predictions.

## Examples

### predict genes in a eukaryotic genome using human parameters
**Args:** `--species=human genome.fasta --gff3=on --outfile=predictions.gff3`
**Explanation:** --species=human uses the human-trained HMM; --gff3=on outputs standard GFF3; --outfile writes to file instead of stdout

### predict genes with RNA-seq hints for improved accuracy
**Args:** `--species=arabidopsis --hintsfile=rnaseq_hints.gff --extrinsicCfgFile=extrinsic.cfg genome.fasta --gff3=on > improved_predictions.gff3`
**Explanation:** --hintsfile provides RNA-seq intron/exon hints; --extrinsicCfgFile sets weights for hint integration; significantly improves sensitivity and specificity

### generate hint file from RNA-seq BAM alignments
**Args:** `bam2hints --in=star_alignments.bam --out=rnaseq_hints.gff`
**Explanation:** converts spliced RNA-seq alignments to intron hints in GFF format; BAM must be sorted by reference and position

### predict genes and output protein sequences
**Args:** `--species=fly --gff3=on --protein=on --codingseq=on genome.fasta > fly_predictions.gff3`
**Explanation:** --protein=on outputs protein sequences embedded in GFF3; --codingseq=on outputs CDS sequences; both useful for downstream functional annotation

### predict only complete genes on the forward strand
**Args:** `--species=zebrafish --genemodel=complete --strand=forward --gff3=on genome.fasta > complete_genes.gff3`
**Explanation:** --genemodel=complete only reports genes with start and stop codons; --strand=forward restricts to forward strand

### run Augustus on a repeat-masked genome with UTR prediction
**Args:** `--species=human human_masked.fasta --gff3=on --softmasking=1 --UTR=on > human_genes.gff3`
**Explanation:** --softmasking=1 uses softmasked lowercase regions to avoid predicting in repeats; --UTR=on adds UTR features (requires species with UTR training)

### predict genes with alternative splicing from evidence
**Args:** `--species=human --hintsfile=hints.gff --alternatives-from-evidence=true --maxtracks=4 --gff3=on genome.fasta > alt_splicing.gff3`
**Explanation:** --alternatives-from-evidence=true reports alternative transcripts when hints suggest them; --maxtracks=4 limits alternative gene structures per locus

### predict genes on a specific region of a chromosome
**Args:** `--species=human --predictionStart=100000 --predictionEnd=500000 chr1.fasta --gff3=on > region_predictions.gff3`
**Explanation:** --predictionStart/--predictionEnd restrict prediction to a specific coordinate range; useful for testing parameters or analyzing specific regions

### predict genes using protein profile for a specific gene family
**Args:** `--species=human --proteinprofile=kinase.profile --gff3=on genome.fasta > kinase_predictions.gff3`
**Explanation:** --proteinprofile uses protein family conservation patterns to improve prediction of genes in a specific family
