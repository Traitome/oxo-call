---
name: annonars
category: genomics
description: A command-line tool for genomic annotation, enabling functional annotation of genomic regions, variants, or reads against reference databases. Supports BED, VCF, and standard genomic formats.
tags: [genomics, annotation, variant-calling, bedtools, functional-annotation]
author: AI-generated
source_url: https://github.com/example/annonars
---

## Concepts

- **Input formats:**annonars accepts multiple genomic file formats including BED ( Browser Extensible Display), VCF (Variant Call Format), and标准genome coordinates. The tool automatically detects the input format based on file extensions (.bed, .vcf, .bedgraph).
- **Annotation databases:** The tool queries local annotation databases containing gene definitions, regulatory elements, and functional annotations. Users must specify database paths via `--db` or `--database-path` flags. Database indices are built automatically on first use.
- **Output modes:** Output can be generated in multiple formats (JSON, TSV, BED) controlled by `--output-format`. The default is TSV for pipe-friendly downstream processing. JSON output includes metadata and confidence scores.
- **Batch processing:** Multiple input files can be processed in a single run using `--batch` or by specifying a manifest file. This dramatically reduces overhead for large-scale projects.

## Pitfalls

- **Missing database path:** If `--db` is not specified and no default database is configured,annonars fails with a cryptic "database not found" error. Always verify database paths before running analyses.
- **Coordinate system mismatch:** Using 0-based coordinates (common in BED files) when the tool expects 1-based coordinates (typical in VCF) causes silent offset errors in output. Always verify coordinate conventions match your input format.
- **Input file permissions:** Failing to set read permissions on input files results in "Permission denied" errors that may be mistaken for format issues. Verify file permissions before execution.
- **Memory limits with large files:** Processing genome-wide BED files without setting `--max-memory` causes excessive memory consumption and potential OOM kills. Use `--chunk-size` to process large files in manageable segments.

## Examples

### Annotate a BED file with gene names
**Args:** `annotate --input variants.bed --db /refdb/annotation --output annotated_variants.bed`
**Explanation:** Annotates genomic intervals in a BED file with gene names from the specified annotation database, writing results to the output file.

### Convert VCF annotations to JSON format
**Args:** `convert --input variants.vcf --output-format json --output results.json`
**Explanation:** Converts variant call format annotations to JSON for programmatic downstream analysis or integration with other tools.

### Process multiple files using a manifest
**Args:** `batch --manifest sample_manifest.txt --output-dir ./results`
**Explanation:** Processes multiple genomic files listed in a manifest file, writing individual results to the specified output directory.

### Filter annotations by functional impact
**Args:** `filter --input annotated.vcf --type missense --min-confidence 0.9 --output filtered.vcf`
**Explanation:** Filters annotated variants to retain only missense mutations with confidence scores above 0.9, useful for variant prioritization.

### Build a custom annotation database
**Args:** `build-db --source custom_annotations.gtf --db-name custom_ref --output custom.db`
**Explanation:** Builds a custom annotation database from a GTF file for use in subsequent annotation runs, enabling project-specific annotation sets.

### Extract only promoter regions
**Args:** `extract --input genome.bed --feature-type promoter --output promoters.bed`
**Explanation:** Extracts genomic features tagged as promoter regions from a comprehensive genome annotation file, useful for regulatory analysis.

### Check database integrity before analysis
**Args:** `validate-db --db /refdb/annotation`
**Explanation:** Validates the integrity and version compatibility of an annotation database before running production analyses to prevent runtime failures.