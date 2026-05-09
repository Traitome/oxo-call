---
name: chromimpute
category: epigenomics | imputation | data harmonization
description: Chromimpute is an epigenomic data harmonization and imputation tool that integrates heterogeneous chromosome-level signals across experiments, cell types, and laboratories into unified reference tracks. It models and corrects batch effects, experimental artifacts, and missing data to produce coherent imputed epigenomic maps.
tags:
  - epigenomics
  - chromatin
  - imputation
  - batch-correction
  - signal-harmonization
  - Chip-seq
  - Dnase-seq
  - reference-epigenome
  - bigwig
author: AI-generated
source_https://github.com/duke-computational-biology/chromimpute
---

## Concepts

- Chromimpute operates on chromosome-segmented signal files (bigWig, BedGraph, or ChIPnapped format) aligned to a reference genome build. All input tracks must share the same chromosome naming convention and coordinate system; mismatched builds will produce silently incorrect imputation results.
- The imputation model learns a latent representation of expected chromatin signal patterns across a reference epigenome compendium, then generates imputed values for each genomic bin by conditioning on observed signal and experimental metadata. The output is a normalized signal track with batch effects removed.
- Batch correction in Chromimpute is performed by modeling the relationship between experimental metadata (e.g., lab of origin, sequencing depth, antibody lot) and observed signal deviation from the expected pattern. Metadata must be supplied as a tab-delimited manifest file with one row per input track and columns for track ID, file path, lab, and any known covariates.
- Chromimpute outputs signal values as continuous floats per genomic bin. Downstream interpretation always requires a **seak** threshold (signal enrichment above background) chosen by the analyst; the tool does not call peaks automatically.
- The reference epigenome used for imputation strongly influences results. Using a reference compiled from different tissue types than your input sample will produce biologically meaningless imputed tracks even if numeric output is generated.

## Pitfalls

- Omitting required metadata columns from the manifest file causes Chromimpute to silently ignore batch-correction covariates, resulting in imputed tracks that still contain lab-specific biases. Always verify that the manifest columns match the expected schema.
- Using input tracks from different genome builds (e.g., hg19 mixed with hg38) produces corrupted imputation output because coordinate offsets are not detected and corrected internally. Liftover all inputs to a consistent build before running chromimpute.
- Setting the bin size too small (e.g., 10 bp for whole-genome ChIP-seq data) dramatically increases memory and runtime without improving imputation quality, since the model's training data has limited resolution at that scale. A bin size of 50–200 bp is appropriate for most use cases.
- Requesting imputation for genomic regions with insufficient training data coverage (e.g., a novel cell type not represented in any reference track) produces predictions that are extrapolations rather than interpolations, which are unreliable. Check the training data overlap before interpreting output for unusual cell types.
- Failing to specify the correct reference epigenome version causes subtle mismatches when combining imputed tracks with external annotation sets, as chromosome coordinate lengths differ between genome builds. Always document the genome build used.

## Examples

### Impute missing signal from a partial epigenomic track
**Args:** `impute --input-files /data/H3K4me3_partial.bigWig --output-dir /results/H3K4me3_imputed --manifest /data/manifest.tsv --reference /ref/epi_reference.gz --genome hg38`
**Explanation:** Imputes missing signal values in genomic bins where the partial input track has no coverage, using the reference epigenome compendium to inform predicted chromatin enrichment.

### Batch-correct signal tracks from multiple laboratories
**Args:** `batch --input-dir /data/tracks --output-dir /results/corrected --manifest /data/manifest.tsv --covariates lab_id,antibody_lot,sequencing_depth --genome hg38`
**Explanation:** Models and removes systematic differences in signal intensity caused by differences in laboratory origin and experimental protocol, so that corrected tracks are directly comparable.

### Normalize signal to a standard scale
**Args:** `normalize --input-file /data/H3K27ac.bigWig --output-file /results/H3K27ac_norm.bigWig --method quantile --genome hg38`
**Explanation:** Rescales input signal values so that the resulting track has a quantile distribution matching the expected reference distribution, enabling fair cross-track comparisons.

### Combine multiple epigenetic marks into a unified track
**Args:** `combine --input-dir /data/marks --output-file /results/combined.bigWig --operation weighted_sum --weights H3K4me3:1.0,H3K27ac:0.8,H3K27me3:-0.5 --genome hg38`
**Explanation:** Merges multiple input signal tracks into a single composite track using analyst-specified weights, producing a multi-dimensional chromatin activity summary at each genomic bin.

### Impute a specific cell-type mark using a custom reference
**Args:** `impute --input-files /data/neuron_H3K9ac.bigWig --output-dir /results/neuron_imputed --manifest /data/neuron_manifest.tsv --reference /ref/neuronal_reference.gz --genome hg38 --bin-size 100`
**Explanation:** Uses a cell-type-specific reference epigenome (rather than the generic default) to impute missing values in a histone modification track from neuronal cells, improving prediction accuracy for tissue-relevant chromatin patterns.