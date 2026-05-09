---
name: aria2
category: Download Manager
description: aria2 is a lightweight multi-protocol command-line download utility that supports concurrent segmented downloads, HTTP/HTTPS, FTP, BitTorrent, and Metalink. Commonly used in bioinformatics pipelines for downloading reference genomes, datasets, and software archives.
tags:
  - download
  - http
  - ftp
  - bittorrent
  - segmented-download
  - bioinformatics
  - pipeline
author: AI-generated
source_url: https://aria2.github.io/
---

## Concepts

- **Segmented concurrent downloads**: aria2 splits a file into multiple pieces and downloads them simultaneously using the `-x` flag to specify connections per server (max 16 for HTTP/FTP). This dramatically speeds up downloads of large files like reference genomes (e.g., GRCh38 ~3 GB) compared to single-connection downloads.

- **Resumable downloads**: With the `-c` flag, aria2 tracks downloaded chunks in a `.aria2` control file. If the download is interrupted (network outage, timeout), running the same command again resumes from the last successfully written byte. Without `-c`, interrupted downloads must restart from zero.

- **Checksum verification**: The `--checksum` flag (or `--check-integrity` for BitTorrent) validates file integrity after download. For bioinformatics data, this prevents silent data corruption in reference FASTA files, BED files, or VCF datasets—corrupted bases could propagate errors into downstream analyses.

- **Multi-source and Metalink support**: aria2 can download the same file from multiple mirror URLs listed in a `.metalink` XML file or `.aria2` control file. This improves reliability when mirrors have varying uptime or geographic latency.

- **Configuration file and RPC mode**: aria2 reads options from `~/.aria2/aria2.conf` using `dir`, `continue`, `max-connection-per-server`, and other directives. The `--conf-path` flag overrides the default location. aria2 also supports an RPC server mode (`--enable-rpc`) for programmatic control in pipeline scripts.

## Pitfalls

- **Omitting the resumable flag**: Running aria2 without `-c` means partial downloads are discarded on interruption. For a 3 GB human genome FASTA downloaded over an unstable connection, losing 2.9 GB and restarting wastes hours of pipeline time.

- **Setting excessive concurrent connections**: Using `-j 16` or higher against a server with rate limiting can trigger IP bans or temporary blocks. Bioinformatics servers like NCBI or ENA may throttle offending clients, causing all subsequent downloads in the pipeline to fail.

- **Assuming default output filename**: Without `-o filename`, aria2 saves the file as the URL's basename (e.g., `GCF_000001405.39_GRCh38.p13_genomic.fna.gz`). If two downloads share the same basename from different URLs, the second overwrites the first silently, causing data loss.

- **Not verifying checksums**: Skipping `--checksum` after downloading large archives means undetected corruption. A single flipped bit in a BED file could misalign genomic coordinates, producing false variant calls in downstream analyses.

- **Incorrect config file permissions**: If `~/.aria2/aria2.conf` is readable only by root (mode 600), aria2 running as a regular user silently ignores all configuration directives and uses defaults, potentially causing unexpected behavior in automated pipelines.

## Examples

### Download a single large file with 8 concurrent connections

**Args:** `-x 8 -d /data/genomes https://ftp.ncbi.nlm.nih.gov/genomes/all/GCF/000/001/405/GCF_000001405.39_GRCh38.p13/GCF_000001405.39_GRCh38.p13_genomic.fna.gz`
**Explanation:** The `-x 8` flag opens 8 parallel connections to the NCBI FTP server, reducing download time for a 1 GB compressed reference file compared to a single-stream download.

### Resume an interrupted download

**Args:** `-c -d /data/rnaseq https://example.com/SRR12345678.fastq.gz`
**Explanation:** The `-c` flag enables resumable mode, so aria2 checks the `.aria2` control file and continues the download from byte offset if the file already exists as a partial download.

### Download and save with a specific output filename

**Args:** `-o GRCh38.p13_reference.fa.gz -z GRCh38.p13_reference.fa.gz https://ftp.ncbi.nlm.nih.gov/genomes/all/GCF/000/001/405/GCF_000001405.39_GRCh38.p13/GCF_000001405.39_GRCh38.p13_genomic.fna.gz`
**Explanation:** The `-o` flag renames the downloaded file to `GRCh38.p13_reference.fa.gz`, preventing conflicts when downloading multiple genome builds in the same directory.

### Download with MD5 checksum verification

**Args:** `--checksum=md5=abc123def456 https://data.example.com/dataset.vcf.gz`
**Explanation:** After downloading, aria2 computes the MD5 hash and compares it against the provided value, ensuring the VCF dataset is not corrupted before it enters a variant-calling pipeline.

### Download with a bandwidth limit to avoid saturating shared network links

**Args:** `--max-download-limit=2M -d /data/cache https://mirror.example.com/annotation.gff3.gz`
**Explanation:** The `--max-download-limit=2M` cap restricts aria2 to 2 MB/s, allowing the download to run in the background without disrupting other users or services sharing the network.

### Download with a custom number of retries and timeout per retry

**Args:** `-m 10 --timeout=60 https://ftp.example.com/homo_sapiens_anno.zip`
**Explanation:** `-m 10` attempts the download up to 10 times if transient HTTP 503 errors occur, and `--timeout=60` waits 60 seconds per attempt before giving up, making the command suitable for unreliable connections.

### Download using a proxy server for authenticated downloads

**Args:** `--http-proxy=http://proxyuser:proxypass@proxy.example.com:8080 -o raw_data.tsv https://data.example.com/results/raw_data.tsv`
**Explanation:** The `--http-proxy` flag routes traffic through an authenticated proxy—useful in restricted HPC environments where external access requires corporate proxy credentials.