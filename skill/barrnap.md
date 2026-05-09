---
name: barrnap
category: Genome Annotation
description: A tool for identifying ribosomal RNA genes in nucleotide sequences using HMM profiles. Detects 5S, 16S, 23S rRNA in bacteria/archaea and 18S, 5.8S, 28S in eukaryotes.
tags: [rRNA, annotation, HMM, genome, prokaryotes, eukaryotes]
author: AI-generated
source_url: https://github.com/tseemann/barrnap
---

## Concepts

- **Input format**: Barrnap accepts FASTA format files containing nucleotide sequences (genomes, contigs, or assembled reads). It does NOT accept protein sequences—use a translation tool first if working with protein data.
- **Output format**: By default, barrnap outputs GFF3 format (compatible with most genome browsers and annotation tools). Use `--outfmt gff` for explicit GFF3 or `--outfmt text` for a simple tabular summary.
- **Kingdom selection**: The `--kingdom` flag is mandatory and determines which HMM profiles are used: `bac` (bacteria, detects 5S/16S/23S), `arc` (archaea, detects 5S/16S/23S), or `euk` (eukaryotes, detects 18S/5.8S/28S). Specifying the wrong kingdom produces no results or irrelevant hits.
- **HMM-based detection**: Barrnap uses bundled HMM profiles (from Infernal and RFAM models) to identify rRNA genes. The sensitivity depends on the evalue threshold—lower values yield fewer but more confident predictions.
- **Threading**: Use `--threads` to parallelize detection across multiple CPU cores, significantly speeding up analysis of large genomes or multiple contigs.

## Pitfalls

- **Omitting the `--kingdom` flag**: Without specifying `bac`, `arc`, or `euk`, barrnap may fail to detect any rRNA genes or produce spurious hits from the wrongorganism models. Always verify the expected kingdom matches your input sequence.
- **Using protein sequences as input**: Barrnap is designed for nucleotide data only. Providing protein translations causes the tool to report zero hits or exit with an error, wasting analysis time.
- **Setting `--evalue` too high**: An excessively loose evalue threshold (e.g., `--evalue 1e-1`) includes weak alignments that are not true rRNA genes, leading to false-positive annotations in downstream steps.
- **Ignoring strand orientation**: Barrnap reports predictions on both forward (`+`) and reverse (`-`) strands. Failing to check the strand column in GFF3 output can cause confusion when interpreting gene directionality.
- **Assuming output overwrite**: By default, barrnap overwrites the output file without prompting. Running barrnap twice on different inputs to the same file will silently replace the first results.

## Examples

### Detect rRNA genes in a bacterial genome assembly
**Args:** `--kingdom bac --threads 4 input.fna`
**Explanation:** This runs barrnap on a bacterial genome using 4 CPU threads; the `bac` kingdom ensures 5S, 16S, and 23S rRNA HMM models are searched.

### Find eukaryotic rRNA contigs in a metagenome assembly
**Args:** `--kingdom euk --evalue 1e-5 assembly.fna`
**Explanation:** The `euk` kingdom switches to 18S/5.8S/28S models, and the stricter evalue reduces false positives from non-rRNA sequence similarities.

### Generate tabular output instead of GFF3
**Args:** `--kingdom bac --outfmt text --out rRNA_hits.txt genomes.fna`
**Explanation:** Using `--outfmt text` produces a simple tab-separated table with coordinates, making downstream parsing in standard shell scripts easier.

### Specify a custom HMM database location
**Args:** `--kingdom arc --dbdir /path/to/custom/hmmprofiles sequence.fna`
**Explanation:** If using custom or updated RFAM models stored in a non-default directory, `--dbdir` points barrnap to the correct HMM files.

### Run with low memory footprint by limiting threads
**Args:** `--kingdom bac --threads 1 --evalue 1e-3 contigs.fna`
**Explanation:** Single-threaded execution (`--threads 1`) reduces RAM usage on memory-constrained systems while still detecting rRNA at a moderate confidence threshold.

### Extract only 16S predictions from bacterial data
**Args:** `--kingdom bac --outfmt text --out 16S_only.txt genome.fna | grep -i 16S`
**Explanation:** After generating text output, piping through grep isolates 16S rRNA predictions specifically; the kingdom ensures 16S models are included in the search.