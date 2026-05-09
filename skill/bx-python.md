---
name: bx-python
category: bioinformatics-data-formats
description: A Python library providing tools for manipulating biological sequences, alignments, and intervals, including format converters and set operations on genomic features.
tags:
  - python
  - bioinformatics
  - genomics
  - alignments
  - intervals
  - maf
  - fasta
  - fastq
  - axt
author: AI-generated
source_url: https://github.com/bxlab/bx-python
---

## Concepts

- **MAF (Multiple Alignment Format) manipulation**: bx-python reads and writes MAF files using indexed access. The `maf_io` module provides the ` MafIndex` class which allows random access by genomic coordinate, enabling efficient retrieval of alignment blocks without scanning entire files. Streams are iterated via `maf_iter` for sequential processing.

- **Interval tree operations**: The `intervals` module implements interval trees for fast overlap queries. When adding genomic intervals to an `IntervalTree`, queries using `find()` return all features overlapping a given range in O(log n + k) time where k is the output size. This is essential for intersectingBED/GFF/MAF features with query regions.

- **Array and cluster operations**: The `arrays` module supports dense and sparse numerical arrays used for scoring alignments and building coverage profiles. The `Cluster` class groups items by distance threshold using agglomerative clustering, useful for grouping nearby genomic features like binding sites or variants.

- **Format conversion via maf_to_fasta and related converters**: Converting between alignment formats requires consistent column ordering. The MAF format stores coordinates in ASCII (1-indexed, half-open intervals), while most downstream tools expect 0-indexed integer coordinates. Streams must be buffered when mixing iterator-based processing with random-access indexing.

- **Set operations on features**: Intersecting, subtracting, and uniting genomic intervals requires sorted inputs and proper handling of boundary cases (touching vs. overlapping intervals). The `operations` submodule provides `junction()`, `intersect()`, and `subtract()` functions that correctly handle half-open coordinate conventions used in genome browsers.

## Pitfalls

- **Using 1-indexed coordinates in code that expects 0-indexed**: bx-python functions generally expect 0-indexed, half-open intervals `[start, end)` to match Python conventions. Passing UCSC-style 1-indexed coordinates causes off-by-one errors where features are shifted by one base, silently corrupting downstream analysis results.

- **Not indexing MAF files before random access queries**: Without an `.index` file created by `maf_parse.py`, the `MafIndex` class falls back to linear scanning which is extremely slow on large files. Index creation is O(n) and lookup is O(log n), making it critical for any workflow querying specific genomic regions repeatedly.

- **Iterating and indexing the same MAF stream simultaneously**: Creating an index while iterating a MAF stream consumes the iterator, preventing simultaneous access. The workaround is to run indexing and iteration as separate steps or to cache the stream to a temporary file before parallel operations.

- **Assuming interval trees are automatically sorted**: IntervalTree queries assume items were added in coordinate order. Adding intervals out of order without calling `tree.finish()` causes incorrect overlap results, as the tree structure depends on sorted insertion for correct query performance.

- **Forgetting strand orientation in interval operations**: Most set operations ignore strand by default, treating overlapping features on opposite strands as identical. When analyzing features like binding sites that are strand-specific, explicitly filter or check `.strand` attributes before performing intersections to avoid false-positive overlaps.

## Examples

### Convert a MAF file to FASTA format
**Args:** input_maf --output-fasta --ensure-complete-reference
**Explanation:** Specifying `--ensure-complete-complete-reference` guarantees that gap characters are added so the reference sequence spans all aligned blocks, producing a valid aligned FASTA output where all sequences have equal length.

### Build a random-access index for a MAF file
**Args:** maf_parse.py input_maf --index
**Explanation:** The `--index` flag creates a `.maf.index` file alongside the input, enabling the `MafIndex` class to perform O(log n) genomic coordinate lookups instead of linear scanning in subsequent analysis steps.

### Extract MAF blocks overlapping a genomic region
**Args:** maf_parse.py input_maf --region=chr2:100000-200000
**Explanation:** The `--region` argument restricts output to alignment blocks that contain at least one query base, dramatically reducing output size and processing time when only a specific locus is needed for downstream analysis.

### Query overlapping intervals using an interval tree
**Args:** using bx.intervals IntervalTree and find method
**Explanation:** Building an interval tree from BED coordinates and using `find(start, end)` efficiently retrieves all features overlapping the query range, which is far faster than brute-force O(n) scans when processing millions of genomic annotations.

### Intersect two BED files to find common genomic features
**Args:** using bx.intervals.operations intersect function
**Explanation:** The `intersect()` function from `bx.intervals.operations` accepts sorted iterators and returns only the portions of intervals that overlap between the two sets, correctly handling half-open coordinate conventions for accurate feature sharing analysis.

### Cluster genomic features by proximity
**Args:** using bx.array.Cluster with distance_threshold parameter
**Explanation:** Setting a `distance_threshold` of 100 groups binding sites or variants within 100 base pairs into the same cluster, enabling analysis of binding site density and regional enrichment without manual grouping logic.

### Parse a FASTQ file and compute quality statistics
**Args:** using bx-seqtools FASTQ parsing functions
**Explanation:** The FASTQ parser in bx-python handles both Sanger and Illumina 1.8+ quality score encoding, automatically converting to integer phred scores for per-base quality summary statistics across millions of reads.

### Merge multiple MAF files into a single output
**Args:** using bx.align.maf ConcatenatingWriter with multiple input streams
**Explanation:** The `ConcatenatingWriter` merges alignment blocks from separate species or assemblies while preserving coordinate sorting, producing a combined MAF file suitable for whole-genome comparisons across multiple datasets.