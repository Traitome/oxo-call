---
name: "AGAT - Another GFF-related Annotation Tool"
category: "Genomics / Annotation Manipulation"
description: "A comprehensive Perl-based suite for validating, converting, filtering, searching, and analyzing GFF3 genome annotation files. Includes companion scripts for sequence extraction, statistics, format conversion, and feature manipulation."
tags: ["GFF3", "annotation", "validation", "sequence-extraction", "bioinformatics", "genomics", "perl", "gene-annotation"]
author: "AI-generated"
source_url: "https://github.com/NBISweden/AGAT"
---

## Concepts

- AGAT (Another GFF-related Annotation Tool) is a multi-script Perl suite where each companion binary (e.g., `agat_convert_validate_gff3.pl`, `agat_sp_statistics.pl`) is invoked directly by its full script name as the first CLI token. The suite operates exclusively on GFF3 annotation files, which encode genomic features as nine tab-delimited columns: seqid, source, type, start, end, score, strand, phase, and attributes.

- Sequence extraction tools (e.g., `agat_sp_extract_sequences.pl`) require both a valid GFF3 annotation file and a corresponding genomic FASTA file. The tool maps coordinates from the GFF3 onto the FASTA to pull out nucleotide or peptide sequences for the annotated features, and the `--gff` and `--fasta` flags are mandatory positional inputs for this workflow.

- Feature filtering and search tools use a hierarchical feature-type system (e.g., gene → mRNA → CDS → exon) inherited from the Sequence Ontology. Filtering with `--feature` or `--attribute` flags operates on this taxonomy, meaning specifying `--feature exon` will extract only exon-level features rather than genes, which can silently produce empty output if the wrong feature type is targeted.

- AGAT's format validation (`agat_convert_validate_gff3.pl --val`) performs deep syntax checking of GFF3 compliance, including phase consistency (CDS features must have phase 0, 1, or 2), attribute format correctness, and cross-referenced parent-child relationships. Files that pass AGAT validation are broadly compatible with other GFF3-aware tools, but validation failures are non-fatal warnings rather than hard stops, so output should always be verified manually.

## Pitfalls

- Passing a GFF2 or GTF file to AGAT without first converting it with `agat_convert_gff2gff3.pl` causes silent failures or garbled output, because AGAT expects strict GFF3 syntax and the nine-column structure. GTF files especially are structurally incompatible and will produce either errors or technically valid but semantically wrong GFF3.

- Using the wrong feature type in `--feature` filtering produces empty output files without warning. For example, `--feature gene` on a GFF3 that only contains `CDS` and `exon` children (no explicit `gene` parent) will yield zero features, because AGAT strictly matches the requested type against the GFF3 feature types present in the file.

- Omitting the mandatory genomic FASTA file (`--fasta`) when running sequence extraction tools (`agat_sp_extract_sequences.pl`) causes the tool to attempt reading the reference from the GFF3 `##FASTA` section if present, but many GFF3 files lack this section, leading to a runtime error. Always explicitly provide `--fasta` for reproducibility.

- Requesting reverse-complement sequences with `--rev` on features already marked with `-.` strand in the GFF3 attributes produces double-reversed sequences, inverting the correct orientation. The tool does not detect conflicting strand specifications and will output the reversed complement of an already reversed feature.

- Running `agat_sp_statistics.pl` on a GFF3 with missing or malformed `##gff-version` directives produces incomplete statistics and may silently skip feature counts, because the parser falls back to permissive mode and assumes partial compliance.

## Examples

### Validate a GFF3 annotation file for syntax correctness
**Args:** `agat_convert_validate_gff3.pl --val input.gff3 --out validated_output.gff3`
**Explanation:** The `--val` flag triggers deep GFF3 compliance checking including phase consistency and attribute formatting, while `--out` redirects the validated copy to a new file.

### Extract nucleotide sequences for all CDS features using a genomic FASTA reference
**Args:** `agat_sp_extract_sequences.pl --gff annotation.gff3 --fasta genome.fa --feature CDS --out cds_sequences.fasta`
**Explanation:** Mapping CDS coordinate ranges from the GFF3 onto the genome FASTA extracts the coding nucleotide sequences for every CDS feature in the annotation.

### Convert a GFF2 annotation file to GFF3 format
**Args:** `agat_convert_gff2gff3.pl --gff input.gff2 --out output.gff3`
**Explanation:** The conversion script rewrites the older GFF2 format into the standard nine-column GFF3 structure, making it compatible with AGAT and other GFF3-aware tools.

### Generate a summary statistics report for a GFF3 annotation
**Args:** `agat_sp_statistics.pl --gff annotation.gff3 --out stats_report.txt`
**Explanation:** The statistics tool counts feature types, calculates feature density per sequence, and reports attribute distribution, producing a comprehensive annotation summary.

### Filter a GFF3 to retain only protein-coding gene features and their direct children
**Args:** `agat_sp_filter_feature_by_attribute.pl --gff annotation.gff3 --attribute parent --value gene: --out gene_features.gff3`
**Explanation:** Using the `--attribute parent` filter with a prefix value extracts features whose parent attribute matches the pattern, isolating gene-level features and their hierarchical children from the full annotation.

### Search a GFF3 for features with a specific alias attribute value
**Args:** `agat_convert_search.pl --gff annotation.gff3 --attribute alias --search "BRCA1" --exact --out brca1_hit.gff3`
**Explanation:** The search tool performs exact matching on the specified `alias` attribute column, extracting only the feature records that contain the exact identifier "BRCA1".