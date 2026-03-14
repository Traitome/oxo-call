---
name: hmmer
category: sequence-utilities
description: Profile hidden Markov model-based search for protein families and functional domain annotation
tags: [protein, hmm, domain, annotation, pfam, homology, protein-family, database-search]
author: oxo-call built-in
source_url: "http://hmmer.org/"
---

## Concepts

- HMMER uses profile HMMs to search for protein families; key programs: hmmbuild, hmmsearch, hmmscan, phmmer, nhmmer.
- hmmsearch: search a profile HMM against a sequence database; hmmscan: search a sequence against a profile database (Pfam).
- hmmbuild: build an HMM from a multiple sequence alignment (MSA); hmmalign: align sequences to an HMM.
- Use -o for main output; --tblout for per-sequence tabular output; --domtblout for per-domain tabular output.
- Use -E for E-value cutoff (sequence-level); --domE for domain-level E-value cutoff.
- Use --cpu N for multi-threading; HMMER is slower than BLAST but more sensitive for distant homologs.
- Pfam database (Pfam-A.hmm) is the most widely used profile database for domain annotation.
- phmmer is BLASTP-like search using per-sequence HMMs; hmmbuild+hmmsearch is for profile-based search.

## Pitfalls

- hmmsearch queries an HMM against a sequence database; hmmscan queries a sequence against an HMM database — opposite directions.
- Press the HMM database with hmmpress before hmmscan: hmmpress Pfam-A.hmm creates index files.
- Without --tblout/--domtblout, HMMER only outputs human-readable text that's hard to parse.
- E-value thresholds need to be appropriate — default 10 is too permissive; use 1e-5 for most searches.
- HMMER sequence IDs must not contain special characters — spaces in headers cause parsing issues.
- For large databases, use -Z to specify the effective database size for accurate E-value calculation.

## Examples

### search a protein database against Pfam HMM profiles (domain annotation)
**Args:** `hmmscan --cpu 8 --tblout pfam_hits.tbl --domtblout pfam_domains.tbl -E 1e-5 Pfam-A.hmm proteins.faa > pfam_output.txt`
**Explanation:** hmmscan for sequence vs profile DB; --tblout per-sequence hits; --domtblout per-domain hits; -E cutoff

### search a protein HMM profile against a sequence database
**Args:** `hmmsearch --cpu 8 --tblout hits.tbl --domtblout domain_hits.tbl -E 1e-10 gene_family.hmm sequences.faa > hmmsearch_out.txt`
**Explanation:** hmmsearch for profile vs sequence DB; opposite direction from hmmscan

### build a profile HMM from a multiple sequence alignment
**Args:** `hmmbuild --cpu 8 gene_family.hmm aligned_sequences.sto`
**Explanation:** input is Stockholm (.sto) or other MSA format; outputs HMM profile for hmmsearch

### press Pfam database for hmmscan indexing
**Args:** `hmmpress Pfam-A.hmm`
**Explanation:** creates Pfam-A.hmm.h3i, .h3m, .h3f, .h3p index files required before running hmmscan

### search proteins with phmmer (BLAST-like single sequence query)
**Args:** `phmmer --cpu 8 --tblout phmmer_hits.tbl -E 1e-5 query_protein.faa target_database.faa > phmmer_out.txt`
**Explanation:** phmmer uses query sequence directly without pre-building HMM; similar to BLASTP
