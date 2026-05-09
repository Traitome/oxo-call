---
name: bamrescue
category: BAM/SAM Manipulation
description: Recover aligned and unaligned reads from damaged or truncated BAM files by scanning raw binary data and reconstructing BAM records.
tags:
  - bam
  - recovery
  - corrupted-files
  - damaged-bam
  - data-rescue
  - alignment-recovery
author: AI-generated
source_url: https://github.com/seqhack/bamrescue
---

## Concepts

- `bamrescue` reads BAM files byte-by-byte and reconstructs valid BAM records even when the central BGZF block or index is partially corrupted, making it useful for extracting data from sequencing runs with I/O errors or incomplete file transfers.
- The tool outputs recovered reads as SAM format to stdout by default, preserving the original quality strings, CIGAR strings, and optional fields (NH, HI, MD, NM) extracted from the binary data.
- It supports scanning only specific reference sequences using `--ref` or `--region` flags, which limits recovery to reads mapped to genomic intervals and significantly reduces processing time for large files.
- Recovery is based on detecting valid BAM magic bytes (`BAM\1`), checking the block size field, and validating the LZFSS compression descriptor, which allows it to skip over corrupted blocks rather than failing entirely.
- The tool preserves read names, mapping qualities, and flags exactly as stored in the binary BAM record, but does not reconstruct the BAM header or reference sequences themselves.

## Pitfalls

- Running `bamrescue` on a healthy BAM file does not guarantee identical output to `samtools view`, because the tool uses a heuristic block-skipping algorithm that may miss or duplicate records at block boundaries, leading to unexpected read count discrepancies.
- Outputting to SAM format loses BGZF compression and the original BAM binary structure, so downstream tools expecting a well-formed BAM with a valid header will require an additional `samtools reheader` or `samtools view -b` conversion step.
- If the BAM file has a corrupted magic header (`BAM\1`), `bamrescue` will exit with code 255 and produce no output, because the recovery logic depends on finding at least one valid block to anchor the scanning process.
- Specifying `--threads` with values exceeding the number of available CPU cores can cause memory thrashing and actually slow down recovery on I/O-bound workloads, particularly when scanning network-mounted files.
- The `--min-quality` threshold filters reads using the base quality value in the binary record, but this field may be zero-padded or corrupted in damaged files, causing valid low-quality reads to be incorrectly discarded.

## Examples

### Recover all readable reads from a truncated BAM file
**Args:** `--input truncated_run.bam --output recovered.sam`
**Explanation:** This scans the entire file and outputs every readable BAM record as SAM to the specified file, including reads that were successfully mapped before the truncation point.

### Limit recovery to a single reference sequence by numeric ID
**Args:** `--input sample.bam --ref 0 --output ref0_recovered.sam`
**Explanation:** Restricting recovery to reference ID 0 targets the first sequence in the BAM header, which is useful when only one chromosome was affected by corruption.

### Recover reads from a specific genomic interval
**Args:** `--input sample.bam --region chr1:1000000-2000000 --output interval_recovered.sam`
**Explanation:** Specifying a genomic range ensures only reads with mapping positions falling within those coordinates are recovered, reducing output size for targeted rescue operations.

### Enable verbose logging to diagnose recovery quality
**Args:** `--input damaged.bam --log-level DEBUG --output rescued.sam`
**Explanation:** Setting DEBUG log level outputs per-block status messages, allowing you to identify exactly which regions of the file were skipped due to corruption and assess recovery completeness.

### Use multiple threads to accelerate recovery of large files
**Args:** `--input large_truncated.bam --threads 8 --output recovered.sam`
**Explanation:** Allocating 8 threads enables parallel block scanning and reconstruction, which significantly reduces wall-clock time when processing files larger than 50 GB on systems with sufficient I/O bandwidth.

### Skip reads with average base quality below a threshold
**Args:** `--input sample.bam --min-quality 20 --output high_quality.sam`
**Explanation:** Filtering by minimum quality removes reads where the average base quality value is below 20, ensuring downstream analyses do not include low-confidence sequences that may have been corrupted in the damaged regions.