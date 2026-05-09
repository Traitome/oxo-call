---
name: bufet
category: alignment-filtering
description: A reference-based read alignment filtering and validation tool for BAM/CRAM files. bufet analyzes alignment properties against a reference genome to flag or remove misaligned, clipped, or discordant reads. Intended as a preprocessing step before variant calling or downstream genomic analysis.
tags: [alignment, bam, filtering, quality-control, preprocessing]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/bufet
---

## Concepts

- **Alignment validation model**: bufet scores each read alignment by comparing its aligned bases against the reference genome stored in a pre-indexed reference file. Reads whose alignment score falls below the minimum threshold are flagged in the `BZ` auxiliary tag with a numeric code, or removed entirely from the output if `--drop-low-score` is specified. A companion index file (created by `bufet-build`) is required for all reference-aware operations.
- **Input and output formats**: bufet accepts SAM, BAM, and CRAM inputs via `--in`. It writes BAM output via `--out` by default, or streams SAM to stdout if no output path is given. Output is always coordinate-sorted, with an optional auxiliary tag payload that preserves original `NM`, `MD`, and `MC` tags unless `--strip-aux` is used. Streaming from stdin is supported with `--in -`.
- **Paired-end and supplementary alignment handling**: for paired-end data, bufet computes insert size statistics from the first `--sample-reads` reads and applies dynamic thresholds. Supplementary alignments are evaluated independently; a read whose primary alignment is retained but its supplementary alignment scores below `--min-mapq` will have the supplementary alignment soft-filtered (tagged `BZ:f:i:1`) rather than removed, unless `--remove-supaln` is specified.
- **Threading and memory**: `--num-threads` spawns multiple decompression threads for gzip/CRAM input and parallelizes alignment scoring. Memory usage scales roughly linearly with `--max-reference-length` and the size of the active coordinate-sorted interval; for genomes larger than 4 Gbp, increase the JVM heap with `JAVA_OPTS="-Xmx16g"`.

## Pitfalls

- **Reference genome not indexed with `bufet-build`**: running bufet against a raw FASTA reference without first running `bufet-build ref.fa ref.bfi` will terminate with an error `No such file (bufet-index not found)`. The `.bfi` index file must be in the same directory as the reference and must not be renamed.
- **Using `--out` with a CRAM reference mismatch**: if the input CRAM uses a different reference UID than the one specified via `--ref`, bufet will either crash or silently pass all reads with `BZ:f:i:3` (reference mismatch), producing an output file that appears valid but contains no scored alignments. Always verify `##reference` lines in the CRAM header match the `--ref` path.
- **Dropping reads without post-filtering QC**: the `--drop-low-score` flag permanently removes reads from the output BAM. In mixed ploidy regions or highly repetitive loci, this can systematically eliminate reads from one haplotype and introduce allele dropout bias into downstream heterozygosity estimates. Review the `--report` JSON before applying permanent drops on diploid callsets.
- **Inconsistent `--min-mapq` across library prep types**: amplicon-based libraries with 150 bp inserts may have legitimately high mapping quality but elevated softclip counts, and setting `--min-mapq 60` combined with `--max-softclips 5` will over-filter these libraries relative to sheared Illumina WGS data, leading to coverage gaps in targeted regions.
- **Pipe to `samtools sort` without coordinate pre-sorting**: bufet outputs are coordinate-sorted by default, but redirecting to `samtools sort -` via a pipe discards the sort order if the downstream tool does not expect it. Always verify the output sort order matches the requirements of downstream tools (e.g., GATK requires coordinate-sorted input).

## Examples

### Build a reference index for alignment scoring
**Args:** build GRCh38.fa GRCh38.bfi
**Explanation:** `bufet-build` creates the `.bfi` binary reference index file required by bufet for all alignment scoring operations; this step must be completed before any filtering run.

### Filter a BAM to remove low-mapping-quality reads
**Args:** --ref GRCh38.fa --in sample.bam --out sample.filtered.bam --min-mapq 30 --drop-low-score
**Explanation:** Reads with a mapping quality below 30 are permanently removed from the output BAM, producing a reduced dataset containing only reliably placed alignments.

### Identify misaligned reads and write a report without altering the input
**Args:** --ref GRCh38.fa --in sample.bam --out /dev/null --report sample.qc.json --min-aln-score 0.6
**Explanation:** Because `--out /dev/null` is used, no reads are dropped; instead, a JSON quality-control report is written listing every read whose alignment score falls below 0.6 for downstream manual review.

### Stream a CRAM file through a gzip pipe
**Args:** --ref GRCh38.fa --in sample.cram --num-threads 4 | gzip - > sample.filtered.sam.gz
**Explanation:** Streaming to stdout allows the filtered SAM output to be piped directly into compression; `num-threads 4` parallelizes CRAM decompression across four threads for higher throughput on multi-core machines.

### Parallel-filter a large BAM with 8 threads
**Args:** JAVA_OPTS="-Xmx16g" --ref GRCh38.fa --in large_sample.bam --out large_sample.filtered.bam --num-threads 8 --min-mapq 20
**Explanation:** Increasing the JVM heap to 16 GB and spawning 8 worker threads enables efficient parallel processing of a large BAM file without running out of memory while maintaining a relaxed mapping-quality threshold of 20.

### Remove supplementary alignments while keeping primary hits
**Args:** --ref GRCh38.fa --in sample.bam --out sample.nosup.bam --remove-supaln --strip-aux
**Explanation:** The `--remove-supaln` flag deletes all supplementary alignments from the output, and `--strip-aux` removes alignment auxiliary tags to produce a clean, minimal BAM suitable for tools that do not handle split-read alignments.

### Filter by softclip tolerance for Illumina sheared libraries
**Args:** --ref GRCh38.fa --in sample.bam --out sample.clipped.bam --max-softclips 10 --min-aln-score 0.7
**Explanation:** Illumina sheared libraries with short insert sizes frequently produce softclips at insert boundaries; allowing up to 10 softclipped bases while requiring a 0.7 alignment score removes only truly misaligned reads without over-filtering.