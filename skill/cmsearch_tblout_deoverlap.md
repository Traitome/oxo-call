---
name: cmsearch_tblout_deoverlap
category: RNA Structure Analysis / Hit Filtering
description: Filters overlapping hits from Infernal cmsearch tabular output, retaining the highest-scoring hit among overlapping entries using bit score or E-value comparison.
tags:
  - infernal
  - cmsearch
  - RNA homology search
  - overlapping hits removal
  - covariance model
  - tabular output
  - RNA annotation
author: AI-generated
source_url: https://github.com/EddyRivasLab/infernal
---

## Concepts

- **Input format**: Reads standard cmsearch `--tblout` tabular output (typically piped via stdin), which contains per-hit records with columns: target name, accession, query name, accession, E-value, bit score, gc-bound bias, sequence-from, sequence-to, strand, truncated flags, model-from, model-to, and score. Any valid cmsearch tabular file is acceptable input.
- **Overlap resolution**: Two hits overlap if they share the same target sequence and their sequence-coordinate ranges intersect. When overlaps are detected, the tool retains the hit with the better (lower) E-value by default, or optionally the better (higher) bit score, and discards the others. A containment relationship (one hit fully inside another) is treated as overlap under the `--def` (default) mode.
- **De-overlap modes**: `--any` treats any coordinate intersection as overlap, removing the lower-scoring hit; `--def` treats only full containment as overlap, keeping the larger hit and removing the inner one when a hit is fully inside another; `--beta` selects for the best E-value, preferring hits with the best overall score.
- **Output**: Writes a de-overlapped cmsearch tabular file to stdout, which is fully compatible with downstream tools expecting standard cmsearch tabular format. Use shell redirection (`>`) or `--oc` to save output to a file.
- **Score comparison tie-breaking**: When two hits have equal E-values or bit scores, the tie-breaking behavior depends on the selected `--beta` flag. Without `--beta`, E-value is the primary criterion; with `--beta`, the raw bit score takes precedence.

## Pitfalls

- **Using `--any` instead of `--def`**: The `--any` mode is more aggressive and may discard more hits than intended, because any base of overlap (even a single nucleotide) triggers removal of the lower-scoring hit. This can lead to legitimate single-exon annotations being lost if flanking UTR or intron sequences slightly overlap neighboring gene models.
- **Confusing strand handling**: By default the tool does not distinguish between hits on the forward (+) and reverse complement (−) strands within the same sequence coordinate range. If your input contains antisense predictions that physically overlap forward-strand hits, the de-overlap logic may unexpectedly remove one strand's predictions, resulting in missing annotation of antisense regulatory elements.
- **Mismatched `--beta` flag**: When `--beta` is set, the tool switches score comparison from E-value to bit score as the primary ranking metric. If you expect E-value-based filtering but have silently enabled `--beta` via a script or alias, the resulting hit ranking will differ, potentially causing low-E-value but high-score hits to be retained over high-E-value but low-score ones.
- **Alpha threshold too strict**: Setting a high `--alpha` value removes any hit with E-value worse than the threshold before overlap resolution even occurs. This means that in dense genomic regions with many weakly significant hits, you may discard all overlapping entries even if one of them would have been the top-scoring survivor after de-overlapping.
- **Reading from stdin without verifying input completeness**: If you pipe cmsearch output into `cmsearch_tblout_deoverlap` and the pipe breaks or cmsearch errors out mid-stream, `cmsearch_tblout_deoverlap` may read a partial/incomplete tabular file and silently produce an output file that appears valid but is truncated, leading to downstream gene annotation gaps.
- **File access mode with named pipes**: `cmsearch_tblout_deoverlap` may not support seeking on named pipes (FIFOs). Attempting `cmsearch --tblout ... | cmsearch_tblout_deoverlap - > out.tblout` can fail if the tool internally calls `fseek`. Always verify that your version supports streaming input or write the raw tabular output to disk first.

## Examples

### Remove overlapping cmsearch hits using default containment-based de-overlap
**Args:** `-E 0.1 --oc deoverlapped.tblout seqfile.tblout`
**Explanation:** The `-E 0.1` flag removes all hits with E-value greater than 0.1 before applying the default `--def` (containment) de-overlap mode, keeping the best-scoring hit among fully-contained overlapping sets.

### Aggressively remove all hits with any coordinate overlap
**Args:** `--any -o strict_deoverlapped.tblout raw.tblout`
**Explanation:** The `--any` flag treats any sequence-coordinate intersection as overlap, removing the lower-E-value hit whenever any nucleotide is shared, producing a strictly non-redundant annotation set.

### De-overlap by bit score ranking using --beta
**Args:** `--beta --oc bitrank_deoverlapped.tblout raw.tblout`
**Explanation:** The `--beta` flag switches the comparison metric from E-value to raw bit score, so the highest-scoring hit by bit score is retained among overlapping groups, which is useful for ranking structural homology candidates.

### Filter by E-value threshold and de-overlap with containment rules
**Args:** `--alpha 0.01 --oc filtered.tblout raw.tblout`
**Explanation:** The `--alpha 0.01` flag discards all hits with E-value worse than 0.01 prior to overlap resolution, then applies the default `--def` mode to remove hits fully contained within higher-scoring ones.

### Suppress header comments from de-overlapped output
**Args:** `--no-comment -o clean_deoverlapped.tblout raw.tblout`
**Explanation:** The `--no-comment` flag strips any comment or track lines from the output so that the de-overlapped tabular file contains only the raw hit records without metadata headers, making it compatible with downstream parsers.

### Save the filtered-out hit records for manual review
**Args:** `--sf discarded_hits.tblout --oc survivors.tblout raw.tblout`
**Explanation:** The `--sf discarded_hits.tblout` flag saves all hits removed during de-overlapping to a separate file for manual curation, while `--oc survivors.tblout` writes the retained hits to the output file.