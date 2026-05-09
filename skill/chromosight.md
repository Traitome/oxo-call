---
name: chromosight
category: chromatin-analysis
description: A tool for detecting chromatin patterns (loops, stripes, borders, fires) in Hi-C interaction matrices using kernel-based pattern detection.
tags: [hi-c, chromatin-contact-maps, loop-detection, stripe-detection, chromatin-organization, 3d-genome]
author: AI-generated
source_url: https://github.com/berasaurus/chromosight
---

## Concepts

- Chromosight detects chromatin patterns in Hi-C matrices stored in `.cool` or `.mcool` format, which are normalized sparse matrix formats that store pairwise contact frequencies between genomic loci.
- The tool uses precomputed or custom kernels to scan Hi-C matrices, computing a pearson correlation-like score at each genomic position to identify pattern instances like loops (enrichment at corner), stripes (enrichment along diagonal extensions), borders (vertical/horizontal enrichment), and fires (focal peaks).
- Output is provided as genomic coordinates (chromosome, start, end) with pattern type and confidence scores, which can be directly visualized in Hi-C heatmaps or used for downstream analysis.
- Patterns can be detected genome-wide or restricted to specific chromosomes or regions using genomic intervals, and results can be filtered by statistical significance (p-value/FDR) and minimum score thresholds.

## Pitfalls

- Using unnormalized (raw) Hi-C contact matrices instead of normalized `.cool` files will produce false positive pattern calls, as raw counts include coverage biases that confound the kernel-based detection algorithm.
- Setting the `--kernel-size` too small relative to the expected pattern size will miss large-scale patterns like wide stripes, while setting it too large will create spurious detections from nearby chromatin contacts blending together.
- Failing to adjust `--threads` for multi-threaded execution on large Hi-C datasets results in unnecessary waiting; however, setting too many threads can cause memory exhaustion when loading high-resolution matrices like 1kb resolution genome-wide data.
- Not filtering results by minimum score (`--min-score`) and significance thresholds can leave hundreds of low-confidence detections that represent background noise rather than true biological patterns, complicating downstream interpretation.
- Attempting to detect patterns on low-coverage Hi-C datasets (fewer than ~50 million reads) without adjusting the `--pearson-threshold` results in unreliable pattern calls with high false discovery rates.

## Examples

### Detect chromatin loops genome-wide from a Hi-CCool file
**Args:** `--patterns loops --input my_data.cool --output loops_detection --min-score 0.5 --pearson-threshold 0.3`
**Explanation:** This runs loop detection using default kernels on a normalized Cooler file, outputting genomic loop coordinates with scores above 0.5 and Pearson correlation above 0.3.

### Detect multiple pattern types including stripes and borders
**Args:** `--patterns loops stripes borders --input HiC_sample.mcool --output multi_patterns --min-score 0.6 --threads 8`
**Explanation:** This detects loops, stripes, and borders simultaneously using 8 threads for faster processing, requiring a minimum score of 0.6 for each detected pattern.

### Run detection on a specific chromosome region
**Args:** `--patterns loops --input sample.cool --output chr19_loops --region chr19:0-60000000 --min-score 0.45`
**Explanation:** This restricts loop detection to chromosome 19 between positions 0 and 60 Mb, reducing computational time and focusing analysis on a specific genomic region.

### Adjust detection sensitivity for high-resolution data
**Args:** `--patterns loops --input highres_1kb.cool --output highres_loops --min-score 0.7 --kernel-size 20000 --pearson-threshold 0.5`
**Explanation:** This uses a larger kernel size (20kb) suitable for high-resolution data to detect larger loops, with stricter thresholds to filter noise inherent in high-resolution matrices.

### Save detection results with detailed statistics
**Args:** `--patterns loops --input experiment.cool --output detailed_loops --min-score 0.4 --save-all-stats --output-format tsv`
**Explanation:** This outputs all detection statistics including p-values and correlation scores in TSV format for downstream filtering and analysis in other software.