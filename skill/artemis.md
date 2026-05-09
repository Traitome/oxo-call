---
name: artemis
category: genome_browser
description: A Java-based genome browser and annotation viewer for EMBL, GenBank, GFF, and FASTA format sequence files, used for visualizing and editing genome annotations
tags: genome_browser, visualization, annotation, DNA, sequence_viewer, EMBL, GenBank, GFF
author: AI-generated
source_url: https://sanger-pathogens.github.io/artemis/
---

## Concepts

- **Input formats**: Artemis reads EMBL, GenBank, GFF2, GFF3, and FASTA format sequence files, automatically detecting the format from the file content or extension, and can also connect to databases via SRS or BioPHP for remote retrieval.
- **Data model**: The tool displays genomic features (genes, CDS, mRNA, tRNA, promoters, repeats, etc.) as graphical annotations on a DNA sequence coordinate axis, with feature colors configurable by feature type or qualifier.
- **Memory management**: As a Java application, Artemis requires explicit heap size specification via `-Xmx` flags (e.g., `-Xmx2000m` for 2GB); default heap may be insufficient for large genomes (e.g., mammalian chromosomes).
- **Output capabilities**: Can export selected features to text, export feature lists to CSV, save edited annotations back to EMBL/GenBank format, and view underlying DNA sequences with optional translation to amino acids.

## Pitfalls

- **Insufficient heap memory**: Running with default Java heap (often 256MB) on large genome files causes `OutOfMemoryError` and session crashes; must allocate adequate memory before launching.
- **Wrong Java version**: Artemis requires Java 8 or later; running on older Java 7 or earlier versions results in silent failures or compatibility errors when loading the JAR.
- **Missing file argument**: Launching without specifying an input file opens only the GUI with no sequence loaded, requiring manual file opening which breaks automated pipelines.
- **Incorrect file format detection**: Large files with non-standard extensions may not auto-detect correctly; specifying the wrong format flag leads to parsing failures or empty feature displays.

## Examples

### Open a local GenBank file for visualization
**Args:** `/path/to/sequence.gb`
**Explanation:** This opens the specified GenBank file in Artemis, parsing and displaying all genomic features on the sequence track.

### Open a file with explicit heap size for large genomes
**Args:** `-Xmx4000m /path/to/large_genome.embl`
**Explanation:** Allocates 4GB of heap memory to handle large genome files without OutOfMemoryError, essential for bacterial genomes over 5MB or eukaryotic chromosomes.

### View a remote database entry by accession
**Args:** `ENTRY:NC_000913`
**Explanation:** Retrieves the E. coli K-12 genome (accession NC_000913) from EMBL via network connection and displays it without requiring a local file.

### Enable DNA and translation viewing
**Args:** `-DNA /path/to/sequence.gff`
**Explanation:** Forces display of the underlying DNA sequence with six-frame translation visible, useful for verifying CDS annotations and codon accuracy.

### Export selected features to a text file
**Args:** `-DUMP /path/to/output.txt /path/to/input.embl`
**Explanation:** Dumps all features from the input file to a tab-delimited text file rather than launching the interactive GUI, enabling batch processing in pipelines.