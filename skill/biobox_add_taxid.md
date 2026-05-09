---
name: biobox_add_taxid
category: taxonomy-annotation
description: A tool for adding NCBI taxonomic identifiers to sequence data by mapping sequence headers or content against reference taxonomy databases. Supports FASTA, FASTQ, and BAM input formats with flexible output options.
tags: [taxonomy, taxid, ncbi, bioinformatics, annotation, sequence-classification]
author: AI-generated
source_url: https://github.com/biobox-tools/biobox_add_taxid
---

## Concepts

- Input sequences in FASTA or FASTQ format are parsed and their headers are matched against a provided taxonomy database (default: NCBI GTDB) to assign taxonomy identifiers. The tool extracts existing identifiers from sequence headers (e.g., `>seq1|taxid:12345`) or uses exact sequence matching when header information is absent.

- Output can be in the same format as input with updated headers containing taxids, or in a tab-separated mapping file that associates sequence names with their assigned taxids, organism names, and confidence scores. The mapping output is useful for downstream analysis pipelines that need structured taxonomy annotations.

- The tool supports multiple matching strategies: header-based extraction (fast), exact sequence identity search (requires reference database), and approximate matching using k-mer based similarity with configurable k-mer size (default: 11) and minimum identity threshold (default: 97%). Different strategies trade off speed versus accuracy for sequences without pre-existing taxonomy annotations.

- Database files must be pre-built using the companion `biobox_taxdb_build` binary. The tool cannot function without a valid taxonomy database. Supported database formats include DBM (default) and BDB for faster random access on large datasets.

- The tool respects NCBI taxonomic hierarchy: when a sequence cannot be matched to a specific species-level taxid, it automatically assigns the closest available parent taxid (genus, family, etc.) based on the `--taxonomic-rank` flag setting.

## Pitfalls

- Not providing a taxonomy database via `--database` or `--taxdb` defaults to online NCBI lookup, which fails if there is no network connectivity or if NCBI's API is rate-limited. Always build a local database for batch processing to avoid API throttling errors that cause incomplete annotations.

- Sequences with duplicate names in the input file will produce ambiguous mappings in the output, as each duplicate receives the same taxid assignment. This leads to downstream tools potentially misinterpreting which sequence corresponds to which taxid in the mapping file.

- Using `--min-identity` values below 90% may assign incorrect taxids for highly similar sequences from different species, particularly in conserved regions like ribosomal RNA genes. This contamination propagates error to all downstream analyses that rely on accurate taxonomy annotations.

- Input files compressed with gzip (`.gz` extension) are not automatically detected; you must explicitly decompress them first or use the companion `gunzip` utility, otherwise the tool reports a parse error and exits with non-zero status.

- The `--output-format json` option produces JSON that includes confidence scores only when `--include-confidence` is specified; omitting this flag results in JSON output lacking score fields, making it impossible to evaluate the reliability of taxid assignments in automated pipelines.

## Examples

### Add taxids to sequences using header-based extraction
**Args:** `--input sequences.fasta --output annotated.fasta --method header`
**Explanation:** This extracts any existing taxid information embedded in sequence headers (e.g., `|taxid:9606|`) and writes them to the output file with normalized formatting, useful for standardizing taxonomy annotations across datasets from different sources.

### Build a custom taxonomy database from a FASTA reference
**Args:** `--reference ref_sequences.fasta --output taxdb.dbm --kmer-size 13`
**Explanation:** Creates a searchable taxonomy database from a custom FASTA file containing reference sequences with taxonomy annotations in their headers. The k-mer size affects both build time and subsequent matching sensitivity, with larger values being more specific but requiring longer sequences.

### Assign taxids using approximate sequence matching
**Args:** `--input unknown_seqs.fasta --database taxdb.dbm --output mapping.tsv --method kmer --min-identity 95`
**Explanation:** Performs k-mer based similarity search against the database, assigning the taxid of the best matching reference sequence when identity exceeds 95%. This enables annotation of sequences that lack pre-existing taxonomy information.

### Output taxonomy mappings in JSON format with confidence scores
**Args:** `--input sequences.fasta --database taxdb.dbm --output results.json --output-format json --include-confidence`
**Explanation:** Produces JSON output containing sequence identifiers, assigned taxids, organism names, and similarity confidence scores. The confidence scores enable downstream tools to filter or weight results based on assignment reliability.

### Limit taxonomy assignment to genus level or higher
**Args:** `--input sequences.fasta --database taxdb.dbm --output annotated.fasta --taxonomic-rank genus --method hybrid`
**Explanation:** Uses hybrid matching (header first, then k-mer if no header taxid) but assigns only genus-level taxids even when species-level matches exist. Useful when analysis requires broad taxonomic grouping rather than precise species identification.

### Process multiple input files with parallel threads
**Args:** `--input-dir ./fastas/ --output-dir ./annotated/ --threads 8 --method header`
**Explanation:** Processes all FASTA files in the input directory using 8 parallel threads, writing annotated files to the output directory with preserved filenames. Directory processing is more efficient than scripting multiple single-file commands for batch workflows.