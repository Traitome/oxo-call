---
name: "blat"
category: "Sequence Alignment"
description: "BLAT (BLAST-Like Alignment Tool) is a ultrafast tool for aligning DNA or protein sequences against a reference database, commonly used for finding exons, promoters, and other genomic features in assembled genomes."
tags: ["alignment", "genome", "dna", "protein", "exon-finding", "ucsc"]
author: "AI-generated"
source_url: "https://genome.ucsc.edu/FAQ/FAQblat.html"
---

## Concepts

- **PSL Output Format**: BLAT outputs alignments in PSL (PSL = Pattern Space Listing) format, a 21-column tab-delimited format where columns include: query name, query size, query start-end, subject name, subject size, subject start-end, match count, mismatch count, repeats, score, strand orientation, query sequence bounds, subject sequence bounds, and the gap block sizes and positions. This format is designed for efficient parsing by downstream tools.

- **Query and Subject Type Matching**: BLAT requires explicit declaration of file types for both query (`-q`) and subject (`-t`) sequences as `dna`, `rna`, or `prot`. Using `-type=prot` on the command line auto-sets both to protein. DNA queries against protein databases (and vice versa) are not supported; attempting this produces zero results with no explicit error message.

- **Index-Based Acceleration**: BLAT achieves its speed by pre-building an index with `blat-build` that partitions the subject genome into overlapping tiles (typically 11-14 bases for DNA, shorter for proteins). Without a pre-built 2bit index file, BLAT falls back to slower direct scanning of the subject file, but the index approach can be orders of magnitude faster for whole-genome searches.

- **Minimum Identity Threshold**: The default `-minIdentity` value is 95 for DNA-DNA alignments but only 75 for protein alignments. For RNA-seq or evolutionary analyses requiring lower identity cutoffs, this parameter must be explicitly set; otherwise, related sequences below the threshold are silently omitted from output.

## Pitfalls

- **Forgetting to Build the 2bit Index**: Running BLAT directly on a large FASTA subject file without pre-indexing causes BLAT to load the entire file into memory and scan it sequentially, which can be orders of magnitude slower and consume excessive RAM. Always use `blat-build` with the appropriate `-tileSize` parameter to create a `.2bit` index for genomic reference sequences.

- **Mismatched Query and Subject Types**: Supplying a DNA query with `-t=prot` or vice versa produces no alignments without warning. Ensure both `-q` and `-t` flags are consistent with the actual biological molecule type in your input files, or use `-type=prot` to enforce protein-mode alignment.

- **Default Identity Threshold Too Restrictive**: For evolutionary studies, cross-species mapping, or degraded RNA-seq data, the default 95% identity cutoff silently discards valid alignments. Set `-minIdentity` to a lower value (e.g., 85-90 for diverged sequences) before assuming a query has no matches.

- **Parsing PSL Output Without Skipping Header**: The PSL text header line (`psLayout`) is printed as the first line of every output file. Scripts that assume output starts directly with data rows will encounter parsing errors or misaligned column assignments. Always skip or strip the first line, or use `-noHead` to suppress it entirely.

- **Large Gap Sizes in Protein Alignments**: BLAT's gap penalties are optimized for genomic DNA and can produce unreasonably large insertions/deletions in protein alignments, distorting alignment quality. The `-gapExt` parameter may need manual tuning; alternatively, consider tools like `tblastn` for protein-to-genome alignment tasks where gap handling is more biologically calibrated.

## Examples

### Align a single DNA query against a whole-genome index
**Args:** `query.fa genome.2bit -out=psl query.psl`
**Explanation:** This aligns sequences from `query.fa` against the pre-indexed genome stored in `genome.2bit`, outputting results in PSL format to `query.psl`. The `.2bit` index enables rapid lookup compared to raw FASTA scanning.

### Find protein homologs in a protein database
**Args:** `-type=prot query.pep subject.pep -out=psl matches.psl`
**Explanation:** Running BLAT in protein mode (`-type=prot`) aligns amino acid sequences, automatically setting both query and subject types to protein. This is suitable for finding remote homologs where DNA-level comparison would miss conservation.

### Align EST sequences with relaxed gap settings
**Args:** `est_query.fa cdna.2bit -t=dna -q=rna -fine -minIdentity=90 out.psl`
**Explanation:** The `-fine` flag enables slower but more accurate gap resolution for EST alignments where sequencing errors create spurious gaps. Combined with `-minIdentity=90`, this captures divergent but genuine expressed transcripts.

### Build a genomic index for repeated use
**Args:** `hg38.fa hg38.2bit`
**Explanation:** `blat-build` constructs a `.2bit` index from the FASTA input file, enabling future BLAT commands to reference `hg38.2bit` instead of `hg38.fa`. The resulting index is memory-mapped for efficient random access.

### Extract high-identity coding exon mappings for visualization
**Args:** `cds.fa genome.2bit -minIdentity=98 -noHead -out=psl exons.psl`
**Explanation:** Setting `-minIdentity=98` filters to near-perfect matches suitable for precise exon boundary mapping. The `-noHead` flag suppresses the PSL header row, producing clean tabular output ready for bedtools or UCSC genome browser track generation.

### Batch-align multiple queries to a reference
**Args:** `-q=prot -t=prot queries.faa ref.pep -out=maf results.maf`
**Explanation:** This performs protein-protein alignment across all sequences in `queries.faa` against `ref.pep`, outputting in MAF (Multiple Alignment Format) suitable for conservation analysis or phylogenetic tools like PhyloFit.

### Generate axt format for UCSC chain file conversion
**Args:** `qseq.fa target.2bit -out=axt -noHead align.axt`
**Explanation:** AXT format produces a compact query-vs-reference alignment block format used by UCSC tools like `axtChain` for syntenic mapping. Suppressing the header with `-noHead` simplifies downstream chaining pipeline integration.