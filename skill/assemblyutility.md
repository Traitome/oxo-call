---
name: assemblyutility
category: genome_assembly
description: A comprehensive utility for genomic sequence assembly operations including read assembly, contig scaffolding, assembly format conversion, and quality assessment. Supports FASTA, FASTQ, and standard assembly output formats.
tags:
- assembly
- genomics
- bioinformatics
- sequence-assembly
- contigs
- scaffolds
author: AI-generated
source_url: https://github.com/assemblyutility/assemblyutility
---

## Concepts

- **Input Formats**: assemblyutility accepts raw sequencing reads in FASTQ format (optionally gzipped), reference-assisted assembly in FASTA, and supports SAM/BAM for assembly-to-reference comparisons. The tool automatically detects format from file extensions (.fastq, .fq, .fasta, .fa, .sam, .bam).
- **Output Modes**: Generated assemblies can be output as raw contigs (FASTA), scaffolded sequences with N-placeholder gaps, or extended consensus sequences. Quality metrics (N50, coverage depth, assembly consensus QV) are written to stderr unless --metrics is specified.
- **Assembly Algorithms**: The tool implements overlap-layout-consensus (OLC) for short reads and de Bruijn graph assembly for larger datasets. Algorithm selection is automatic based on read length and coverage depth unless overridden by --algorithm flag.
- **Memory Model**: Large assemblies use a streaming approach where read information is hashed and only assembled segments are held in memory. The --memory-limit flag controls maximum RAM allocation (default 8GB).

## Pitfalls

- **Mismatched read orientation**: Specifying --paired-end when inputs are single-end reads causes assembly failure with misleading error messages; always verify read layout in input files before setting read orientation flags.
- **Insufficient coverage for assembly**: Assembly with coverage below 10x produces fragmented, unreliable contigs that may pass without warning; check coverage with external tools before running assembly.
- **Contaminated reference sequences**: Using a reference with adapter contamination or vector sequences leads to assembled contigs containing spurious sequences; always run contamination screening before reference-guided assembly.
- **Format mismatch in batch processing**: Mixing FASTQ and FASTA in multi-file input without explicit --format flag results in partial assembly failure; explicitly specify format when batch inputs have mixed extensions.

## Examples

### Assemble short reads into contigs using default settings
**Args:** input Reads.fq.gz --output assembly_output.fasta
**Explanation:** Uses automatic algorithm selection and memory management to assemble compressed FASTQ reads into FASTA contigs, writing output to the specified file.

### Assemble with explicit de Bruijn graph algorithm
**Args:** input sample1.fq --algorithm debruijn --kmer-size 31 --output dbg_assembly.fasta
**Explanation:** Forces de Bruijn graph assembly with 31-mer size, overriding automatic algorithm selection for improved assembly of high-coverage Illumina data.

### Generate quality metrics alongside assembly
**Args:** input reads.fq --output assembled.fa --metrics metrics.txt
**Explanation:** Writes assembly statistics including N50, contig count, and total assembly length to metrics.txt while producing the primary FASTA output.

### Reference-guided scaffolding with existing contigs
**Args:** input contigs.fasta --reference ref_genome.fa --output scaffolded.fa --scaffolding-mode hybrid
**Explanation:** Uses reference sequence to scaffold existing contigs, inserting N-gaps at misjoined regions and producing a scaffolded FASTA output.

### Convert existing assembly to BED format for visualization
**Args:** input old_assembly.fa --convert-to bed --output assembly_regions.bed
**Explanation:** Converts FASTA assembly to BED format coordinates for genome browser visualization without performing new assembly.

### Filter low-complexity regions from final assembly
**Args:** input raw_assembly.fa --filter-complexity --min-complexity 50 --output filtered.fasta
**Explanation:** Removes low-complexity (high-repetition) contigs below 50% complexity threshold, producing a filtered assembly suitable for downstream analysis.