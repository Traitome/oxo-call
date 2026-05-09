---
name: aprfinder
category: DNA Damage Detection
description: A tool for identifying apurinic/apyrimidinic (AP) sites in DNA sequences. Scans nucleotide sequences and reports the genomic coordinates and confidence scores of predicted abasic sites resulting from depurination or depyrimidination damage events.
tags: [DNA damage, AP site, abasic site, depurination, DNA repair, damage detection, genomics]
author: AI-generated
source_url: https://github.com/aprfinder/aprfinder
---

## Concepts

- **AP Site Prediction Model**: aprfinder identifies abasic sites by scanning input sequences for patterns characteristic of depurination and depyrimidination events. The algorithm evaluates local sequence context, base composition, and thermodynamic properties to assign a confidence score (0-100) to each predicted site.
- **Input Format Requirements**: The tool accepts FASTA format for single or multiple sequences, raw sequence files (.seq or .txt), and optionally Binary Alignment Map (BAM) files when aligned reads are provided with a reference genome. Header lines must begin with '>' followed by a unique sequence identifier.
- **Output Structure**: Results are delivered in three complementary formats simultaneously: (1) a BED file listing genomic coordinates of each AP site, (2) a detailed TXT report including flanking sequences (±10 bp by default) and individual confidence scores, and (3) a summary JSON file aggregating statistics across all input sequences.
- **Threshold and Filtering Behavior**: The minimum score threshold defaults to 25, meaning sites scoring below this value are excluded from output. Higher thresholds (e.g., 50, 75) progressively reduce sensitivity but increase specificity, which is critical when working with low-quality or ancient DNA where background noise is elevated.
- **Multi-Sequence Processing**: When given a multi-FASTA file, aprfinder processes each sequence independently and aggregates results into a single output set, maintaining distinct identifiers in the 'name' column of BED output. The tool does not automatically merge or deduplicate overlapping sites across different input sequences.

## Pitfalls

- **Failing to Specify Output Directory Creates Cluttered Working Directories**: If the `-o/--output` flag is omitted, aprfinder writes output files to the current working directory with default names (e.g., `aprfinder_results.bed`), overwriting any pre-existing results without warning. This is especially problematic in shared computing environments where analyses are run repeatedly.
- **Confusing Sequence Type Parameters Produces Invalid Results**: The `-t/--seq-type` parameter accepts 'dna', 'rna', or 'auto'. Specifying 'dna' when the input contains RNA sequences (or vice versa) generates systematically incorrect predictions because the algorithm applies the wrong nucleotide substitution matrices and thermodynamic parameters. The 'auto' mode is recommended for metagenomic samples with ambiguous composition.
- **Ignoring the Flanking Sequence Window Causes Downstream Annotation Errors**: By default, aprfinder extracts ±10 bp flanking each AP site. If downstream tools expect different flanking window sizes, failing to adjust `-w/--flank-window` to the correct value (e.g., ±20 or ±50) results in missing sequence context during motif annotation, potentially causing false negatives in functional enrichment analyses.
- **Using Default Threshold for Ancient or Damaged DNA Undermines Sensitivity**: For degraded ancient DNA (aDNA) with elevated background damage, the default threshold of 25 produces excessive false positives. Users must increase the threshold (typically to 40-60) to maintain acceptable precision, but this simultaneously risks losing authentic damage signals that are inherently lower-scoring due to short fragment lengths and post-mortem chemical modification.
- **Assuming Coordinate System Matches Reference Without Verification**: aprfinder outputs genomic coordinates in 1-based format by default, but can be switched to 0-based format using `--zero-based`. Many downstream tools (like BEDTools) require 0-based coordinates. Mixing coordinate systems leads to systematic 1-bp offset errors in all downstream overlap analyses, which are difficult to detect without manual validation.

## Examples

### Predict AP sites in a single FASTA sequence with default settings
**Args:** `input.fasta -o results/`
**Explanation:** Scanning a single sequence file with all default parameters outputs predicted AP sites to the specified directory, using the standard threshold of 25 and ±10 bp flanking windows. The results directory will contain BED, TXT, and JSON output files.

### Detect high-confidence AP sites in multiple sequences with stringent filtering
**Args:** `multifasta.fa -o strict_output/ -t dna --threshold 60`
**Explanation:** Processing a multi-FASTA file while applying a high confidence threshold of 60 significantly reduces the number of reported sites, retaining only those with strong evidence. The `-t dna` flag explicitly specifies DNA mode for accurate parameter selection.

### Extract AP sites from RNA sequences with extended flanking context
**Args:** `rna_transcripts.fa -o rna_results/ -t rna -w 25`
**Explanation:** When analyzing RNA sequences (which undergo depurination differently than DNA), specifying `-t rna` ensures the correct scoring model is applied. The `-w 25` flag extends the flanking window to ±25 bp, capturing larger sequence motifs required for downstream RNA structure prediction.

### Generate AP site BED file in zero-based coordinates for UCSC genome browser
**Args:** `genome_regions.fa -o browser_output/ --zero-based`
**Explanation:** Producing a zero-based BED file compatible with the UCSC Genome Browser and associated tools like BedTools and IGV. This is the standard coordinate system for genome browser tracks and must match the reference genome's convention.

### Process ancient DNA with aDNA-specific damage settings
**Args:** `ancient_sample.fa -o aDNA_output/ --threshold 45 --damage-profile extended`
**Explanation:** Analyzing degraded ancient DNA samples requires elevated thresholds and specialized damage profile modeling. The `--damage-profile extended` flag enables enhanced modeling of characteristic aDNA damage patterns (C-to-T and G-to-A transitions), reducing false positives from random degradation artifacts.

### Output only genomic positions as a minimal BED file
**Args:** `sequence.fa -o minimal/ --format bed --min-fields`
**Explanation:** Generating a stripped-down BED file containing only chromosome, start, and end coordinates without score or strand columns. This minimal format is required by certain downstream tools that cannot parse extended BED files with additional fields.

### Auto-detect sequence type with verbose logging for debugging
**Args:** `unknown_origin.fa -o debug_output/ -t auto -v`
**Explanation:** Processing sequences of uncertain nucleotide composition using automatic type detection while enabling verbose output logging (-v) to diagnose detection decisions. Verbose mode prints the detected sequence type, number of valid bases scanned, and any parsing warnings to stderr for troubleshooting.