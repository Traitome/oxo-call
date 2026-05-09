---
name: bakta
category: Genome Annotation
description: Fast bacterial genome annotation tool that identifies CDS, rRNA, tRNA, CRISPR arrays, and other genomic features. Includes antimicrobial resistance and virulence gene detection using AMRFinder Plus. Outputs GenBank, EMBL, GFF3, and other standard formats.
tags:
  - bacterial-genomics
  - genome-annotation
  - prokaryotes
  - antimicrobial-resistance
  - virulence-factors
  - bioinformatics
  - contig-annotation
author: AI-generated
source_url: https://github.com/oschkopp/bakta
---

## Concepts

- **Data Model**: Bakta annotates bacterial genome sequences in FASTA, GenBank, or EMBL format by scanning for coding sequences (CDS), ribosomal RNA (rRNA), transfer RNA (tRNA), tmRNA, CRISPR arrays, and other genome features. It uses a built-in database of bacterial protein families and integrates AMRFinder Plus for antimicrobial resistance (AMR) and virulence gene detection.

- **Input Formats**: Accepts raw FASTA files containing assembled contigs, as well as GenBank and EMBL formats. Minimum contig length can be specified to filter out small sequences (default: 1 bp). The tool can also incorporate existing annotations via the `--keep-old-annotations` flag to preserve or extend prior annotations.

- **Output Formats**: Generates multiple standard formats simultaneously: GenBank (`.gbk`), EMBL (`.embl`), FASTA (`.ffa`), GFF3 (`.gff`), and tab-separated summary files (`.tsv`). JSON output is available for programmatic parsing. A summary SVG provides visual overview of the annotation.

- **Database Management**: Bakta requires a local database that is automatically downloaded on first run. Use `bakta-build` to manually download/update the database to a custom path. The `--skip-crispr` flag disables CRISPR array detection to reduce runtime on datasets known to lack CRISPR systems.

- **Genetic Code**: Default translation table is NCBI Table 11 (Bacterial, Archaeal, and Plant Plastid Code). Use `--translation-table` or `--tt` to specify alternative genetic codes for organelles or non-standard bacteria.

## Pitfalls

- **Missing Database on First Run**: On initial execution, Bakta automatically downloads ~300 MB of database files. Running without internet access or in a firewalled environment causes failure. Use `bakta-build` to pre-download the database to avoid this issue in automated pipelines.

- **Insufficient Memory for Large Genomes**: Annotating large bacterial genomes (>10 Mb) or metagenome assemblies with many contigs can consume significant RAM. The tool may crash or become unresponsive if available memory is insufficient. Monitor memory usage and consider splitting large contig sets.

- **Duplicate Contig Names in Input**: If the input FASTA file contains duplicate sequence names/IDs, Bakta may produce unexpected output or overwrite results. Ensure all contig headers are unique before annotation.

- **Overwriting Previous Results**: By default, Bakta exits if the output directory already exists (error: "Output directory already exists"). Use `--force` to overwrite, or specify a new output directory to prevent accidental data loss.

- **Incompatible Translation Table**: Specifying an incorrect genetic code via `--translation-table` leads to incorrect protein translations for all CDS predictions. Verify the appropriate code for your organism (e.g., Table 4 for Mycoplasma, Table 6 for some protozoa).

## Examples

### Annotate a bacterial genome assembly in FASTA format
**Args:** --output /path/to/output --prefix strain_1 input.fasta
**Explanation:** This runs Bakta on the input FASTA file, writing all output files (GenBank, GFF3, FASTA, TSV) to the specified output directory with the given prefix for filenames.

### Generate only GenBank output with custom database path
**Args:** --db /local/bakta_db --output /path/to/output --format genbank input.fasta
**Explanation:** Uses a locally pre-downloaded database (avoiding repeated downloads) and produces only GenBank format output, reducing disk space usage compared to generating all formats.

### Annotate with 8 threads for faster processing
**Args:** --threads 8 --output /path/to/output input.fasta
**Explanation:** Enables multi-threaded execution to significantly speed up annotation, especially beneficial for larger genomes or batch processing multiple files.

### Skip CRISPR array detection for faster runtime
**Args:** --skip-crispr --output /path/to/output input.fasta
**Explanation:** Disables CRISPR detection when the organism is known to lack CRISPR systems, reducing runtime and memory usage for organisms like Mycoplasma or highly reduced genomes.

### Keep existing annotations and extend with new predictions
**Args:** --keep-old-annotations --output /path/to/output input.gbk
**Explanation:** Takes an existing GenBank file with annotations and extends it with additional predictions (AMR, virulence genes) rather than re-annotating from scratch.

### Annotate with custom genetic code (Table 4 for Mycoplasma)
**Args:** --translation-table 4 --output /path/to/output input.fasta
**Explanation:** Uses NCBI Translation Table 4 (Mycoplasma/Spirillum code) instead of the default bacterial code, ensuring correct protein translation for organisms using this genetic code.

### Generate summary only in TSV format for downstream parsing
**Args:** --output /path/to/output --format tsv input.fasta
**Explanation:** Outputs only the tab-separated summary file containing feature coordinates, IDs, and product names for integration into pipelines that require structured tabular data.

### Annotate with lambda prophage detection enabled
**Args:** --include-lambdap --output /path/to/output input.fasta
**Explanation:** Enables detection of lambda prophage regions in the genome, useful for E. coli and related Enterobacteriaceae where prophage integration is biologically relevant.

### Force overwrite existing output directory
**Args:** --force --output /path/to/existing_output input.fasta
**Explanation:** Overwrites all files in an existing output directory, useful in automated pipelines where previous runs may need to be replaced without manual cleanup.
---