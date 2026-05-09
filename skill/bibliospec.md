---
name: bibliospec
category: Mass Spectrometry / Spectral Library Creation
description: Converts MS/MS search results and raw spectral data into BiblioSpec spectral library format (.blib SQLite database) for use in targeted proteomics workflows with Skyline.
tags: [proteomics, msms, spectral-library, conversion, blib, sqmass, sky]
author: AI-generated
source_url: https://github.com/ProteoWizard
---

## Concepts

- **Output Format (.blib):** Bibliospec produces a SQLite-based spectral library file (`.blib`) containing peptide sequences, precursor masses, fragmentation ions, and associated protein metadata. This format is the native library format for Skyline and supports targeted proteomics quantitation.
- **Input Format Flexibility:** The tool accepts multiple MS/MS search result formats including MGF (Mascot Generic Format), ms2, SQT, mascot DAT files, pepXML, and mzIdentML. Each format requires specific flags to correctly parse fragmentation patterns and peptide assignments.
- **Enzyme and Cleavage Specificity:** Bibliospec models tryptic cleavage rules to define peptide boundaries. The `--enzyme` flag controls which protease rule is applied (e.g., trypsin, chymotrypsin, lys-c), directly affecting which peptides enter the library and their correct monoisotopic masses.
- **Charge State and Fragment Ion Handling:** Input files may contain singly, doubly, or triply charged precursors. The `--charge` flag specifies which charge states to include, and ion type annotations (b-ions, y-ions, neutral losses) are extracted based on the `--ion-type` specification.
- **Instrument and Collision Energy Models:** Fragmentation patterns vary by instrument type (ion trap, Q-TOF, Orbitrap). The `--instrument` flag selects the appropriate theoretical ion series coefficients used when creating predicted spectra from sequence data alone.

## Pitfalls

- **Mismatched File Extensions:** Providing an .mzML file directly without search results is incorrect—bibliospec expects search result files (MGF, DAT, pepXML), not raw MS data. Raw files must first be processed through a search engine before conversion.
- **Incorrect Precursor Mass Tolerance:** Specifying `--pm-tolerance` with a unit mismatch (e.g., setting 0.5 when the format requires parts-per-million) produces systematically shifted peptides, rendering the library unsuitable for targeted workflow calibration.
- **Missing or Incompatible Ion Types:** Omitting `--ion-type` when the input file lacks explicit fragment annotations causes bibliospec to fall back to default ion prediction, which may not match the actual fragmentation observed on the instrument.
- **Duplicate Peptide Entries:** Running bibliospec on overlapping input files without deduplication creates redundant peptide entries in the .blib, inflating library size and causing ambiguous identification in Skyline.
- **Enzyme Misconfiguration:** Selecting the wrong `--enzyme` value (e.g., trypsin for a non-tryptic search) leads to incorrect cleavage site assignments and masses, producing a library that fails to match experimental spectra during acquisition.

## Examples

### Convert an MGF file to a BiblioSpec spectral library
**Args:** `SpectraWideGap.mgf --output LibraryWideGap.blib --enzyme trypsin`
**Explanation:** Specifies the MGF input file containing Mascot search results, directs output to a named .blib file, and constrains peptide entries to fully tryptic cleavage sites.

### Build a library from a Mascot DAT file with high mass accuracy
**Args:** `run01_fht.seq --output run01.blib --pm-tolerance 20 --pm-unit ppm --ion-type b y`
**Explanation:** Uses SQT-format input with 20 parts-per-million precursor tolerance and explicitly requests both b-ion and y-ion series for inclusion in the spectral library.

### Create a library specifying doubly charged precursors only
**Args:** `ionTrap_results.mgf --output doubly.blib --charge 2`
**Explanation:** Filters the input to include only doubly charged precursor spectra, which is appropriate for ion trap data where +2 ions dominate and single-charge ions are sparse.

### Generate a library with Lys-C specificity
**Args:** `lysC_search.dat --output lysC_lib.blib --enzyme lysc`
**Explanation:** Uses Lys-C cleavage rules when creating the library, appropriate when the original search was configured for Lys-C digestion rather than standard trypsin.

### Convert multiple search result files into a single consolidated library
**Args:** `runA.mgf runB.ms2 runC.sqt --output combined.blib --enzyme trypsin --merge`
**Explanation:** Aggregates spectra from three separate search result files into one .blib database, using merge semantics to combine and deduplicate entries based on shared peptide sequences.