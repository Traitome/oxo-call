---
name: rseqc
category: quality-control
description: RSeQC — RNA-seq quality control toolkit; infer_experiment, read_distribution, junction_annotation, bam_stat, tin for comprehensive RNA-seq QC
tags: [rseqc, rna-seq, qc, strand, read-distribution, junction, bam, rnaseq-qc, tin, infer-experiment]
author: oxo-call built-in
source_url: "https://rseqc.sourceforge.net/"
---

## Concepts
- RSeQC is a collection of Python scripts; common tools: `infer_experiment.py`, `read_distribution.py`, `junction_annotation.py`, `junction_saturation.py`, `bam_stat.py`, `inner_distance.py`, `read_duplication.py`, `tin.py`.
- All RSeQC scripts are installed in the PATH after `pip install RSeQC` or `conda install -c bioconda rseqc`; the `.py` extension is required for most commands.
- A **BED12 reference gene model** (`-r ref.bed`) is required by most tools; download from UCSC Table Browser or convert GTF with `gtfToGenePred | genePredToBed` or use bundled models.
- `infer_experiment.py` detects library strandedness (sense/antisense/unstranded) by comparing read orientations to the gene model; crucial before running featureCounts/STAR/HTSeq.
- `read_distribution.py` reports the fraction of reads mapping to CDS, UTRs, introns, intergenic regions; useful for assessing RNA-seq data quality.
- `junction_annotation.py` classifies splice junctions as known (in annotation), partial-novel (one end known), or complete-novel; outputs BED and R plots.
- `tin.py` (Transcript Integrity Number): measures mRNA degradation per transcript (0–100); median TIN < 70 indicates significant RNA degradation.
- `bam_stat.py` provides a quick summary of a BAM file: total reads, mapped, paired, uniquely mapped, etc.
- `inner_distance.py` estimates the inner distance (fragment size minus read lengths) for paired-end libraries; needed to set STAR `--alignMatesGapMax`.
- Input BAM files must be coordinate-sorted and indexed; run `samtools sort` and `samtools index` first.

## Pitfalls
- The `.py` suffix is required on most RSeQC commands (e.g. `infer_experiment.py`, not `infer_experiment`); scripts are installed as standalone executables.
- BED reference file must match the genome assembly used for alignment (chromosome names must match exactly — chr1 vs 1); mismatches silently give incorrect results.
- `infer_experiment.py` samples only 200,000 reads by default; increase with `-s 2000000` for more accurate strandedness detection in lowly expressed transcriptomes.
- `tin.py` is very slow on large BAMs; limit to a representative subset of transcripts with `-c` to reduce runtime.
- DANGER: interpreting `infer_experiment.py` results: "1++,1--" means reads map in sense direction (forward strand library); "1+-,1-+" means antisense; "Undetermined" means unstranded.
- RSeQC uses a lot of memory for WGS-scale BAMs; for RNA-seq QC, sub-sample to 20M reads first with `samtools view -s 0.1` if runtime is an issue.
- Python 3 is required for RSeQC ≥ 4.0; older versions require Python 2.7; check with `python --version` and install via conda if needed.

## Examples

### infer library strandedness from a BAM
**Args:** `infer_experiment.py -r hg38.bed -i sorted.bam -s 2000000`
**Explanation:** -r provides the BED12 gene model; -s 2000000 samples 2M reads for accuracy; output shows fraction of reads supporting each strand orientation

### get read distribution across genomic features
**Args:** `read_distribution.py -r hg38.bed -i sorted.bam`
**Explanation:** reports percentage of reads in CDS exon, UTR, intron, intergenic etc.; useful for detecting genomic contamination or rRNA enrichment

### annotate splice junctions
**Args:** `junction_annotation.py -r hg38.bed -i sorted.bam -o sample_junctions`
**Explanation:** -o sets the output prefix; produces sample_junctions.junction.bed, .xls, .r, and .pdf files; classifies novel vs known junctions

### check saturation of junction detection
**Args:** `junction_saturation.py -r hg38.bed -i sorted.bam -o sample_sat`
**Explanation:** estimates how many more junctions would be discovered with more sequencing depth; plots detection saturation curve

### compute BAM statistics
**Args:** `bam_stat.py -i sorted.bam`
**Explanation:** prints total reads, mapped, paired, uniquely mapped, duplicates; quick sanity check for alignment success before downstream analysis

### measure transcript integrity (RNA quality)
**Args:** `tin.py -i sorted.bam -r hg38.bed`
**Explanation:** computes TIN score per transcript (0-100); median TIN < 70 indicates RNA degradation; output tab-delimited; generates summary.txt and .xls

### estimate inner distance for paired-end RNA-seq
**Args:** `inner_distance.py -r hg38.bed -i sorted.bam -o inner_dist`
**Explanation:** estimates fragment insert size distribution; output .pdf shows inner distance histogram; median value guides STAR/HISAT2 settings

### check for read duplication rate
**Args:** `read_duplication.py -i sorted.bam -o duplication`
**Explanation:** plots read duplication rate by read occurrence count; high duplication (>50%) indicates over-amplification or low-complexity library
