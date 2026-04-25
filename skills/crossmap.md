---
name: crossmap
category: utilities
description: Genomic coordinate conversion between genome assemblies (e.g., hg19 to hg38) for various file formats
tags: [coordinates, liftover, assembly, vcf, bed, bam, genome, conversion, chain, UCSC]
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
- --chromid controls output chromosome ID style: a=as-is, l=long (with chr), s=short (no chr), n=no-change.
- --ref-consistent checks if reference allele is consistent during VCF liftover.
- --compress compresses output VCF with gzip automatically.
- --unmap-file specifies custom filename for unmapped entries.
- viewchain subcommand displays chain file contents in human-readable format.

## Pitfalls

- Chain file must match the source→target genome versions (e.g., hg19→hg38, NOT hg38→hg19).
- VCF conversion requires the target genome FASTA for REF allele verification.
- Some variants fail liftover — check the .unmap output file and investigate failed records.
- CrossMap does not update variant IDs — rsIDs or local IDs remain unchanged after conversion.
- BAM conversion may produce coordinate-unsorted output — always re-sort with samtools sort after.
- For VCF, multiallelic records should be split before CrossMap for accurate conversion.
- CrossMap has subcommands (bed, vcf, bam, gff, wig, bigwig, etc.); use subcommand before chain file.
- --chromid default is 'a' (as-is); use 'l' for long style (chr1) or 's' for short style (1) if needed.
- Compressed remote files (URLs) are not supported; download first for remote inputs.
- --naive-bed-parsing can help with non-standard BED files that don't conform to strict format.

## Examples

### convert BED file from hg19 to hg38 coordinates
**Args:** `bed hg19ToHg38.over.chain.gz input_hg19.bed output_hg38.bed`
**Explanation:** CrossMap bed subcommand; hg19ToHg38.over.chain.gz chain file; input_hg19.bed input BED; output_hg38.bed output BED; chain file specifies conversion direction

### convert VCF file from hg19 to hg38 with target reference
**Args:** `vcf hg19ToHg38.over.chain.gz input_hg19.vcf hg38_reference.fa output_hg38.vcf`
**Explanation:** CrossMap vcf subcommand; hg19ToHg38.over.chain.gz chain file; input_hg19.vcf input VCF; hg38_reference.fa target genome FASTA for REF allele check; output_hg38.vcf output VCF; unlifted variants go to .unmap

### convert GFF/GTF annotation from one assembly to another
**Args:** `gff hg19ToHg38.over.chain.gz annotation_hg19.gtf output_hg38.gtf`
**Explanation:** CrossMap gff subcommand; hg19ToHg38.over.chain.gz chain file; annotation_hg19.gtf input GTF; output_hg38.gtf output GTF; handles both GFF and GTF format conversion

### convert BAM file from one genome build to another
**Args:** `bam hg19ToHg38.over.chain.gz input_hg19.bam output_hg38.bam`
**Explanation:** CrossMap bam subcommand; hg19ToHg38.over.chain.gz chain file; input_hg19.bam input BAM; output_hg38.bam output BAM; re-sort output with samtools sort; update header SQ lines with new coordinates

### convert VCF with short chromosome style output
**Args:** `vcf hg19ToHg38.over.chain.gz input.vcf hg38.fa output.vcf --chromid s --compress`
**Explanation:** CrossMap vcf subcommand; hg19ToHg38.over.chain.gz chain file; input.vcf input VCF; hg38.fa target genome FASTA; output.vcf output VCF; --chromid s outputs short chromosome IDs (1, 2, X instead of chr1, chr2, chrX); --compress gzip compresses output

### convert BED with custom unmap file
**Args:** `bed hg19ToHg38.over.chain.gz input.bed output.bed --unmap-file failed_liftover.bed`
**Explanation:** CrossMap bed subcommand; hg19ToHg38.over.chain.gz chain file; input.bed input BED; output.bed output BED; --unmap-file failed_liftover.bed specifies custom filename for entries that fail liftover

### view chain file contents
**Args:** `viewchain hg19ToHg38.over.chain.gz`
**Explanation:** CrossMap viewchain subcommand; hg19ToHg38.over.chain.gz chain file; displays chain file in human-readable block-to-block format; useful for understanding alignment structure

### convert BigWig file
**Args:** `bigwig hg19ToHg38.over.chain.gz input_hg19.bw output_hg38.bw`
**Explanation:** CrossMap bigwig subcommand; hg19ToHg38.over.chain.gz chain file; input_hg19.bw input BigWig; output_hg38.bw output BigWig; signal track conversion maintains BigWig format

### convert Wiggle/bedGraph format
**Args:** `wig hg19ToHg38.over.chain.gz input.wig output.wig`
**Explanation:** CrossMap wig subcommand; hg19ToHg38.over.chain.gz chain file; input.wig input Wiggle; output.wig output Wiggle; handles Wiggle and bedGraph format conversion; preserves signal values

### convert MAF mutation format
**Args:** `maf hg19ToHg38.over.chain.gz input.maf hg38.fa output.maf`
**Explanation:** CrossMap maf subcommand; hg19ToHg38.over.chain.gz chain file; input.maf input MAF; hg38.fa target genome FASTA; output.maf output MAF; for Mutation Annotation Format conversion; requires target genome FASTA like VCF
