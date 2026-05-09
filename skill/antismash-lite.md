---
name: antismash-lite
category: Bioinformatics - Secondary Metabolite Detection
description: Predicts biosynthetic gene clusters (BGCs) in bacterial and fungal genomes using profile hidden Markov models (HMMs). Identifies NRPS, PKS, terpenes, RiPPs, and other secondary metabolite production regions.
tags: [biosynthetic gene clusters, secondary metabolites, antiSMASH, natural products, genome mining, BGC prediction, NRPS, PKS]
author: AI-generated
source_url: https://antismash.secondarymetabolites.org
---

## Concepts

- AntiSMASH analyzes nucleotide sequences (FASTA or GenBank format) to identify biosynthetic gene clusters by detecting characteristic domain profiles (KS, AT, NRPS, Cy, etc.) and conserved gene arrangements.
- The tool supports multiple input formats: raw FASTA sequences, GenBank files with annotation, and allows output in HTML (visualization), GenBank (cluster sequences), JSON (parsed data), and text (summary) formats.
- Cluster types detected include: NRPS (non-ribosomal peptide synthetases), PKS (polyketide synthases - type I, II, III), terpenes, RiPPs (ribosomally synthesized peptides), ladderanes, aminocoumarins, ectoines, and siderophores.
- Taxonomic origin matters: use `--taxonomy bacteria` (default) or `--taxonomy fungi` since different HMM libraries are trained for bacterial vs. fungal sequences.
- Full analysis requires downloaded database files (knowngene clusters, pfam profiles) in the installation directory; running without these enables only basic prediction mode.

## Pitfalls

- **Short or fragmented input sequences**: Contigs shorter than 10kb often fail to contain complete clusters, leading to missed predictions or incomplete annotations.
- **Output format mismatches**: AntiSMASH auto-detects output format from extension; using `--output` with wrong extension or not specifying `--genbank`/`--json` when needed produces unexpected file types.
- **Missing database runtime files**: Running in strict mode or with `--knownclusterblast` fails if the required database files (downloaded separately) are not present in the default/antismash directory.
- **Memory exhaustion on large genomes**: Genomes over 10MB may require `--limit` or running in batches; single large sequences consume excessive RAM during domain detection.
- **Wrong organism type selected**: Using bacterial defaults for fungal genomes misses fungal-specific cluster types (e.g., NRPS-like in fungi); using wrong taxonomy reduces detection accuracy.

## Examples

### Basic run with a FASTA input file
**Args:** `--input-fasta input_genome.fasta`
**Explanation:** Runs AntiSMASH on a nucleotide FASTA file using default settings (bacterial taxonomy, HTML output) with the genome filename as the input.

### Specify bacterial taxonomy explicitly
**Args:** `--input-fasta input.fasta --taxonomy bacteria`
**Explanation:** Explicitly sets the bacterial HMM library for cluster detection, which is the default but improves clarity for downstream pipelines.

### Generate JSON output for downstream analysis
**Args:** `--input-fasta genome.fasta --json`
**Explanation:** Produces machine-parseable JSON output containing cluster coordinates, types, and gene annotations for integration with other bioinformatics tools.

### Save results to a specific directory
**Args:** `--input-fasta genome.fasta --output-dir ./antismash_results`
**Explanation:** Writes all output files to the specified directory instead of creating a new directory named after the input file.

### Enable specific analysis modules
**Args:** `--input-fasta genome.fasta --enable clusterblast,smcog-trees`
**Explanation:** Activates optional modules: ClusterBlast for cluster homology comparison and smcog-trees for phylogenetic analysis of similar gene families.

### Enable all detection options with relaxed thresholds
**Args:** `--input-fasta genome.fasta --enable all --loose`
**Explanation:** Runs all available detection modules and uses relaxed stringency thresholds to maximize cluster candidate identification at potential false positive cost.