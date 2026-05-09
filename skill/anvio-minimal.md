---
name: anvio-minimal
category: Pangenomics
description: A tool in the Anvi'o platform that classifies gene clusters into minimal, core, and accessory categories based on their occurrence across genomes, enabling analysis of conserved versus variable genomic elements in microbial pangenomes.
tags: [anvio, pangenomics, gene-clusters, microbial-genomics, HMM, core-genome, accessory-genome]
author: AI-generated
source_url: https://anvio.org/programs/anvio-minimal/
---

## Concepts

- **Minimal gene clusters** are those present in a subset of genomes (between the core threshold and a defined minimum) but not universally. Anvi-minimal classifies each gene cluster as core (in ≥90% of genomes by default), minimal (in a smaller defined fraction), or accessory (rare/unique), based on per-genome occurrence data stored in the pan database.
- **The pan database (`.pan-db`)** is the primary input. It is created by `anvi-pan-genome` and stores gene clusters, their functional annotations, and occurrence matrices. Anvi-minimal reads this database, computes cluster statistics, and appends new summary layers describing minimality across all genomes in the pan.
- **HMM-based identification** of gene clusters uses hidden Markov models trained on protein families. Anvi-minimal leverages these cluster assignments to determine which genomes contain each gene, and the tool can regenerate these HMM profiles if the `--regen-hmms` flag is used, which may alter previously defined cluster boundaries.
- **Output includes a new view** in the pan database called "mininality" (percentage of genomes containing each gene cluster) and summary statistics printed to stdout. This data integrates directly with `anvi-display-pan` for interactive visualization of core versus variable genes.
- **Functional enrichment analysis** can be performed downstream using `anvi-compute-functional-enrichment` on the same pan database to identify which gene functions are enriched in minimal versus accessory clusters, making the tool essential for comparative genomics studies.

## Pitfalls

- **Using an outdated or incompatible pan database** causes the tool to fail with a SQLite schema mismatch error. The pan database must be created with the same Anvi'o version being used to run anvi-minimal; mixing versions (e.g., v7 input to v8 tool) produces cryptic errors about missing columns.
- **Omitting the `--output-dir` flag** results in the summary being written only to stdout without persistent output files. The terminal output is lost if the session terminates, requiring a complete re-run to regenerate the data, which is especially problematic for large pangenomes.
- **Setting `--min-occurrence` too low** creates an overly broad "minimal" category that includes nearly all gene clusters, reducing analytical value. Conversely, a too-high threshold excludes biologically meaningful variable genes, collapsing the accessory set inappropriately.
- **Running on an unmerged pan database** (with genomes having different contigs-db sources) produces unreliable occurrence counts because some gene clusters may be fragmented or incompletely identified, leading to systematic undercounting of gene presence in certain genomes.
- **Assuming gene clusters are unchanged after running `anvi-rename-gene-caller`** without re-running HMM profiling. If gene callers renamed genes across the pan database, the cluster assignments stored in the pan.db may reference stale gene caller IDs, causing anvi-minimal to skip or misclassify affected clusters.

## Examples

### Classify gene clusters as core, minimal, and accessory from an existing pan database
**Args:** `-p PAN.db --output-dir MINIMAL_OUTPUT`
**Explanation:** This reads the pan database and classifies each gene cluster by its occurrence fraction across all genomes, writing the results to the specified output directory for downstream visualization and analysis.

### Recompute HMM profiles and classify clusters with custom occurrence thresholds
**Args:** `-p PAN.db --output-dir MINIMAL_OUTPUT --regen-hmms --min-occurrence 4 --max-occurrence 12`
**Explanation:** Using `--regen-hmms` forces re-computation of hidden Markov model searches for gene clusters while custom thresholds define minimal as genes appearing in at least 4 but no more than 12 genomes.

### Generate only the gene cluster summary table without interactive data
**Args:** `-p PAN.db --just-do-it --output-dir MINIMAL_OUTPUT`
**Explanation:** The `--just-do-it` flag skips functional enrichment updates and produces only the tabular summary of gene cluster occurrence, useful for rapid screening of large pangenomes.

### Export minimality statistics for specific genomes in a subset
**Args:** `-p PAN.db --output-dir MINIMAL_OUTPUT --genomes-storage GENOMES-STORAGE.db --sample-name "TaxonA"`
**Explanation:** When a genomes storage file is provided along with a sample name, anvi-minimal generates statistics filtered to the specified subset of genomes rather than the full pan.

### Classify clusters and directly feed results to functional enrichment analysis
**Args:** `-p PAN.db --output-dir MINIMAL_OUTPUT --compute-functional-enrichment`
**Explanation:** Adding the enrichment flag triggers automatic computation of which functions are statistically associated with minimal versus core clusters, saving a separate results file ready for downstream interpretation.

### Re-classify clusters after updating functional annotations in the pan database
**Args:** `-p PAN.db --output-dir MINIMAL_OUTPUT --enforce --destination-db PAN-REFINED.db`
**Explanation:** When annotations have been updated externally, `--enforce` overwrites existing classifications and `--destination-db` writes the refined results to a new pan database rather than modifying the input in place.