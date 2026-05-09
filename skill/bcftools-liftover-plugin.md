---
name: bcftools-liftover-plugin
category: genomic-coordinate-conversion
description: A bcftools plugin for liftover (coordinate conversion) of VCF/BCF files between different genome assemblies using UCSC chain files. Converts variant positions from one reference build (e.g., hg19) to another (e.g., hg38) while handling indels, structural variants, and complex alignments.
tags: [liftover, coordinate-conversion, vcf, bcf, genome-assembly, variant-calling, bcftools, plugin, ucsc, chain-file]
author: AI-generated
source_url: https://samtools.github.io/bcftools/2023/10/01/bcftools-liftover-plugin/
---

## Concepts

- **Chain file requirement**: The plugin requires a UCSC chain file (`.chain`) that defines alignments between two genome assemblies. Chain files contain the source assembly (e.g., hg19), target assembly (e.g., hg38), and a series of alignment blocks mapping coordinates across chromosomes.
- **Plugin loading**: This is a bcftools plugin that must be invoked via `bcftools +liftover` (note the `+` prefix). The plugin reads VCF/BCF input and produces liftover'd output in VCF/BCF format to stdout by default.
- **Reverse-strand handling**: The `--flip` option reverses the chain direction, allowing liftover from the target assembly back to the source assembly using the same chain file. Without `--flip`, liftover proceeds from the chain's source to target.
- **Multi-mapping records**: Variants may map to multiple positions when alignments span gaps or repeated regions. The `-a` option controls alignment selection behavior, and multiple mappings may be flagged using appropriate filters.
- **Rejection mechanism**: When using `--reject`, records that fail liftover are output to a separate file with a numeric code indicating the failure reason (e.g., 0 for success, 1 for unmapped, 2 for ambiguous).

## Pitfalls

- **Genome assembly mismatch**: Using a chain file built for hg19→hg38 but attempting to liftover from hg38→hg19 without the `--flip` flag produces incorrect coordinates, silently generating invalid variant positions in the output VCF.
- **Unmapped record loss**: Records that cannot be lifted over (e.g., no alignment in the chain file) are silently dropped unless `--reject` is specified, causing data loss without warning in large-scale pipeline runs.
- **Chromosome naming mismatch**: VCF files using non-standard chromosome names (e.g., "1" instead of "chr1") will fail to liftover if the chain file uses different naming conventions, as the plugin performs exact string matching on chromosome names.
- **Missing required options**: Omitting the `--chain` option results in an error, but overlooking complementary options like `--allow-extra-chrom` may cause failures with non-standard chromosome contigs present in custom VCF files.
- **Multiple mapping ambiguity**: Variants that align to multiple positions in the target assembly are not automatically flagged; using strict liftover without accounting for these can propagate incorrectly mapped variants downstream.

## Examples

### Basic VCF liftover from hg19 to hg38
**Args:** `--chain hg19ToHg38.over.chain input.vcf.gz -O z -o output.hg38.vcf.gz`
**Explanation:** Loads the specified chain file to convert variant positions and outputs a compressed VCF file with coordinates in the hg38 assembly.

### Liftover with rejection file for failed records
**Args:** `--chain hg19ToHg38.over.chain input.vcf.gz --reject rejected.vcf.gz -O z -o output.hg38.vcf.gz`
**Explanation:** Captures all records that cannot be lifted over to the specified rejection file, enabling downstream analysis of failed variants with their failure codes.

### Reverse liftover from hg38 to hg19 using flip
**Args:** `--chain hg19ToHg38.over.chain --flip input.hg38.vcf.gz -O z -o output.hg19.vcf.gz`
**Explanation:** Uses the flip option to reverse the chain direction, allowing conversion back to the older genome assembly without obtaining a separate reverse chain file.

### Liftover excluding unmapped records silently
**Args:** `--chain hg19ToHg38.over.chain input.vcf.gz -a drop -O z -o output.hg38.vcf.gz`
**Explanation:** Drops records that fail to map (no valid alignment found) rather than writing them to a separate file, producing a clean output VCF with only successfully lifted records.

### Liftover with secondary alignments kept
**Args:** `--chain hg19ToHg38.over.chain input.vcf.gz -a best -O z -o output.hg38.vcf.gz`
**Explanation:** Selects only the best alignment for variants with multiple possible mappings, discarding secondary alignments to avoid ambiguous coordinate assignments.

### Allow non-standard chromosomes during liftover
**Args:** `--chain hg19ToHg38.over.chain --allow-extra-chrom input.vcf.gz -O z -o output.hg38.vcf.gz`
**Explanation:** Processes chromosome contigs that may not appear in the standard chain file, preventing errors when working with patch sequences or custom genomic annotations.

### Liftover using companion binary for chain preparation (reference)
**Args:** `chain2sam.pl hg19 hg38 hg19ToHg38.chain > hg19ToHg38.over.chain`
**Explanation:** Uses an external chain preparation script to convert UCSC chain format into a compatible bcftools chain file format before running the liftover plugin.