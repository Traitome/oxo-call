---
name: rseqc
category: qc
description: RSeQC — RNA-seq quality control toolkit; infer_experiment, read_distribution, junction_annotation, bam_stat, tin for comprehensive RNA-seq QC
tags: [rseqc, rna-seq, qc, strand, read-distribution, junction, bam, rnaseq-qc, tin, infer-experiment]
author: oxo-call built-in
source_url: "https://rseqc.sourceforge.net/"
---

## Concepts
- RSeQC is a collection of Python scripts; common tools: `infer_experiment.py`, `read_distribution.py`, `junction_annotation.py`, `junction_saturation.py`, `bam_stat.py`, `inner_distance.py`, `read_duplication.py`, `tin.py`, `geneBody_coverage.py`, `FPKM_count.py`.
- All RSeQC scripts are installed in the PATH after `pip install RSeQC` or `conda install -c bioconda rseqc`; the `.py` extension is required for most commands.
- A **BED12 reference gene model** (`-r ref.bed`) is required by most tools; download from UCSC Table Browser or convert GTF with `gtfToGenePred | genePredToBed` or use bundled models.
- `infer_experiment.py` detects library strandedness (sense/antisense/unstranded) by comparing read orientations to the gene model; crucial before running featureCounts/STAR/HTSeq.
- `read_distribution.py` reports the fraction of reads mapping to CDS, UTRs, introns, intergenic regions; useful for assessing RNA-seq data quality.
- `junction_annotation.py` classifies splice junctions as known (in annotation), partial-novel (one end known), or complete-novel; outputs BED and R plots.
- `tin.py` (Transcript Integrity Number): measures mRNA degradation per transcript (0–100); median TIN < 70 indicates significant RNA degradation.
- `bam_stat.py` provides a quick summary of a BAM file: total reads, mapped, paired, uniquely mapped, etc.
- `inner_distance.py` estimates the inner distance (fragment size minus read lengths) for paired-end libraries; needed to set STAR `--alignMatesGapMax`.
- `geneBody_coverage.py` calculates read coverage over gene bodies to detect 5' or 3' bias; useful for assessing RNA integrity and library preparation quality.
- `FPKM_count.py` calculates raw fragment count, FPM (fragments per million), and FPKM for each gene; useful for expression-level QC before differential analysis.
- Input BAM files must be coordinate-sorted and indexed; run `samtools sort` and `samtools index` first.
- Most RSeQC scripts accept multiple input BAM files separated by commas, a text file with BAM paths (one per line), or a directory containing BAM files.

## Pitfalls
- RSeQC is a multi-script suite. Each tool is a separate Python script: infer_experiment.py, read_distribution.py, junction_annotation.py, junction_saturation.py, bam_stat.py, inner_distance.py, read_duplication.py, tin.py, geneBody_coverage.py, FPKM_count.py, etc. There is NO single 'rseqc' command. Each script is invoked directly by name with its own flags.
- The `.py` suffix is required on most RSeQC commands (e.g. `infer_experiment.py`, not `infer_experiment`); scripts are installed as standalone executables.
- BED reference file must match the genome assembly used for alignment (chromosome names must match exactly — chr1 vs 1); mismatches silently give incorrect results.
- `infer_experiment.py` samples only 200,000 reads by default; increase with `-s 2000000` for more accurate strandedness detection in lowly expressed transcriptomes.
- `tin.py` is very slow on large BAMs; limit to a representative subset of transcripts with `-c` to reduce runtime.
- interpreting `infer_experiment.py` results: "1++,1--" means reads map in sense direction (forward strand library); "1+-,1-+" means antisense; "Undetermined" means unstranded.
- RSeQC uses a lot of memory for WGS-scale BAMs; for RNA-seq QC, sub-sample to 20M reads first with `samtools view -s 0.1` if runtime is an issue.
- Python 3 is required for RSeQC ≥ 4.0; older versions require Python 2.7; check with `python --version` and install via conda if needed.
- `geneBody_coverage.py` requires a housekeeping gene list for accurate coverage estimation; without it, results may be biased toward highly expressed genes.
- `FPKM_count.py` only accepts BAM format (not SAM); ensure input files are properly sorted and indexed BAM files.
- Some scripts like `bam2wig.py` require chromosome size files (`-s chrom.sizes`) which must be downloaded separately from UCSC.

