---
name: batvi
category: variant-analysis
description: A bioinformatics tool for predicting variant effects and functional impacts on genomic sequences, analyzing nucleotide changes and their biological consequences.
tags:
  - variant
  - genomics
  - effect-prediction
  - bioinformatics
  - vcf
author: AI-generated
source_url: https://github.com/batvi-project/batvi
---

## Concepts

- **Input Format**: batvi accepts variant calls in standard VCF (Variant Call Format) files, requiring at minimum CHROM, POS, ID, REF, ALT, and QUAL columns. A valid VCF header (##fileformat line) must be present for correct parsing.
- **Output Annotations**: The tool generates predictions including functional class (synonymous, missense, nonsense, frameshift), protein position changes, and conservation scores across multiple species alignments.
- **Reference Genome**: batvi requires a compatible reference genome index (created via companion tool batvi-build) to determine transcript contexts and calculate regulatory impacts accurately.
- **Batch Processing**: Multiple VCF files can be processed in a single run using the `--input-file-list` flag, with results aggregated into a unified output table.

## Pitfalls

- **Unindexed Reference**: Running batvi without first building the reference genome index causes the tool to fail with a missing index error, wasting computation time on large datasets.
- **Mismatched Genome Build**: Using transcript annotations from a different genome build (e.g., GRCh37 annotations with GRCh38 reference) produces incorrect mapping positions and erroneous effect predictions.
- **Empty or Invalid VCF**: Submitting VCF files missing required columns or with malformed records causes parsing errors that halt the analysis without partial results.
- **Memory Exhaustion**: Processing whole-genome VCF files with millions of variants without setting appropriate memory limits via `--max-memory` triggers out-of-memory failures on clusters.

## Examples

### Predict effects for a single-sample VCF file
**Args:** `--input variants.vcf --reference hg38.fa --output effects.tsv`
**Explanation:** This runs effect prediction on the variant call file using the hg38 reference genome, writing predictions to a tab-separated output file containing functional annotations for each variant.

### Annotate variants with conservation scores
**Args:** `--input variants.vcf --reference hg38.fa --conservation --output conservation.tsv`
**Explanation:** The conservation flag enables multiple sequence alignment scoring, adding PhyloP and GERP conservation metrics to the output for each variant position.

### Process multiple VCF files in batch mode
**Args:** `--input-file-list sample_list.txt --reference hg38.fa --output batch_results/`
**Explanation:** This processes all VCF files listed in the text file (one path per line) sequentially, storing individual result files in the specified output directory.

### Generate JSON-formatted predictions
**Args:** `--input variants.vcf --reference hg38.fa --output-format json --output predictions.json`
**Explanation:** Switching output format to JSON provides machine-parseable predictions ideal for integration with downstream bioinformatics pipelines.

### Filter for high-confidence missense variants only
**Args:** `--input variants.vcf --reference hg38.fa --filter "effect_type=missense && qual>=30" --output high_conf.tsv`
**Explanation:** This applies a combined filter requiring both missense effect classification and quality score above 30, reducing output to clinically relevant high-confidence variants.

### Specify non-default memory allocation
**Args:** `--input large_wgs.vcf --reference hg38.fa --max-memory 16G --output large_results.tsv`
**Explanation:** Setting memory to 16GB prevents out-of-memory failures when processing whole-genome call sets containing millions of variants.

### Use transcript annotations from custom GTF
**Args:** `--input variants.vcf --reference hg38.fa --annotation-file custom_annotations.gtf --output custom_effects.tsv`
**Explanation:** Providing a custom GTF file overrides default annotation sources, enabling analysis with project-specific or alternative transcript databases.