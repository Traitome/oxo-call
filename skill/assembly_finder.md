---
name: assembly_finder
category: Genomics
description: A command-line tool for discovering, downloading, and managing genome assemblies from local databases and remote repositories like NCBI. Supports filtering by organism, assembly level, and sequence quality. Useful for bulk retrieval of reference genomes and comparative genomics workflows.
tags: [genome, assembly, download, ncbi, reference-genome, fasta, bioinformatics]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/assembly_finder
---

## Concepts

- **Assembly Identity**: Each assembly has a unique NCBI GenBank or RefSeq accession (e.g., GCF_000001405.40) that definitively identifies the version and source of the genome. Using the correct accession prevents downloading outdated or incorrect assemblies.
- **Assembly Levels**: Assemblies are categorized by quality—'complete genome' (chromosome-level), 'scaffold', 'contig', or ' chromosome'. The level determines suitability for different analyses; complete genomes are required for precise gene positioning.
- **Output Formats**: The tool outputs in standard formats (FASTA for sequences, AGP for assembly descriptions, CSV/JSON for metadata), enabling direct integration into downstream pipelines like alignment or annotation tools.
- **Local Database Index**: A local SQLite index caches remote assembly metadata to reduce network calls. The index is auto-updated but can be manually refreshed with `--update-index` to ensure current organism names and accessions.

## Pitfalls

- **Ambiguous Organism Names**: Using common names like "human" instead of the binomial "Homo sapiens" may match multiple assemblies (e.g., different strains or outdated entries). This leads to downloading the wrong assembly or confusing results in batch mode.
- **Incomplete Download**: Cancelling a download mid-way or using an unstable network connection leaves truncated files. The tool validates checksums for complete entries but partial FASTA files will silently fail alignment tools.
- **Outdated Index**: Running searches without `--update-index` on an old database returns obsolete accessions that may have been superseded or withdrawn, compromising reproducibility and causing errors in reproducible research.
- **Permission Issues**: Default download paths in system directories (e.g., /usr/local/share) require root write access. Configuring a custom output directory with `-o` avoids permission errors and allows user-managed storage.

## Examples

### Download the latest human reference genome from RefSeq
**Args:** `--organism "Homo sapiens" --source refseq --level complete --download`
**Explanation:** Specifies the organism, restricts to RefSeq (higher quality), requests complete genome level, and triggers the download with no output path flag (uses current directory).

### Search for all bacterial assemblies from a specific genus
**Args:** --genus salmonella --list-only
**Explanation:** The `--list-only` flag shows matching accessions without downloading, allowing inspection before bulk retrieval.

### Update the local assembly database Index
**Args:** --update-index
**Explanation:** Downloads fresh metadata from NCBI registries, ensuring searches reflect current accessions and organism names.

### Download a specific assembly by accession to a named folder
**Args:** -i GCF_000001405.40 -o ~/genomes/human_ref/
**Explanation:** The `-i` flag bypasses organism search and directly fetches the specified assembly, `-o` sets the output directory.

### Find assemblies with minimum 50x coverage
**Args:** --organism "mus musculus" --min-coverage 50 --list-only
**Explanation:** Filters mouse assemblies by sequence coverage, useful for selecting high-quality genomes for variant calling.

### Download multiple assemblies in batch mode
**Args:** --organism zebrafish --source genbank --level scaffold --batch --download
**Explanation:** The `--batch` flag processes all matching entries, enabling bulk retrieval for comparative studies.

### Export assembly metadata to CSV
**Args:** --organism "arabidopsis thaliana" --metadata-only --format csv
**Explanation:** Outputs a CSV file with assembly stats (size, level, accession) without downloading sequences, useful for planning analysis.