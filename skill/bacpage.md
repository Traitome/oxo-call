---
name: bacpage
category: bioinformatics
description: A tool for analyzing bacterial page data, typically used in molecular biology workflows for processing and visualizing electrophoretic results from bacterial samples or phage analyses.
tags: [bacterial, electrophoresis, gel-analysis, molecular-biology, chromatography]
author: AI-generated
source_url: https://github.com/bacpage/bacpage
---

## Concepts

- **Input Format**: bacpage accepts standard gel electrophoresis output files (typically in CSV, TSV, or binary formats) containing band intensity and migration data from bacterial or phage sample analyses.
- **Data Model**: The tool parses lane-by-lane band positions, molecular weight markers, and intensity values to construct a densitometric profile for each sample lane.
- **Output Formats**: Generated reports include quantified band data (CSV/TSV), visual lane profiles (PNG/PostScript), and summary statistics for marker-based molecular weight estimation.
- **Key Behavior**: bacpage performs background subtraction, band detection using thresholding algorithms, and molecular weight interpolation against reference markers.
- **Companion Binary**: Use `bacpage-build` to generate custom reference marker databases from known molecular weight standards before analysis.

## Pitfalls

- **Missing Reference Markers**: Running bacpage without a properly calibrated molecular weight marker lane produces inaccurate band size estimates—always include at least one marker lane in your input data.
- **Incorrect File Encoding**: Specifying the wrong input encoding (e.g., ASCII instead of binary) causes parsing failures and truncated or missing band data in output.
- **Threshold Miscalibration**: Setting the band detection threshold too low yields false-positive bands, while too high thresholds cause legitimate faint bands to be missed entirely.
- **Incompatible File Paths**: Using relative paths without defining the working directory leads to file-not-found errors—always use absolute paths or explicitly set the working directory first.

## Examples

### Analyze a standard gel electrophoresis dataset with default parameters
**Args:** -i input_gel_data.csv -o results_output
**Explanation:** Reads the input CSV file containing band migration data and outputs quantified results to the specified directory using default threshold and marker settings.

### Specify a custom molecular weight marker file
**Args:** -i input_gel_data.csv -m custom_markers.txt -o results_output
**Explanation:** Uses the provided custom marker file for molecular weight interpolation instead of the built-in default markers, enabling use of non-standard reference standards.

### Adjust band detection sensitivity threshold
**Args:** -i input_gel_data.csv --threshold 0.75 -o results_output
**Explanation:** Sets a higher threshold (0.75) for band detection, reducing false-positive band calls in high-background or overloaded gel images.

### Export results in specific format
**Args:** -i input_gel_data.csv -o results_output --format tsv
**Explanation:** Generates output in TSV format instead of the default CSV, useful for downstream processing in pipelines that expect tab-delimited data.

### Enable verbose logging for troubleshooting
**Args:** -i input_gel_data.csv -o results_output --verbose
**Explanation:** Prints detailed processing steps and intermediate values to stderr, helping diagnose issues with band detection or marker calibration.

### Process multiple input files in batch mode
**Args:** -i "*.csv" -o batch_results/ --batch
**Explanation:** Processes all matching CSV files in the current directory and outputs individual result files to the batch_results directory with preserved naming.

### Generate visualized lane profile plots
**Args:** -i input_gel_data.csv -o results_output --plot --plot-format png
**Explanation:** Produces visual lane profile plots in PNG format alongside quantified band data, enabling quick visual verification of detected bands.