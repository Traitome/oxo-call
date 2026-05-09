---
name: bigslice
category: Bioinformatics - spliced alignment
description: Finds spliced alignments of mRNA or cDNA sequences to a genomic database, accounting for canonical and non-canonical introns. Part of the GMAP/GTAP suite for genome-wide alignment of EST, cDNA, and RNA-seq data.
tags: [genomics, alignment, splicing, RNA, cDNA, introns, EST, genome]
author: AI-generated
source_url: http://gmap.homolog.com/
---

## Concepts

- **Genomic Index Requirement**: bigslice requires a pre-built genomic index created by `bigslice-build`. Without a valid index, alignments fail immediately. The index stores compressed genomic sequences for rapid lookup during spliced alignment.
- **Spliced Alignment Model**: bigslice detects alignment gaps corresponding to introns using canonical splice site motifs (GT-AG, GC-AG, AT-AC) and allows for non-canonical splice sites when configured. Gaps smaller than thespecified intron length are treated as insertions.
- **Input Format Flexibility**: Accepts FASTA, FASTQ, and multi-line formats as input. Sequences may be provided via standard input when using `-` as the input filename. Raw sequence data without line breaks is also supported.
- **Output Format Control**: Generates alignments in multiple formats including GFF3, FASTA (for aligned sequences), and a native `.msg` format. The output format is controlled via `-f` and `-o` flags, with SAM output requiring specific formatting flags.

---

## Pitfalls

- **Missing Genomic Index**: Running bigslice without first building an index with `bigslice-build` produces no alignments and only an error message. Users must ensure the index path is correct and accessible.
- **Incorrect Splice Site Settings**: Using `--canonical` when the data contains non-canonical introns causes false negative alignments. Conversely, allowing all splice sites increases false positives and alignment runtime.
- **Memory Exhaustion with Large Genomes**: Specifying insufficient memory for the index via `-m` when aligning to mammalian-scale genomes causes crashes. The recommended minimum is 2GB for human genome indexes.
- **Conflicting Format Flags**: Combining incompatible output format flags (e.g., `-f 3` for SAM with `-o`) produces malformed output. Ensure flags are compatible with the desired output type.

---

## Examples

### Align EST sequences to a pre-built human genome index

**Args:** `-D /path/to/hg38_index -d hg38 input.fasta > aligned.gff`
**Explanation:** Aligns EST sequences from input.fasta to the hg38 index stored in the specified directory, outputting results in GFF format.

### Output alignments in SAM format for downstream processing

**Args:** `-D /path/to/hg38_index -d hg38 -f 3 input.fasta`
**Explanation:** Produces SAM format output, which is directly compatible with standard bioinformatics tools like samtools,IGV, and variant callers.

### Find alignments with a specific minimum exon length

**Args:** `-D /path_to_index -d genome -m 50 input.fasta`
**Explanation:** Ensures that reported exons in spliced alignments are at least 50 bases long, filtering out very short exons that may represent alignment noise.

### Allow non-canonical splice sites for atypical transcripts

**Args:** `-D /path_to_index -d genome --canonical=0 input.fasta`
**Explanation:** Enables detection of non-standard splice site pairs beyond GT-AG, GC-AG, and AT-AC, necessary for non-model organisms or cDNA with mutation data.

### Limit maximum intron size to 500kb

**Args:** `-D /path_to_index -d genome -i 500000 input.fasta`
**Explanation:** Restricts reported introns to a maximum of 500 kilobases, useful for excluding alignment artifacts inrepeat-rich regions or when analyzing specific genomic contexts.

### Process input sequences from standard input

**Args:** `-D /path_to_index - d genome -f