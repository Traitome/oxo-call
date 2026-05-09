---
name: assembly_uploader
category: data-upload
description: Command-line tool for uploading genome assembly files to remote databases or repositories (NCBI, ENA, or custom servers). Supports FASTA, GenBank, Multi-FASTA, and AGP formats with mandatory metadata fields.
tags: ["assembly", "upload", "fasta", "genbank", "bioinformatics", "data-submission", "ena", "ncbi"]
author: AI-generated
source_url: https://example.com/assembly_uploader_docs
---

## Concepts

- **Input formats**: assembly_uploader accepts FASTA files (.fasta, .fa, .fna), GenBank files (.gb, .gbk), and AGP files for describing complex assemblies. Files can be gzipped (.gz) to reduce bandwidth during upload.
- **Metadata requirements**: Each upload requires at minimum a species name (scientific name), strain identifier, and sequencing technology (e.g., "Illumina", "PacBio", "Oxford Nanopore"). Missing metadata causes the upload to queue in "pending" state indefinitely.
- **Authentication**: Uploads require API key or institutional credentials passed via `--api-key` or `--credentials-file`. Anonymous uploads are blocked by all major databases; unauthenticated requests return HTTP 401.
- **Batch mode**: Multiple assemblies can be queued in a manifest file (JSON or TSV) using `--manifest`. Batch uploads process sequentially — a single assembly failure does not halt the entire batch, but the failed entries are logged for retry.
- **Output states**: After upload, assemblies receive a handle/Accession (e.g., "GCF_000001405.40") or an error code. The tool polls for status until complete unless `--no-wait` is specified.

## Pitfalls

- **Omitting version numbers**: Specifying `--species "Homo sapiens"` without a strain or isolate tag results in ambiguous submissions. The database may reject the assembly or assign it to the wrong taxon, requiring manual curation later.
- **Uploading pre-indexed assemblies**: If you upload an assembly that has already been submitted to the same database under a different Accession, the system flags a duplicate. The new upload may be held pending review for weeks.
- **Mismatched file format extension**: Renaming a GenBank file to `.fasta` and uploading causes parsing errors. The uploader detects format via magic bytes, not extension — but a mismatch triggers validation failure.
- **Exceeding file size limits**: Most databases cap single-file uploads at 10 GB compressed. Files larger than this return HTTP 413 and the upload aborts; the only solution is splitting with `--split` or using Aspera/FTP batch transfer.
- **Ignoring update vs. new submission**: Using `--update GCF_000001405.40` on a handle that no longer exists returns 404. The tool does not auto-create — an update to a non-existent Accession fails silently in some versions.

## Examples

### Upload a single bacterial genome FASTA to the database
**Args:** `--file strains/EColi_K12.fasta --species "Escherichia coli K-12" --technology "Illumina" --api-key $NCBI_API_KEY`
**Explanation:** This uploads a single-isolate genome with taxonomy and sequencing metadata. The API key authenticates the user; the database assigns a provisional Accession.

### Submit a complex metagenome assembly using AGP
**Args:** `--file assembles/metag1.agp --species "metagenome" --project "Tara Oceans" --technology "Oxford Nanopore" --manifest manifest.json`
**Explanation:** AGP files describe how contigs map to scaffolds. The manifest references all component FASTA files; the project tag organizes the submission in the web portal.

### Retry a failed batch upload from a manifest
**Args:** `--manifest failed_submissions.json --resume`
**Explanation:** The `--resume` flag reads the previous log of failed uploads and re-queues only those entries, avoiding re-upload of successful assemblies.

### Upload a GenBank file without waiting for processing
**Args:** `--file uploads/ plasmid_clone.gb --species "synthetic construct" --no-wait --output-id`
**Explanation:** The `--no-wait` flag returns immediately with a job ID for polling later, useful in pipelines where you need non-blocking behavior.

### Check upload status using a received handle
**Args:** `--status GCF_000012345.678`
**Explanation:** Queries the database for the current state (uploading, validated, failed, released) of a previously submitted assembly. Returns structured status output for automation scripts.