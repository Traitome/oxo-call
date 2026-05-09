---
name: bucketcache
category: Data Management / Caching
description: A transparent caching layer for cloud storage that caches bioinformatics files locally while maintaining the master copy in S3, GCS, or Azure. Used to accelerate repeated access to large genomic datasets.
tags: [cache, cloud-storage, s3, bioinformatics, performance, i/o-optimization]
author: AI-generated
source_url: https://github.com/CloudMmers/bucketcache
---

## Concepts

- `bucketcache` operates as a transparent caching proxy between bioinformatics tools and cloud storage backends (S3, Google Cloud Storage, Azure Blob Storage). Files appear as regular files to downstream tools like `samtools`, `bcftools`, or `tabix`, but are fetched from local cache on subsequent accesses.
- The tool supports two primary caching strategies: **write-through** (data is written to both cache and remote simultaneously, ensuring consistency) and **write-back** (data is written to cache first and lazily synced to remote, optimizing write performance for batch jobs).
- Cache configuration is defined in a YAML file that specifies the remote bucket URL, local cache directory path, maximum cache size (in GB), and expiration policies. The tool monitors file access patterns to determine which files to retain or evict.
- `bucketcache` can be invoked in two modes: as a **daemon** that mounts a FUSE filesystem for transparent access, or as a **companion binary** (`bucketcache-prefetch`) to explicitly populate the cache before running analysis pipelines.
- The cache system tracks metadata (ETag, size, last-modified) of remote objects and automatically invalidates stale cached entries when the remote file changes, preventing analysis on outdated data.

## Pitfalls

- **Stale cache entries**: If remote files are updated directly (bypassing bucketcache), the cached copy may not reflect the latest version. Running analyses on stale index files (.tbi, .csi) can produce incorrect variant calls or missing alignments.
- **Insufficient cache size**: When the local cache directory fills up, `bucketcache` evicts files based on LRU policy. Frequently accessed index files may be evicted mid-pipeline, forcing expensive re-downloads and causing job failures or inconsistent results.
- **Path mismatch in mount configuration**: If the mount point path does not match the paths used by downstream tools, files are accessed directly from remote storage, negating the performance benefits and accumulating egress costs.
- **Incomplete cache warm-up**: Running a pipeline without prefetching critical files first (e.g., reference genome FASTA, variant call sets) causes the first invocation to pay full latency costs, slowing down initial runs significantly.
- **Permission desync between cache and remote**: Running as a different user (e.g., root vs. compute user) can cause cache permission errors, resulting in "Permission denied" failures when tools attempt to read or write cached files.

## Examples

### Mount an S3 bucket as a cached local filesystem
**Args:** mount --config /path/to/cache.yaml /mount/point
**Explanation:** This mounts the remote S3 bucket at the specified local path, allowing bioinformatics tools to access files as if they were local while `bucketcache` handles fetching and caching transparently.

### Prefetch a BED file before running a variant calling pipeline
**Args:** prefetch --config /path/to/cache.yaml s3://bucket/genome/regions.bed.gz
**Explanation:** Explicitly downloads the BED file into the local cache before pipeline execution, eliminating remote latency during the critical analysis phase.

### Configure bucketcache with a 500 GB local cache limit
**Args:** cache-server --config /path/to/cache.yaml --max-size 500
**Explanation:** Starts the caching daemon with a 500 GB size limit, controlling disk usage on shared compute nodes while ensuring adequate capacity for most genomics workloads.

### List cached files to verify which files are stored locally
**Args:** cache-stat --config /path/to/cache.yaml --list
**Explanation:** Queries the cache metadata to verify which remote objects are currently stored locally, helping diagnose whether slowdowns are due to cache misses.

### Evict specific cached files to free up space for new datasets
**Args:** cache-evict --config /path/to/cache.yaml --prefix s3://bucket/old-cohort/
**Explanation:** Manually removes cached files matching a prefix, allowing users to free cache space for new analyses without waiting for automatic LRU eviction.