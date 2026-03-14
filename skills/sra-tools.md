---
name: sra-tools
category: utilities
description: NCBI SRA toolkit for downloading and converting sequencing data from the Sequence Read Archive
tags: [sra, ncbi, download, fastq, sequencing-data, public-data, accession]
author: oxo-call built-in
source_url: "https://github.com/ncbi/sra-tools"
---

## Concepts

- SRA Tools downloads sequencing data from NCBI SRA using accession numbers (SRR, ERR, DRR formats).
- fasterq-dump is the modern replacement for fastq-dump — it's much faster and uses local disk for temp storage.
- fasterq-dump: fasterq-dump SRR123456 -O output_dir/ -e N; auto-splits paired-end reads into _1.fastq and _2.fastq.
- prefetch downloads .sra files locally first, then fasterq-dump converts: prefetch SRR123456 && fasterq-dump SRR123456.
- prefetch + fasterq-dump is faster than direct fasterq-dump for multiple accessions or slow connections.
- Use -e N for multi-threading in fasterq-dump; -O for output directory.
- For large datasets, use the AWS S3 or cloud-optimized approach: --cloud-instance or aws s3 sync.
- The SRA file format stores reads efficiently; .sra files must be converted to FASTQ for most tools.

## Pitfalls

- fastq-dump is deprecated — always use fasterq-dump for new workflows.
- fasterq-dump requires temp disk space (3-4x the final FASTQ size) — ensure sufficient disk space.
- For paired-end data, fasterq-dump --split-files creates separate _1 and _2 FASTQ files automatically.
- Without --outdir (-O), fasterq-dump outputs to the current directory.
- The .sra files downloaded by prefetch are stored in ~/ncbi/public/sra/ by default.
- Large SRA downloads may timeout on poor connections — use prefetch for resilient downloads.

## Examples

### download and convert an SRA accession to FASTQ
**Args:** `fasterq-dump SRR123456 -O output_directory/ -e 8`
**Explanation:** fasterq-dump downloads and converts; -O output directory; -e 8 threads; auto-splits PE reads

### download SRA file first then convert (more reliable)
**Args:** `prefetch SRR123456 -O sra_downloads/`
**Explanation:** prefetch downloads .sra file; then run: fasterq-dump sra_downloads/SRR123456/SRR123456.sra -O fastq_output/

### download multiple SRA accessions in batch
**Args:** `prefetch --option-file accession_list.txt -O sra_downloads/`
**Explanation:** --option-file with one accession per line; downloads all; then loop through fasterq-dump for conversion

### convert SRA to compressed FASTQ
**Args:** `fasterq-dump SRR123456 -O output/ -e 8 && gzip output/SRR123456_1.fastq output/SRR123456_2.fastq`
**Explanation:** fasterq-dump then gzip; fasterq-dump does not natively gzip output

### validate an SRA file integrity
**Args:** `vdb-validate SRR123456.sra`
**Explanation:** vdb-validate checks SRA file integrity before conversion; useful after download
