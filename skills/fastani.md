---
name: fastani
category: comparative-genomics
description: FastANI — fast pairwise Average Nucleotide Identity estimation for microbial genomes; uses MinHash-like sketching for high-throughput comparisons
tags: [fastani, ani, comparative-genomics, microbial, prokaryote, taxonomy, species-boundary, genome, mash, phylogeny]
author: oxo-call built-in
source_url: "https://github.com/ParBLiSS/FastANI"
---

## Concepts
- FastANI estimates Average Nucleotide Identity (ANI) between whole-genome assemblies; the species boundary is typically 95% ANI.
- Input: assembled genome FASTA files (one genome per file); accepts both uncompressed and gzipped FASTA.
- Output: tab-separated file with columns: query path, reference path, ANI (%), number of bidirectional fragment mappings, total query fragments.
- `--query` / `-q` specifies the query genome; `--refList` / `-rl` specifies a text file listing one reference genome path per line.
- `--queryList` / `-ql` and `--refList` / `-rl` enable all-vs-all or many-vs-many comparisons in a single run.
- `--fragLen` (default 3000 bp): the length of fragments used for alignment; shorter values increase sensitivity for highly divergent genomes.
- `--minFraction` (default 0.2): minimum fraction of the query genome that must align to compute an ANI estimate; lower values keep more distant comparisons.
- `--threads` / `-t`: number of parallel threads; FastANI parallelises over query–reference pairs.
- ANI < 70% returns no estimate (below the reliable detection threshold); for such distances, use MASH or other sketch-based tools.
- All-vs-all matrix: run with `--queryList` = `--refList` to compare all genomes against each other.
- Output ANI values are reported once per ordered pair (query → reference); symmetrised results require post-processing.
- `--visualize` outputs mapping coordinates for visualization (single genome pair only).
- `--matrix` outputs ANI as lower triangular matrix in PHYLIP format for phylogenetic analysis.
- `--kmer` / `-k` sets k-mer size (default 16, max 16); smaller values increase sensitivity.
- `--maxRatioDiff` filters spurious matches; default 10.0, lower values exclude HGT regions.

## Pitfalls
- FastANI is designed for complete or near-complete prokaryotic genomes (>1 Mb); results for highly fragmented assemblies or short sequences may be unreliable.
- The `--minFraction` filter can silently drop pairs with low coverage; if many pairs are missing from the output, lower `--minFraction` and check.
- Very large all-vs-all jobs (thousands of genomes) produce large output files and require significant memory; pre-filter genomes by MASH distance to reduce the pairwise space.
- Do NOT use FastANI on metagenome-assembled genomes (MAGs) with >50% completeness issues; use checkm2 to verify completeness first.
- Input genome paths in `--queryList` and `--refList` must be absolute or consistently relative to the working directory where FastANI is called.
- using compressed (gzip) FASTA without ensuring FastANI was compiled with zlib support may cause silent failures or wrong results.
- `--visualize` only works for single genome pair comparisons; using with lists will cause error.
- ANI < 70% produces no output line; this is expected behavior, not an error.
- `--matrix` output is lower triangular; full matrix requires post-processing to symmetrize.
- Default `--fragLen` 3000bp may be too large for small genomes (viruses, plasmids); use 500-1000bp instead.

## Examples

### compute ANI between two genomes
**Args:** `--query query.fasta --ref reference.fasta --output result.txt`
**Explanation:** fastANI command; --query query.fasta query genome; --ref reference.fasta reference genome; --output result.txt output file; computes ANI from query.fasta against reference.fasta; output has ANI%, fragment counts; 95% threshold defines species boundary

### all-vs-all comparison of a genome collection
**Args:** `--queryList genomes.list --refList genomes.list --output all_vs_all.tsv --threads 16`
**Explanation:** fastANI command; --queryList genomes.list query genome list; --refList genomes.list reference genome list (same file for all-vs-all); --output all_vs_all.tsv output file; --threads 16 parallelises across pairs; genomes.list has one genome path per line; computes ANI for all ordered pairs

### one-to-many comparison: one query vs many references
**Args:** `--query new_isolate.fa --refList reference_db.list --output ani_results.tsv --threads 8`
**Explanation:** fastANI command; --query new_isolate.fa query genome; --refList reference_db.list reference genome list; --output ani_results.tsv output file; --threads 8 parallel threads; compares a single new isolate against all references in the list; useful for species identification or novelty detection

### filter results to show only high-ANI pairs (same species)
**Args:** `--queryList genomes.list --refList genomes.list --output raw.tsv --minFraction 0.5 --threads 16`
**Explanation:** fastANI command; --queryList genomes.list query genome list; --refList genomes.list reference genome list; --output raw.tsv output file; --minFraction 0.5 requires 50% of the query to align; --threads 16 parallel threads; increases reliability by filtering out low-coverage comparisons

### adjust fragment length for highly similar genomes
**Args:** `--query q.fa --ref r.fa --output result.txt --fragLen 1500`
**Explanation:** fastANI command; --query q.fa query genome; --ref r.fa reference genome; --output result.txt output file; --fragLen 1500 uses shorter fragments; increases sensitivity for genomes with many repetitive regions or large insertions/deletions

### build a genome list file for batch comparison
**Args:** `find /genomes -name '*.fna' > genomes.list`
**Explanation:** find command; /genomes directory; -name '*.fna' finds all .fna genome files; > genomes.list output to list file; creates a text file listing all .fna genome paths; use this list with --queryList or --refList for batch FastANI jobs

### check FastANI version
**Args:** `--version`
**Explanation:** fastANI command; --version flag; prints the installed FastANI version; important for reproducibility reporting in publications

### generate PHYLIP matrix for phylogenetic analysis
**Args:** `--queryList genomes.list --refList genomes.list --output ani.tsv --matrix --threads 16`
**Explanation:** fastANI command; --queryList genomes.list query genome list; --refList genomes.list reference genome list; --output ani.tsv output file; --matrix creates additional .matrix file in PHYLIP format; --threads 16 parallel threads; use for NJ tree construction with tools like QuickTree

### visualize genome alignment regions
**Args:** `--query query.fa --ref ref.fa --output result.txt --visualize`
**Explanation:** fastANI command; --query query.fa query genome; --ref ref.fa reference genome; --output result.txt output file; --visualize outputs mapping coordinates to result.txt.visual; shows conserved regions between two genomes

### compare small genomes (viruses/plasmids)
**Args:** `--query virus1.fa --ref virus2.fa --output result.txt --fragLen 500`
**Explanation:** fastANI command; --query virus1.fa query virus genome; --ref virus2.fa reference virus genome; --output result.txt output file; --fragLen 500 uses shorter fragments for small genomes; default 3000bp is too large for viral genomes

### increase sensitivity for divergent genomes
**Args:** `--query query.fa --ref ref.fa --output result.txt --kmer 12 --minFraction 0.1`
**Explanation:** fastANI command; --query query.fa query genome; --ref ref.fa reference genome; --output result.txt output file; --kmer 12 smaller k-mer increases sensitivity; --minFraction 0.1 keeps more distant comparisons

### filter horizontal gene transfer regions
**Args:** `--queryList genomes.list --refList genomes.list --output ani.tsv --maxRatioDiff 0.05 --threads 16`
**Explanation:** fastANI command; --queryList genomes.list query genome list; --refList genomes.list reference genome list; --output ani.tsv output file; --maxRatioDiff 0.05 excludes regions with abnormal coverage ratios; --threads 16 parallel threads; helps filter HGT events
