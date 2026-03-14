---
name: centrifuge
category: metagenomics
description: Rapid and memory-efficient taxonomic classification of sequencing reads using FM-index
tags: [metagenomics, taxonomy, classification, reads, 16S, wgs, kraken]
author: oxo-call built-in
source_url: "https://ccb.jhu.edu/software/centrifuge/manual.shtml"
---

## Concepts

- Centrifuge classifies reads by aligning them to a reference database using an FM-index; it is faster and more memory-efficient than Kraken for large DBs.
- centrifuge-build constructs an index from a FASTA collection with taxonomy metadata; NCBI taxonomy nodes.dmp and names.dmp are required.
- centrifuge outputs two files: a per-read classification TSV and a summary report TSV; centrifuge-kreport converts these to Kraken-style reports.
- Paired-end reads are passed with -1 and -2; single-end with -U; reads can be FASTQ or FASTA, gzipped or uncompressed.
- --report-file writes a per-taxon abundance summary during the run; this is a quick overview but the Kraken-style report has more detail.
- Centrifuge databases are available pre-built from the JHU website (nt, bacteria, archaea, viruses, human); building custom DBs is possible but time-consuming.

## Pitfalls

- Not specifying -x (index prefix) causes centrifuge to fail — always provide the full path prefix to the index files.
- Centrifuge index files (.1.cf, .2.cf, .3.cf) must all be present; a missing file causes a cryptic error at startup.
- --min-hitlen default is 22; raising it (e.g., 30) increases precision but reduces recall for shorter reads.
- Centrifuge does not handle interleaved FASTQ natively; split into separate R1/R2 files before using -1/-2.
- The summary report from centrifuge is not in Kraken format by default; use centrifuge-kreport to get Pavian/Krona-compatible output.
- Memory-mapped index loading (default) can be slow on network filesystems; copy indexes to local SSD before running.

## Examples

### classify paired-end reads against a pre-built bacterial/viral database
**Args:** `centrifuge -x /databases/bv_bacteria -1 R1.fastq.gz -2 R2.fastq.gz -S classifications.tsv --report-file report.tsv -p 16`
**Explanation:** -x sets the database prefix; -S is the classification output; --report-file gives per-taxon summary; -p 16 for threads

### classify single-end reads and generate Kraken-style report
**Args:** `centrifuge -x /databases/nt -U reads.fastq.gz -S classifications.tsv --report-file report.tsv -p 16 && centrifuge-kreport -x /databases/nt classifications.tsv > kraken_report.txt`
**Explanation:** two-step: centrifuge classifies, centrifuge-kreport converts to Kraken-compatible format for Pavian/Krona visualization

### build a custom centrifuge index from bacterial reference genomes
**Args:** `centrifuge-build -p 16 --taxonomy-tree nodes.dmp --name-table names.dmp --conversion-table seqid2taxid.map genomes.fasta custom_db`
**Explanation:** --taxonomy-tree and --name-table are NCBI taxonomy files; --conversion-table maps sequence IDs to taxids

### classify reads with increased sensitivity for viral detection
**Args:** `centrifuge -x /databases/viral -U reads.fastq.gz -S viral_hits.tsv --report-file viral_report.tsv -p 8 --min-hitlen 16`
**Explanation:** --min-hitlen 16 increases sensitivity for short viral reads at the cost of specificity

### convert centrifuge output to Krona-compatible format
**Args:** `centrifuge-kreport -x /databases/bv_bacteria classifications.tsv | cut -f1,3,4,5,6 > krona_input.txt && ktImportTaxonomy krona_input.txt -o taxonomy.html`
**Explanation:** centrifuge-kreport produces Kraken-format; ktImportTaxonomy from KronaTools creates interactive HTML

### remove human reads by classifying against human genome and excluding matches
**Args:** `centrifuge -x /databases/hg38 -1 R1.fastq.gz -2 R2.fastq.gz -S human_classifications.tsv -p 16 --un-conc non_human_%.fastq.gz`
**Explanation:** --un-conc writes paired reads that did NOT classify against the human database; useful for host depletion
