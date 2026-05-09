---
name: asgal
category: RNA Editing Detection
description: A tool for detecting A-to-I (adenosine-to-inosine) RNA editing sites from RNA-seq alignments. It identifies hyper-edited inosine residues by comparing RNA-seq reads against a reference genome, employing filtering, coverage thresholds, and statistical validation to distinguish genuine editing events from sequencing or alignment artifacts.
tags:
  - RNA editing
  - A-to-I editing
  - RNA-seq
  - variant calling
  - inosine
  - post-transcriptional
  - epigenetics
author: AI-generated
source_url: https://github.com/UCSC-Analytics/asgal
---

## Concepts

- **A-to-I Editing Biology**: Inosine is interpreted as guanosine by reverse transcriptase and sequencing machinery, causing RNA-seq reads covering edited sites to appear as A→G mismatches relative to the reference genome. `asgal` exploits this signal by scanning aligned reads for adenine positions that show guanine nucleotides, then filters and validates clusters of such events to infer editing sites.

- **Input Requirements**: `asgal` operates on coordinate-sorted BAM files aligned to a reference genome, along with a reference genome sequence (FASTA) and gene annotation file (GTF/GFF3) to contextualize editing sites within annotated transcripts. Stranded or unstranded library protocols are supported, but strand information critically affects editing site assignment orientation; stranded libraries with correct library preparation direction are required for accurate detection.

- **Output Formats**: Results are produced in BED-like and VCF-like formats, reporting chromosome, position, strand, coverage, editing ratio (proportion of reads showing A→G at that position), and statistical significance. Editing sites are also annotated with overlapping gene names and transcript IDs, enabling downstream biological interpretation.

- **Companion Binary asgal-build**: Before running detection, the reference genome must be indexed using `asgal-build` to generate lookup tables that accelerate scanning. The indexing step is mandatory; running `asgal` without pre-built indexes will fail. The index files are genome-specific and must be regenerated if the reference genome changes.

- **Editing Ratio and Coverage Thresholds**: `asgal` applies dual thresholds: a minimum coverage (number of reads covering a site) and a minimum editing ratio (fraction of reads displaying the A→G mismatch). These parameters are tunable and represent the primary knobs for controlling sensitivity versus specificity; overly permissive thresholds produce many false positives, while overly strict thresholds miss genuine low-expression editing sites.

## Pitfalls

- **Using an Unindexed Reference Genome**: Running the detector without first running `asgal-build` produces errors or silently returns empty results. Always execute the indexing step before any detection runs, and ensure the index files remain accessible in the same directory tree as the reference genome.

- **Misinterpreting Library Strandedness**: RNA-seq libraries prepared with dUTP-based protocols generate reads in the reverse complement orientation relative to the original transcript. If the strand direction is mis-specified or assumed to be unstranded when the library is actually stranded, `asgal` will assign editing sites to the wrong strand and mis-annotate overlapping genes, corrupting downstream biological conclusions.

- **Confusing DNA Editing with RNA Editing**: `asgal` is designed specifically for A-to-I RNA editing events. Editing events involving other nucleotide changes (T→C, C→T) or genomic SNPs that produce A→G mismatches by chance are not filtered out by default. Users must independently validate that observed A→G mismatches represent genuine post-transcriptional editing rather than genomic variants, typically by comparing RNA-seq data against matched genomic DNA sequencing.

- **Insufficient Coverage for Low-Expression Sites**: Editing sites with fewer than 5–10 covering reads are statistically unreliable and `asgal` may either discard them or assign them spuriously low editing ratios. When studying tissues or conditions with low RNA-seq depth, increasing sequencing depth or aggregating replicates is necessary to achieve reliable editing site detection.

- **Inconsistent Reference Genome Version**: The alignment file (BAM), reference genome (FASTA), gene annotation (GTF), and `asgal` indexes must all correspond to the exact same genome assembly and version (e.g., GRCh38). Mixing files from different genome builds causes positional mismatches, silent mis-annotations, and nonsensical output.

## Examples

### Index a reference genome for editing site detection
**Args:** `build -r hg38.fa -o hg38_index/`
**Explanation:** The companion binary `build` constructs the lookup index from the reference FASTA file into a named output directory, which is a prerequisite for all subsequent detection runs.

### Detect A-to-I editing sites from a stranded RNA-seq BAM
**Args:** `detect -b SRR123456.sorted.bam -r hg38.fa -a hg38.gtf -i hg38_index/ --strand both -o results/`
**Explanation:** Running detection on a sorted BAM with strand-aware parameters and the required index directory produces editing site calls annotated with gene context.

### Require minimum coverage of 10 reads and editing ratio of 0.2
**Args:** `detect -b sample.bam -r hg38.fa -a hg38.gtf -i hg38_index/ --min-coverage 10 --min-editing-ratio 0.2 -o filtered_results/`
**Explanation:** Setting explicit coverage and editing ratio thresholds filters out sites supported by fewer than 10 reads or with an editing proportion below 20%, reducing false positive calls.

### Generate a VCF-formatted output for downstream analysis
**Args:** `detect -b sample.bam -r hg38.fa -a hg38.gtf -i hg38_index/ --output-format vcf -o editing_sites.vcf`
**Explanation:** Requesting VCF output produces a file compatible with standard genomic variant tools, enabling intersection with genomic databases and population frequency filters.

### Run detection on multiple BAM files in a directory
**Args:** `detect -b batch_dir/ -r hg38.fa -a hg38.gtf -i hg38_index/ -o batch_results/ --batch`
**Explanation:** Using batch mode processes all BAM files within the specified directory sequentially, producing per-sample output files and a merged summary table for cohort-level analysis.