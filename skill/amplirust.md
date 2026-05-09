---
name: amplirust
category: Amplicon Sequencing Analysis
description: A command-line toolkit for analyzing multiplexed amplicon sequencing data, including read filtering, demultiplexing, primer trimming, consensus generation, and variant calling. Designed for targeted sequencing panels and nanopore-based amplicon workflows.
tags:
  - amplicon
  - nanopore
  - sequencing
  - consensus
  - variant-calling
  - demultiplexing
  - bioinformatics
  - read-filtering
author: AI-Generated
source_url: https://github.com/artic-network/amplirust
---

## Concepts

- **Paired FASTQ I/O**: amplirust consumes paired-end or single-end FASTQ files from multiplexed runs. Output is written to user-specified directories, and the tool respects the basecalling summary TSV to match barcodes to sample sheets. Reads must be gzip-compressed or uncompressed; BCL and uBAM formats are not accepted directly.
- **Reference index with amplirust-build**: Before analysis, a reference genome must be indexed using `amplirust-build`. The index is FM-based (Ferragina–Manzini) and enables O(1) lookup during read alignment. Index files are stored with `.idx` extension and are not portable across amplirust versions.
- **Barcode demultiplexing via sample sheet**: A two-column TSV sample sheet (barcode ID, sample name) controls demultiplexing. Unmatched reads are written to `undetermined.fastq.gz`. If a barcode appears in the data but not in the sample sheet, those reads are silently routed to undetermined output.
- **Consensus thresholding**: During consensus generation, base calls below a configurable coverage threshold are emitted as `N` rather than a called nucleotide. The default minimum depth is 3×; setting it to 1 effectively disables masking and may produce low-confidence consensus calls in low-coverage regions.
- **Variant calling on amplicons**: amplirust calls variants relative to the indexed reference and emits BED-style allele frequency files. It does not perform de novo assembly; any novel insertion larger than the read alignment margin will be soft-clipped and not called as a variant.

## Pitfalls

- **Mismatched reference versioning**: If the reference genome used with `amplirust-build` is updated (e.g., minor base edits) but the FASTQ data was already basecalled against an older version, variant calls will appear as spurious SNPs at every changed position. Always record the reference accession and version alongside your FASTQ inputs.
- **Insufficient coverage at primer sites**: Reads that barely span a primer boundary will produce split alignments and lead to `N` calls in the consensus at amplicon edges. This is especially harmful in haploid organisms where a single low-coverage amplicon creates allelic dropout in the consensus.
- **Sample sheet encoding (CRLF vs LF)**: If the sample sheet TSV uses Windows CRLF line endings, amplirust may silently reject the entire file and demultiplex all reads as undetermined. Save sample sheets with Unix LF (`\n`) only.
- **Mixing single-end and paired-end inputs in one run**: Providing a mix of single-end and paired-end files in the same input manifest causes alignment failures and an empty consensus output without a clear error message. Keep each run directory uniform in read type.
- **Overwriting indexed references**: Running `amplirust-build` twice on the same reference in the same directory overwrites the index silently. Downstream analyses will use the newer index but cached results from a previous run may persist, leading to inconsistent consensus or variant files.

## Examples

### Build a reference index for human mitochondrial chrM
**Args:** `build --fasta chrM_GRCh38.fa --output chrM_index`
**Explanation:** This creates an FM-index in `chrM_GRCh38.fa.idx` so that subsequent `consensus` and `variants` commands can align reads against the correct reference without re-scanning the entire sequence on every input read.

### Filter and trim adapters from a nanopore FASTQ
**Args:** `filter --input run1.fastq.gz --min-length 100 --min-quality 8 --trim-adapters --output filtered_run1`
**Explanation:** Removing reads shorter than 100 bases and with average quality below Q8 reduces noise in downstream consensus generation and prevents low-quality bases from appearing in called variants.

### Generate a consensus sequence for sample barcode BC01
**Args:** `consensus --index chrM_index.idx --reads filtered_run1/BC01.fastq.gz --min-depth 5 --output consensus_BC01`
**Explanation:** With a minimum depth of 5, positions covered by fewer than 5 reads are masked as `N`, producing a higher-confidence consensus compared to the default threshold of 3.

### Demultiplex a run with 24 barcoded samples
**Args:** `demux --input run1.fastq.gz --sample-sheet samples.tsv --output demux_run1 --force`
**Explanation:** The `--force` flag allows demux to overwrite existing output files; without it, a second run over the same input fails with a file-exists error if output directories already exist from a previous attempt.

### Call variants for a single sample against the indexed reference
**Args:** `variants --index chrM_index.idx --reads filtered_run1/BC01.fastq.gz --min-af 0.05 --output variants_BC01.vcf`
**Explanation:** Setting `--min-af 0.05` includes low-frequency alleles down to 5% abundance, which is useful for detecting mixed infections or heteroplasmy in mitochondrial datasets.

### Run the full pipeline (filter, demux, consensus, variants) in one command
**Args:** `run --sample-sheet samples.tsv --index chrM_index.idx --input run1.fastq.gz --min-quality 8 --min-length 100 --min-depth 3 --output full_results`
**Explanation:** The `run` subcommand chains all four steps automatically, reducing the risk of forgetting to apply consistent filter parameters across separate demux, consensus, and variant steps.