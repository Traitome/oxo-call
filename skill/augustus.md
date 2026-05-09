---
name: augustus
category: Genome Annotation / Gene Prediction
description: Ab initio gene prediction tool for identifying protein-coding genes in eukaryotic genomic DNA sequences. Uses hidden Markov models trained on species-specific parameters to predict gene structures including exons, introns, start/stop codons, and UTRs.
tags: [gene-prediction, ab-initio, genome-annotation, eukaryotic-genomes, HMM, Augustus]
author: AI-generated
source_url: https://bioinf.uni-goettingen.de/pages/users/stanke/augustus.html
---

## Concepts

- Augustus reads genomic DNA sequences in FASTA format and predicts protein-coding gene structures using species-specific HMM parameters. Output formats include GFF3, GenBank, and JSON—specify with `--gff3`, `--genbank`, or `--json` flags respectively.

- The `--species` flag is mandatory and selects from dozens of pre-trained species models (e.g., `human`, `mouse`, `arabidopsis`, `fly`). Using an incorrect or unrelated species file produces meaningless predictions because gene structure models are species-specific.

- Augustus supports "hints mode" via `--extrinsicspec` or hint files (EST, protein, RNA-seq alignments) which dramatically increases prediction accuracy. Pure ab initio prediction alone is rarely suitable for real genomes—combine with evidence-based hints for reliable gene models.

- The companion tool `augustus-train` creates custom species parameters from a training set of known gene annotations. After training, specify your custom species with `--species=yourSpecies` in prediction runs.

- Prediction outputs include predicted protein sequences (`--proteinoutput`) and coding sequence translations (`--cds`) alongside genomic coordinates. These protein FASTA files can be used directly for downstream homology searches or functional annotation.

## Pitfalls

- Omitting the `--species` flag causes Augustus to fail with an error, but specifying the wrong species leads to silently incorrect predictions—always verify the species model matches your organism of interest before running.

- Running on entire chromosomes without chunking (`--chunk size`) consumes excessive memory and takes hours. Split large sequences into smaller chunks (e.g., 1-2 Mb) and predict separately for reasonable runtime and resource usage.

- Using default output format (GenBank) without redirecting to a file causes predictions to print to stdout, mixing with progress messages. Always use `--outfile` to direct output to a specific file for downstream processing.

- Not providing any hint information results in ab initio predictions that miss genes with non-canonical structures. For genomes with any available evidence (RNA-seq,proteins, ESTs), always incorporate hints via `--hints` or `--extrinsicspec`.

- Forgetting that Augustus predicts on the forward strand only by default. To predict on both strands, add `--strand=both`—otherwise antisense genes will be missed entirely.

## Examples

### Predict genes in a FASTA file using the human species model

**Args:** `--species=human input.fa --outfile=predictions.gff3 --gff3=on`
**Explanation:** This runs Augustus using the pre-trained human HMM parameters to predict protein-coding genes in the input genomic sequence, outputting in GFF3 format for compatibility with genome browsers and downstream tools.

### Generate gene predictions with RNA-seq hints for a fungal genome

**Args:** `--species=candida_albicans input.fa --hints=hints.gff --outfile=hint_predictions.gff3 --gff3=on`
**Explanation:** This incorporates RNA-seq-derived hints to guide gene prediction, significantly improving accuracy by using evidence from transcript alignment rather than relying solely on ab initio models.

### Create custom gene predictions using custom-trained species parameters

**Args:** `--species=myCustomModel input.fa --outfile=custom_predictions.gff3 --gff3=on`
**Explanation:** This uses parameters trained specifically on your organism using `augustus-train`, providing more accurate predictions than generic pre-trained species that may not reflect your organism's gene structures.

### Output predicted protein sequences alongside genomic annotations

**Args:** `--species=arabidopsis input.fa --outfile=annotations.gff3 --proteinoutput=predicted_proteins.fa --gff3=on`
**Explanation:** This generates both genome annotations and translated protein FASTA sequences, enabling immediate functional annotation through homology searches without additional translation steps.

### Predict genes on both DNA strands in a smaller genome

**Args:** `--species=fly scaffold.fa --outfile=both_strands.gff3 --strand=both --gff3=on`
**Explanation:** The `--strand=both` flag ensures predictions occur on both forward and reverse complement strands, capturing genes encoded on the negative strand which constitute roughly half of protein-coding genes.

### Run Augustus with optimized memory usage for large genomic sequences

**Args:** `--species=zebrafish chr1.fa --outfile=chr1_genes.gff3 --gff3=on --chunk=100000 --decompressprobs`
**Explanation:** Chunking breaks large chromosomes into 100kb segments for manageable memory usage and parallel processing; `--decompressprobs` reduces memory footprint for very large inputs at minimal accuracy cost.