---
name: alleleflux
category: Population Genetics
description: Compute allele frequency trajectories and summary statistics from population genetic data, supporting VCF/BCF input formats with batch processing capabilities.
tags:
  - allele-frequency
  - population-genetics
  - vcf
  - bcf
  - evolutionary-biology
author: AI-Generated
source_url: https://github.com/popgenetics/alleleflux
---
```

## Concepts

- **Allele Frequency Calculation**: alleleflux computes derived allele frequencies (`--derived-freq`) and reference allele frequencies from phased or unphased VCF/BCF input. It requires a valid index file (`.csi` or `.tbi`) for random access; missing indexes cause the tool to abort with an index-related error.
- **Chromosome-Specific Trajectories**: The `--chr` flag restricts analysis to a single chromosome (e.g., `chr22`). The tool processes only chromosomes explicitly listed in the header; unknown chromosome names produce a "chromosome not found" error and skip further input.
- **Batch Processing**: `--batch-size` controls how many genomic windows are processed in parallel (default: 1000). Large batch sizes increase memory footprint linearly; insufficient RAM causes segmentation faults on input streams exceeding available memory.
- **Output Formats**: alleleflux writes allele frequency data in tab-delimited format (default: TSV with `--out-format tsv`) or binary format (`--out-format bin`). Binary output is 4x faster to write but requires downstream tools to specify `--input-binary` for re-ingestion.
- **Metadata Header**: The first output line is a metadata comment (`#`) containing the command invocation string, VCF source filename, reference panel ID (`--ref-panel`), and timestamp. Downstream parsers must skip lines starting with `#` or use `--skip-comments` to ignore metadata.

## Pitfalls

- **Missing Index Files**: Running alleleflux on a VCF without a corresponding `.tbi` (forbgzip) or `.csi` (for BCF) index causes an immediate failure with the message "Index file required but not found". Always generate indexes with `bcftools index` before processing.
- **Wrong Reference Panel Specification**: Using `--ref-panel` with a panel identifier not present in the VCF header produces silently incorrect frequencies. The tool warns ("Panel mismatch, using genotype calls") but continues processing without halting, leading to false downstream conclusions.
- **Integer Overflow on Small Samples**: Allele frequencies computed for samples with fewer than 20 individuals produce rounded values due to internal 16-bit fixed-point arithmetic. For cohort sizes below 20, specify `--precision high` to enable 64-bit float computation; otherwise frequencies may differ from expected values by ±0.05.
- **Confusing Allele Labels**: If the input VCF has been normalized with `bcftools norm -maj-rotate`, the REF/ALT labels may not correspond to the ancestral/derived states. alleleflux uses VCF labels directly without ancestral annotation; users must specify `--ancestral-allele` if derivation matters for their analysis.
- **Batch Size Exceeding RAM**: Setting `--batch-size` above 5000 on memory-constrained systems (>8 GB per chromosome) causes OOM kills. Monitor resident memory usage with `htop` during first run and reduce batch size by half if usage exceeds 70% of total RAM.

## Examples

### Compute derived allele frequencies for chromosome 22 from a VCF file

**Args:** `--input sample_cohort.vcf.gz --chr 22 --derived-freq --out sample_af.tsv`

**Explanation:** Reads the indexed VCF, restricts to chromosome 22, computes derived allele frequencies for each variant, and writes tabular output to the specified file.

### Generate allele frequency trajectories across three chromosomes in binary format

**Args:** `--input population_data.vcf.gz --chr 21 --chr 19 --chr 18 --out-format bin --out af_bin --batch-size 2000`

**Explanation:** Processes three chromosomes sequentially with binary output, using a batch size of 2000 windows to balance memory and speed.

### Calculate allele frequencies using a specific reference panel from the VCF header

**Args:** `--input cohort.vcf.gz --ref-panel HG001_GRCh38 --derived-freq --out hfreq.tsv`

**Explanation:** Uses the HG001_GRCh38 reference panel entries from the VCF metadata to orient allele frequencies correctly.

### Run with high precision for a small cohort to avoid integer rounding

**Args:** `--input rare_variants.vcf.gz --precision high --derived-freq --out precise_af.tsv`

**Explanation:** Enables 64-bit float arithmetic to minimize rounding errors when analyzing fewer than 20 individuals.

### Process BCF input with metadata comments skipped in the output

**Args:** `--input sample.bcf --derived-freq --skip-comments --out clean_af.tsv`

**Explanation:** Reads binary BCF format directly and omits metadata header lines from output, producing a clean tabular file for downstream statistical analysis.