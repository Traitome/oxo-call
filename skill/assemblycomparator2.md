---
name: assemblycomparator2
category: Genomics
description: Compare genome assemblies and compute quality metrics such as N50, contig counts, total length, and alignment-based statistics to evaluate assembly quality and similarity.
tags: [assembly, comparison, N50, genomics, quality-metrics]
author: AI-generated
source_url: https://github.com/GMOD/assemblycomparator2
---

## Concepts

- **Input Format**: AssemblyComparator2 accepts multiple FASTA files as assembly inputs. Each input file should represent a distinct assembly (e.g., from different assemblers, versions, or samples) to be compared side-by-side. Files must be valid FASTA format with `.fa`, `.fasta`, `.fna`, or `.fa.gz` extensions.
- **Metrics Computed**: The tool calculates standard assembly quality metrics including N50, L50, total assembled length, number of contigs/scaffolds, longest contig, GC content, and BUSCO scores when reference annotations are provided. These metrics enable quantitative comparison between assemblies.
- **Output Modes**: Results can be generated as tabular text reports (default), JSON for programmatic parsing, or HTML for visual inspection. The `--outfmt` flag controls output format selection.
- **Reference-Based Comparison**: When a reference genome is supplied via `--reference`, AssemblyComparator2 computes alignment-based metrics including coverage, identity percentage, and synteny conservation, allowing evaluation of assembly accuracy against a known reference.
- **Threshold Filtering**: The `--min-length` parameter filters contigs below a specified size before metric calculation, ensuring statistics reflect high-quality assembly regions and preventing small fragments from skewing N50 and other metrics.

## Pitfalls

- **Mismatched File Extensions**: Providing files with unrecognized extensions (e.g., `.txt` or `.seq`) causes the tool to fail silently without processing. Always ensure input assemblies use standard FASTA extensions or explicitly specify the format with `--format fasta`.
- **Memory Exhaustion with Large Genomes**: For large assemblies (>1 GB total sequence), insufficient RAM causes the process to terminate abnormally. Use the `--chunk-size` parameter to process assemblies in segments, or allocate more memory before running the comparison.
- **Inconsistent Reference Naming**: When using `--reference`, chromosome/contig names in the reference must match those in the query assembly exactly. Mismatched naming conventions (e.g., "chr1" vs "1") produce zero alignment coverage without raising errors, leading to misleading quality assessments.
- **Mixed File Compression Formats**: Combining compressed (`.gz`) and uncompressed input files in a single run causes inconsistent metric calculations. Process all files with the same compression status, or decompress everything before comparison.
- **Ignoring BUSCO Dependencies**: BUSCO score computation requires HMMER and NCBI datasets to be pre-downloaded. Running with `--busco` on a system without these resources produces NA values instead of scores, which can break downstream automated analysis pipelines.

## Examples

### Compare two assemblies and generate a default text report
**Args:** `assembly1.fa assembly2.fa --outfmt text`
**Explanation:** This runs a side-by-side comparison of two assemblies using default settings, outputting a human-readable text report containing N50, contig counts, total length, and other standard metrics for both assemblies.

### Compare three assemblies and output JSON for programmatic parsing
**Args:** `assembler_a.fa assembler_b.fa meta.asm.fa --outfmt json --output comparison_results.json`
**Explanation:** The JSON output format enables integration with automated pipelines and scripting workflows by providing structured data that can be easily parsed by tools like jq or Python libraries.

### Compare assemblies with reference genome for alignment-based metrics
**Args:** `draft_assembly.fa --reference ref_genome.fa --busco --outfmt html`
**Explanation:** Including a reference genome enables alignment-based quality assessment, while HTML output provides a visual report with interactive charts for manual inspection and reporting.

### Filter small contigs before comparison
**Args:** `v1_assembly.fa v2_assembly.fa --min-length 1000 --outfmt text`
**Explanation:** Setting a minimum contig length of 1000 bp excludes fragmented sequences from metric calculations, producing N50 and L50 values that better reflect high-quality assembly regions.

### Process large genome assemblies in memory-efficient chunks
**Args:** `large_asm1.fa large_asm2.fa --chunk-size 500M --outfmt text`
**Explanation:** For genomes exceeding available RAM, the `--chunk-size` parameter processes 500 MB segments sequentially, preventing memory exhaustion while still producing accurate comparative metrics across the full assemblies.