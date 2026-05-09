---
name: ale
category: Genome Assembly Quality Assessment
description: Bayesian assembly likelihood evaluator that assesses genome assembly quality using aligned read data. It computes a log-likelihood score without requiring a reference genome, making it useful for evaluating de novo assembly quality.
tags: [assembly, quality, evaluation, likelihood, bioinformatics, genome, sequencing]
author: AI-generated
source_url: https://github.com/seqan/seqan/tree/master/apps/ale
---

## Concepts

- **Input Requirements**: ALE requires two primary inputs—a genome assembly in FASTA format and aligned sequencing reads in BAM or CRAM format. The reads must be aligned to the assembly, not to a separate reference.
- **Likelihood Scoring**: The tool computes a log-likelihood score representing how well the assembly explains the observed read data. Higher positive scores indicate better concordance between reads and assembly.
- **Error Models**: ALE incorporates sequencing error models, insert size distributions, and coverage information to assess assembly quality across different regions.
- **No Reference Genome Needed**: Unlike other quality metrics, ALE evaluates assemblies without a reference genome, making it valuable for de novo assembly assessment.
- **Output Format**: Results are written to a JSON file containing the overall log-likelihood score, per-scaffold quality scores, and summary statistics.

## Pitfalls

- **Using Wrong Alignment Target**: Aligning reads to a reference genome instead of the assembly will produce meaningless scores. Reads must be aligned to the assembly being evaluated.
- **Inconsistent Read Groups**: If BAM files lack proper read group information or have inconsistent RG tags across reads, ALE may fail or produce inaccurate results.
- ** Extremely High Coverage Bias**: Regions with coverage >1000x can disproportionately influence likelihood scores and mask problems in other regions.
- ** Ignored Quality Scores**: Failing to provide quality score (QUAL) information in the BAM file reduces the accuracy of error modeling.
- **Misinterpreting Negative Scores**: Negative log-likelihood scores are not necessarily bad—they indicate the assembly has room for improvement, but very large negative values may signal major issues.

## Examples

### Basic assembly quality evaluation
**Args:** --help
**Explanation:** Displays all available command-line options and their descriptions for the ALE evaluator.

### Evaluate assembly with default settings
**Args:** assembly.fa aligned_reads.bam output.json
**Explanation:** Runs ALE with default parameters, requiring only the assembly FASTA, aligned BAM, and output JSON filename.

### Specify multiple libraries with different insert sizes
**Args:** -l 180 -l 250 -l 320 assembly.fa_lib1_lib2.bam output.json
**Explanation:** Provides three paired-end libraries with mean insert sizes of 180bp, 250bp, and 320bp for more accurate likelihood computation.

### Set custom minimum read mapping quality
**Args:** -q 20 assembly.fa aligned_reads.bam output.json
**Explanation:** Filters reads with mapping quality below 20 before computing likelihood scores, reducing noise from ambiguously mapped reads.

### Adjust K-mer length for likelihood computation
**Args:** -k 25 assembly.fa aligned_reads.bam output.json
**Explanation:** Uses 25-mers for k-mer based likelihood calculations, which can improve accuracy for larger genomes.

### Generate detailed per-scaffold output
**Args:** -v assembly.fa aligned_reads.bam output.json
**Explanation:** Produces verbose output including per-scaffold quality scores alongside the overall assembly likelihood score.