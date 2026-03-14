---
name: crossmap
category: utilities
description: Genomic coordinate conversion between genome assemblies (e.g., hg19 to hg38) for various file formats
tags: [coordinates, liftover, assembly, vcf, bed, bam, genome, conversion]
author: oxo-call built-in
source_url: "https://crossmap.sourceforge.net/"
---

## Concepts

- CrossMap converts genomic coordinates between genome assemblies using chain files from UCSC.
- Supported formats: BED, GFF/GTF, VCF, BAM/SAM, BigWig, Wiggle, FASTA.
- Download chain files from UCSC: http://hgdownload.soe.ucsc.edu/goldenPath/hg19/liftOver/hg19ToHg38.over.chain.gz
- Usage: crossmap bed chain_file input.bed output.bed
- For VCF conversion: CrossMap needs the target genome FASTA to update REF alleles: crossmap vcf chain_file input.vcf target.fa output.vcf
- Variants that cannot be lifted over are written to <output.vcf>.unmap file.
- For BAM conversion: coordinate sort and reindex after CrossMap.

## Pitfalls

- Chain file must match the source→target genome versions (e.g., hg19→hg38, NOT hg38→hg19).
- VCF conversion requires the target genome FASTA for REF allele verification.
- Some variants fail liftover — check the .unmap output file and investigate failed records.
- CrossMap does not update variant IDs — rsIDs or local IDs remain unchanged after conversion.
- BAM conversion may produce coordinate-unsorted output — always re-sort with samtools sort after.
- For VCF, multiallelic records should be split before CrossMap for accurate conversion.

## Examples

### convert BED file from hg19 to hg38 coordinates
**Args:** `bed hg19ToHg38.over.chain.gz input_hg19.bed output_hg38.bed`
**Explanation:** bed subcommand; chain file specifies conversion direction; output is lifted-over BED

### convert VCF file from hg19 to hg38 with target reference
**Args:** `vcf hg19ToHg38.over.chain.gz input_hg19.vcf hg38_reference.fa output_hg38.vcf`
**Explanation:** vcf subcommand; requires target FASTA for REF allele check; unlifted variants go to .unmap

### convert GFF/GTF annotation from one assembly to another
**Args:** `gff hg19ToHg38.over.chain.gz annotation_hg19.gtf output_hg38.gtf`
**Explanation:** gff subcommand handles both GFF and GTF format conversion

### convert BAM file from one genome build to another
**Args:** `bam hg19ToHg38.over.chain.gz input_hg19.bam output_hg38.bam`
**Explanation:** bam subcommand; re-sort output with samtools sort; update header SQ lines with new coordinates
