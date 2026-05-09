---
name: CADD Threshold Applicator
category: variant-annotation
description: A threshold-filtering tool for CADD (Combined Annotation Dependent Depletion) scored variant files. Takes a VCF annotated with CADD scores and outputs variants meeting or exceeding a specified PHRED-scaled score cutoff. Useful for clinical variant prioritisation and research-grade variant filtering pipelines.
tags:
  - cadscore
  - vcf-filtering
  - variant-scoring
  - genomic-filtering
  - clinical-variants
  - phred-score
  - bioinformatics
  - snv-calling
author: AI-generated
source_url: https://github.com/kircherlab/CADD-toolkit
---

## Concepts

- The tool reads a VCF file whose INFO field contains a `CADD` (or `CADD_Phred`) annotation added by CADD or a compatible annotator. Each variant's score must be expressed as a PHRED-scaled float value (e.g., `CADD=25.3`). Input VCFs missing this annotation will produce zero or all-pass results with no error message.
- The primary filtering argument is a numeric threshold value (e.g., `--threshold 15`). Any variant whose CADD score is **greater than or equal to** the threshold is retained in the output VCF. Variants below the threshold are silently excluded from the output.
- The tool respects the genome build of the input VCF (typically GRCh37 or GRCh38). A mismatched or omitted genome build annotation may cause silently incorrect scores or zero output if the tool validates chromosome names against a reference contig list.
- Output can be written as a filtered VCF file (`-o / --output`), as a count-only report (`--count`), or as a list of genomic positions (`--BED` mode). Each output format serves a different downstream use case in bioinformatics pipelines.
- When annotating pre-existing CADD scores from an already-annotated VCF (without re-running the full CADD model), the tool operates purely as a passthrough filter, making it extremely fast compared to full CADD annotation runs.

## Pitfalls

- Providing a VCF that has **not** been annotated with CADD scores in the INFO field will result in an **empty output file** with no warning or error raised. Always verify that `CADD=` or `CADD_Phred=` tags are present in the INFO column of the input VCF before running the threshold tool.
- Using the **wrong genome build** (e.g., passing `--build hg38` on a GRCh37-annotated file) causes the tool to either throw an error or silently produce incorrect filtering results because chromosome names and contig lengths differ between builds.
- Setting a **threshold value that is too low** (e.g., `0` or negative) will output **all variants** in the input VCF, defeating the purpose of filtering and potentially bloating downstream analysis files. Typical research thresholds range from 15 to 30; clinical filtering often requires ≥ 20.
- Forgetting to specify the **output path** (`-o / --output`) will cause output to be written to stdout or to a default path, which may overwrite existing files or produce results in an unexpected location depending on the tool version.
- Mixing coordinate-sorted and unsorted input VCFs without the `--sort` flag causes **output ordering issues** that break downstream tools expecting sorted VCF, such as bedtools intersect or GATK tools that rely on coordinate ordering.

## Examples

### Filter variants with CADD PHRED score ≥ 20
**Args:** `-i input.vcf --threshold 20 -o filtered.vcf`
**Explanation:** This selects all variants whose CADD PHRED score is ≥ 20 and writes them to a new VCF file, which is the standard approach for prioritising likely deleterious variants in research studies.

### Output only the count of passing variants
**Args:** `-i input.vcf --threshold 25 --count`
**Explanation:** Running with `--count` skips VCF output and instead prints a single integer representing the number of variants exceeding the threshold, which is useful for quick quality-control checks in a pipeline.

### Export passing variants as a BED-style genomic position list
**Args:** `-i input.vcf --threshold 28 --BED passing_positions.bed`
**Explanation:** The `--BED` output mode converts each passing variant into a genomic interval (chr:start-end) suitable for direct use with bedtools, UCSC Genome Browser track hubs, or other interval-based bioinformatics tools.

### Apply a lenient threshold for exploratory analysis
**Args:** `-i input.vcf --threshold 10 -o lenient.vcf`
**Explanation:** Using a low threshold of 10 captures more moderate-effect variants for exploratory analysis, which is appropriate in discovery phases before subsequent functional validation narrows the candidate set.

### Filter using GRCh38 build specification explicitly
**Args:** `-i input_hg38.vcf --build hg38 --threshold 22 -o hg38_filtered.vcf`
**Explanation:** Explicitly specifying `--build hg38` ensures chromosome name validation and contig checking are performed against the correct reference, preventing silent failures when the input VCF header does not declare the genome build clearly.

### Pipe filtered output directly into another bioinformatics tool
**Args:** `-i input.vcf --threshold 20 | bcftools view -`
**Explanation:** Because the tool writes to stdout when `-o` is omitted, its output can be piped directly into tools like `bcftools`, `bedtools`, or custom scripts, enabling compact one-liner workflows without intermediate file creation.