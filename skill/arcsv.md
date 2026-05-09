---
name: arcsv
category: variant-calling-utilities
description: A utility for filtering, selecting, and computing statistics on Variant Call Format (VCF) files. Operates on bgzip-compressed and tabix-indexed VCF/BCF inputs. Supports chromosome-level region queries, sample-level subsetting, functional impact filtering, and multi-sample cohort operations.
tags:
  - vcf
  - bcf
  - variant-filtering
  - bioinformatics
  - genomics
  - snp
  - indel
author: AI-generated
source_url: https://github.com/arcsv/arcsv
---

## Concepts

- **VCF/BCF format**: `arcsv` reads bgzip-compressed VCF (`.vcf.gz`) or BCF (`.bcf`) files. Input files must be sorted and accompanied by a Tabix index (`.vcf.gz.tbi` or `.bcf.csi`) or a CSI index (`.bcf.csi`). Unsorted or unindexed inputs cause silent failures or incomplete output; always sort with `bcftools sort` before piping into `arcsv`.
- **Chromosome-level region specification**: Regions are passed as `chrom:start-end` strings (1-based, end-inclusive). Using the tabix coordinate system, a region like `chr1:1000-2000` selects variants whose POS falls within that interval. Overlapping variants at region boundaries are included; non-overlapping records are excluded.
- **Sample and genotype filters**: `arcsv --samples` restricts output to listed sample names; `arcsv --filter` applies a logical expression to FILTER and INFO fields. Genotype-level expressions (e.g., `GT == "0/1"`) filter individual genotype calls before variant-level records are emitted, reducing output to only sites with qualifying non-reference calls.
- **Output format control**: By default `arcsv` outputs to stdout in uncompressed VCF format. The `--output-type z` flag writes bgzip-compressed VCF, and `--output-type b` writes BCF. Pipe compressed output directly to tools that accept streams (e.g., `bcftools view`) without decompression.

## Pitfalls

- **Forgetting to sort before indexing**: If the input VCF is not coordinate-sorted, the Tabix index will be silently corrupted, causing `arcsv` to skip or misplace variants when a region is specified. Always run `bcftools sort -O z input.vcf.gz > sorted.vcf.gz && bcftools index sorted.vcf.gz` before using region queries.
- **Incorrect chromosome naming conventions**: Mixing `chr1` and `1` naming styles produces zero results with no warning. Verify that the chromosome column in your query matches the sequence dictionary in the VCF header exactly before running region-based operations.
- **Overfiltering with overly restrictive genotype expressions**: Applying strict genotype filters (e.g., `GT == "1/1" && DP > 30`) on low-coverage data often returns zero records, giving the false impression that no variants exist. Check sample-level depth distributions with `arcsv --stats` first to calibrate thresholds.
- **Piping into tools that require sorted input**: Some downstream tools (e.g., `bcftools consensus`) require sorted input internally. Piping unsorted `arcsv` output into these tools causes index mismatches or silent errors. Insert an explicit sort step in the pipeline chain.
- **Omitting `--output-type b` when writing BCF for downstream tools**: Writing to the default VCF format and then renaming to `.bcf` produces an invalid file. Use `--output-type b` explicitly when BCF output is required, or use `bcftools view` to convert afterward.

## Examples

### Filter variants by QUAL threshold and functional impact
**Args:** `--input NA12878.vcf.gz --filter "QUAL > 30 && INFO/IMPACT == HIGH" --remove-filtered-all`
**Explanation:** Removing all records with `QUAL