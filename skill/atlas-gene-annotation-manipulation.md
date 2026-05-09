---
name: atlas-gene-annotation-manipulation
category: genomics/gene-annotation
description: A comprehensive tool for manipulating gene annotations including combining multiple annotation files, filtering by genomic features, converting between standard formats (GTF, BED, GFF), and extracting specific genomic regions.
tags: [gene-annotation, gtf, bed, gff, gff3, genomics, sequence-manipulation, format-conversion]
author: AI-generated
source_url: https://github.com/hgc-devel/atlas-gene-annotation-manipulation
---

## Concepts

- The tool operates on gene annotation files, supporting GTF (Gene Transfer Format), BED (Browser Extensible Data), and GFF (General Feature Format) as primary I/O formats, each with distinct field structures and coordinate conventions
- Annotations are processed as discrete genomic features where each entry contains chromosome, start/end coordinates, strand information, and feature-specific attributes like gene ID, transcript ID, and biotype
- Filtering operations can target specific genomic regions via chromosome and coordinate ranges, and attributes can be matched using exact value comparisons or regex patterns
- Output format selection determines the schema used for writing results; GTF employs a 9-column structure, BED uses variable column counts, and GFF3 implements a 9-column format with embedded attributes
- Multiple input files can be merged using merge mode, though overlapping features on the same strand are flagged as conflicts rather than automatically deduplicated

## Pitfalls

- **Coordinate system mismatch**: Specifying genomic coordinates in the wrong format (1-based versus 0-based) produces silent errors where off-target regions are extracted or filtered, degrading downstream analysis
- **BED format column count errors**: Attempting to output BED format without providing the correct column count for the selected fields causes malformed output files that fail to parse in downstream tools like UCSC or IGV
- **Missing required attributes during conversion**: Converting between formats without providing mandatory attributes (gene_id in GTF, Name in GFF3) generates invalid output that downstream tools reject
- **Unintentional merging of overlapping features**: Using merge mode on a BED file containing proximal but distinct genes merges them into single entries, destroying annotation granularity
- **Chromosome naming inconsistency**: Mixing chromosome naming conventions (chr1 versus 1) across input files produces duplicate or missing features after merge operations

## Examples

### Extract a genomic region from a GTF file
**Args:** extract-region -i annotations.gtf.gz --chromosome 3 --start 1000000 --end 2000000 | gzip > region_3_1-2Mb.gtf.gz
**Explanation:** Extracting a specific chromosomal interval from a compressed GTF file preserves annotations overlapping the coordinates, and piping through gzip maintains compression for downstream compatibility.

### Filter GTF entries by gene biotype
**Args:** filter -i annotations.gtf.gz --attribute gene_biotype --value protein_coding --operation exact-match | gzip > protein_coding.gtf.gz
**Explanation:** Using exact-match filtering on the gene_biotype attribute isolates protein-coding genes from a mixed annotation file, while non-matching entries are discarded from the output stream.

### Convert a GFF3 file to sorted BED format
**Args:** convert -i genome_annotation.gff3 --input-format gff3 --output-format bed --output-version 6 --sort-by chromosome,start | gzip > genome_annotation.bed.gz
**Explanation:** Converting between formats requires specifying both input and output formats explicitly, and sorting by chromosome then genomic position ensures compatibility with genome browsers and overlap analysis tools.

### Merge multiple BED files and report overlapping entries
**Args:** merge -i encode_ctcf.bed.gz encode_h3k27ac.bed.gz -m overlap --strand-specific --report-conflicts | gzip > merged_peaks.bed.gz
**Explanation:** The strand-specific flag ensures separate processing of positive and negative strands during merge operations, and conflict reporting identifies regions with conflicting annotations requiring manual review.

### Extract gene annotations and write as compressed GFF3
**Args:** extract-genes -i transcripts.gtf.gz --attribute gene_id --unique --output-format gff3 | gzip > gene_set.gff3.gz
**Explanation:** Using --unique with gene_id attribute deduplicates entries where multiple transcripts exist for the same gene, producing one entry per gene rather than per transcript in the output file.

### Filter annotations using a regular expression pattern
**Args:** filter -i homo_sapiens.gtf.gz --attribute gene_name --value "^BRCA[0-9]*$" --operation regex --invert-match | gzip > non_brca.gtf.gz
**Explanation:** Applying regex pattern matching with --invert-match excludes any gene names matching the BRCA pattern, useful for creating control gene sets excluding disease-associated loci.

### Build a gene annotation index for fast querying
**Args:** atlas-gene-annotation-manipulation-build -i annotation.gtf.gz --index-name grch38_annotation_idx --force
**Explanation:** Building an index creates auxiliary files that dramatically accelerate region-based queries on large annotation files, though the --force flag is required when overwriting existing indices.