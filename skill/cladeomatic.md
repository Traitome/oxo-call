---
name: cladeomatic
category: phylogenetic-analysis
description: A rapid gene-by-gene approach for bacterial outbreak analysis and allelic typing, enabling high-throughput comparison of genomic loci across samples using reference schemas and allelic profiling.
tags:
- whole-genome-sequencing
- outbreak-analysis
- allele-calling
- gene-by-gene
- bacterial-typing
- chewbbа
- mlst
- phylogenetic
author: AI-generated
source_url: https://github.com/achtman-lab/CladeOMatic
---

## Concepts

- **Schema-based loci definition**: CladeOMatic operates on a predefined "schema" — a curated set of genomic loci (genes or gene fragments) extracted from one or more reference sequences. Each locus is identified by a unique name and numeric allele identifier, enabling consistent cross-sample comparison independent of assembly quality.

- **Three-stage workflow**: The tool follows a build → annotate → call pipeline. First, `cladeomatic-build` defines and extracts locus sequences from references. Second, `cladeomatic-annotate` identifies locus matches in query assemblies. Third, `cladeomatic-call` assigns allelic designations and produces allelic profiles as tab-delimited or aligned FASTA output.

- **Allelic profile output**: The primary output is an allele matrix where rows are samples and columns are locus names, with cell values being the called allele number (integer). Novel alleles receive temporary designations (e.g., allele "0" or "NEW") and full sequences are dumped for later curation. The profile is directly compatible with PHYLOVIZ for visualization and epidemiological analysis.

- **Mixed allele handling**: When a locus shows mixed signals (e.g., heterozygous calls or paralogues), CladeOMatic prioritizes a consensus call but can emit a warning flag. Users may need to inspect raw BLAST-like alignments to resolve ambiguous loci manually.

- **Input format flexibility**: The tool accepts FASTA and GenBank formats for references and assemblies. Output alignments are produced as FASTA with full locus sequences, enabling downstream maximum-likelihood or neighbor-joining phylogenetics with standard tools like RAxML or IqTree.

## Pitfalls

- **Using an unverified schema**: Building a schema from a single incomplete or misannotated reference causes systematic allele calling failures across all samples. Consequences include excessive "novel" allele calls and inflated phylogenetic diversity that does not reflect true evolutionary signal.

- **Ignoring mixed-allele warnings**: When a sample contains two distinct sequences at a locus (e.g., due to duplicated regions or contamination), CladeOMatic may assign a consensus or partial allele. Continuing analysis without manual review produces incorrect pairwise distances and distorts cluster definitions.

- **Mismatched reference and query species**: If query genomes belong to a different lineage or species than the reference schema, locus detection sensitivity drops sharply. The result is many missing loci (null alleles) that artificially inflate inter-sample distances and lead to false-positive distinct clusters.

- **Reusing allele numbers across schema updates**: allele identifiers are schema-specific. Adding new loci to an existing schema without reassigning allele numbers causes silent collisions where different loci share the same numeric identifier in downstream analysis, corrupting phylogenetic trees.

- **Insufficient sequence overlap at locus boundaries**: Loci defined with very short flanking regions may fail to align correctly when query sequences contain large indels or structural variants near the locus edges, resulting in truncated or misaligned allelic sequences that skew phylogenies.

## Examples

### Build a locus schema from a curated reference sequence
**Args:** `build --reference refsequence.gbk --output-schema myschema --locus-prefix genes --min-locus-length 100 --overlap 20`
**Explanation:** Extracts all detectable loci from a GenBank reference file using a sliding window approach, creating a schema file with prefixed locus names and enforcing minimum length and overlap thresholds for robust downstream matching.

### Annotate locus matches in a query assembly
**Args:** `annotate --assembly sample1.fasta --schema myschema --output-dir annotations --identity-cutoff 90 --coverage-cutoff 80`
**Explanation:** Scans a query assembly against the predefined schema, reporting locus hits that meet minimum percent identity and coverage filters, and writing per-locus alignment summaries for manual inspection.

### Call alleles across multiple samples with a prebuilt schema
**Args:** `call --samples sample1.fasta sample2.fasta sample3.fasta --schema myschema --output-profile allelic_profile.tsv --output-fasta aligned_alleles.fasta --novel-handling assign-new`
**Explanation:** Performs batch allelic typing for three input assemblies, producing a tab-delimited allele matrix and aligned FASTA of called locus sequences, automatically assigning new numeric designations to allele sequences not present in the schema.

### Export a PhyloViZ-compatible edge list from called profiles
**Args:** `call --profile allelic_profile.tsv --export-edges outbreak_edges.csv --distance-method goeBURST --min-subclones 3`
**Explanation:** Converts the allelic profile matrix into an edge list suitable for PhyloViZ visualization, using goeBURST distance calculations to define clonal complexes and specifying a minimum number of supporting loci for subclique formation.

### Perform recursive schema update by incorporating novel alleles
**Args:** `build --reference novel_alleles.fasta --existing-schema myschema --output-schema myschema_v2 --update-mode merge`
**Explanation:** Takes a set of allele sequences flagged as novel during previous calling rounds and merges them into the existing schema, creating a versioned schema that eliminates "0" calls in future analyses and preserves consistent allele numbering across iterative updates.