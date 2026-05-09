---
name: arcs
category: Assembly / Read Filtering
description: A bioinformatics tool for filtering and processing long-read data during genome assembly. It performs quality-based read selection, length filtering, and coverage-aware subsampling to improve assembly quality.
tags: [assembly, long-reads, nanopore, pacbio, filtering, quality-control]
author: AI-generated
source_url: https://github.com/(repository)/arcs
---

## Concepts

- **Input format**: Accepts FASTA or FASTQ format sequences, typically long reads from Oxford Nanopore or PacBio instruments. The tool reads from stdin or files specified with positional arguments.
- **Filtering criteria**: Reads are filtered based on minimum/maximum length thresholds, estimated accuracy (based on q-score), and read coverage depth when a reference is provided. Default minimum length is 500bp for typical workflows.
- **Output behavior**: Filtered reads are written to stdout in the same format as input (FASTA/FASTQ). Reads that fail filters are optionally logged to stderr for review. The tool preserves read headers and quality strings unchanged.
- **Mode of operation**: Acts as a stream filter—reads are processed sequentially, making it memory-efficient for large datasets. Supports both exclusion (-e) and inclusion (-i) filtering modes for flexible sample selection.

## Pitfalls

- **Setting minimum length too low**: Using a minimum length below 500bp can include degraded reads that harm assembly quality, leading to fragmented or erroneous contigs in downstream steps.
- **Ignoring read quality metrics**: Failing to set quality score thresholds (-q) when working with error-prone long reads can pass low-fidelity reads that create false overlaps, increasing assembly error rate.
- **Filter conflicts causing empty output**: Combining contradictory filters (e.g., minimum length 1000 and maximum length 500, or exclusion and inclusion for the same reads) results in zero reads output without explicit warning.
- **Not accounting for coverage skew**: Over-filtering high-coverage regions while preserving low-coverage areas can create bias in assembly graphs, causing collapse of repeat-rich regions.

## Examples

### Filter long reads longer than 1kb
**Args:** `-m 1000 -o filtered.fq reads.fq`
**Explanation:** This retains only reads at least 1000bp, removing short fragments common in degraded DNA samples that reduce assembly continuity.

### Keep only high-confidence Nanopore reads (q-score ≥ 12)
**Args:** `-q 12 -i highq.fq input.fq`
**Explanation:** Quality-score filtering removes reads with many errors, improving overlap detection accuracy in assembly.

### Exclude specific contaminant sequences by name
**Args:** `-e contaminants.fq -o clean.fq raw_reads.fq`
**Explanation:** Exclusion mode removes known contaminant reads (adapter sequences, PhiX spike-ins) before assembly begins.

### Subsample to approximately 30x coverage
**Args:** `-c 30 --genome-size 5g -o subsampled.fq input.fq`
**Explanation:** Coverage-aware subsampling reduces computational burden while maintaining sufficient depth for reliable assembly.

### Filter out very long reads (>50kb) to remove potential chimeras
**Args:** `-M 50000 -o no_chimeras.fq reads.fq`
**Explanation:** Extremely long reads often represent chimeric molecules; removing them prevents false assembly joins.

### Combine length and quality filters for strict selection
**Args:** `-m 1000 -q 15 -o strict.fq input.fq`
**Explanation:** Stringent dual filtering produces a high-confidence but smaller dataset that yields more accurate assemblies at the cost of coverage.