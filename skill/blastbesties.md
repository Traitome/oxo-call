---
name: blastbesties
category: sequence_alignment
description: Wrapper tool for BLAST searches that identifies optimal database matches and best reciprocal hits. Processes nucleotide and protein sequences against reference databases with configurable scoring parameters and multiple output formats.
tags:
  - blast
  - sequence alignment
  - homology search
  - bioinformatics
  - database search
author: AI-generated
source_url: https://www.ncbi.nlm.nih.gov/books/NBK21091/
---

## Concepts

- **BLAST Algorithm Variants**: blastbesties supports multiple algorithm modes depending on query and database types: blastn (nucleotide-to-nucleotide), blastp (protein-to-protein), blastx (translated nucleotide-to-protein), tblastn (protein-to-translated nucleotide), and tblastx (translated nucleotide-to-translated nucleotide). Selecting the correct algorithm is critical—using nucleotide query against a protein database or vice versa will produce no meaningful results.

- **Input Database Preprocessing**: Reference databases must be formatted with makeblastdb or equivalent before searching. The database must match the query type (nucleotide queries require nucleotide databases, protein queries require protein databases). Unformatted FASTA files will cause immediate execution failure without informative error messages.

- **Output Format Modes**: Results can be generated in several formats including tabular (with customizable columns), XML (for programmatic parsing), HTML (for interactive browsing), and ASN.1 (for archive storage). Tabular output with custom columns is most efficient for downstream scripting pipelines; default output includes query ID, subject ID, percent identity, alignment length, and E-value.

- **e-value Threshold Filtering**: The statistical significance threshold (evalue parameter) defaults to 10.0 but should be adjusted based on query length and database size. Shorter queries or larger databases require stricter (lower) e-value thresholds to avoid false positives; an E-value of 0.001 is appropriate for short sequences while 0.05 may suffice for longer queries against small databases.

- **Scoring Matrix Selection**: Protein searches use substitution matrices (BLOSUM62 by default, or PAM70, PAM250) that define amino acid match/mismatch scores. Nucleotide searches use match rewards and mismatch penalties (default: +2/-3). Using an inappropriate matrix for sequence composition will yield suboptimal alignments or miss true homologs.

## Pitfalls

- **Database Type Mismatch**: Running a nucleotide query against a protein database (or the reverse) produces zero results with no error. Users often assume the tool failed when the issue is fundamental incompatibility between query sequence type and database分子. Always verify query type using tools like `blastdbcmd -dbtype nucl` before execution.

- **Ignoring the E-value Caveat**: High-identity matches with poor (high) E-values can still be biologically meaningless, especially for short queries in large databases. A 30% identity match across 20 amino acids might achieve E-value 0.001 by chance alone. Always examine both percent identity and alignment length in context—not just E-value alone.

- **Unformatted Input Sequences**: FASTA headers containing special characters (pipe symbols, brackets, excessive whitespace) cause parsing failures or silent truncation. Sequences with internal stop codons (*) or non-standard amino acid letters (B, J, Z, X) without proper handling may be excluded from searches. Clean input sequences before processing.

- **Missing.task Definition**: Omitting the task parameter forces the default algorithm regardless of query characteristics. For nucleotide queries, this defaults to megablast (optimized for highly similar sequences) when more sensitive settings (blastn) may be needed to detect divergent homologs. Explicitly setting task improves recall for distant matches.

- **Overlooking the Low-Complexity Filter**: By default, seg filtering masks low-complexity regions causing legitimate biologically meaningful repeats to be ignored. For repetitive protein domains or nucleotide microsatellites, disabling filtering may be necessary to capture full alignment, but this increases false positive rates substantially.

## Examples

### Search a protein query against a protein database using BLASTP
**Args:** -query protein.fasta -db swissprot -outfmt 11 -out results.asn -evalue 0.001 -task blastp
**Explanation:** Uses BLASTP algorithm with strict E-value threshold of 0.001 against SwissProt protein database, outputting ASN.1 format suitable for archive storage.

### Perform nucleotide search with tabular output
**Args:** -query gene.fasta -db nt -outfmt 6 -out hits.tsv -evalue 0.01 -qcov_hspper 50
**Explanation:** Searches nucleotide database with tabular output (fmt 6) requiring 50% query coverage per HSP to filter partial alignments.

### Run fast local alignment with megablast
**Args:** -query refseq.fasta -db nr -outfmt 0 -out results.txt -task megablast -reward 1 -penalty -3
**Explanation:** Executes megablast optimized for highly similar sequences with standard nucleotide scoring to find close matches.

### Translate and search protein database
**Args:** -query genomic_seq.fasta -db nr -outfmt 7 -out xml_results.xml -evalue 0.0001 -task tblastn
**Explanation:** Uses tblastn to translate nucleotide query in six frames and search against protein database for distant homologs.

### List available databases without running search
**Args:** -dump_databases
**Explanation:** Lists all formatted databases available in the default directory, useful for verifying database existence before constructing search commands.

### Apply custom scoring matrix for protein alignment
**Args:** -query protein.fasta -db pfam -outfmt 6 -out matches.tsv -matrix PAM250 -gapopen 11 -gapextend 1
**Explanation:** Uses PAM250 substitution matrix (more sensitive for distant relationships) with specified gap opening and extension penalties.

### Disable low-complexity filtering
**Args:** -query repeat_protein.fasta -db cdd -outfmt 6 -out results.tsv -seg no -evalue 10.0
**Explanation:** Disables low-complexity filtering to retain alignments in repetitive sequence regions while accepting higher E-value threshold.

### Extract subject sequences from alignment
**Args:** -query seq.fasta -db uniprot -outfmt 15 -out seqs.fasta -max_target_seqs 100
**Explanation:** Outputs subject sequences (fmt 15) rather than alignments, extracting top 100 hits directly for downstream analysis.

### Calculate statistical significance manually
**Args:** -query query.fasta -db nr -outfmt 10 -out stats.out -evalue 1000 -dbsize 50000000 -searchsp 200000000
**Explanation:** Uses artificially high E-value threshold with explicit database and search space sizes for custom significance calculations.