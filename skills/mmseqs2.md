---
name: mmseqs2
category: sequence-search
description: Ultra-fast protein and nucleotide sequence search and clustering
tags: [protein, clustering, search, homology, database, metagenomics]
author: oxo-call built-in
source_url: "https://github.com/soedinglab/MMseqs2/wiki"
---

## Concepts

- MMseqs2 works on internal databases created with createdb; use easy-* wrappers to skip manual DB steps for common workflows.
- easy-search / easy-cluster / easy-linclust accept FASTA directly and handle DB creation, search, and result conversion internally.
- Sensitivity is controlled with -s (1=fast, 7.5=default, up to 7.5 for exhaustive); higher values are slower but find more distant homologs.
- easy-linclust is linear-time clustering suitable for very large datasets (billions of sequences); easy-cluster is slower but more sensitive.
- Output format for convertalis mimics BLAST tabular (--format-mode 0); mode 4 gives a TSV with headers, mode 2 gives SAM.
- All MMseqs2 commands require a temporary directory (--tmp-dir or positional tmp arg); use a fast local disk path for best performance.

## Pitfalls

- Forgetting the tmp directory argument causes an error — always supply a writable tmp path as the last positional argument.
- easy-cluster and easy-linclust use representative sequences, not consensus; use result2repseq to extract representative FASTAs.
- Coverage parameters (--min-seq-id, -c, --cov-mode) default to lenient values; tighten them for species-level clustering.
- --cov-mode 0 computes coverage over query and target; mode 1 is query-only, mode 2 is target-only — choose appropriately.
- MMseqs2 databases are not cross-compatible with different versions; regenerate DBs after upgrading.
- Protein searches against nucleotide DBs require translated search mode (--search-type 2 or 3); mixing types without this flag gives empty results.

## Examples

### search protein FASTA against UniRef50 and output BLAST tabular results
**Args:** `easy-search query.fasta uniref50.fasta results.m8 tmp --format-mode 0 --threads 16 -s 7.5`
**Explanation:** -s 7.5 is max sensitivity; --format-mode 0 gives BLAST-style TSV; tmp is the temp directory

### cluster protein sequences at 90% identity
**Args:** `easy-cluster proteins.fasta cluster_90 tmp --min-seq-id 0.9 -c 0.8 --cov-mode 0 --threads 16`
**Explanation:** --min-seq-id 0.9 sets 90% identity threshold; -c 0.8 requires 80% coverage of both query and target

### fast linear-time clustering of large metagenomic protein set at 50% identity
**Args:** `easy-linclust proteins.fasta cluster_50 tmp --min-seq-id 0.5 -c 0.8 --threads 32`
**Explanation:** easy-linclust scales linearly — preferred for >10M sequences; same threshold flags as easy-cluster

### build an MMseqs2 database from a FASTA file
**Args:** `createdb proteins.fasta proteinsDB`
**Explanation:** creates proteinsDB, proteinsDB.index, etc.; required before using search/cluster subcommands directly

### search one MMseqs2 DB against another and convert results to TSV
**Args:** `search queryDB targetDB resultDB tmp -s 6 --threads 16 && convertalis queryDB targetDB resultDB results.tsv --format-mode 4`
**Explanation:** search writes binary resultDB; convertalis converts to human-readable TSV with column headers (--format-mode 4)

### extract representative sequences from a cluster result
**Args:** `result2repseq proteinsDB proteinsDB cluster_result repseqDB && convert2fasta repseqDB representatives.fasta`
**Explanation:** result2repseq pulls the cluster representative; convert2fasta writes FASTA output

### perform translated nucleotide-to-protein search
**Args:** `easy-search reads.fasta proteins.fasta hits.m8 tmp --search-type 2 --threads 16`
**Explanation:** --search-type 2 translates the query nucleotides in all 6 frames before searching against a protein target
