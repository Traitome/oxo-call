---
name: breaktracer
category: Structural Variant Analysis
description: A tool for tracing and characterizing genomic breakpoints from structural variant calls. Reads VCF/BCF files or tabular breakpoint reports and reconstructs breakpoint-spanning sequences using a reference genome, enabling validation, annotation, and visualization of structural variant junctions.
tags:
  - structural-variants
  - breakpoints
  - vcf
  - genome-alignment
  - sv-validation
  - junction-analysis
author: AI-generated
source_url: https://github.com/breaktracer/breaktracer
---

## Concepts

- **Breakpoint Model**: Each structural variant is represented by a breakpoint pair (left/right coordinates on the reference). Breaktracer fetches flanking sequences from the reference genome, assembles junction-spanning contigs, and writes aligned breakpoint graphs in JSON or dot format for downstream inspection.
- **Input Formats**: Primary inputs are VCF/BCF files (annotated with INFO/ALT entries) or tab-delimited breakpoint tables with columns `chrom1`, `pos1`, `chrom2`, `pos2`, `svtype`. Plain FASTA/FASTQ sequences can also be passed directly with `--query-seqs`. The tool auto-detects file format from the extension.
- **Reference Genome**: A reference FASTA file must be indexed with a companion `.fai` file. Breaktracer uses `htslib` internally for random-access fetching, so the reference must be bgzipped. Any species with a standard genomic reference is supported, provided the chromosome names in the VCF match the reference sequence dictionary.
- **Output Modes**: Default output is a TSV report with columns `sv_id`, `chrom1`, `pos1`, `chrom2`, `pos2`, `svtype`, `flank_left`, `flank_right`, `junc_seq`, `entropy_score`. Using `--output-graph` produces a GML graph file for network analysis. The `--verbose` flag appends per-breakpoint alignment cigar strings to the report.
- **Scoring and Filtering**: Breaktracer computes an entropy-based confidence score (0–100) for each breakpoint based on sequence complexity in the junction region. Breakpoints with scores below the `--min-score` threshold (default 20) are flagged but not discarded unless `--drop-low-confidence` is set.

## Pitfalls

- **Mismatched chromosome naming conventions**: If your VCF uses chr-prefixed chromosome names (e.g., chr1) but the reference index uses non-chr names (1), Breaktracer silently returns empty flanking sequences for every entry. Always verify that `--reference` sequence dictionary matches your VCF `##contig` headers.
- **Missing or unindexed reference FASTA**: Running Breaktracer without a corresponding `.fai` index on the reference FASTA causes the process to abort with a generic "file not found" error even though the FASTA itself exists. Build the index with `samtools faidx reference.fa` before running any Breaktracer command.
- **Incorrectly paired INFO fields in VCF**: Breaktracer expects a `MATEID` or `CHR2/POS2` field for translocation-style breakpoints. If these fields are absent, the tool treats each record as a deletion or inversion without checking partner coordinates, leading to spurious `svtype` assignments in the output report.
- **Using `--drop-low-confidence` with a high `--min-score` threshold**: Setting `--min-score 80` combined with `--drop-low-confidence` can silently discard the majority of real breakpoints, especially for low-coverage or high-repetitiveness regions. Always inspect the count summary printed to stderr before trusting a filtered dataset.
- **Output file overwrites without confirmation**: When `--output` points to an existing file, Breaktracer appends new rows without prompting. This can cause duplicate entries in downstream analyses if the same run is accidentally repeated.

## Examples

### Trace breakpoints from a VCF file using the GRCh38 reference
**Args:** `--input-vcf variants.vcf.gz --reference GRCh38.fa --output breakpoint_report.tsv`
**Explanation:** Reads each VCF record, fetches 100 bp of flanking sequence on each side of every breakpoint from GRCh38, assembles the junction, and writes the summary TSV.

### Generate a breakpoint graph in GML format for visualization in Cytoscape
**Args:** `--input-breakpoints bp_table.tsv --reference hg38.fa --output-graph bp_network.gml`
**Explanation:** Reads a tab-delimited breakpoint table (instead of VCF), constructs a graph where nodes are genomic positions and edges are structural variant connections, and exports it as GML.

### Trace breakpoints with custom flanking window size and minimum entropy score
**Args:** `--input-vcf small_sv.vcf --reference hg38.fa --flank-size 200 --min-score 50 --output filtered_report.tsv`
**Explanation:** Fetches 200 bp on each side of every breakpoint (instead of the default 100) and only includes breakpoints with an entropy score of at least 50 in the output TSV.

### Process a query FASTA of breakpoint-spanning contigs and write a JSON alignment report
**Args:** `--query-seqs contigs.fa --reference hg38.fa --output-alignment alignments.json --format json`
**Explanation:** Directly aligns user-supplied FASTA contigs against the reference without a VCF, performs local alignment around putative junctions, and writes per-contig alignment records in JSON.

### Trace breakpoints with verbose cigar output and low-confidence entries dropped
**Args:** `--input-vcf structural_variants.vcf.gz --reference hg38.fa --drop-low-confidence --verbose --output verbose_report.tsv`
**Explanation:** Removes breakpoints scoring below the default threshold (20) from the output and appends per-breakpoint alignment cigar strings to each row of the TSV report for manual inspection of alignment quality.