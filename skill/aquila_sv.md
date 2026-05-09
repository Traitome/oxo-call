---
name: aquila_sv
category: Structural Variant Detection
description: A long-read structural variant detection tool that identifies deletions, insertions, duplications, inversions, and translocations from Oxford Nanopore or PacBio sequencing data using split-read and read-depth signals.
tags: [sv, structural-variants, long-reads, nanopore, pacbio, variant-calling, genomics]
author: AI-generated
source_url: https://github.com/bioinfo aquila_sv
---

## Concepts

- Aquila SV processes aligned long-read BAM/CRAM files against a reference genome to detect structural variants using both split-read alignment signals and read-depth coverage changes across the genome.
- The tool requires an index built with `aquila_sv-build` on the reference genome before variant calling; without this index, alignment-based detection will fail or produce no results.
- Output formats include VCF for standard variant exchange, BED for genomic intervals, and JSON for detailed read-level evidence—VCF is the preferred format for downstream analysis pipelines.
- Minimum read support thresholds control sensitivity: lower values (1-2 reads) increase recall but add false positives, while higher values (5+ reads) improve precision at the cost of missing smaller variants.

## Pitfalls

- Running Aquila SV without first creating a reference index causes immediate failure with an index-related error, as the tool relies on pre-computed lookup structures for efficient read alignment processing.
- Setting the minimum support threshold too low (1 read) produces high false-positive rates because random alignment artifacts can be misinterpreted as true structural variants, requiring extensive downstream filtering.
- Using outdated read aligners that produce non-standard BAM formatting (older SAMtools versions) causes parsing failures or silent data loss during read extraction, resulting in incomplete variant calls.
- Specifying an output filename without the correct extension for the chosen format silently defaults to VCF output, potentially overwriting existing files without warning if the filename matches.
- Insufficient memory allocation causes the tool to terminate without processing chromosomes with high read depth, leaving them absent from output despite the tool reporting successful completion.

## Examples

### Detect structural variants from a Nanopore BAM file
**Args:** -r hg38.fa -b ont_reads.bam -o output.vcf -t 8
**Explanation:** This runs SV detection on Nanopore reads aligned to the hg38 reference, outputing results to a VCF file using 8 threads for parallel processing.

### Run with relaxed support threshold for high-sensitivity discovery
**Args:** -r hg38.fa -b pacbio.bam -o discovery.vcf -m 2 -t 16
**Explanation:** Lowering the minimum read support to 2 reads increases sensitivity for detecting rare variants, appropriate for exploratory analysis.

### Generate JSON output with detailed read evidence
**Args:** -r hg38.fa -b ont_reads.bam -o detailed_calls.json -f json --min-af 0.1
**Explanation:** Using JSON format with a 10% allele frequency threshold captures allelic fraction information helpful for mosaic or subclonal variant analysis.

### Process a specific genomic region only
**Args:** -r hg38.fa -b pacbio.bam -o regional_calls.vcf -m 3 -L chr2:100000000-150000000 -t 4
**Explanation:** Restricting analysis to chromosome 2 between 100-150Mb dramatically reduces runtime while focusing on a specific region of interest.

### Run with strict filtering for high-confidence variant calls
**Args:** -r hg38.fa -b ont_reads.bam -o high_confidence.vcf -m 5 -f vcf --min-qual 20
**Explanation:** Requiring at least 5 supporting reads and a quality score of 20 filters out low-confidence calls, suitable for downstream clinical or validation workflows.

### Save output in BED format for genomic interval analysis
**Args:** -r hg38.fa -b pacbio.bam -o sv_intervals.bed -f bed -m 3 -t 8
**Explanation:** Using BED format outputs genomic intervals only without genotype information, useful for overlap analysis with other genomic features.

### Build reference index before running variant detection
**Args:** hg38.fa hg38_index
**Explanation:** Creating the reference index is a prerequisite step that pre-processes the genome for efficient read alignment during SV detection.