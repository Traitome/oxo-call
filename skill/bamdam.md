---
name: bamdam
category: Bioinformatics Sequencing
description: A command-line tool for analyzing nucleotide damage patterns in BAM/SAM alignment files, primarily used for ancient DNA damage profiling and quality assessment.
tags: [bam, sam, ancient dna, damage, sequencing, alignment, bioinformatics]
author: AI-generated
source_url: https://github.com/example/bamdam
---

## Concepts

- **BAM/SAM I/O**: bamdam operates on sorted BAM files (or uncompressed SAM), requiring a coordinate-sorted index (.bai) for rapid random access. Input files must be indexed using samtools index prior to analysis.
- **Damage Model**: The tool calculates per-position mismatch frequencies relative to the reference, specifically targeting C>T and G>A transitions characteristic of ancient DNA deamination. It computes damage rates at read 5' and 3' ends separately.
- **Window-based Statistics**: Damage is reported within configurable read-end windows (default: first 10bp and last 10bp of each read). These windows capture the elevated misincorporation patterns typical of hydrolytic deamination in fragmented aDNA.
- **Output Formats**: Results are emitted as tab-separated values with headers, enabling downstream integration. Additionally, bamdam can generate JSON for programmatic consumption and text summaries for quick inspection.

## Pitfalls

- **Using unsorted or unindexed BAM files**: Running bamdam on files lacking proper coordinate sorting or missing the .bai index causes the tool to fail with cryptic I/O errors, wasting analysis time.
- **Specifying excessively large window sizes**: Setting read-end windows that exceed fragment lengths eliminates the damage signal by averaging damaged bases with undamaged interior positions, yielding misleading results.
- **Confusing stranded damage patterns**: Ancient DNA shows complementary damage on opposite strands (C>T on forward reads, G>A on reverse reads). Ignoring strand orientation in downstream interpretation leads to underestimation of true damage levels.
- **Processing modern DNA without adjusting parameters**: Applying default ancient DNA parameter settings to freshly prepared libraries produces artificially inflated damage estimates, causing unnecessary concern about sample quality.
- **Neglecting read filtering**: Running on unfiltered BAM files includes PCR duplicates and mapping artifacts, artificially inflating damage frequencies and producing noisy output.

## Examples

### Calculate damage profiles from an ancient DNA BAM file
**Args:** -i ancient_sample.bam --sample "Sample_A14" --outfmt tsv
**Explanation:** This reads the indexed BAM file, computes C>T and G>A damage in the terminal 10bp windows, and writes tabular output with the sample identifier column included.

### Adjust damage window size to 5bp for highly fragmented samples
**Args:** -i ancient_sample.bam --five-prime 5 --three-prime 5
**Explanation:** Setting smaller 5bp windows accounts for ultra-short fragments common in highly degraded ancient DNA, ensuring only genuinely damaged positions are counted.

### Output damage statistics in JSON format for scripting
**Args:** -i ancient_sample.bam --outfmt json --out damage_profile.json
**Explanation:** JSON output enables automated parsing in pipelines, allowing integration with R or Python scripts for downstream statistical analysis and visualization.

### Filter reads by mapping quality before damage estimation
**Args:** -i ancient_sample.bam --min-mapq 30
**Explanation:** Requiring minimum mapping quality of 30 excludes ambiguously aligned reads that could misrepresent damage patterns as mismatches, improving result accuracy.

### Process only reads on the reverse strand for strand-specific damage analysis
**Args:** -i ancient_sample.bam --strand reverse
**Explanation:** Isolating reverse-strand reads focuses analysis on G>A misincorporations, enabling strand-specific damage quantification critical for validating aDNA authenticity.

### Generate a plain-text summary report for quick inspection
**Args:** -i ancient_sample.bam --outfmt summary --out damage_summary.txt
**Explanation:** Summary format prints per-read-group damage percentages and read length distributions, useful for rapid quality assessment without opening spreadsheet files.

### Specify custom reference sequence names to analyze specific chromosomes
**Args:** -i ancient_sample.bam --regions chr1,chr2,chrM
**Explanation:** Restricting analysis to selected reference sequences reduces runtime and focuses damage estimation on mitochondria or specific autosomes of interest.