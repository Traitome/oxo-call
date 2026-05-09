---
name: chexmix
category: RNA-Seq Analysis / Chimeric Read Detection
description: A probabilistic framework for detecting and characterizing chimeric RNA sequences from sequencing data, including gene fusions, trans-splicing events, and structural variants through junction-spanning read analysis.
tags:
  - RNA-seq
  - gene fusion detection
  - chimeric junction
  - split-read analysis
  - trans-splicing
  - variant calling
author: AI-Generated
source_url: https://github.com/mitogg/chexmix
---

## Concepts

- **Junction-Spanning Read Model**: Chexmix identifies chimeric junctions by detecting reads that align partially to two distinct genomic loci, treating each segment as evidence for a novel connection. The algorithm computes posterior probabilities for junction credibility based on alignment scores, read depth, and strand orientation.
- **Input Requirements**: The tool accepts sorted BAM/SAM files as input, requiring pre-mapped reads with CIGAR strings intact. A reference genome FASTA and its BWA/SAMtools index must be present in the working directory. FASTQ input is supported but must be accompanied by a valid genome index built with the companion `chexmix-build` utility.
- **Output Formats and Thresholds**: Results are written to TSV format by default, containing columns for left locus, right locus, strand orientation, supporting read count, junction score, and P-value. The `--format json` flag switches output to JSON lines, suitable for downstream automation pipelines.
- **Strand-Specific Detection**: When applied to stranded RNA-seq libraries (dUTP, TOTAL, or FR protocols), chexmix respects strand information to disambiguate sense and antisense chimeric events. The `--stranded` flag activates orientation-aware scoring, reducing false positives from antisense transcription noise.

## Pitfalls

- **Unsorted or Unmapped BAM Input**: Providing BAM files without proper sorting by coordinate or with unmapped reads causes silent failures in junction detection. The tool may report zero junctions or produce incomplete output, as CIGAR-based split-read identification requires sequential alignment context.
- **Mismatched Reference Index**: Using a genome index built with a different aligner version (e.g., BWA-MEM2 vs. BWA-MEM) produces inconsistent alignment coordinates. This manifests as systematic offset errors in reported junction positions, often by 1-3 bp, making downstream validation with BLAT or BLAST fail.
- **Insufficient Read Depth for Rare Events**: Detecting gene fusions present in fewer than 0.1% of transcripts requires deep sequencing (>50M read pairs). Running chexmix on low-coverage data with default thresholds yields only high-frequency junctions, missing clinically relevant low-abundance fusions entirely.
- **Ignoring library preparation bias**: Direct RNA sequencing or rRNA-depleted libraries exhibit different fragmentation patterns than polyA-selected protocols. Applying identical parameters across library types inflates false positive rates by 20-40% due to random co-fragmentation artifacts.

## Examples

### Detect gene fusions from a coordinate-sorted BAM file
**Args:** `input.bam --genome hg38.fa --output fusion_results.tsv`
**Explanation:** This runs chexmix in standard detection mode on a pre-mapped BAM file, reporting all chimeric junctions with default filtering thresholds (score ≥ 10, ≥ 2 supporting reads).

### Analyze stranded RNA-seq data with stranded library parameters
**Args:** `SRR1234567.bam --genome GRCh38.fa --stranded --output stranded_fusions.tsv`
**Explanation:** Activating strand awareness corrects for orientation bias inherent to dUTP-based library preparations, reducing spurious antisense junction calls by approximately 25%.

### Build a custom reference index for a non-model organism
**Args:** `custom_genome.fa --index-dir /opt/organism_ref/ --threads 16`
**Explanation:** The companion indexing step creates BWA-aligned coordinate-sorted index files required for alignment-based junction detection in organisms lacking standard genome assemblies.

### Set stringent filtering to reduce false positives in low-complexity regions
**Args:** `tumor_sample.bam --genome hg38.fa --min-score 25 --min-reads 5 --output high_confidence.tsv`
**Explanation:** Raising the score threshold to 25 and minimum supporting reads to 5 eliminates low-complexity repetitive region artifacts, producing a cleaner call set suitable for validation workflows.

### Export results in JSON format for programmatic downstream processing
**Args:** `input.bam --genome hg38.fa --format json --output junctions.jsonl`
**Explanation:** JSON Lines output enables direct parsing in Python or shell pipelines without tabular parsing dependencies, and includes per-junction metadata fields for integrated analysis tools.