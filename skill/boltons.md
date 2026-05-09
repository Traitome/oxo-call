---
name: boltons
category: utility_library
description: A comprehensive Python utility library providing efficient data structures, file operations, and iteration helpers useful in bioinformatics pipelines for atomic file writing, progress tracking, caching, and data manipulation.
tags: [python, utility, file-operations, iteration, caching, data-structures, bioinformatics]
author: AI-generated
source_url: https://boltons.readthedocs.io/
---

## Concepts

- **Atomic File Writing**: The `boltons.fileutils.atomic` module provides `AtomicWriter` for crash-safe file writes—critical when writing large genomics data (VCF, BAM indices) to prevent corrupt output if the pipeline is interrupted.
- **Lazy Iteration**: `boltons.iterutils` offers `backed_iter`, `chunked_iter`, and `windowed_iter` for processing large FASTQ/BAM files in memory-efficient chunks without loading entire files into memory.
- **Caching Utilities**: `boltons.cacheutils` provides `@cached` decorators and `LRU` implementations to cache expensive computations like sequence alignments or variant calls across pipeline stages.
- **Dictionary Nesting**: `boltons.dictutils` supports nested dot-notation access (`dict.get_recursive()`) for manipulating complex nested JSON/JSONL outputs from tools like GATK or samtools.
- **Progress Reporting**: `boltons.tqdmutils` integrates with tqdm for progress bars when processing multiple samples or genomic regions in batch scripts.

## Pitfalls

- **Using In-Memory Cache for Massive Datasets**: Applying `@cached` decorators to functions returning huge numpy arrays or DataFrames can exhaust RAM, causing OOM kills on HPC clusters—use `boltons.cacheutils.on_disk` or file-based caching instead.
- **AtomicWriter Fails on Network mounts**: `AtomicWriter` relies on `os.rename()` which is not atomic on NFS or certain network filesystems; writes may still corrupt on shared storage—always verify write success.
- **Iterutils Consumes Iterators Once**: Functions from `iterutils` like `backed_iter` consume the underlying iterator; re-reading requires re-creating the iterator, causing missing data if the source file stream is not seekable.
- **Import Errors in Isolated Environments**: Boltons is a third-party library; if the bioinformatics environment lacks internet access, ensure boltons is pre-installed in the container/Singularity image.
- **Thread Safety with Fileutils**: `AtomicWriter` is not thread-safe by default; concurrent writes from multiple pipeline processes to the same file can cause race conditions—use explicit locking.

## Examples

### Write a VCF file atomically to prevent corruption during pipeline interruption
**Args:** `-c "from boltons.fileutils import AtomicWriter; aw = AtomicWriter('/data/output.vcf'); aw.write(open('/data/output.vcf').read())"`
**Explanation:** Uses AtomicWriter to perform a temp-file-then-rename pattern ensuring the output file is either complete or untouched if the process crashes.

### Process a large FASTQ file in fixed-size chunks for memory efficiency
**Args:** `-c "from boltons.iterutils import chunked_iter; reads = chunked_iter(open('large.fq'), chunksize=10000); [process(batch) for batch in reads]"`
**Explanation:** Chunks the FASTQ file into 10,000-line batches, allowing downstream processing without loading the entire gigabyte-scale file into RAM.

### Cache an expensive BWA alignment function call across multiple samples
**Args:** `-c "from boltons.cacheutils import cached; @cached; def align_sample(sample): return bwa.align(sample)"`
**Explanation:** Decorator caches alignment results by argument hash, avoiding redundant recomputation when re-running failed samples or restarting the pipeline.

### Access nested JSON output from GATK using dot notation
**Args:** `-c "from boltons.dictutils import NestedDict; data = NestedDict(gatk_json); print(data['variantAnnotations.coverage.mean'])" `
**Explanation:** Allows dot-notation traversal of deeply nested GATK JSON outputs without chained `.get()` calls or KeyError handling.

### Create an LRU cache to hold top N most-recently accessed reference sequences
**Args:** `-c "from boltons.cacheutils import LRU; seq_cache = LRU(maxsize=100); seq_cache['chr1'] = fasta.fetch('chr1')"`
**Explanation:** Provides a bounded cache evicting least-recently-used entries when full, useful for keeping frequently-accessed chromosomes in memory during variant calling.