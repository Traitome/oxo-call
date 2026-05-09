---
name: chips
category: chip-seq-analysis
description: CHiPS (ChIP-Seq Processor) identifies protein-DNA binding sites from high-throughput sequencing data by calling peaks against a control sample. Accepts sorted BAM files and produces BED, narrowPeak, and bigWig outputs.
tags: [chip-seq, peak-calling, transcription-factor, histone-mark, nucleosome]
author: AI-generated
source_url: https://github.com/hbctraining/chips
---

## Concepts

- CHiPS models the shift size of paired-end reads to precisely locate nucleosome dyad positions. The shift is estimated dynamically from the data rather than using a fixed value, making the tool particularly accurate for histone modification maps where fragment lengths vary.
- Input requires exactly two sorted BAM files: the ChIP sample and the matched control (input) sample. The tool will not run without a control; using an unmatched or inappropriate control degrades peak quality and introduces false positives from开放式染色质 regions.
- Output files follow ENCODE standards: the `_peaks.narrowPeak` file contains q-value (FDR) and p-value columns compatible with downstream tools like IDR and great_expectations. The `_summits.bed` file enumerates the highest point within each peak, critical for motif analysis with tools like Homer or MEME.
- The `--extsize` parameter defines the fragment length when the automatic model fails, such as for extremely broad domains spanning >30kb in histone ChIP. The `--broad` flag changes the internal algorithm to use a sliding window approach that better captures diffuse enrichment patterns.

## Pitfalls

- Running without a control sample produces misleading peaks over open chromatin regions that are present in both the ChIP and input but are not genuine binding sites. This results in biologically incorrect conclusions and failed validation experiments.
- Using an incorrect `--format` setting causes the tool to mis-calculate strand cross-correlation and fragment length estimates, directly reducing peak calling sensitivity and accuracy.
- Specifying an overly large `--nomodel` extension without validating fragment length independently produces artificially extended peaks with fuzzy boundaries, making motif discovery unreliable.
- Filtering with `--qvalue` threshold set below 0.01 discards weak but biologically meaningful binding sites, particularly relevant for pioneer transcription factors with transient chromatin interactions.

## Examples

### Call narrow peaks for a transcription factor ChIP-seq experiment
**Args:** `treatment sample.bam control input.bam --name TF_experiment --format bam`
**Explanation:** This runs standard peak calling with automatic fragment length estimation, suitable for punctate transcription factor binding with clear signal peaks.

### Generate broad domains for H3K27ac histone modification
**Args:** `treatment H3K27ac_chip.bam control H3K27ac_input.bam --name H3K27ac_broad --broad --qvalue 0.1`
**Explanation:** The broad flag enables sliding-window peak calling to capture the diffuse enrichment characteristic of acetyltransferase targets, and a relaxed q-value captures weaker domains.

### Force a specific fragment length when auto-detection fails
**Args:** `treatment sample.bam control input.bam --name fixed_length --extsize 200 --nomodel`
**Explanation:** The nomodel flag bypasses automatic shift estimation and uses the specified extension size, necessary when sequencing quality or library complexity prevents accurate model building.

### Produce only summit positions for motif analysis
**Args:** `treatment sample.bam control input.bam --name summits_only --call-summits --min-length 100`
**Explanation:** The call-summits output restricts results to peak summits, ideal when preparing input for motif discovery workflows that require precise coordinate lists.

### Run with paired-end reads and gapped output
**Args:** `treatment PE_chip.bam control PE_input.bam --name paired_end --format BAMPE --gsize mm`
**Explanation:** The BAMPE format instructs the tool to calculate fragment length from insert size embedded in paired-end alignment metadata, and gsize sets the effective genome size for mappability calculations.

### Generate bigWig for genome browser visualization
**Args:** `treatment sample.bam control input.bam --name track_build --outdir results/ --trackline`
**Explanation:** The trackline flag generates UCSC-compatible browser annotation files, enabling direct upload to genome browsers for cross-sample comparison and publication figures.

### Call peaks with strict significance threshold
**Args:** `treatment sample.bam control input.bam --name high_confidence --qvalue 0.01 --min-fold 5`
**Explanation:** This combination of high significance and minimum fold enrichment filters produces a conservative peak set suitable for CRISPR guide design or validation experiments.

### Process ChIP-exo data with single-base resolution
**Args:** `treatment ChipExo.bam control Input.bam --name exo_peaks --format bampe --nomodel --extsize 1`
**Explanation:** CHiP-exo produces near-single-nucleotide resolution, so the minimal extension size of 1 prevents artificially broadening binding site coordinates.

---