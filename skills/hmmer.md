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
- Pfam HMMs have three curated thresholds: GA (gathering), TC (trusted), NC (noise); use --cut_ga for Pfam-standard annotation.
- --cut_ga uses the gathering threshold (most commonly used); --cut_tc is stricter; --cut_nc is most permissive.
- -A saves multiple alignment of all hits; useful for building new MSAs from search results.
- --max disables all heuristic filters for maximum sensitivity (much slower).
- --F1, --F2, --F3 control the three-stage heuristic filter thresholds (MSV, Viterbi, Forward).
- -Z sets effective database size for accurate E-value calculation; important for custom databases.

## Pitfalls

- hmmsearch queries an HMM against a sequence database; hmmscan queries a sequence against an HMM database — opposite directions.
- Press the HMM database with hmmpress before hmmscan: hmmpress Pfam-A.hmm creates index files.
- Without --tblout/--domtblout, HMMER only outputs human-readable text that's hard to parse.
- E-value thresholds need to be appropriate — default 10 is too permissive; use 1e-5 for most searches.
- HMMER sequence IDs must not contain special characters — spaces in headers cause parsing issues.
- For large databases, use -Z to specify the effective database size for accurate E-value calculation.
- --cut_ga, --cut_tc, --cut_nc only work if the HMM file contains these curated thresholds; most Pfam HMMs have them.
- --max disables all heuristics and is extremely slow; only use for small queries or when sensitivity is critical.
- Default --cpu is 2 for hmmsearch; explicitly set --cpu for parallel processing.
- hmmscan requires the database to be pressed first; hmmsearch does not require pressing.
- -A alignment output can be very large; use with caution on large databases.
- Domain scores (--domE, --domT) are conditional on sequence already passing sequence thresholds.

## Examples

### search a protein database against Pfam HMM profiles (domain annotation)
**Args:** `hmmscan --cpu 8 --tblout pfam_hits.tbl --domtblout pfam_domains.tbl -E 1e-5 Pfam-A.hmm proteins.faa > pfam_output.txt`
**Explanation:** hmmscan subcommand for sequence vs profile DB; --cpu 8 threads; --tblout pfam_hits.tbl per-sequence hits; --domtblout pfam_domains.tbl per-domain hits; -E 1e-5 E-value cutoff; Pfam-A.hmm HMM database; proteins.faa input sequences; output to pfam_output.txt

### search a protein HMM profile against a sequence database
**Args:** `hmmsearch --cpu 8 --tblout hits.tbl --domtblout domain_hits.tbl -E 1e-10 gene_family.hmm sequences.faa > hmmsearch_out.txt`
**Explanation:** hmmsearch subcommand for profile vs sequence DB; --cpu 8 threads; --tblout hits.tbl per-sequence hits; --domtblout domain_hits.tbl per-domain hits; -E 1e-10 E-value cutoff; gene_family.hmm HMM profile; sequences.faa sequence database; output to hmmsearch_out.txt

### build a profile HMM from a multiple sequence alignment
**Args:** `hmmbuild --cpu 8 gene_family.hmm aligned_sequences.sto`
**Explanation:** hmmbuild subcommand; --cpu 8 threads; gene_family.hmm output HMM profile; aligned_sequences.sto input Stockholm-format MSA

### press Pfam database for hmmscan indexing
**Args:** `hmmpress Pfam-A.hmm`
**Explanation:** hmmpress subcommand; Pfam-A.hmm HMM database to index; creates .h3i, .h3m, .h3f, .h3p index files required before running hmmscan

### search proteins with phmmer (BLAST-like single sequence query)
**Args:** `phmmer --cpu 8 --tblout phmmer_hits.tbl -E 1e-5 query_protein.faa target_database.faa > phmmer_out.txt`
**Explanation:** phmmer subcommand; --cpu 8 threads; --tblout phmmer_hits.tbl per-sequence hits; -E 1e-5 E-value cutoff; query_protein.faa query sequence; target_database.faa target sequences; output to phmmer_out.txt

### use Pfam gathering threshold for standard annotation
**Args:** `hmmscan --cpu 8 --cut_ga --tblout pfam_ga.tbl --domtblout pfam_ga_dom.tbl Pfam-A.hmm proteins.faa > pfam_ga.txt`
**Explanation:** hmmscan subcommand; --cpu 8 threads; --cut_ga uses Pfam's curated gathering threshold; --tblout pfam_ga.tbl per-sequence hits; --domtblout pfam_ga_dom.tbl per-domain hits; Pfam-A.hmm HMM database; proteins.faa input sequences; output to pfam_ga.txt

### use trusted cutoff for high-confidence annotation
**Args:** `hmmscan --cpu 8 --cut_tc --tblout pfam_tc.tbl Pfam-A.hmm proteins.faa > pfam_tc.txt`
**Explanation:** hmmscan subcommand; --cpu 8 threads; --cut_tc uses stricter trusted cutoff; --tblout pfam_tc.tbl per-sequence hits; Pfam-A.hmm HMM database; proteins.faa input sequences; output to pfam_tc.txt

### save multiple alignment of significant hits
**Args:** `hmmsearch --cpu 8 -A hits_alignment.sto --tblout hits.tbl -E 1e-5 query.hmm database.faa > hmmsearch_out.txt`
**Explanation:** hmmsearch subcommand; --cpu 8 threads; -A hits_alignment.sto saves Stockholm-format alignment; --tblout hits.tbl per-sequence hits; -E 1e-5 E-value cutoff; query.hmm HMM profile; database.faa sequence database; output to hmmsearch_out.txt

### run with maximum sensitivity (disable heuristics)
**Args:** `hmmsearch --cpu 8 --max --tblout max_hits.tbl -E 1e-3 query.hmm database.faa > max_out.txt`
**Explanation:** hmmsearch subcommand; --cpu 8 threads; --max disables all heuristic filters for maximum sensitivity; --tblout max_hits.tbl per-sequence hits; -E 1e-3 E-value cutoff; query.hmm HMM profile; database.faa sequence database; output to max_out.txt

### set effective database size for accurate E-values
**Args:** `hmmsearch --cpu 8 -Z 1000000 --tblout hits.tbl -E 1e-5 query.hmm database.faa > hmmsearch_out.txt`
**Explanation:** hmmsearch subcommand; --cpu 8 threads; -Z 1000000 sets effective database size to 1M sequences; --tblout hits.tbl per-sequence hits; -E 1e-5 E-value cutoff; query.hmm HMM profile; database.faa sequence database; output to hmmsearch_out.txt

### use bit score threshold instead of E-value
**Args:** `hmmsearch --cpu 8 --tblout hits.tbl -T 25 query.hmm database.faa > hmmsearch_out.txt`
**Explanation:** hmmsearch subcommand; --cpu 8 threads; --tblout hits.tbl per-sequence hits; -T 25 sets bit score threshold to 25; query.hmm HMM profile; database.faa sequence database; output to hmmsearch_out.txt

### search with domain-specific E-value threshold
**Args:** `hmmsearch --cpu 8 --tblout hits.tbl --domtblout domains.tbl -E 1e-5 --domE 1e-3 query.hmm database.faa > hmmsearch_out.txt`
**Explanation:** hmmsearch subcommand; --cpu 8 threads; --tblout hits.tbl per-sequence hits; --domtblout domains.tbl per-domain hits; -E 1e-5 sequence E-value cutoff; --domE 1e-3 domain E-value cutoff; query.hmm HMM profile; database.faa sequence database; output to hmmsearch_out.txt
