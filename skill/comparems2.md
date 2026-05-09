---
name: comparems2
category: Mass Spectrometry / Metabolomics
description: A tool for comparing MS2 (tandem mass) spectra from mass spectrometry experiments. Computes spectral similarity scores, performs peak matching, and identifies analogous fragmentation patterns between query spectra and reference libraries for metabolomics annotation and quality control.
tags:
  - mass-spectrometry
  - metabolomics
  - ms2
  - spectral-comparison
  - peak-matching
  - similarity-scoring
author: AI-generated
source_url: https://github.com/comparems2/comparems2
---

## Concepts

- **Spectral Data Model**: comparems2 operates on MS2 (MS/MS) spectra containing precursor ion m/z values and their corresponding fragment ion peaks with intensities. Each spectrum is represented as a peak list (m/z, intensity pairs) and the tool computes pairwise similarity using established metrics such as dot product, cosine similarity, or weighted fragment matching.

- **Input Formats**: The tool supports standard mass spectrometry data formats including MGF (Mascot Generic Format), mzXML, mzML, and plain text peak lists. Reference libraries are typically provided as spectral databases in MGF or JSON format. Query spectra can be loaded from single files or batch-processed from directory inputs.

- **Output Modes**: comparems2 produces similarity score matrices, ranked hit lists, and alignment tables. Results can be exported as TSV, CSV, or JSON for downstream bioinformatics analysis. The tool also supports verbose text output for manual inspection of matched peaks between pairs of spectra.

- **Scoring Algorithms**: Multiple similarity metrics are available including normalized dot product, spectral entropy-based similarity, and fragment ion coverage. The default cosine-based scoring normalizes for peak intensity variations and is robust against intensity-dependent biases common in metabolomics data.

## Pitfalls

- **Mismatched Ion Polarities**: Comparing spectra from different ion polarities (positive vs. negative mode) will produce meaningless low similarity scores. Always ensure query and reference spectra were acquired in the same ionization mode, or use polarity-specific reference libraries.

- **Incorrect Precursor m/z Tolerance**: Setting the precursor tolerance too narrow will cause valid matches to be missed due to instrument mass calibration variations, while excessively wide tolerances may yield false positives from unrelated spectra. A tolerance of 0.1-0.5 Da is typical for instrument-grade data.

- **Unfiltered Noise Peaks**: Including low-intensity noise peaks in the input spectra degrades similarity calculations, especially for dot product-based metrics that are sensitive to peak density. Always apply a minimum intensity threshold (e.g., filter peaks below 1% of the base peak) before comparison.

- **Incompatible Fragment m/z Matching**: The default fragment binning for matching may not suit high-resolution instrument data. Using integer-level binning for high-res FT-MS data will artificially inflate scores for unrelated spectra; adjust the fragment tolerance to match your instrument resolution (e.g., 0.01 Da for Orbitrap data).

- **Memory Limits with Large Libraries**: Loading very large spectral libraries into memory for pairwise comparison can exhaust available RAM and cause crashes. Use batch processing or the library chunking feature when working with libraries containing millions of spectra.

## Examples

### Compare a single query spectrum against a reference library

**Args:** --query spectrum.mgf --library reference_library.mgf --output results.tsv --score cosine

**Explanation:** This compares a single query spectrum from MGF file against all spectra in the reference library using cosine similarity scoring, writing the ranked hit list to a tab-separated output file.

### Batch process multiple query spectra with parallel computation

**Args:** --query-dir ./query_spectra/ --library reference.mgf --output batch_results/ --score dot_product --threads 8

**Explanation:** This processes all spectra in the query directory in parallel using 8 threads, comparing each against the reference library and saving individual result files in the output directory.

### Adjust precursor and fragment tolerances for high-resolution data

**Args:** --query highres.mgf --library library.json --precursor-tol 0.05 --fragment-tol 0.01 --score weighted-cosine

**Explanation:** This sets tight precursor (0.05 Da) and fragment (0.01 Da) tolerances appropriate for Orbitrap high-resolution MS2 data, using a weighted cosine score that emphasizes matches at high m/z values.

### Filter peaks before comparison to remove noise

**Args:** --query noisy.mgf --library clean_reference.mgf --min-intensity 100 --output filtered_results.tsv

**Explanation:** This applies intensity filtering to remove peaks below 100 arbitrary intensity units before comparison, improving specificity by reducing noise-driven false matches.

### Export results in JSON format with detailed peak matches

**Args:** --query test.mgf --library reference.mgf --output detailed.json --format json --verbose

**Explanation:** This exports results in JSON format including verbose information about which peaks were matched between query and reference, useful for manual curation and debugging.

### Use spectral entropy-based similarity metric

**Args:** --query query.mgf --library reference.mgf --output entropy_scores.tsv --score entropy

**Explanation:** This uses spectral entropy similarity instead of cosine, which is more robust to missing peaks and can better handle incomplete fragmentation spectra common in tandem MS.

### Run with precursor isotope pattern matching enabled

**Args:** --query precursor.mgf --library reference.mgf --output matches.tsv --isotope-match --score cosine

**Explanation:** This enables precursor isotope pattern matching which considers the isotopic distribution of the precursor ion, adding confidence to matches by verifying the correct isotopic pattern alongside fragment similarity.