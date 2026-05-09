---
name: annotwg
category: variant_annotation
description: A whole-genome variant annotation tool that adds functional, regulatory, and population frequency annotations to VCF files. Annotates variants with gene information, protein change predictions, dbSNP identifiers, conservation scores, and allele frequencies from population databases.
tags: [variant_annotation, vcf, genomics, functional_annotation, whole_genome, snp, indel]
author: AI-generated
source_url: https://github.com/annotwg/annotwg
---

## Concepts

- **Input Format**: annotwg accepts VCF (Variant Call Format) files as primary input. The input VCF must be properly formatted with at least CHROM, POS, ID, REF, ALT, QUAL, and FILTER columns. Gzip-compressed (.gz) VCF files are supported when indexed with tabix.
- **Annotation Types**: The tool adds multiple annotation categories: gene-centric annotations (gene name, transcript, coding change, protein alteration), dbSNP identifiers from reference databases, population allele frequencies (ExAC, gnomAD), conservation scores (PhyloP, CADD), and pathogenicity predictions (SIFT, PolyPhen).
- **Output Format**: Default output replaces the INFO column with annotation fields in VCF format. Use `--output-format bed` for BED track output, or `--output-format table` for tab-delimited text. The tool can also emit JSON (--output-format json) for programmatic downstream processing.
- **Database Build**: Specify the reference genome with `--build hg38` or `--build hg19`. Annotation quality depends critically on genome build consistency—the tool will error if VCF chromosome naming doesn't match the selected build.
- **Batch Processing**: Process multiple VCF files in a single run by supplying a file list (one path per line) to `--input-list`. Use `--jobs` to parallelize across cores; each input file spawns an independent worker process.

## Pitfalls

- **Genome Build Mismatch**: Using `--build hg19` on a VCF aligned to hg38 produces incorrect gene assignments and annotation failures. Check your VCF header (`##fileformat` and contig lines) to confirm the reference build before running.
- **Duplicate Variant IDs**: Supplying a VCF with pre-existing ID column entries (e.g., rs numbers) that conflict with dbSNP annotations may cause the tool to skip new annotations or overwrite existing IDs silently. Use `--preserve-ids` to keep original IDs intact.
- **Memory Limits on Large VCFs**: Whole-genome VCF files (>50GB) require substantial RAM for index loading. The tool may crash with out-of-memory errors when annotating without first splitting chromosomes. Use `--split-by-chrom` to process each chromosome independently.
- **Missing Index Files**: Running annotwg without pre-built tabix indices for database files causes immediate failure. Always run `--build-indices` on your annotation databases before the first annotation session.
- **Inconsistent Chromosome Notation**: VCFs using "chr1" notation paired with annotation databases built without "chr" prefixes (or vice versa) result in zero annotations with no warning. Verify chromosome format consistency before processing.

## Examples

### Annotate a single VCF file with gene and functional information
**Args:** --input variants.vcf --output annotated.vcf --annotations gene,func
**Explanation:** This annotates each variant with gene name, transcript ID, and protein change using the default hg38 build gene database.

### Add dbSNP identifiers and population frequencies to a VCF
**Args:** --input variants.vcf --output annotated.vcf --annotations dbsnp,freq --populations gnomad,exac
**Explanation:** The tool adds rsIDs from dbSNP and extracts allele frequencies from gnomAD and ExAC population datasets.

### Annotate with conservation scores and pathogenicity predictions
**Args:** --input variants.vcf --output annotated.vcf --annotations conservation,pathogenicity --tools cadd,polyphen,sift
**Explanation:** This adds conservation (PhyloP) and damaging predictions from CADD, PolyPhen, and SIFT to assess variant impact.

### Output annotations in tabular format instead of VCF
**Args:** --input variants.vcf --output annotated.tsv --output-format table --columns chrom,pos,ref,alt,gene,protein_change,cadd_score
**Explanation:** Exports a TSV file with selected columns instead of modifying the VCF INFO field, useful for downstream filtering spreadsheets.

### Process multiple VCF files in parallel using 8 cores
**Args:** --input-list vcf_files.txt --output-dir ./annotated/ --jobs 8 --annotations gene,freq
**Explanation:** Reads a file containing paths to multiple VCF files and processes them concurrently using 8 parallel workers for faster throughput.

### Annotate a hg19-aligned VCF with appropriate genome build
**Args:** --input variants_hg19.vcf --output annotated_hg19.vcf --build hg19 --annotations gene,dbsnp
**Explanation:** Uses the hg19 gene and dbSNP annotation databases, ensuring correct chromosome and coordinate mappings for older alignment data.

### Split output by chromosome to manage file sizes
**Args:** --input large_genome.vcf --output-dir ./chr_annotated/ --split-by-chrom --annotations gene
**Explanation:** Creates separate output files per chromosome to avoid single massive files, simplifying downstream chromosome-specific analysis.