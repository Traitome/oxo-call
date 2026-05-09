---
name: bifidoannotator
category: genomics/annotation
description: A bioinformatics tool for annotating genomic features in bacterial and microbial genomes. Supports functional annotation, gene prediction validation, and taxonomy-dependent annotation strategies. Operates via command-line interface accepting standard bioinformatics formats.
tags: [genomics, annotation, bacteria, microbial, gene-calling, functional-annotation]
author: AI-generated
source_url: https://github.com/bifidoannotator/bifidoannotator
---

## Concepts

- **Input Formats**: bifidoannotator accepts FASTA (raw sequences), GenBank (with feature annotations), and FASTA with quality scores. Multi-FASTA files are processed sequentially; single-sequence inputs are recommended for gene-level annotations.
- **Output Files**: Generates annotated output in GTF (for downstream RNA-seq analysis) or BED format (for genome visualization). The output format is controlled via the `-o` flag and defaults to GTF if not specified.
- **Annotation Modes**: Three primary modes exist — `predict` (gene prediction only), `annotate` (functional annotation using ORF translation), and `full` (combines both prediction and functional assignment using homology scoring).
- **Database Integration**: Requires a pre-indexed annotation database built with the companion tool `bifidoannotator-build`. Without a loaded database, the tool falls back to intrinsic gene-calling which produces lower-confidence annotations.

## Pitfalls

- **Using Intrinsic Mode Without a Database**: Running bifidoannotator without `--db` or `--index` defaults to intrinsic prediction, which misses many hypothetical proteins in bacterial genomes. Consequences: incomplete annotation files leading to false negatives in downstream analysis.
- **Mismatched Output Format for Downstream Tools**: Specifying BED output when a downstream tool requires GTF (or vice versa) causes parsing failures. Always verify format compatibility with tools like IGV or Cufflinks before running pipelines.
- **Ignoring Sequence Type Settings**: The `--seq-type` flag defaults to `dna` but users processing RNA sequences (cDNA) forget to set `--seq-type rna`, causing incorrect codon frames andframe-shift errors in functional translations.
- **Database Version Mismatch**: Using an outdated annotation database with newer tool versions produces inconsistent annotations due to changed feature identifiers. Always rebuild databases with `bifidoannotator-build` when upgrading the tool.
- **Input File Encoding Issues**: Non-ASCII characters in sequence headers (common when downloading from NCBI) cause parsing errors. Users must sanitize input headers or use `--strip-headers` to remove problematic characters.

## Examples

### Annotating a bacterial genome with a pre-built database
**Args:** `--db bifido_db --input genome.fasta --output annotated.gtf`
**Explanation:** Uses a pre-indexed database for homology-based annotation, producing GTF output suitable for RNA-seq quantification workflows.

### Predicting genes without a database (intrinsic mode)
**Args:** `--input genome.fasta --output predictions.bed --mode predict`
**Explanation:** Runs gene prediction without functional annotation, useful when no database exists or for quick initial gene-calling before database loading.

### Producing BED format annotation for visualization
**Args:** `--input sequences.fasta --output view_annotations.bed --format bed`
**Explanation:** Generates BED output which can be directly loaded into genome browsers like IGV or UCSC for manual inspection.

### Processing RNA sequences with correct frame detection
**Args:** --input cdna.fasta --output rna_annotations.gtf --seq-type rna
**Explanation:** Sets the sequence type to RNA so the tool uses the correct codon table for translation, preventing frameshift errors in functional annotation.

### Using full annotation mode combining prediction and functional assignment
**Args:** --db mydatabase --input contigs.fasta --output full_annot.gtf --mode full
**Explanation:** Combines gene prediction with functional annotation using homology scoring, providing the most complete annotation output for publication-quality datasets.

### Stripping non-standard headers before processing
**Args:** --input raw_seq.fasta --output clean.gtf --strip-headers
**Explanation:** Removes non-ASCII and problematic characters from sequence headers that would otherwise cause parsing failures, enabling processing of NCBI-downloaded sequences.

### Running with custom scoring thresholds for short contigs
**Args:** --input short_contigs.fasta --output short_annot.gtf --min-orf 90 --mode full
**Explanation:** Lowers the minimum ORF length to 90bp (default is 150bp) to capture smaller peptides common in fragmented bacterial assemblies.

### Parallel processing for large multi-chromosome datasets
**Args:** --input genome_dir/ --output annotations.gtf --threads 8 --mode full
**Explanation:** Enables multi-threaded processing with 8 threads for large genomes with multiple chromosomes, significantly reducing runtime on multi-core systems.