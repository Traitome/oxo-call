---
name: beast-pype
category: phylogenetic-analysis
description: A Python-based pipeline tool that automates Bayesian phylogenetic analysis workflows using BEAST (Bayesian Evolutionary Analysis by Sampling Trees). It handles input validation, XML configuration generation, execution management, and post-analysis result processing.
tags:
  - bayesian
  - phylogenetics
  - beast
  - molecular-evolution
  - xml
  - mcmc
author: AI-generated
source_url: https://github.com/beast-pype/beast-pype
---

## Concepts

- **Input Format Requirements**: beast-pype accepts multiple sequence formats including FASTA, NEXUS, and PHYLIP for alignments. The input alignment must contain no ambiguous characters, and taxon labels must be unique and consistent across all partitions in multi-locus analyses. Character sets and codon positions are specified separately for partitioned analyses.

- **XML Configuration Generation**: The core workflow converts validated alignments into BEAST-readable XML files. Subcommand `beast-pype xml` generates the analysis configuration with nested elements for substitution models, clock models, and tree priors. Each partition can receive independent model specifications, and the tool supports common models such as HKY, GTR, and codon-based substitutions.

- **MCMC Convergence Monitoring**: beast-pype tracks effective sample sizes (ESS) for key parameters during execution. The `beast-pype diagnose` subcommand analyzes log files and reports whether chains have achieved sufficient mixing. Recommended minimum ESS thresholds vary by parameter type; branch-related parameters typically require higher values than rate parameters.

- **Multi-locus Analysis Support**: The tool manages partitioned datasets by creating per-partition model specifications while maintaining a single tree topology. Partition linking options control whether clock rates, substitution matrices, or tree priors are shared across loci. The `--partition` flag accepts tab-delimited specification files for complex analyses.

## Pitfalls

- **Ignoring Tip Date Calibration**: Specifying incorrect or missing sampling dates causes the analysis to assume zero-time branches, producing meaningless evolutionary rate estimates. Dates must be provided in decimal years or ISO 8601 format and verified against the alignment metadata before running.

- **Misconfiguring Partition Independence**: Treating all partitions as completely independent when they share evolutionary processes leads to inflated parameter uncertainty. Linking clock models or substitution parameters across partitions is essential for multi-locus analyses of the same taxa.

- **Insufficient Chain Length**: Running short MCMC chains produces low ESS values that invalidate parameter estimates. Preliminary runs should use at least 10 million states with logging every 1000 states before scaling to final analyses with 50-100 million states for complex models.

- **Neglecting Taxon Set Validation**: Including duplicate or misspelled taxon labels in the input causes BEAST to fail silently or produce incorrect trees. The validation step must be run before XML generation to catch labeling inconsistencies.

- **Using Inappropriate Substitution Models**: Overly complex models such as independent GTR for small alignments (fewer than 50 taxa) lead to overfitting and unreliable branch length estimates. Model selection should consider sequence length, number of taxa, and biological hypotheses being tested.

## Examples

### Validate input alignment before analysis
**Args:** `validate --input sequences.fasta --format fasta`
**Explanation:** Running validation checks for ambiguous characters, duplicate labels, and format compliance before any analysis step.

### Generate XML configuration with HKY substitution model
**Args:** `xml --input alignment.nex --model-type hky --clock-type ucld --output beast_config.xml`
**Explanation:** Converting a NEXUS alignment to BEAST XML with a hierarchical Bayesian clock model suitable for evolutionary rate estimation.

### Run partitioned multi-locus analysis
**Args:** `run --xml multi_locus.xml --partitions partition.txt --length 50000000 --log 5000 --threads 8`
**Explanation:** Executing a five-locus partitioned analysis with 50 million MCMC states logged every 5000 iterations using 8 parallel threads.

### Diagnose convergence and ESS adequacy
**Args:** `diagnose --log beast.log --burnin 10 --min-ess 200`
**Explanation:** Analyzing the BEAST log file after discarding the first 10% of samples to verify all parameters exceed minimum ESS thresholds.

### Convert results to Nexus tree format
**Args:** `trees --input trees.nex --format nexus --output rooted_trees.nex --root midpoint`
**Explanation:** Converting sampled trees from BEAST output into Nexus format with midpoint rooting applied for downstream phylogenetic visualization.

### Estimate divergence times with calibrated priors
**Args:** `xml --input genes.fasta --clock-type relaxed --treePrior calibrated --bounds 2.5 7.0 --output timed.xml`
**Explanation:** Generating XML with a relaxed clock model and node height priors bounded between 2.5 and 7.0 million years for divergence time estimation.

### Process concatenated alignment with partition specifications
**Args:** `concatenate --input locus1.fasta locus2.fasta --output concat.fasta --partitions codon_positions.txt`
**Explanation:** Combining multiple locus alignments into a single file with codon position partitions specified for analysis.

### Generate XML with morphological data partition
**Args:** `xml --input combined.nex --partition morphological: morphology --partition nucleotide: genes --model-type hky --output morphology_xml.xml`
**Explanation:** Creating XML with separate substitution models for morphological and nucleotide partitions within a combined dataset.

### Run analysis with fixed random seed for reproducibility
**Args:** `run --xml analysis.xml --seed 42 --length 100000000 --log 10000`
**Explanation:** Executing BEAST analysis with a fixed random seed for reproducible MCMC results across identical computing environments.

### Extract summary statistics from log file
**Args:** `summary --log output.log --burnin 25 --stats posterior likelihood rate --output stats.txt`
**Explanation:** Extracting posterior mean and 95% highest posterior density intervals for specified statistics after appropriate burn-in removal.