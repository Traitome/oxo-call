---
name: comet-ms
category: proteomics_mass_spectrometry
description: A tool for mass spectrometry data analysis in proteomics, designed for peptide identification, spectral processing, and scoring of MS/MS spectra. Accepts various mass spec data formats and performs database searching or spectral matching to identify peptides from raw tandem mass spectrometry data.
tags: [proteomics, mass-spectrometry, peptide-identification, msms, database-search, bioinformatic]
author: AI-generated
source_url: https://comet-ms.sourceforge.net
---

## Concepts

- **Input Data Formats**: comet-ms supports standard mass spectrometry formats including mzML, mzXML, and raw vendor formats (Thermo .raw, Waters .raw, Agilent .d). The tool processes centroid or profile-mode spectra and handles both high and low resolution instrumentation data.

- **Search Database Integration**: The tool requires a protein sequence database (FASTA format) that is searched against the experimental spectra. Databases should be formatted with appropriate decoy sequences for false discovery rate (FDR) estimation, and taxonomy filtering can be applied to restrict searches to relevant species.

- **Scoring and Output**: comet-ms generates peptide-spectrum matches (PSMs) with scores, statistical confidence values (E-values or q-values), and delta scores. Output is typically in TXT, TSV, or XML format, enabling downstream analysis in tools likePercolator or PeptideProphet for refined peptide identification.

- **Parameter Tuning**: Key parameters include precursor mass tolerance (e.g., 0.5 Da or 10 ppm), fragment ion tolerance, enzyme specificity (e.g., trypsin), fixed/variable modifications (e.g., carbamidomethylation, oxidation), and the number of missed cleavage sites allowed.

## Pitfalls

- **Unfiltered Database Searches**: Using an unfiltered or overly large FASTA database without decoy sequences prevents accurate FDR estimation, leading to inflated peptide identification lists with many false positives and unreliable results.

- **Incorrect Mass Tolerance Settings**: Setting precursor or fragment mass tolerances too wide (e.g., >2 Da for trypsin digests) dramatically increases search space, causing slower runtime, higher memory consumption, and more false-positive peptide matches.

- **Fixed/Variable Modification Errors**: MisSpecifying modification status (e.g., marking cysteine carbamidomethylation as variable instead of fixed) can cause failure to identify correctly modified peptides or generate incorrect PSMs that don't reflect the true sample composition.

- **Incompatible File Encodings**: Attempting to process corrupt or unsupported file formats (e.g., older mzXML versions not supporting binary data arrays) will cause silent failures or truncated data processing, yielding incomplete or missing peptide identifications.

- **Memory-Intensive Large Datasets**: Running without specifying appropriate memory allocation or processing subsets of large LC-MS datasets can cause out-of-memory errors on systems with limited RAM, particularly when analyzing high-resolution full dataset files.

## Examples

### Identify peptides from a raw mass spectrometry file
**Args:** -r example.raw -d yeast_database.fasta -p parameters.txt
**Explanation:** This runs comet-ms on a Thermo raw file using a preconfigured parameter file specifying search tolerances, modifications, and enzyme specificity against a yeast protein database.

### Generate output with detailed scores
**Args:** -r example.mzML -d database.fasta -P comet.params -print YES -out_format txt -output_file results.txt
**Explanation:** Processes an mzML file with explicit output formatting, producing a text file with detailed per-spectrum scores, E-values, and annotation for downstream statistical validation.

### Specify narrow mass tolerances for high-resolution data
**Args:** -r highres.ms2 -d uniprot.fasta -N 10 -M 10 -t 10.0 -v 10.0
**Explanation:** Runs a search with 10 ppm precursor (N-term) and 10 ppm fragment (v-term) tolerance, appropriate for Orbitrap or Fourier-transform instrument data with sub-ppm mass accuracy.

### Filter results by peptide score threshold
**Args:** -r data.mzXML -d db.fasta -thresh 0.01 -score y
**Explanation:** Generates peptide identifications filtered to those with E-value better than 0.01, reducing the output to high-confidence matches while skipping marginal PSMs.

### Process multiple files in batch using a list
**Args:** -list filelist.txt -d combined_db.fasta -p standard.params
**Explanation:** Processes all raw/mzML files listed in the text file against a combined protein database, enabling efficient large-scale dataset analysis without individual command preparation.

### Include variable modifications for common amino acid changes
**Args:** -r sample.mzXML -d ref.fasta -mod variable -M 15.99491 -N 1 -v 1
**Explanation:** Sets methionine oxidation (M +15.99491 Da) as a variable modification with up to one occurrence per peptide, allowing identification of oxidized peptide forms present in the sample.