---
name: collect_mgf
category: Mass Spectrometry
description: A command-line tool for collecting, merging, and preprocessing Mascot Generic Format (MGF) mass spectrometry data files. It aggregates multiple MGF spectra into combined outputs, filters by precursor features, and normalizes intensity values across datasets.
tags:
- mass-spectrometry
- mgf
- proteomics
- peak-collection
- bioinformatics
author: AI-generated
source_url: https://github.com/MassSpecToolkit/collect_mgf
---

## Concepts

- **MGF Format Structure**: MGF files contain spectrum entries delimited by `BEGIN IONS` and `END IONS` tags, with metadata lines (TITLE, RTINSECONDS, PEPMASS, CHARGE) followed by peak data as `m/z intensity` pairs. Each spectrum is independent, making parallel processing straightforward.

- **Input Handling**: The tool accepts multiple MGF files as input arguments or via a manifest file (one path per line). It processes files sequentially by default but supports parallel execution with `--threads` to improve throughput on multi-core systems. Input files must be valid MGF; malformed entries are skipped with warnings.

- **Output Modes**: Three primary output modes exist: `--merge` (concatenates all spectra into a single output file), `--split` (writes each input file to a separate output with prefix/suffix), and `--filter` (retains spectra matching criteria such as precursor mass range, charge state, or minimum peak count).

- **Normalization Options**: Intensity normalization can be applied via `--normalize` with methods `tic` (total ion current scaling to 1e6), `sqrt` (square root transformation), or `log` (log10 after adding 1 to avoid log(0)). Normalization occurs before filtering to ensure consistent threshold application.

---

## Pitfalls

- **Duplicate Spectrum Identification**: When merging multiple MGF files that share spectra (identical TITLE and PEPMASS values), the tool may create duplicate entries by default. Use `--unique-by title,pepmass` to deduplicate based on those fields; otherwise downstream search engines may report inflated spectral counts or false positives.

- **Charge State Parsing Errors**: MGF files often have inconsistent CHARGE declarations (e.g., "1+" vs "1+ ", missing spaces, ranges like "2-4"). The tool's `--min-charge` and `--max-charge` filters evaluate the parsed integer; malformed charge strings default to 0 and will be excluded without warning unless `--preserve-unknown-charge` is set.

- **Memory Usage with Large Files**: Processing MGF files with tens of thousands of spectra without streaming (`--no-stream`) loads entire files into memory. For datasets exceeding available RAM, operations may fail silently or produce incomplete outputs. Always use `--stream` for files >500 MB.

- **Intensity Overflow in Normalization**: When normalizing with `--normalize tic`, the scaling factor multiplies peak intensities. If original intensities are extremely high (e.g., >1e9), the output may exceed floating-point precision, resulting in truncation or scientific notation that下游 tools cannot parse. Pre-scale with `--scale-factor` before normalization.

---

## Examples

### Merge multiple MGF files into a single output
**Args:** `--merge -o combined.mgf file1.mgf file2.mgf file3.mgf`
**Explanation:** Concatenates all spectrum entries from the three input files into one MGF file named combined.mgf, preserving original metadata and peak data unchanged.

### Filter spectra by precursor mass range
**Args:** `--filter --min-mass 500 --max-mass 1500 -o filtered.mgf input.mgf`
**Explanation:** Retains only spectra where the PEPMASS value falls between 500 and 1500 Da, writing the subset to filtered.mgf; useful for targeting a specific proteomic window.

### Normalize intensities using total ion current
**Args:** `--normalize tic -o normalized.mgf raw.mgf`
**Explanation:** Scales all peak intensities so the sum equals 1,000,000 (1e6), enabling comparison between spectra acquired on different instruments or with different acquisition times.

### Remove duplicate spectra based on title and precursor mass
**Args:** `--unique-by title,pepmass -o deduped.mgf messy.mgf`
**Explanation:** Identifies spectra with matching TITLE and PEPMASS fields and keeps only the first occurrence, eliminating redundant entries that would skew quantitative analysis.

### Process MGF files in parallel with 4 threads
**Args:** `--threads 4 --merge -o parallel_combined.mgf *.mgf`
**Explanation:** Uses 4 concurrent threads to read and merge all .mgf files matching the glob pattern, significantly speeding up processing when working with many large files.

### Retain only spectra with charge state 2 or 3
**Args:** `--min-charge 2 --max-charge 3 -o doubly_triply_charged.mgf input.mgf`
**Explanation:** Filters out spectra with charge states outside the 2-3 range, which is typical for tryptic peptides analyzed in MS/MS; reduces dataset size for database searches targeting multiply-charged precursors.