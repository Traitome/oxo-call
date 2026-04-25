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
- --split-files separates paired-end reads into _1.fastq and _2.fastq; --split-3 puts unpaired reads in a separate file.
- --skip-technical filters out technical reads (barcodes, linkers) common in 10x Genomics and similar protocols.
- --min-read-len filters reads by length; useful for quality control before downstream analysis.
- --fasta outputs FASTA format instead of FASTQ; useful when quality scores are not needed.
- vdb-validate checks SRA file integrity after download; detects corruption before conversion.
- prefetch --option-file accepts a text file with multiple accessions for batch downloads.
- --ngc specifies a dbGaP authorization file for controlled-access data downloads.

## Pitfalls
- fastq-dump is deprecated — always use fasterq-dump for new workflows.
- fasterq-dump requires temp disk space (3-4x the final FASTQ size) — ensure sufficient disk space.
- For paired-end data, fasterq-dump --split-files creates separate _1 and _2 FASTQ files automatically.
- Without --outdir (-O), fasterq-dump outputs to the current directory.
- The .sra files downloaded by prefetch are stored in ~/ncbi/public/sra/ by default.
- Large SRA downloads may timeout on poor connections — use prefetch for resilient downloads.
- --split-3 behavior differs from --split-files; it creates a third file for unpaired reads.
- fasterq-dump does NOT compress output; pipe to gzip or compress afterward.
- --mem limits RAM for sorting; default 100MB may be insufficient for large datasets.
- --temp specifies temp directory; use fast storage (SSD, /tmp) for better performance.
- dbGaP data requires --ngc file; downloads fail without proper authorization.
- vdb-validate may report warnings for valid files; check error vs warning carefully.

## Examples

### download and convert an SRA accession to FASTQ
**Args:** `fasterq-dump SRR123456 -O output_directory/ -e 8`
**Explanation:** fasterq-dump companion binary; SRR123456 SRA accession; -O output_directory/ output directory; -e 8 threads; downloads and converts to FASTQ; auto-splits paired-end reads

### download SRA file first then convert (more reliable)
**Args:** `prefetch SRR123456 -O sra_downloads/`
**Explanation:** prefetch companion binary; SRR123456 SRA accession; -O sra_downloads/ output directory for .sra file; then run: fasterq-dump sra_downloads/SRR123456/SRR123456.sra -O fastq_output/

### download multiple SRA accessions in batch
**Args:** `prefetch --option-file accession_list.txt -O sra_downloads/`
**Explanation:** prefetch companion binary; --option-file accession_list.txt text file with one accession per line; -O sra_downloads/ output directory; downloads all listed accessions; then loop through fasterq-dump for conversion

### convert SRA to compressed FASTQ
**Args:** `fasterq-dump SRR123456 -O output/ -e 8 && gzip output/SRR123456_1.fastq output/SRR123456_2.fastq`
**Explanation:** fasterq-dump companion binary; SRR123456 accession; -O output/ output directory; -e 8 threads; && gzip compresses output; fasterq-dump does not natively gzip output

### validate an SRA file integrity
**Args:** `vdb-validate SRR123456.sra`
**Explanation:** vdb-validate companion binary; SRR123456.sra input SRA file; checks SRA file integrity before conversion; useful after download

### get statistics for an SRA run without downloading reads
**Args:** `sra-stat --quick --xml SRR123456`
**Explanation:** sra-stat companion binary; --quick avoids full file scan; --xml output format; SRR123456 SRA accession; retrieves run metadata (read count, base count, layout)

### download an ENA/EBI accession using prefetch
**Args:** `prefetch ERR123456 -O sra_downloads/`
**Explanation:** prefetch companion binary; ERR123456 ENA accession; -O sra_downloads/ output directory; supports ERR (ENA) and DRR (DDBJ) accessions; stores .sra file for subsequent fasterq-dump conversion

### list all reads in an SRA file
**Args:** `fasterq-dump SRR123456 --stdout -e 4 | head -40`
**Explanation:** fasterq-dump companion binary; SRR123456 accession; --stdout streams to stdout; -e 4 threads; | head -40 preview first 40 lines; preview reads without full download

### check available disk space before a large download
**Args:** `fasterq-dump SRR123456 --check-space`
**Explanation:** fasterq-dump companion binary; SRR123456 accession; --check-space estimates disk space; exits without downloading; prevents failures from insufficient storage

### download a single-end SRA accession and skip technical reads
**Args:** `fasterq-dump SRR123456 -O output/ -e 8 --skip-technical`
**Explanation:** fasterq-dump companion binary; SRR123456 accession; -O output/ output directory; -e 8 threads; --skip-technical omits technical reads (barcodes, linkers); important for single-cell or 10x Genomics datasets

### convert SRA to FASTA format (no quality scores)
**Args:** `fasterq-dump SRR123456 -O output/ --fasta -e 8`
**Explanation:** fasterq-dump companion binary; SRR123456 accession; -O output/ output directory; --fasta outputs FASTA instead of FASTQ; -e 8 threads; useful for applications that don't need quality scores

### filter reads by minimum length
**Args:** `fasterq-dump SRR123456 -O output/ -e 8 --min-read-len 80`
**Explanation:** fasterq-dump companion binary; SRR123456 accession; -O output/ output directory; -e 8 threads; --min-read-len 80 filters reads shorter than 80bp; useful for QC and removing adapter dimers

### use custom temp directory on fast storage
**Args:** `fasterq-dump SRR123456 -O output/ -e 8 --temp /scratch/tmp --mem 500M`
**Explanation:** fasterq-dump companion binary; SRR123456 accession; -O output/ output directory; -e 8 threads; --temp /scratch/tmp fast SSD temp storage; --mem 500M increases sort memory for large datasets

### download controlled-access dbGaP data
**Args:** `prefetch SRR123456 --ngc prj_12345.ngc -O sra_downloads/`
**Explanation:** prefetch companion binary; SRR123456 accession; --ngc prj_12345.ngc dbGaP authorization file; -O sra_downloads/ output directory; required for controlled-access human sequence data

### validate SRA file with deep consistency check
**Args:** `vdb-validate SRR123456.sra --CONSISTENCY-CHECK yes -v`
**Explanation:** vdb-validate companion binary; SRR123456.sra input file; --CONSISTENCY-CHECK yes performs deep validation; -v verbose output; catches corruption that md5 alone misses

### list contents of a kart file before download
**Args:** `prefetch --list my_study.kart`
**Explanation:** prefetch companion binary; --list displays contents; my_study.kart kart file input; shows accessions without downloading; useful for verifying cart contents before large downloads

### convert only aligned reads from BAM-based SRA
**Args:** `fasterq-dump SRR123456 -O output/ -e 8 --only-aligned`
**Explanation:** fasterq-dump companion binary; SRR123456 accession; -O output/ output directory; -e 8 threads; --only-aligned extracts only aligned reads; useful for targeted re-sequencing data

### concatenate paired reads into one file per spot
**Args:** `fasterq-dump SRR123456 -O output/ -e 8 --concatenate-reads`
**Explanation:** fasterq-dump companion binary; SRR123456 accession; -O output/ output directory; -e 8 threads; --concatenate-reads writes whole spots as single entries; useful for interleaved format requirement
