---
name: cgt
category: variant_calling
description: A bioinformatics tool for detecting and genotyping genetic variants from sequencing data. Performs variant calling using configured algorithms and outputs results in standard formats for downstream analysis.
tags: [variant-calling, genomics, snp-detection, genotyping, vcf]
author: AI-generated
source_url: https://github.com/example/cgt
---

## Concepts

- **Input Format**: Accepts aligned sequencing data in BAM/SAM format as primary input, along with a reference genome in FASTA format. The tool reads read alignments and identifies discrepancies from the reference.
- **Output Format**: Produces variant calls in VCF (Variant Call Format) files, including SNP and indel information with quality scores, genotype likelihoods, and read depth metrics.
- **Variant Detection Algorithm**: Uses read-backed phasing and local reassembly to identify polymorphisms, applying statistical models to distinguish true variants from sequencing errors.
- **Companion Tools**: The cgt-build utility creates index files for the reference genome to accelerate alignment lookup during variant calling runs.
- **Quality Filtering**: Applies configurable read and base quality thresholds to filter out low-confidence calls, with default settings optimized for Illumina sequencing data.

## Pitfalls

- **Missing Reference Index**: Running cgt without first creating the reference index with cgt-build causes the tool to fail with a file-not-found error, as it cannot efficiently access the genome sequences.
- **Incompatible Read Group Information**: If BAM files lack proper read group headers (@RG), variant calling may produce biased genotypes or fail to leverage library-specific error models, leading to reduced accuracy.
- **Excessive Memory Usage**: Specifying too many parallel threads without considering system memory can cause out-of-memory errors, especially when processing large genomes or high-coverage datasets.
- **Ignoring Quality Scores**: Failing to adjust minimum quality thresholds for low-coverage data results in many false positive variant calls, while overly strict thresholds for high-coverage data may discard true variants.

## Examples

### Call variants from a BAM file against a reference genome
**Args:** --reference ref.fa --input alignments.bam --output variants.vcf
**Explanation:** This command reads the aligned reads from the BAM file, compares them to the reference genome, and outputs all identified variants to a VCF file.

### Call variants with specific quality and depth filters
**Args:** --reference ref.fa --input alignments.bam --output filtered.vcf --min-quality 30 --min-depth 10
**Explanation:** Applies stricter filtering criteria, only including variants with quality scores of 30 or higher and at least 10 reads supporting the call.

### Use multi-threading to accelerate variant calling
**Args:** --reference ref.fa --input alignments.bam --output variants.vcf --threads 8
**Explanation:** Runs the variant calling algorithm using 8 parallel threads, significantly reducing runtime on multi-core systems for large datasets.

### Call variants on a specific genomic region
**Args:** --reference ref.fa --input alignments.bam --output region_variants.vcf --region chr1:1000000-2000000
**Explanation:** Restricts analysis to a 1 Mb region on chromosome 1, useful for targeted analysis or testing parameters on smaller data subsets.

### Include indel calling alongside SNP detection
**Args:** --reference ref.fa --input alignments.bam --output all_variants.vcf --call-indels
**Explanation:** Enables detection of insertion and deletion events in addition to single nucleotide polymorphisms, producing a more comprehensive variant set.

### Output genotype likelihoods for downstream phasing
**Args:** --reference ref.fa --input alignments.bam --output with_likelihoods.vcf --emit-all-fields
**Explanation:** Includes additional genotype likelihood fields in the output VCF, preserving all information needed for subsequent haplotype phasing or advanced filtering.