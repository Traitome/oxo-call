---
name: chap
category: sequence_alignment
description: A fast alignment tool for comparing protein or nucleotide sequences against indexed databases, using compressed hash tables for efficient memory usage and rapid lookup.
tags: [bioinformatics, sequence-analysis, alignment, indexing, genomics, proteomics]
author: AI-generated
source_url: https://github.com/bioinformatics-chap/chap
---

## Concepts

- **Index-based alignment**: CHAP uses pre-built index files (created by `chap-build`) to accelerate sequence searches, converting a target database into a compact lookup structure stored in multiple files (.cts, .bkt, .amb).
- **Hash table format**: The index consists of three key file types - .cts (contains the reference sequences with position metadata), .bkt (backtrack information for fast traversal), and .amb (ambiguous base/amino acid position markers).
- **Input formats**: CHAP accepts FASTA (.fa, .fasta) and FASTQ (.fq, .fastq) input for queries, and supports both nucleotide and protein sequences by detecting the alphabet used.
- **Reporting modes**: Three output modes exist - 0 for map (shows best mapping), 1 for map+unmapped (includes failed sequences), and 2 for map+ref (adds reference details).
- **Seed-and-extend**: The algorithm finds exact-match seeds using the hash index, then extends alignments with a banded dynamic programming approach to verify compatibility.

## Pitfalls

- **Mismatched index type**: Using a nucleotide index (built with default parameters) for protein queries produces no alignments because the k-mer length and hash functions differ; rebuild with `--protein` flag.
- **Corrupt or missing index files**: If any of the three index components (.cts, .bkt, .amb) are moved or truncated after building, CHAP fails with a cryptic "index read error" without specifying which file is faulty.
- **Excessive memory consumption**: Setting `-k` (max alignments per read) too high combined with many concurrent queries can exhaust RAM because each alignment stores a separate result record.
- **Wrong reporting mode**: Specifying `--report 2` on large reference databases produces massive output files since every matching position includes full reference context, causing disk space issues and slow downstream parsing.
- **File permission errors**: Building an index in a read-only directory succeeds, but running `chap` later silently skips writing temporary files, leading to incorrect results or truncated output.

## Examples

### Build a nucleotide sequence database index

**Args:** ./reference.fa -t chapref
**Explanation:** Creates three index files (chapref.cts, chapref.bkt, chapref.amb) from the reference FASTA, storing hash table for nucleotide queries; the -t flag specifies the base output name.

### Build a protein sequence database index

**Args:** ./protein_db.fa -t protidx --protein
**Explanation:** Creates an index using protein alphabet (20 amino acids instead of 4 nucleotides) with adjusted k-mer length, enabling subsequent protein similarity searches.

### Align queries against an index with default settings

**Args:** ./queries.fa -x nucleic_idx -S results.sam
**Args:** ./queries.fa -x protein_idx -S prot_results.sam
**Explanation:** Performs standard alignment of input sequences against the pre-built index (-x specifies index), outputting SAM format results; default report mode 0 returns only best mappings.

### Align with multiple alignment candidates per read

**Args:** ./queries.fa -x idx -k 10 -a
**Explanation:** Requests up to 10 potential alignments per query (-k 10) and includes all in output (-a flag); useful for downstream variant calling where secondary alignments matter.

### Output unmapped sequences alongside alignments

**Args:** ./queries.fa -x idx -S out.sam --report 1
**Explanation:** Uses report mode 1 to include both successfully mapped and unmapped sequences, allowing identification of reads with no viable match in the target database.

### Control alignment sensitivity via minimum score threshold

**Args:** ./queries.fa -x idx -S out.sam --min-score 30
**Explanation:** Filters alignments below score 30, reducing false positives in noisy data at cost of missing true low-similarity matches; score scales with alignment length and mismatch count.

### Run with reduced thread count for shared systems

**Args:** ./queries.fa -x idx -S out.sam -p 2
**Explanation:** Limits CHAP to 2 parallel threads (-p 2) instead of default auto-detection, preventing resource contention on multi-user compute nodes.