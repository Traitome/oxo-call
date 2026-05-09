---
name: bedgovcf
category: Genomics / File Format Conversion
description: Converts between BED (Browser Extensible Data) and VCF (Variant Call Format) file formats for genomic variant data, enabling interoperability between different genomics analysis tools and pipelines.
tags: [bed, vcf, conversion, variant-calling, genomics, format-conversion, bioinformatics]
author: AI-generated
source_url: https://bedtools.readthedocs.io/en/latest/
---

## Concepts

- **Bidirectional Conversion**: bedgovcf converts genomic variants from BED format to VCF format and vice versa, preserving chromosome coordinates, variant IDs, reference/alternative alleles, and quality scores where specified in the source file.

- **Coordinate System Handling**: The tool handles both 0-based (BED standard) and 1-based (VCF standard) coordinate systems automatically, applying the appropriate offset during conversion based on the output format selected.

- **Annotation Preservation**: When converting VCF to BED, INFO fields and FORMAT fields can be preserved as additional BED columns (typically columns 4-12), allowing downstream tools to retain critical variant metadata such as DP (depth), GQ (genotype quality), and custom annotations.

- **Header Processing**: The tool parses VCF header lines (#CHROM, INFO, FORMAT) to correctly map field names to BED column headers, and conversely generates proper VCF headers when converting BED to VCF.

## Pitfalls

- **Coordinate Offset Errors**: Failing to account for the 0-based vs 1-based coordinate difference results in off-by-one errors for variant positions, causing variants to appear at incorrect genomic locations in the output file.

- **Missing Required Fields**: Converting VCF to BED without required columns (chrom, start, end, name, score, strand) can cause downstream tools to fail or produce invalid BED files that are rejected byGenome browsers.

- **Inconsistent Chromosome Naming**: Using different naming conventions (chr1 vs 1, NC_000001.11 vs chr1) between input files results in failed merging or annotation errors in downstream analyses.

- **Overflow of Variant IDs**: When multiple VCF records share the same ID field, converting to BED can cause name collisions, leading to data loss or overwriting of variant information in the output.

## Examples

### Convert a BED file to VCF format
**Args:** `-i variants.bed -o variants.vcf`
**Explanation:** Reads the BED file containing variant coordinates and genotypes, then outputs a valid VCF file with proper headers and formatted variant records.

### Convert a VCF file to BED format
**Args:** `-i variants.vcf -o variants.bed`
**Explanation:** Parses VCF variant records and their INFO/FORMAT fields, converting them to BED format with chromosome, start, end, name, and score columns.

### Specify reference genome for coordinate validation
**Args:** `-i variants.bed -o variants.vcf -g hg38.fa`
**Explanation:** Uses the provided FASTA reference to validate variant coordinates and ensure reference alleles match the genome sequence.

### Preserve specific INFO fields as BED columns
**Args:** `-i variants.vcf -o variants.bed -fields DP,AF,Gene`
**Explanation:** Selectively extracts the DP (depth), AF (allele frequency), and Gene INFO fields from the VCF and places them in additional BED columns.

### Convert with automatic chromosome naming conversion
**Args:** `-i variants.bed -o variants.vcf -usechr`
**Explanation:** Adds the "chr" prefix to chromosome names during conversion if the input uses UCSC naming while the desired output uses RefSeq naming, or vice versa.

### Handle multi-allelic variants by splitting
**Args:** `-i multiallelic.vcf -o split.vcf -split`
**Explanation:** Splits multi-allelic VCF records into separate lines in the output BED file, ensuring each alternative allele gets its own record with unique coordinates.

### Convert only specific genomic regions
**Args:** `-i variants.vcf -o region.bed -chr chr1:1000000-2000000`
**Explanation:** Filters the input VCF to include only variants within the specified chromosome 1 region (positions 1,000,000 to 2,000,000) before converting to BED format.