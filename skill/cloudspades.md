---
name: cloudspades
category: genome-assembly
description: A cloud-native distributed genome assembler based on SPAdes, designed for scalable de novo assembly of microbial and small eukaryotic genomes using heterogeneous sequencing data.
tags: [de-novo-assembly, microbial-genomics, distributed-computing, de-bruijn-graph, cloud-native]
author: AI-generated
source_url: https://github.com/ablab/spades
---

## Concepts

- **Distributed Architecture**: CloudSPAdes partitions read data and k-mer graphs across multiple compute nodes, distributing memory and CPU load for assemblies that exceed single-machine capacity. The `--cloud` flag enables this distributed mode, requiring an object storage prefix (e.g., `--S3-bucket` or `--gs-prefix`) for intermediate data staging.

- **Heterogeneous Read Support**: CloudSPAdes accepts mixed sequencing libraries from Illumina, IonTorrent, Oxford Nanopore, and PacBio (HiFi and CLR) formats. Input reads must be in FASTQ format (standard or gzip-compressed). The assembler automatically selects k-mer sizes and error-correction parameters based on detected library types, but explicit `--pe` (paired-end), `--s` (single), `--pacbio`, or `--nanopore` flags can override auto-detection.

- **k-mer Size Selection**: CloudSPAdes uses a cascading k-mer strategy (default: 21, 33, 55) where multiple k-mer sizes are run sequentially to resolve repetitive regions. The `--careful` mode applies repeat resolution algorithms to reduce contigs with ambiguous connections. Custom k-mer sizes are specified via `--ion-torrent`, `--illumina`, or explicit `--contigs-history` re-use from prior assemblies.

- **Output Artifacts**: The assembler produces several output files in the specified output directory: `contigs.fasta` (final assembled sequences), `scaffolds.fasta` (scaffold-level assemblies), `assembly_graph.fastg` (connectivity graph for visualization), and `params.txt` (assembly statistics including N50, total length, and largest contig). JSON-formatted logs are available in `.json` files for downstream parsing.

- **Memory and Thread Scaling**: Memory requirements scale roughly as `genome_size × coverage × kmer_size / 8`. For large datasets, CloudSPAdes auto-tunes thread allocation based on `--threads` and `--memory` flags, but oversubscribing threads relative to available cores causes I/O contention. The `--unified-memory` flag forces a shared-memory model suitable for multi-core single-node execution.

## Pitfalls

- **Incorrect Cloud Storage Configuration**: Specifying a non-existent bucket prefix (e.g., mistyped S3 bucket name or missing trailing slash) causes silent data staging failures where CloudSPAdes attempts local storage instead of distributed processing. This results in assemblies completing without distributed scaling, defeating the cloud-native purpose and potentially causing out-of-memory errors on single nodes.

- **K-mer Size Mismatch with Read Length**: Setting `--k-mer-size` values larger than the read length causes immediate failure with an "invalid k-mer size" error. For short Illumina reads (100 bp), k-mers above 127 are invalid. For Nanopore reads, k-mers above 127 may produce excessively fragmented assemblies due to error rates compounding in longer k-mers.

- **Mixed Library Quality Neglect**: Running CloudSPAdes without the `--careful` mode on datasets with significantly different quality profiles (e.g., mixing freshly extracted high-quality DNA with degraded archival samples) leads to chimeric contigs where low-quality reads introduce spurious connections in the de Bruijn graph. The assembler may still produce plausible-looking but biologically incorrect assemblies.

- **Insufficient Object Storage Permissions**: CloudSPAdes requires read/write permissions to the specified cloud bucket for intermediate graph and read partition files. Read-only IAM roles cause partial assembly failures where early stages succeed but checkpoint writes fail, leaving corrupted intermediate data. Full bucket access with lifecycle policies enabled prevents accumulation of orphaned temporary files.

- **Deprecated or Conflicting K-mer History**: Re-using `--contigs-history` from a previous CloudSPAdes run on different read sets produces assemblies where k-mer connections reflect the original read graph rather than current data. This leads to contig sequences that include reads absent from the current input, contaminating results with phantom coverage patterns visible in alignment validation.

## Examples

### Assemble paired-end Illumina reads with automatic k-mer selection
**Args:** `--careful -1 reads_R1.fastq.gz -2 reads_R2.fastq.gz -o assembly_output --threads 16`
**Explanation:** The `--careful` flag enables repeat resolution to reduce ambiguously connected contigs, while `-1`/`-2` specify paired-end input files and `--threads` allocates compute resources for faster completion.

### Assemble Oxford Nanopore reads with PacBio CLR fallback
**Args:** `--nanopore ont_reads.fastq.gz --pacbio pacbio_reads.fastq.gz -o hybrid_assembly --cloud --S3-bucket s3://my-bucket/spades-staging/`
**Explanation:** Mixed long-read input flags allow CloudSPAdes to leverage high-quality Nanopore data alongside PacBio reads, with `--cloud` and `--S3-bucket` enabling distributed computation with cloud object storage for intermediate data.

### Assemble IonTorrent reads with custom k-mer sizes
**Args:** `--ion-torrent iontorrent_reads.fastq --k-mer-size 21,33,55,77 -o ion_assembly --memory 64`
**Explanation:** Explicit `--ion-torrent` flags override auto-detection for IonTorrent data characteristics, and `--k-mer-size` specifies a cascading k-mer range optimized for the read length and error profile.

### Resume an interrupted assembly using checkpoint history
**Args:** `--contigs-history prior_run/contigs.info -1 new_reads_R1.fastq.gz -2 new_reads_R2.fastq.gz -o resumed_assembly`
**Explanation:** The `--contigs-history` flag re-uses k-mer graph topology from a prior run, allowing CloudSPAdes to continue from checkpoints rather than restarting read error correction from scratch.

### Perform meta-genome assembly with trusted contig seeding
**Args:** `--meta --pe1 reads.fastq.gz -o meta_assembly --untrusted-contigs input_contigs.fasta --careful`
**Explanation:** The `--meta` flag optimizes assembly parameters for multi-species datasets, while `--untrusted-contigs` allows inclusion of pre-assembled sequences (e.g., from prior binning) as trusted seeds for strain-level resolution.