---
name: beast2-xml
category: Phylogenetics
description: Generates XML input files for BEAST2 from aligned sequence data (FASTA, NEXUS, CSV) for Bayesian evolutionary analysis, molecular dating, and phylogenetic inference.
tags: beast2, phylogenetics, bayesian, xml, evolutionary-analysis, molecular-dating, phylogenetic-inference
author: AI-generated
source_url: https://www.beast2.org/
---

## Concepts

- **Input Data Formats**: beast2-xml accepts multiple sequence alignment formats including FASTA, NEXUS, CSV, and Phylip. All input sequences must be properly aligned with consistent lengths and unambiguous character states (standard IUPAC nucleotides or amino acids).

- **XML Output Structure**: The tool generates a complete BEAST2 XML file containing data, site model, clock model, tree model, and prior specifications. This XML serves as the configuration file for the BEAST2MCMC analysis engine.

- **Model Specification Options**: beast2-xml supports various evolutionary models (JC69, HKY, GTR for nucleotides; LG, WAG, PAM for amino acids), clock models (strict, relaxed exponential, relaxed lognormal), and tree priors (Constrained, Coalescent, Birth-Death).

- **Data Type Inference**: The tool automatically detects whether sequences are nucleotide or amino acid data, but this can be manually overridden using the datatype parameter. Incorrect type inference leads to analysis errors.

- **Taxon Set Definitions**: Users must specify taxon sets for species, loci, or partitions using the taxonset parameter. Missing or inconsistent taxon labels across partitions will cause XML validation failures.

## Pitfalls

- **Mismatched Sequence Lengths**: Providing unaligned sequences with varying lengths causes the tool to fail during XML generation. BEAST2 requires all sequences in a partition to be identical in length.

- **Duplicate or Missing Taxon Labels**: Duplicate taxon names in the input file result in overwritten sequences, while missing labels referenced in other options cause XML parsing errors in BEAST2.

- **Unsupported Characters in Sequences**: Non-standard IUPAC characters (like 'N' for ambiguity is supported, but symbols like '-' or '?' unless properly specified as missing data) or numeric characters in nucleotide data trigger parsing failures.

- **Incorrect Clock-Tree Model Combinations**: Specifying an incompatible clock model for a tree model (e.g., using a birth-death model with a coalescent prior) produces XML that BEAST2 will reject at runtime.

- **Overspecified Taxa with Large Datasets**: Generating XML for datasets with thousands of taxa without adjusting memory allocation in BEAST2 leads to out-of-memory errors during analysis execution.

## Examples

### Generate basic XML from FASTA alignment
**Args:** `-i sequences.fasta -o analysis.xml`
**Explanation:** Creates a minimal BEAST2 XML file using default models (HKY, strict clock, coalescent) from a FASTA input file.

### XML with specific nucleotide substitution model
**Args:** `-i alignments.fasta -o beast_input.xml -s GTR -c relaxed lognormal`
**Explanation:** Uses the GTR substitution model with a relaxed lognormal clock to account for rate variation across branches.

### Specify nucleotide data type explicitly
**Args:** `-i coding_seqs.fasta -o output.xml --datatype nucleotide`
**Explanation:** Forces nucleotide data type interpretation, useful when automatic detection may be ambiguous.

### Multi-locus analysis with partition
**Args:** `-i locus1.fasta locus2.fasta -o multi.xml -p partition`
**Explanation:** Generates XML with partitioned data allowing separate models for each locus, enabling linked or unlinked analysis.

### Specify tree prior and output BEAST file
**Args:** `-i taxa.fasta -o dated.xml --treeprior "birth-death" --chainlength 10000000`
**Explanation:** Uses a birth-death tree prior with a longer Markov chain for more robust phylogenetic inference with estimated divergence times.

### Generate XML with calibration constraints
**Args:** `-i calibrated.fasta -o calibrated_analysis.xml --treeprior " calibrated"`
**Explanation:** Creates XML incorporating previously specified calibration points for molecular dating analysis with relaxed molecular clock.