---
name: atlas
category: RNA-seq Analysis
description: Genome-guided transcript assembler and expression quantifier for RNA-seq data. Atlas takes sorted BAM alignment files and assembles transcripts, estimates expression in FPKM/RPKM, and outputs GTF annotations.
tags: [rna-seq, transcript-assembly, expression-quantification, genome-guided, fpkm]
author: AI-generated
source_url: https://github.com/atlas-org/atlas
---

## Concepts

- **Input Format**: Atlas requires position-sorted BAM files from RNA-seq aligners (e.g., STAR, TopHat). The BAM must be indexed and contain spliced alignment information with N/CIGAR operations for introns.
- **Output Modes**: Atlas operates in two primary modes: `assemble` builds novel transcript isoforms from alignments, and `quantify` computes expression values (FPKM/RPKM) against a reference annotation.
- **Data Model**: Transcripts are modeled as collections of exons with start/end coordinates, strand orientation, and assigned expression values stored in GTF and TXT output files.
- **Multi-sample Handling**: Multiple BAM files can be processed in a single run to enable differential expression comparison across conditions; sample information is specified via a sample sheet.

## Pitfalls

- **Unsorted or Misaligned BAM Input**: Providing a name-sorted instead of position-sorted BAM causes silent failures or incomplete assemblies because atlas traverses genomic coordinates in order.
- **Missing Read Groups**: BAM files lacking proper read group tags trigger downstream expression quantification errors, as atlas cannot correctly attribute reads to samples.
- **Incompatible Reference Genomes**: Using a different genome build (e.g., hg19 vs. hg38) between the aligner index and atlas reference causes massive mis-mapping of alignments to incorrect loci.
- **Memory Exhaustion with Large Datasets**: Processing whole-genome RNA-seq datasets without adjusting the `--memory` parameter leads to crashes; assemblies with millions of transcript fragments require proportional RAM allocation.

## Examples

### Assemble transcripts from an RNA-seq alignment file
**Args:** assemble --input sorted.bam --reference ref.gtf --output_dir ./assembly/
**Explanation:** This runs the assembler in genome-guided mode, using existing annotations to constrain transcript construction while discovering novel isoforms present in the alignment file.

### Quantify expression against a reference annotation
**Args:** quantify --input sample1.bam --annotation ref.gtf --output expression_out.txt
**Explanation:** Computes FPKM expression values for each transcript in the provided GTF by normalizing read counts by transcript length and total mapped reads.

### Run both assembly and quantification in one command
**Args:** run --input sample1.bam --reference ref.gtf --output_dir ./combined_results/ --mode both
**Explanation:** Executes the complete atlas pipeline, first assembling novel transcripts then quantifying all isoforms (reference and novel) in a single workflow.

### Process multiple samples with a sample manifest
**Args:** quantify --manifest sample_sheet.csv --reference ref.gtf --output_dir ./de_analysis/
**Explanation:** Reads a CSV containing sample ID and BAM file paths for all conditions, then computes expression matrices across all samples suitable for differential expression analysis.

### Adjust memory allocation for large genomes
**Args:** assemble --input large_sample.bam --reference hg38.gtf --output_dir ./results/ --memory 32G
**Explanation:** Explicitly allocates 32GB of RAM to accommodate the computational overhead of assembling transcripts from a complex mammalian dataset with high read depth.