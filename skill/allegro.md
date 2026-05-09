---
name: allegro
category: genetic_analysis
description: A software package for family-based genetic linkage and association analysis. Allegro performs pedigree-based statistical tests to identify genomic regions co-segregating with traits in families, supporting both parametric (LOD score) and non-parametric linkage methods.
tags:
  - genetics
  - linkage
  - pedigree
  - family-based
  - association
  - LOD score
  - genome analysis
author: AI-generated
source_url: https://www.sagebionetworks.org/
---

## Concepts

- **Pedigree Input Format**: Allegro accepts standard PED/MAP file pairs where PED files contain family ID, individual ID, paternal/maternal IDs, sex, phenotype, and genotype columns for each marker. The MAP file defines marker names and positions.
- **Analysis Modes**: The tool supports parametric linkage analysis (using a specified disease model) and non-parametric linkage (NPL) analysis, both outputting LOD scores or NPL scores to assess evidence of linkage.
- **Output Statistics**: Results include LOD scores, allele sharing statistics, penetrance ratios, and p-values for each analyzed marker or genomic position, enabling identification of linked chromosomal regions.
- **Phenotype Coding**: Affected status must be encoded as 1 (unaffected), 2 (affected), or -9/unknown for unknown phenotype. Quantitative traits can be analyzed using the -q flag with appropriate trait values.

## Pitfalls

- **Mismatched PED/MAP Files**: Using a MAP file with different marker identifiers or order than the PED file will cause marker genotypes to be misaligned, leading to completely incorrect linkage results.
- **Unset Phenotype Values**: Individuals with missing phenotype data (coded incorrectly as 0 or left blank) are excluded from analysis by default, reducing statistical power and potentially introducing bias if exclusions are not documented.
- **Incorrect Sex Chromosome Handling**: Failing to specify the --sex-specific flag when analyzing X-linked markers can lead to erroneous results, as allele calls and inheritance patterns differ between X and autosomes.
- **Small Family Samples**: Analyzing individual nuclear families with few affected relatives produces underpowered linkage statistics; LOD scores below 3.0 should be interpreted with extreme caution regardless of nominal "significant" threshold.

## Examples

### Run parametric linkage analysis on a PED/MAP file pair
**Args:** `--pedigree data.ped --map data.map --model dominant --output results.txt`
**Explanation:** Performs parametric linkage analysis assuming a dominant disease model, outputting LOD scores for each marker to results.txt.

### Execute non-parametric allele sharing analysis
**Args:** `--pedigree family.ped --map family.map --npl --output npl_results.txt`
**Explanation:** Performs non-parametric linkage analysis using allele sharing methods, suitable when disease model is unknown.

### Analyze X-linked markers with sex-specific inheritance
**Args:** `--pedigree xlinked.ped --map xlinked.map --sex-specific --output xlinked_out.txt`
**Explanation:** Configures analysis for X-chromosome markers where inheritance differs by sex, applying appropriate hemizygote rules for males.

### Specify a custom disease allele frequency for parametric analysis
**Args:** `--pedigree data.ped --map data.map --model recessive --freq 0.01 --output rec_out.txt`
**Explanation:** Sets the disease allele frequency to 1% for a recessive model, affecting expected genotype frequencies under the hypothesized model.

### Run analysis with multiple phenotypes in the same PED file
**Args:** `--pedigree multi.ped --map multi.map --phenotype-column 7 --output multi_out.txt`
**Explanation:** Uses phenotype data from column 7 instead of default column 6, allowing analysis of multiple traits from a single pedigree file.

### Adjust penetrance values for a specific genetic model
**Args:** `--pedigree data.ped --map data.map --model generic --penetrance 0.8,0.4,0.0 --output pen_out.txt`
**Explanation:** Specifies penetrance values for genotypes AA, Aa, and aa respectively, essential for accurate parametric LOD score calculation.

###Perform genome-wide scan with pre-specified marker intervals
**Args:** `--pedigree data.ped --map data.map --scan --step 50000 --output scan_out.txt`
**Explanation:** Performs genome-wide linkage scan with 50kb intervals between test points, generating a dense map of linkage evidence across chromosomes.