## Examples

### infer library strandedness with infer_experiment.py
**Args:** `infer_experiment.py -r hg38.bed -i sorted.bam -s 2000000`
**Explanation:** infer_experiment.py command; -r provides the BED12 gene model; -s 2000000 samples 2M reads for accuracy; output shows fraction of reads supporting each strand orientation

### get read distribution with read_distribution.py
**Args:** `read_distribution.py -r hg38.bed -i sorted.bam`
**Explanation:** read_distribution.py command; reports percentage of reads in CDS exon, UTR, intron, intergenic etc.; useful for detecting genomic contamination or rRNA enrichment

### annotate splice junctions with junction_annotation.py
**Args:** `junction_annotation.py -r hg38.bed -i sorted.bam -o sample_junctions`
**Explanation:** junction_annotation.py command; -o sets the output prefix; produces sample_junctions.junction.bed, .xls, .r, and .pdf files; classifies novel vs known junctions

### check saturation of junction detection with junction_saturation.py
**Args:** `junction_saturation.py -r hg38.bed -i sorted.bam -o sample_sat`
**Explanation:** junction_saturation.py command; estimates how many more junctions would be discovered with more sequencing depth; plots detection saturation curve

### compute BAM statistics with bam_stat.py
**Args:** `bam_stat.py -i sorted.bam`
**Explanation:** bam_stat.py command; prints total reads, mapped, paired, uniquely mapped, duplicates; quick sanity check for alignment success before downstream analysis

### measure transcript integrity (RNA quality)
**Args:** `tin.py -i sorted.bam -r hg38.bed`
**Explanation:** tin.py command; computes TIN score per transcript (0-100); median TIN < 70 indicates RNA degradation; output tab-delimited; generates summary.txt and .xls

### estimate inner distance with inner_distance.py
**Args:** `inner_distance.py -r hg38.bed -i sorted.bam -o inner_dist`
**Explanation:** inner_distance.py command; estimates fragment insert size distribution; output .pdf shows inner distance histogram; median value guides STAR/HISAT2 settings

### check read duplication rate with read_duplication.py
**Args:** `read_duplication.py -i sorted.bam -o duplication`
**Explanation:** read_duplication.py command; plots read duplication rate by read occurrence count; high duplication (>50%) indicates over-amplification or low-complexity library

### calculate gene body coverage with geneBody_coverage.py
**Args:** `geneBody_coverage.py -r hg38.housekeeping.bed -i sorted.bam -o geneBody_cov`
**Explanation:** geneBody_coverage.py command; -r provides a housekeeping gene BED file for unbiased coverage estimation; detects 5' or 3' coverage bias indicating RNA degradation or library prep issues; outputs .txt, .pdf, and .r files

### calculate FPKM counts with FPKM_count.py
**Args:** `FPKM_count.py -i sorted.bam -r hg38.bed -o fpkm_counts`
**Explanation:** FPKM_count.py command; calculates raw fragment count, FPM, and FPKM per gene; useful for expression-level QC and cross-sample comparison before differential analysis; outputs tab-delimited text file

### convert BAM to BigWig with bam2wig.py
**Args:** `bam2wig.py -i sorted.bam -s hg38.chrom.sizes -o output -u`
**Explanation:** bam2wig.py command; -s provides chromosome sizes file from UCSC; -u skips multi-mapping reads; automatically converts to BigWig if wigToBigWig is in PATH; useful for genome browser visualization
