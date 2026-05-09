---
name: beagle-lib
category: Genetics / Genotype Imputation & Phasing
description: BEAGLE is a software tool for genotype phasing, haplotype inference, and genetic variant imputation in sequencing and SNP array data. It implements Bayesian clustering and graphical model algorithms to phase haplotypes and impute untyped variants.
tags: [genotype, imputation, phasing, haplotype, genetics, VCF, SNP, sequencing, BEAGLE]
author: AI-generated
source_url: https://faculty.washington.edu/browning/beagle/beagle.html
---

## Concepts

- **Input Format:** BEAGLE accepts VCF (Variant Call Format) files containing genotype data. The VCF must include genotype likelihoods (GL or PL fields) or genotype posterior probabilities. Both phased reference panels (using the `gt` or `hd` format) and unphased query datasets are supported.
- **Output Format:** BEAGLE outputs phased VCF files with updated GT fields and imputed genotype posterior probabilities. The output VCF includes probability annotations (GP field) for imputed alleles and genotypes.
- **Reference Panels:** High-quality phased reference panels (such as 1000 Genomes, HRC, or TopMed) dramatically improve imputation accuracy. The reference panel is specified via the `ref` parameter using a VCF/BCF file.
- **Chromosome-Specific Processing:** BEAGLE processes each chromosome independently and requires specification of the chromosome via the `chrom` parameter (e.g., `chrom=22` for chromosome 22).
- **Algorithm Modes:** BEAGLE supports different analysis modes: genotype imputation (`impersonate=true`), haplotype phasing only, and rare variant calling. The `iterations` parameter controls the number of refinement iterations.

## Pitfalls

- **Missing Genotype Likelihoods:** Providing VCF files without GL, PL, or GP genotype probability fields leads to BEAGLE treating all genotypes as equally likely, severely degrading imputation accuracy. Always include proper likelihoods from sequencing reads or SNP array confidence scores.
- **Mismatched Reference Panel:** Using a reference panel from a different population or genetic ancestry than the query samples reduces imputation accuracy and may introduce false variants. Select a reference panel matched to the样本 population.
- **Incorrect Chromosome Specification:** Failing to set the `chrom` parameter or using incorrect chromosome names prevents BEAGLE from properly aligning variants with the reference panel and causes analysis failure.
- **Large Files Without Memory Allocation:** Processing whole-genome VCF files (especially sequencing data) requires substantial RAM. The default Java heap may be insufficient, leading to out-of-memory errors. Increase Java heap with `-Xmx` flag (e.g., `-Xmx8g` for 8GB).
- **Overlapping Variant Regions:** Running BEAGLE on overlapping genomic regions without proper batching creates duplicate variant records in the output, corrupting downstream analysis.

## Examples

### Phase genotypes from a VCF file using default settings
**Args:** `ref=reference_panel.vcf.gz chrom=1 iterations=10 sample=my_data.vcf.gz`
**Explanation:** This runs BEAGLE to phase haplotypes and impute missing genotypes in chromosome 1, using a reference panel for imputation guidance with 10 refinement iterations.

### Impute genotypes with a genetic map for recombination modeling
**Args:** `ref=1000G_phase3_chr22.vcf.gz chrom=22 gmap=genetic_map_chr22.txt sample=query_chr22.vcf.gz out=imputed_chr22`
**Explanation:** Including a genetic map enables BEAGLE to model recombination hotspots, improving phasing accuracy especially in regions of high linkage disequilibrium decay.

### Run BEAGLE with 15 iterations and increased window size
**Args:** `ref=refpanel.vcf chrom=20 iterations=15 window=10000 sample=input.vcf.gz`
**Explanation:** Increasing iterations refines haplotype estimates more thoroughly, while the window parameter controls the genomic segment size processed in each step for memory efficiency.

### Impute SNP array data with allele error modeling
**Args:** `ref=HRC_chr1.vcf.gz chrom=1 impute=true sample=snp_array_data.vcf.gz`
**Explanation:** For SNP array data, enabling the impute flag improves accuracy by modeling allele genotyping errors specific to array technology rather than sequencing reads.

### Phase an entire chromosome with larger Java heap memory
**Args:** `ref=reference_chr3.vcf.gz chrom=3 sample=query_chr3.vcf.gz`
**Explanation:** Running BEAGLE with insufficient memory causes crashes; requesting more heap allows processing of larger chromosome-wide datasets without interruption.