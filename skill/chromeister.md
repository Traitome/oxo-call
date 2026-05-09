---
name: chromeister
category: Genome Assembly / Web Integration
description: A bioinformatics tool for downloading and processing genomic data from web sources, with capabilities for chromosome-level assembly and annotation. Integrates with web-based genomic databases to retrieve reference sequences and metadata.
tags: [genome, assembly, web-scrape, chromosomes, reference-database, download]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/chromeister
---

## Concepts

- **Web-based genomic data retrieval**: chromeister connects to online genomic databases (NCBI, Ensembl, UCSC) to download chromosome sequences, annotations, and metadata in standard formats (FASTA, GTF, BED).
- **Streaming I/O architecture**: Input can be direct URLs, accession numbers (e.g., GCA_000001405), or local FASTA files; output streams to stdout in FASTA/FASTQ format for pipeline integration.
- **Chromosome-aware processing**: The tool maintains chromosome naming conventions (chr1, chr2, mt, etc.) and can filter, subset, or combine specific chromosomes from multi-chromosome assemblies.
- **Companion index builder**: Use `chromeister-build` to create indexed reference databases for fast lookups; indexes are compatible with downstream alignment tools accepting indexed genomes.

## Pitfalls

- **Assuming web access is always available**: If the machine lacks internet connectivity or the remote database is down, chromeister will fail silently or timeout. Always verify network access with `--ping` before large batch downloads.
- **Mixing chromosome naming schemes**: Reference databases use different conventions (chr1 vs 1, chrMT vs mt). Mixing incompatible sources causes downstream alignment failures—use `--normalize-names` to enforce consistency.
- **Ignoring file size limits**: Large multi-chromosome assemblies (human, mouse) can exceed several gigabytes. Without specifying `--split-chromosomes`, the tool may consume excessive memory and crash on resource-limited nodes.

## Examples

### Download human chromosome 1 reference sequence
**Args:** `--accession GCF_000001405.40 --chromosome chr1 --output fasta`
**Explanation:** Downloads the GRCh38 reference sequence for chromosome 1 using the NCBI assembly accession and outputs in FASTA format.

### Retrieve multiple chromosomes in a single run
**Args:** `--accession GCF_000001405.40 --chromosome chr1,chr2,chr3,chrM --output fasta`
**Explanation:** Fetches four specified chromosomes (including mitochondrial DNA) in one request, reducing network overhead.

### Stream FASTA output to another bioinformatics tool
**Args:** `--accession GCF_000001405.40 --chromosome chr1 --output stdout | bwa mem - ref.fa reads.fq`
**Explanation:** Pipes chromosome 1 directly into bwa mem for alignment, avoiding intermediate disk writes.

### Create an indexed reference database
**Args:** GCF_000001405.40 --reference-name GRCh38 --index-dir /refs/grch38/
**Explanation:** Uses the companion `chromeister-build` binary to generate indexed files for fast queries by downstream tools.

### Download with normalized chromosome naming
**Args:** `--accession GCF_000001405.40 --output fasta --normalize-names`
**Explanation:** Converts all chromosome names to the chr-prefixed format (e.g., 1 → chr1, MT → chrM) ensuring compatibility with tools expecting UCSC conventions.

### Filter to protein-coding genes only
**Args:** --accession GCF_000001405.40 --feature-type gene --gene-biotype protein_coding --output genomic.gtf
**Explanation:** Downloads gene annotations filtered to protein-coding genes only, excluding pseudogenes and non-coding RNAs.

### Check database connectivity before batch jobs
**Args:** `--ping --timeout 10`
**Explanation:** Performs a lightweight connectivity check against the configured genomic database to verify the service is reachable before launching resource-intensive downloads.

### Split output by individual chromosome files
**Args:** --accession GCF_000001405.40 --split-chromosomes --output-dir /refs/hg38/
**Explanation:** Writes each chromosome to a separate FASTA file (chr1.fa, chr2.fa, etc.) in the specified directory, reducing memory usage and enabling parallel processing.