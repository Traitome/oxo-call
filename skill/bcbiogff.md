---
name: bcbiogff
category: Genomic Annotation / GFF3 Processing
description: A bioinformatics command-line tool for processing, validating, and manipulating GFF3 (General Feature Format) annotation files. Supports filtering, conversion, extraction, sorting, and validation operations on genomic feature files commonly used in bioinformatics pipelines.
tags: [gff3, genomics, annotation, validation, conversion, bioinformatics, genome-annotation]
author: AI-generated
source_url: https://github.com/bcbio/bcbiogff
---

## Concepts

- **GFF3 Format Model**: bcbiogff operates on the GFF3 (General Feature Format version 3) data model, where each feature line contains 9 tab-separated columns: seqid, source, type, start, end, score, strand, phase, and attributes. The attributes column uses key=value pairs separated by semicolons and is the primary way to store metadata like gene IDs, transcripts, and functional annotations.

- **Input/Output Formats**: The tool accepts GFF3 as primary input and can output filtered GFF3, BED format (for genome browsers), or CSV/TSV for downstream analysis. It handles both plain text (.gff, .gff3) and compressed (.gff.gz) files transparently via gzip detection.

- **Feature Hierarchy and Relationships**: bcbiogff understands the parent-child hierarchy defined in GFF3 (gene → mRNA → exon, CDS), allowing operations that traverse this hierarchy. The tool uses the ID and Parent attributes to establish relationships, enabling queries like "extract all CDS features belonging to gene X".

- **Strand and Coordinate Handling**: Genomic coordinates in GFF3 are 1-based and inclusive on both ends, unlike 0-based half-open intervals in BED format. bcbiogff preserves strand information (+, -, or .) and can convert between coordinate systems when exporting to other formats.

## Pitfalls

- **Incorrect Strand Interpretation**: Forgetting that GFF3 uses + and - to indicate strand, while . indicates unstranded features. Mixing up strand conventions when converting to BED format (which uses + and - but treats . as 0) will cause misalignment in genome browsers.

- **Coordinate System Confusion**: GFF3 is 1-based inclusive, but many downstream tools expect 0-based half-open coordinates (like BED). Failing to account for this offset when extracting genomic regions will cause off-by-one errors in downstream analysis.

- **Malformed Attribute Parsing**: GFF3 attributes must follow key=value format, but many files contain escaping issues (e.g., semicolons within values not properly escaped, or spaces around equals signs). bcbiogff may fail silently or produce incorrect output when encountering non-compliant attribute lines.

- **Parent Reference Errors**: When filtering or extracting features, references to non-existent Parent IDs will result in orphaned features. This breaks the hierarchical structure and causes issues in downstream analyses that depend on proper gene-transcript-exon relationships.

- **Compressed File Memory Usage**: Processing large compressed GFF files without sufficient memory can cause the tool to crash. The decompression happens in memory, so a 10GB compressed file may require 40GB+ RAM depending on compression ratio.

## Examples

### Extract all protein-coding genes from a GFF3 file
**Args:** `--type gene --attribute-filter "biotype=protein_coding" input.gff3`
**Explanation:** This filters the input to only include gene features where the biotype attribute equals protein_coding, useful for gene set enrichment analysis.

### Convert GFF3 to BED format for genome browser visualization
**Args:** `--to-bed input.gff3 output.bed`
**Explanation:** Converts the GFF3 features to BED format, which is compatible with UCSC Genome Browser and IGV for visualization tracks.

### Extract features within a specific genomic region
**Args:** `--region chr1:1000000-2000000 input.gff3 --output filtered.gff3`
**Explanation:** Extracts all features overlapping the specified chromosome 1 region from position 1,000,000 to 2,000,000, preserving the full feature hierarchy.

### Validate GFF3 file syntax and structural integrity
**Args:** `--validate input.gff3`
**Explanation:** Checks the GFF3 file for common syntax errors, duplicate feature IDs, broken Parent references, and coordinate validity issues, outputting a report.

### Sort GFF3 file by genomic coordinates
**Args:** `--sort --output sorted.gff3 input.gff3`
**Explanation:** Sorts all features first by chromosome (seqid), then by start position, organizing the file for efficient region-based queries and compatibility with tools expecting sorted input.

### Extract all transcripts for a specific gene
**Args:** --parent-filter "ID=gene:AT1G01010" input.gff3 --type mRNA`
**Explanation:** Uses the Parent attribute filter to retrieve all mRNA features that have gene:AT1G01010 as their parent, following the GFF3 hierarchy.

### Merge multiple GFF3 files into one
**Args:** --merge file1.gff3 file2.gff3 --output merged.gff3`
**Explanation:** Combines two or more GFF3 files into a single output, handling duplicate IDs and sorting the result by genomic coordinates.

### Extract only CDS features to a protein annotation format
**Args:** --type CDS --to-gtf input.gff3 output.gtf`
**Explanation:** Extracts coding sequence (CDS) features and converts them to GTF format, useful for running tools like Cufflinks or StringTie that require GTF input.