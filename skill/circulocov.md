---
name: circulocov
category: Coverage Analysis
description: A bioinformatics tool for calculating and visualizing coverage depth across circular DNA/RNA sequences. Accepts aligned BAM/CRAM files and reference sequences, outputs coverage statistics, per-base depth, and uniformity metrics tailored for circular genomes such as viruses or plasmids.
tags: ['coverage', 'circular-genome', ' BAM', 'depth-analysis', 'virology', 'plasmid']
author: AI-generated
source_url: https://github.com/circulocov/circulocov
---

## Concepts

- **Circular genome handling**: circulocov treats the reference as a circular topology, meaning base position N+1 wraps to position 1 without edge artifacts. This is essential for viral genomes (e.g., Hepatitis Delta Virus, Ring DNA viruses) where coverage at the termini must be computed as continuous rather than fragmented.
- **Input formats**: The tool accepts aligned sequencing reads in BAM or CRAM format paired with a reference FASTA representing the circular genome. It requires an index file (.bai/.crai) for efficient random access. Unaligned reads are ignored unless explicitly specified.
- **Output modes**: Default output is a plain text coverage table with columns: position, depth, base. Optional outputs include a summary statistics file (mean, median, min, max, uniformity score) and a BEDGRAPH track for genome browser visualization. The uniformity metric is computed as the inverse coefficient of variation (1 - CV) expressed as percentage.
- **Strandedness awareness**: When the `--strand` flag is used, circulocov reports coverage separated by read orientation (+/-), distinguishing forward and reverse strand contributions. This is critical for analyzing antisense transcription or replication intermediates in circular RNA.

## Pitfalls

- **Using linear reference without circular flag**: Passing a linear reference FASTA for a known circular virus without the `--circular` flag results in under-coverage at the genome termini (first and last ~50bp), because reads spanning the junction are split and not properly realigned. This underestimates the true coverage depth by up to 15% in compact viral genomes.
- **Forgetting to index the BAM file**: Running circulocov on an unindexed BAM file causes the tool to scan the entire file sequentially, drastically increasing runtime (5-10x slower). For whole-genome viral datasets exceeding 1GB, this may lead to memory exhaustion or timeout errors.
- **Confusing read orientation with strand specificity**: The `--strand` flag reports stranded coverage, but not distinguishing forward and reverse strand reads is a common mistake when analyzing replication intermediates or viral transcripts. Relying on combined depth alone can mask strand-specific depletion patterns that indicate active replication or transcriptional bias.
- **Specifying wrong genome length**: If the reference FASTA header contains a different length than the actual assembled genome (e.g., when using a draft assembly), coverage positions beyond the actual sequence are treated as zero depth but included in statistics, skewing mean and uniformity calculations.

## Examples

### Calculate coverage depth for a circular viral genome

**Args:** `--reference ref.fasta --input alignments.bam --circular --output coverage.txt`

**Explanation:** This runs circulocov in default mode, treating the reference as circular and outputting per-base depth values to coverage.txt. The `--circular` flag ensures bases near the ends are merged seamlessly.

### Generate summary statistics without per-base file

**Args:** `--reference ref.fasta --input alignments.bam --circular --stats-only --output viral_cov_stats.txt`

**Explanation:** When only summary statistics are needed, the `--stats-only` flag skips the per-base table, producing a smaller output file containing mean, median, min, max, and uniformity score.

### Export coverage as BEDGRAPH for visualization

**Args:** `--reference ref.fasta --input alignments.bam --circular --bedgraph cov.bedgraph`

**Explanation:** Exports coverage in BEDGRAPH format compatible with UCSC Genome Browser or IGV for visualizing coverage tracks alongside annotations. The circular junction is handled, ensuring smooth visualization across the genome termini.

### Analyze strand-specific coverage

**Args:** `--reference ref.fasta --input alignments.bam --circular --strand --output stranded_cov.txt`

**Explanation:** Uses the `--strand` flag to report forward (+) and reverse (-) strand coverage separately for each position, useful for distinguishing replication intermediates from transcripts in positive-sense RNA viruses.

### Filter by minimum mapping quality

**Args:** `--reference ref.fasta --input alignments.bam --circular --min-mapq 30 --output highq_cov.txt`

**Explanation:** Applies a minimum mapping quality threshold (MAPQ ≥ 30) to exclude ambiguous or low-quality reads from the coverage calculation, reducing false depth signals from misaligned regions.