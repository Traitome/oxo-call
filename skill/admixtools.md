---
name: admixtools
category: population genetics / admixture analysis
description: A software package for analyzing genetic admixture in human (and other) populations. Provides tools for computing f-statistics, D-statistics, f4-statistics, admixture dating (rolloff), and model-based ancestry proportions. Built on the EIGENSTRAT format for efficient processing of large genotype datasets.
tags:
  - genetics
  - ancestry
  - population-structure
  - f-statistics
  - admixture
  - PCA
  - ancestry-inference
author: AI-generated
source_url: https://github.com/DReichLab/EIGENSTRAT
---

## Concepts

- **EIGENSTRAT format:** admixtools uses a specific three-file format (.ind, .snp, .geno) where .ind contains individual metadata (ID, sex, population label), .snp contains SNP metadata (ID, chromosome, position, reference/alternate alleles), and .geno stores genotypes as a compressed binary matrix. This format is memory-efficient for large datasets.

- **f-statistics framework:** The tool computes f2, f3, and f4 statistics to quantify shared drift between populations. f3-statistics (ABBA-BABA test) detect admixture; negative f3 values indicate a target population is admixed between two source populations. f4-statistics measure symmetric difference in allele sharing.

- **qpAdm admixture modeling:** The qpAdm tool fits a model where a test population is modeled as a linear combination of source populations (e.g., test = w1*source1 + w2*source2 + w3*source3 + w4*source4). It reports mixing weights (proportions) and standard errors, allowing assessment of which ancestral populations contributed to the target.

- **rolloff dating:** The rolloff command estimates the time of admixture events by modeling the decay of linkage disequilibrium with genetic distance. It fits an exponential decay curve and reports the estimated number of generations since admixture, which can be converted to years using a generation time (typically ~25-30 years).

- **Companion tools:** admixtools includes convertf (format conversion), smartpca (PCA analysis), and twstats (tree statistics). The companion binary `convertf` is often needed to transform PLINK or other formats into EIGENSTRAT format before main analyses.

## Pitfalls

- **Mismatched allele coding:** Failing to specify -strand in convertf can flip reference/alternate alleles, causing all subsequent f-statistics to have opposite signs. Always verify allele frequencies match expected values after conversion using a known population as a sanity check.

- **Insufficient pop list specification:** Many admixtools programs require explicit pop lists (-p onepar. or -q prefix). Using an incorrect or incomplete population list can silently drop samples or produce NA values, leading to misleading admixture estimates or failed fitting.

- **Using admixtools instead of subcommands:** Users often invoke just `admixtools` expecting it to run a specific analysis. The tool is a wrapper; valid subcommands include `qp3Pop`, `qpDstat`, `qpAdm`, `rolloff`, `smartpca`, `convertf`, and `twstats`. Running the bare command prints usage help.

- **Conflicting population labels:** Duplicate population labels in the .ind file cause ambiguity in which individuals represent a given source. Ensure each population label is unique within a sample set, or use individual IDs that clarify group membership.

- **Missing or malformatted .snp file:** The .snp file requires strict formatting with tab/space delimiters; chromosome values like "chr1" vs "1" must match between the .snp file and any reference genome. Inconsistent naming breaks coordinate matching in downstream analyses.

## Examples

### Convert PLINK PED/MAP files to EIGENSTRAT format
**Args:** -pinfo poplist:poplist.txt -inPlink ./input -outEigenstrat ./output
**Explanation:** This runs convertf to transform PLINK PED/MAP files into EIGENSTRAT format, reading sample metadata from poplist.txt and writing three output files to the output directory.

### Compute f3-statistics to test for admixture
**Args:** qp3Pop -p leftpar. -o output:results.out > log.txt
**Explanation:** This computes f3-statistics for all triples defined in leftpar, testing whether populations show evidence of admixture using the Outgroup f3-statistic formulation; results are written to results.out.

### Compute D-statistics (ABBA-BABA test)
**Args:** qpDstat -p leftpar. -v > dstat_results.txt
**Explanation:** This runs qpDstat with leftpar to compute D-statistics comparing allele sharing patterns between populations, displaying verbose output (-v flag) to see intermediate values.

### Model ancestry proportions using qpAdm
**Args:** qpAdm -p leftpar.:left_ancestors.txt -o output:fitted_model.txt -info:allpops.txt
**Explanation:** This runs qpAdm to model each target population as a mixture of source populations listed in left_ancestors.txt, outputting fitted mixing weights to fitted_model.txt using allpops.txt for complete population information.

### Date an admixture event using rolloff
**Args:** rolloff -p leftpar.:admix_pairs.txt -g genetic_map.hg19.txt > rolloff_out.txt
**Explanation:** This runs rolloff to estimate the time since admixture by modeling LD decay, reading genetic map coordinates from hg19.txt and outputting generations-since-admixture estimates.

### Perform PCA analysis on genotype data
**Args:** smartpca -i eigenstrat_input.geno -a eigenstrat_input.snp -b eigenstrat_input.ind -k 20 -o pca_output.evec -e pca_eval.txt
**Explanation:** This runs smartpca to compute the first 20 principal components from EIGENSTRAT format input files, outputting PC coordinates to pca_output.evec and eigenvalues to pca_eval.txt.

### Convert FAM/BED to EIGENSTRAT using convertf
**Args:** convertf -p par.PED_EIGENSTRAT
**Explanation:** This runs convertf instructed by parameter file par.PED_EIGENSTRAT to convert FAM/BED format to EIGENSTRAT .ind/.snp/.geno files for use with other admixtools commands.

### Compute f4-statistics between four populations
**Args:** qpF4 -p leftpar.:f4_pairs.txt -o f4_out.txt
**Explanation:** This runs qpF4 to compute f4-statistics for all quadruplets defined in the parameter file, measuring asymmetric allele sharing patterns between the specified population sets.