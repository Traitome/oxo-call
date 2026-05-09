---
name: bifrost-httr
category: Graph-Based Sequence Analysis
description: Query a Bifrost colored de Bruijn graph by traversing paths with reference or query sequences. Finds matching k-mer paths, reports genomic coordinates, and outputs alignment-style results for variant analysis.
tags:
- de-bruijn-graph
- k-mer
- sequence-query
- graph-traversal
- variant-detection
- genomics
author: AI-generated
source_url: https://github.com/pmelsted/bifrost
---

## Concepts

- **Colored de Bruijn Graph Model**: Bifrost represents genomic data as a colored de Bruijn graph where nodes are k-mers and edges connect overlapping (k-1)-mers. Sequences are mapped by traversing the graph following k-mer overlaps.
- **Input Graph Format**: bifrost-httr requires a pre-built Bifrost graph file (`.bfg` format) created via `bifrost build` or `bifrost buildGT`. The graph contains all k-mer sequences and their connectivity information.
- **Query Sequence Format**: Query sequences are provided as FASTA or FASTQ files. The tool finds paths through the graph that represent each query sequence, reporting any mismatches or branching points encountered during traversal.
- **Output Options**: Results include path coordinates, node sequences, coverage per color, and optionally graph topology for matched regions. Output formats include plain text, BED, and GPS formats for downstream analysis.

## Pitfalls

- **Using an Unbuilt or Empty Graph**: Attempting to query with bifrost-httr on a graph that was not properly built with sufficient input sequences will yield no results. Always verify the graph file exists and contains k-mers using `bifrost view` before querying.
- **Mismatched k-mer Size**: If the query sequences have a different k-mer size than the graph was built with, the traversal will fail or produce errors. The k-mer size must match exactly between graph construction and querying.
- **Large Query Files Without Memory Optimization**: Querying very large FASTA files without specifying appropriate memory flags can cause excessive RAM usage. Use `--max-mem` or stream queries in batches to prevent system crashes.

## Examples

### Query a single reference sequence against a built graph
**Args:** -g graph.bfg -q reference.fa -o query_output.txt
**Explanation:** Maps the reference sequence through the colored de Bruijn graph by traversing overlapping k-mers and outputs matching path information to the specified file.

### Query multiple sequences and output BED format
**Args:** -g graph.bfg -q reads.fq -o matches.bed --bed
**Explanation:** Processes all sequences in the FASTQ file against the graph and writes genomic coordinates in BED format for visualization or downstream variant calling.

### Limit traversal to specific colors only
**Args:** -g graph.bfg -q query.fa -o output.txt --colors 1,3,5
**Explanation:** Restricts graph traversal to only consider paths present in colors 1, 3, and 5, useful when focusing on specific samples or populations in a multi-sample dataset.

### Set a minimum path coverage threshold
**Args:** -g graph.bfg -q query.fa -o output.txt --min-cov 5
**Explanation:** Only reports paths where the k-mer coverage across all colors meets or exceeds the threshold of 5, filtering out low-frequency or potentially erroneous paths.

### Enable verbose logging for debugging
**Args:** -g graph.bfg -q query.fa -o output.txt --verbose
**Explanation:** Outputs detailed traversal logs including branching points, dead ends, and alternative paths found during query processing, useful for diagnosing unexpected results.

### Query with a maximum memory limit
**Args:** -g graph.bfg -q large_query.fa -o output.txt --max-mem 8G
**Explanation:** Restricts bifrost-httr to use a maximum of 8GB of RAM when processing the query, preventing excessive memory consumption on shared systems.

### Output results in GPS format for genome visualization tools
**Args:** -g graph.bfg -q query.fa -o results.gps --gps
**Explanation:** Writes query results in GPS (Genome Position String) format, which can be loaded directly into genome browsers like JBrowse for interactive visualization.

### Find paths with exact sequence matching only
**Args:** -g graph.bfg -q query.fa -o output.txt --exact
**Explanation:** Only reports paths where the complete query sequence has an exact match through the graph with no mismatches or branching, skipping partial or ambiguous matches.

### Control the number of threads for parallel processing
**Args:** -g graph.bfg -q query.fa -o output.txt -t 4
**Explanation:** Uses 4 parallel threads to process query sequences, improving throughput on multi-core systems at the cost of higher CPU usage.

### Generate a summary report of all query results
**Args:** -g graph.bfg -q query.fa -o summary.txt --summary
**Explanation:** Produces a concise summary including total queries, match rates, and average path lengths rather than detailed per-sequence output.