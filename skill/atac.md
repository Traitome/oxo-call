---
name: atac
category: Chromatin Accessibility / Epigenomics
description: A bioinformatics toolkit for processing ATAC-seq data, including quality control, alignment, peak calling, footprinting, and differential accessibility analysis. Operates on FASTQ inputs and produces genomic signal tracks and BED-format peak sets.
tags:
  - ATAC-seq
  - chromatin-accessibility
  - epigenomics
  - peak-calling
  -Tn5
  - epigenetics
  - next-generation-sequencing
author: AI-Generated
source_https://github.com/Goz3rr/ATACseq: https://github.com/Goz3rr/ATACseq
---

## Concepts

- ATAC-seq uses a hyperactive Tn5 transposase to tagment (cut and tag with sequencing adapters) open chromatin regions. The `atac` tool accepts raw FASTQ files and automates adapter trimming, alignment to a reference genome, and generation of BigWig signal tracks and BED peak files.
- The default alignment output is coordinate-sorted BAM; downstream steps (peak calling, read counting) depend on this sorted ordering, so reordering a BAM with `samtools sort` before passing it back to `atac` steps will produce incorrect or failed results.
- Chromatin accessibility is strand-specific in nature: Tn5 inserts adapters in a staggered pattern relative to开放染色质区域。`atac` applies a built-in +4/-4 bp offset correction to shifted Tn5 tags to correctly represent the underlying开放区域边界。Skipping this correction yields peaks that are systematically shifted ~50 bp upstream of their true locations.
- Peak calling in ATAC-seq benefits from a nucleosomal periodicity correction; `atac` internally uses cross-correlation between positive and negative strand read densities to infer the mononucleosomal peak and applies a dynamic fragment-length filter, ensuring that nucleosome-occupied regions are excluded from the开放 chromatin peak set.
- Blacklist regions (e.g., hg38 chrEBV, ENCODE consensus lists) contain highly accessible but biologically uninformative signal (satellite repeats, segmental duplications). Including them in statistical testing inflates the apparent number of significant peaks and can cause spurious differential calls.

## Pitfalls

- Specifying a genome identifier that is not in `atac`'s built-in genome table (e.g., `"hg38"` vs `"GRCh38"`) causes alignment to silently use the wrong chromosome sizes and BAI indices, producing a BAM that appears valid but generates incorrectly positioned peaks.
- Omitting `--no-fragment-length` when your library is single-end ATAC-seq data causes `atac` to assume a paired-end insert-size distribution and incorrectly estimates the nucleosomal fragment cutoff, resulting in inclusion of long nucleosomal fragments in the open chromatin peak set and reduced peak resolution.
- Running peak calling without a matched control (input or ATAC-seq from a knockout sample) and not setting `--no-model` causes the tool to attempt cross-correlation-based peak modeling, which can fail on low-complexity libraries (fewer than 10,000 reads) and exit with a non-descriptive error.
- Using `--threads` values greater than the number of available CPU cores causes I/O contention that paradoxically slows processing; most ATAC-seq pipelines are disk-bound during alignment, so 2–4 threads beyond the core count yields diminishing returns.
- Failure to specify `--out-dir` when running multiple samples sequentially causes output files to be written to the current working directory, and subsequent runs overwrite files from earlier samples, silently merging results and corrupting downstream differential analysis.

## Examples

### Call open chromatin peaks from a paired-end ATAC-seq FASTQ pair
**Args:** `callpeaks -t sample_R1.fastq.gz sample_R2.fastq.gz -g hg38 -o results/sample_peaks.bed --call-summits`
**Explanation:** The `-t` flag supplies both FASTQ files as a pair; `callpeaks` automatically infers paired-end mode, estimates fragment length via cross-correlation, and `--call-summits` ensures that a single summit per peak is reported for downstream motif analysis.

### Align FASTQ to mm10 and produce a sorted BAM with an inline quality filter
**Args:** `align -t mouse_ATAC_R1.fastq.gz mouse_ATAC_R2.fastq.gz -g mm10 -o aligned/mouse_atac.bam -q 30 --max-fragment-length 2000`
**Explanation:** The `-q 30` threshold removes low-mapping-quality reads that map to multiple genomic positions, reducing false-positive accessibility signals in repetitive regions. `--max-fragment-length 2000` excludes very long fragments that arise from Tn5 integration into nucleosome-bound DNA.

### Generate a BigWig coverage track normalized to reads per million mapped
**Args:** `bam-to-bigwig -i aligned/sample_sorted.bam -g hg38 -o tracks/sample_rpm.bigwig --normalize RPM`
**Explanation:** RPM normalization enables direct comparison between samples with different sequencing depths; without it, samples with deeper coverage appear to have more accessible chromatin purely as an artifact of read count.

### Perform differential accessibility analysis between treatment and control BED files
**Args:** `differential -t treatment_peaks.bed -c control_peaks.bed -g hg38 -o diff_results/deseq2_output.tsv --method DESeq2 --min-fold-change 2`
**Explanation:** The `--min-fold-change 2` flag enforces a minimum 2-fold expression change threshold, filtering out statistically significant but biologically negligible accessibility changes that are likely driven by library depth variation.

### Report cross-correlation metrics to diagnose library quality before peak calling
**Args:** `qc -i aligned/sample_sorted.bam -g hg38 -o qc_report/sample_qc.json --cross-corr --fragment-lengths`
**Explanation:** Cross-correlation analysis reports the fragment length corresponding to the nucleosomal periodicity peak; if the reported fragment length is below 100 bp or above 250 bp, the library likely has excessive free Tn5 or nucleosomal contamination, indicating a need to re-prepare the library before proceeding to peak calling.