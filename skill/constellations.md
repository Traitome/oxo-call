---
name: constellations
category: Genome Analysis
description: A tool for analyzing and visualizing genomic structural relationships, connecting variants across populations, and identifying haplotype blocks within genetic datasets. Constellations constructs interactive networks of genetic relationships based on linkage disequilibrium and co-occurrence patterns.
tags: ['genomics', 'structural-variation', 'haplotype-analysis', 'population-genetics', 'linkage-disequilibrium', 'network-analysis', 'visualization']
author: AI-generated
source_url: https://github.com/bioinformatics-tools/constellations
---

## Concepts

- **Input Formats**: Accepts VCF files (single or multi-sample), PLINK PED/MAP formats, and BCF files. Files must be bgzip-compressed and indexed with tabix for VCF/BCF inputs. Header lines with sample identifiers are required for population-level analysis.

- **Data Model**: Constructs a graph-based representation where nodes represent haplotypes or variants and edges represent LD values (r² or D'). The tool calculates pairwise linkage statistics between all variants within a user-specified window (default 1 Mb). Edge weights correspond to the strength of statistical association.

- **Output Types**: Generates network files (GraphML, GEXF), LD matrices (CSV/TSV), haplotype block definitions (BED), and interactive HTML visualizations. Network outputs preserve node attributes including allele frequencies, statistical significance, and genomic coordinates.

- **Key Behaviors**: Performs independent phasing quality assessment, identifies recombination hotspots, and outputs block boundaries in standard BED format. Processing mode determines whether analysis targets population-level LD structure or individual haplotype phases.

## Pitfalls

- **Using uncompressed VCF files**: The tool requires bgzip-compressed and tabix-indexed VCF inputs. Providing plain text VCF files causes immediate failure with a malformed header error. Always run `bgzip file.vcf && tabix -p vcf file.vcf.gz` before processing.

- **Specifying an excessively large analysis window**: Setting `--window-size` above 5 Mb causes memory overflow on standard workstations (16 GB RAM). The tool attempts to load all pairwise LD calculations into memory, which scales quadratically with window size. Use chromosome-stratified analysis for large datasets.

- **Ignoring missing genotype calls**: By default, constellations treats missing genotypes (./.) as heterozygous calls, skewing LD calculations toward zero. Always filter sites with missingness above 5% using `--max-missing 0.95` or apply pre-filtering with bcftools to avoid false negatives in LD estimates.

- **Confusing population and individual modes**: Running in population mode (`--mode population`) on a single-sample VCF produces sparse networks with minimal edges. Individual haplotype analysis requires pedigreed families or phased diploid calls. Verify input contains multiple unrelated samples for population-level analysis.

- **Using outdated reference coordinates**: The tool requires genome builds matching your input VCF (e.g., GRCh38, GCF_000001405). Specifying the wrong build in `--reference` causes coordinate misalignments when exporting BED files. Always confirm reference match between your VCF header and tool configuration.

## Examples

### Calculate pairwise LD for variants within a 500 kb window
**Args:** `--input variants.vcf.gz --window-size 500000 --output ld_matrix.csv --statistic r2`
**Explanation:** Computes LD (r²) between all variant pairs separated by ≤500 kb, exporting a CSV matrix useful for downstream haplotype block detection in R or Python.

### Generate a population-level haplotype network
**Args:** `--input cohort.vcf.gz --mode population --min-r2 0.8 --network-output population_network.graphml`
**Explanation:** Builds a graph where nodes are haplotypes connected when LD exceeds 0.8, exporting GraphML format for visualization in Cytoscape or Gephi.

### Identify recombination hotspots from LD decay
**Args:** `--input population.vcf.gz --mode population --detect-hotspots --hotspot-output hotspots.bed`
**Explanation:** Analyzes LD decay distance across the chromosome to locate recombination hotspots, outputting block boundaries in BED format for functional interpretation.

### Filter low-quality sites before network construction
**Args:** `--input raw.vcf.gz --max-missing 0.05 --min-af 0.01 --qual-filter --output filtered.vcf.gz`
**Explanation:** Removes sites with >5% missing genotypes and minor allele frequency