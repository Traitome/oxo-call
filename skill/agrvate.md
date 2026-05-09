---
name: agrvate
category: Bioinformatics::Sequence Annotation
description: A bioinformatics toolkit for processing and analyzing genomic annotation files (GFF3/GTF formats). agrvate provides utilities for quality control, conversion, filtering, and statistical analysis of gene annotation data, enabling researchers to validate and manipulate feature coordinates across genomic datasets.
tags:
  - gff3
  - gtf
  - annotation
  - genomics
  - feature-extraction
  - conversion
  - bioinformatics
author: AI-generated
source_url: https://github.com/NBISweden/AGAT
---

## Concepts

- **Input formats**: agrvate primarily handles GFF3 and GTF (GTF2) annotation files, which store genomic features such as genes, transcripts, exons, and CDS regions with their coordinates, strands, and attributes. The tool auto-detects format version and validates syntax.
- **Data model**: Genomic features are represented as records with seqid, source, feature type, start/end coordinates, score, strand, phase, and attributes (key-value pairs). Feature types follow the GFF3 specification hierarchy (gene > mRNA > exon/CDS).
- **Output modes**: agrvate can convert between GFF3 and GTF formats, filter features by type/attribute, extract specific genomic regions, validate annotation consistency, and generate summary statistics.
- **Companion utilities**: The agrvate suite includes specialized binaries for distinct tasks (e.g., agrvate_convert for format conversion, agrvate_quality for validation, agrvate_stats for reporting).

## Pitfalls

- **Using incorrect feature type names**: Specifying feature types case-sensitively (e.g., "MRNA" instead of "mRNA") causes filter operations to return zero matches, leading to empty output files without warning.
- **Ignoring strand orientation**: Failing to account for strand information (+/-) when extracting features results in incorrect gene models, especially for antisense transcripts or overlapping genes on opposite strands.
- **Assuming 1-based coordinates**: GFF3 uses 1-based inclusive coordinates while BED format uses 0-based coordinates; conflating these leads to off-by-one errors in feature positions when converting between formats.
- **Processing unsorted input files**: Running agrvate on non-chronologically sorted GFF3 files may produce incomplete results or corrupt output, as the tool expects records ordered by seqid and position.

## Examples

### Convert a GTF file to GFF3 format

**Args:** `-i input.gtf -o output.gff3 convert`
**Explanation:** Converts gene annotation from GTF (commonly used by RNA-seq aligners) to GFF3 (standard bioinformatics exchange format), preserving all attribute mappings.

### Filter and keep only protein-coding genes

**Args:** `-i annotations.gff3 -o genes.gff3 filter --feature_type gene --attribute_filter biotype:protein_coding`
**Explanation:** Extracts only gene features where the biotype attribute equals protein_coding, removing all other annotation types like pseudogenes or ncRNA genes.

### Extract features from a specific genomic region

**Args:** `-i genome.gff3 -o region.gff3 extract --region chr1:100000-500000`
**Explanation:** Pulls all annotation records overlapping chromosome 1 positions 100,000 to 500,000, useful for targeted analysis of specific loci.

### Generate statistics about an annotation file

**Args:** `-i genes.gff3 -o stats.txt stats`
**Explanation:** Produces a summary report containing counts of each feature type, distribution of gene biotypes, total gene lengths, and other quality metrics for the annotation.

### Validate GFF3 syntax and consistency

**Args:** `-i annotations.gff3 -o valid.gff3 validate --check_structure`
**Explanation:** Verifies that all parent-child relationships are correct (e.g., mRNA has gene as parent), reports malformed records, and removes invalid entries from the output.