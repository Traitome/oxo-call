---
name: blastalign
category: Sequence Alignment
description: A command-line tool for aligning two or more sequences using the BLAST algorithm. Performs local pairwise or multiple sequence alignments, outputting results in standard alignment formats (BLAST, SAM, FASTA) for downstream bioinformatics analysis.
tags:
  - sequence-alignment
  - blast
  - local-alignment
  - pairwise-alignment
  - bioinformatics
author: AI-generated
source_url: https://blast.ncbi.nlm.nih.gov/blast/FAQ/CDI.shtml
---

## Concepts

- **Input Format**: Accepts FASTA, FASTQ, or Plain text formats for query and subject sequences. Multi-sequence inputs are supported via file or stdin with one sequence per entry.
- **Output Formats**: Generates alignments in BLAST (-outfmt 0-16), SAM, or FASTA formats. Default is BLAST text format. Use -outfmt 6 for tabular BLAST column output (like BLAST+).
- **Scoring Parameters**: Uses E-value threshold (default 10.0), word size, gap penalties, and substitution matrix (e.g., BLOSUM62 for protein). Adjust via -evalue, -word_size, -gapopen, -gapextend, and -matrix flags.
- **Algorithm Variants**: Supports blastn (nucleotide), blastp (protein), blastx (translated nucleotide vs protein), and tblastn (protein vs translated nucleotide) modes via the -task or explicit program flags.

## Pitfalls

- **Using the wrong -task option**: Selecting blastn for protein sequences (or vice versa) produces meaningless alignments or no hits. Always match the task to your input sequence type.
- **Ignoring the E-value threshold**: Setting -evalue too high (e.g., 100) includes spurious alignments; setting it too low (e.g., 1e-100) may miss valid homologs in divergent sequences.
- **Forgetting to specify both query and subject**: Without -query and -subject (or -infmt inputs), blastalign runs in single-sequence mode, producing no alignment output.
- **Mismatch between input sequence type and scoring matrix**: Using BLOSUM62 with nucleotide inputs causes undefined behavior or errors. Use -matrix NUCL for nucleotide alignments.
- **Omitting -outfmt for scripted pipelines**: Default text output is hard to parse. Use -outfmt 6 (tabular) or -outfmt 5 (XML) for reproducible automated workflows.

## Examples

### Align two nucleotide sequences in FASTA format
**Args:** -query seq1.fasta -subject seq2.fasta -task blastn -evalue 0.001 -outfmt 6 -out results.tsv
**Explanation:** Performs a BLASTN alignment with stringent E-value (0.001), outputting tabular results for easy parsing in downstream scripts.

### Translate a nucleotide query against a protein database
**Args:** -query coding_seq.fasta -subject protein_db.fasta -task blastx -evalue 1e-5 -outfmt 5 -out results.xml
**Explanation:** Uses blastx to translate the query in six frames and align against protein sequences, outputting XML format for structured analysis.

### Align protein sequences using BLOSUM80 scoring
**Args:** -query protein1.fasta -subject protein2.fasta -task blastp -matrix BLOSUM80 -evalue 0.0001 -outfmt 0
**Explanation:** Uses stricter BLOSUM80 matrix for more sensitive protein alignment, reporting results in standard BLAST text format.

### Get alignments via stdin with custom gap penalties
**Args:** -query - -subject db.fasta -gapopen 10 -gapextend 2 -outfmt 6
**Explanation:** Reads query from stdin (single dash), applies custom gap penalties (open=10, extend=2), and outputs tabular format.

### Run a database search with compositional adjustments
**Args:** -query input.fasta -subject nr_db -task blastp -comp_based_stats 1 -evalue 10 -outfmt 6 -out hits.tsv
**Explanation:** Uses compositional adjustment (comp_based_stats 1) to reduce bias, searching against NR database and outputting all hits above E-value 10.

### Retrieve XML output for programmatic parsing
**Args:** -query gene.fasta -subject ref_proteins.fasta -task blastp -evalue 0.01 -outfmt 5 -out xml_output.xml
**Explanation:** Generates XML output suitable for programmatic parsing by scripts, containing detailed alignment annotations.