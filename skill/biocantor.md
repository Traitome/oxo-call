---
name: biocantor
category: genomics/annotation
description: A Python library and command-line tool for managing, converting, and querying genomic annotations across multiple standard formats including GFF3, GTF, and BED. Provides tools for extracting features, filtering annotations, and performing set operations on genomic intervals.
tags: bioinformatics, genomics, GFF3, GTF, BED, annotation, gene, transcript, interval
author: AI-generated
source_url: https://github.com/ biocomantor/biocantor
---

## Concepts

- **Annotation Data Model**: Biocantor represents genomic features as a hierarchical model with genes as parent features containing transcripts, which in turn contain exons. Coordinates in GFF3/GTF are 1-based inclusive, while BED format uses 0-based half-open intervals — always account for this when converting between formats to avoid off-by-one errors in downstream analysis.

- **Multiple I/O Formats**: The tool supports reading from and writing to GFF3, GTF, and BED formats. Each format has specific conventions: GFF3 uses the 9-column structure with a feature type in column 3, GTF requires a transcript_id and gene_id in the attributes column, and BED format is 0-based with chromStart excluded from the interval.

- **Feature Extraction and Filtering**: Biocantor can extract specific feature types (genes, transcripts, exons, CDS) from annotations and filter by genomic coordinates, chromosome name, or attribute values. This enables targeted analysis without loading entire annotation files into memory.

- **Interval Operations**: The tool supports genomic interval operations including overlap detection, windowing around features, and set operations between multiple annotation sets. These are essential for comparing predictions against reference annotations.

## Pitfalls

- **Coordinate System Confusion**: Converting coordinates from GFF3/GTF (1-based, inclusive end) to BED (0-based, half-open) without adjustment will shift all features by one base pair. Always verify coordinate systems match your downstream tools — some aligners expect 0-based while most annotation formats use 1-based.

- **Missing Required Attributes**: GTF format strictly requires `gene_id` and `transcript_id` attributes in column 9. Biocantor will fail or produce malformed output if these attributes are missing, unlike GFF3 which has more flexible attribute requirements. Validate GTF files before conversion.

- **Feature Type Mismatches**: GFF3 and GTF use standardized feature types (gene, mRNA, exon, CDS for GFF3; transcript, exon, CDS for GTF). Using custom or non-standard feature type names will cause parsing failures or silent data loss when extracting specific feature types. Always check feature type spelling and case sensitivity.

- **Duplicate Feature IDs**: Annotations with duplicate gene or transcript IDs will cause unexpected behavior during interval operations, potentially generating false positive overlaps. Biocantor can filter duplicates but this must be explicitly enabled — by default it assumes well-formed input.

## Examples

### Convert a GFF3 annotation file to GTF format
**Args:** convert input.gff3 output.gtf --input-format gff3 --output-format gtf
**Explanation:** This converts gene annotations from GFF3 to GTF format, which is required by many RNA-seq quantification tools like StringTie and featureCounts.

### Extract all genes from a chromosome and save to BED format
**Args:** extract input.gff3 --feature-type gene --chr chr1 -o chr1_genes.bed
**Explanation:** Extracts only gene features located on chromosome 1 and writes them in BED format for use as priors in genome browsers or as custom annotation inputs.

### Filter transcripts by minimum length threshold
**Args:** filter input.gff3 --min-length 300 --feature-type transcript -o long_transcripts.gtf
**Explanation:** Removes transcripts shorter than 300 base pairs, useful for filtering out likely non-functional or partial transcript predictions before expression analysis.

### Validate gene annotation file for common format errors
**Args:** validate input.gff3 --check-id-uniqueness --check-coordinate-order
**Explanation:** Checks the annotation file for duplicate feature IDs and ensures genomic coordinates are properly ordered, preventing errors in downstream pipelines.

### Find overlaps between a prediction file and reference annotations
**Args:** overlap predictions.bed reference.gtf --reciprocal 0.8 -o validated_predictions.gtf
**Explanation:** Identifies predicted intervals that overlap at least 80% reciprocally with reference annotations, commonly used for validating predicted gene models or regulatory elements.

### Extract exons from a GTF file and convert to BED12 format
**Args:** extract input.gtf --feature-type exon -o exons.bed --bed12
**Explanation:** Extracts exon features and outputs them in BED12 format, which compactly represents the exon structure of multi-exon transcripts in a single line.

### Convert annotation to JSON for programmatic processing
**Args:** convert annotation.gff3 annotation.json --json-pretty
**Explanation:** Exports the genomic annotation as JSON, enabling integration with Python scripts or web services that require structured genomic feature data.