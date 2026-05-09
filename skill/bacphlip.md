---
name: bacphlip
category: genomics
description: A bioinformatics tool for analyzing bacteriophage sequences and predicting phage-related proteins in bacterial genomes. bacphlip identifies phage gene candidates, annotates structural proteins, and generates phylogenetic profiles for phage-bacterial interaction studies.
tags: [phage, virus, bacteria, Bacillus, genomics, annotation, phylogenetic-analysis]
author: AI-generated
source_url: https://github.com/bacphlip/bacphlip
---

## Concepts

- bacphlip accepts FASTA or GenBank format input files containing nucleotide or protein sequences from bacterial genomes. It processes whole-genome assemblies to identify prophage regions and predict phage-derived genes.
- The tool outputs multiple result files including `.phage_pred.tsv` (predicted phage regions), `.genes.fasta` (extracted gene sequences), and `.summary.txt` (statistical summary of findings). Output files preserve the input basename and append the appropriate extension.
- bacphlip uses hidden Markov model (HMM) profiles for phage marker gene detection. The tool comes bundled with pretrained HMM profiles for major phage families (Myoviridae, Siphoviridae, Podoviridae) and supports custom HMM profile loading via the `--hmmdb` flag.
- The tool supports parallel processing via the `--threads` flag, which spawns multiple worker processes to handle large genomes faster. Thread count defaults to 1 but should be set to the number of available CPU cores for optimal performance.

## Pitfalls

- Running bacphlip on incomplete or highly fragmented genome assemblies (e.g., thousands of contigs) produces false positives because short contigs lack the contextual nucleotide patterns needed for accurate prophage prediction. Consequences include spurious phage region calls and wasted downstream validation effort.
- Using outdated HMM profile databases causes poor detection sensitivity for novel or divergent phage types. The tool will still run but may miss legitimate phage genes, particularly in genomes with recently acquired prophages that differ from canonical phage families.
- Specifying more threads than available CPU cores causes resource contention and can actually slow down processing or trigger system errors on memory-constrained systems. Always verify core availability with `nproc` or system monitoring tools before setting `--threads`.
- Omitting the required output directory (`-o/--outdir`) results in files being written to the current working directory, which may lack write permissions or cause directory clutter. Always explicitly specify a dedicated output directory.

## Examples

### Predict prophage regions in a Bacillus genome assembly

**Args:** `-i genome.fasta -o output_dir/ --threads 4`

**Explanation:** This runs bacphlip on the input genome FASTA file using 4 parallel threads, writing all results to the specified output directory.

### Use custom HMM profiles for phage detection

**Args:** `-i genome.fasta -o output_dir/ --hmmdb custom_phage_hmms.txt`

**Explanation:** This loads a custom HMM profile database instead of the default bundled profiles, enabling detection of phage types not covered by the standard models.

### Adjust detection stringency for relaxed predictions

**Args:** `-i genome.fasta -o output_dir/ --evalue 1e-3 --coverage 0.5`

**Explanation:** This lowers the E-value threshold to 1e-3 and reduces minimum protein coverage to 50%, yielding more predictions at the cost of higher false positive rates.

### Extract only predicted structural proteins

**Args:** `-i genome.fasta -o output_dir/ --filter structural --format fasta`

**Explanation:** This filters results to retain only phage structural protein predictions and outputs them in FASTA format for downstream phylogenetic analysis.

### Run in verbose mode to debug detection issues

**Args:** `-i genome.fasta -o output_dir/ --verbose --log debug.log`

**Explanation:** This enables verbose logging and writes detailed debug information to debug.log, helping diagnose why certain phage regions may be missed or incorrectly predicted.