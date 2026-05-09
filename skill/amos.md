---
name: AMOS Assembler
category: Genome Assembly / Sequence Assembly
description: A whole-genome assembler and associated tools for assembling DNA sequence reads into contiguous sequences (contigs) and scaffolds. AMOS uses a bank-based data structure for read storage and supports multiple sequencing technologies.
tags: [assembly, genome-assembly, overlap-layout-consensus, reads, contigs, scaffolds]
author: AI-generated
source_url: https://sourceforge.net/projects/amos/
---
## Concepts

- AMOS uses a **bank** as its primary data structure—a persistent, indexed repository that stores reads, quality scores, overlaps, contigs, and scaffolds. All downstream tools operate on data stored within a bank, so the bank must be initialized and populated before assembly begins.
- Input reads are accepted in **FASTA, FASTQ, Sanger, and Illumina formats**; the assembler outputs results in **AMOS, ACE, and CAF formats**. Quality score encoding (Phred+33 vs Phred+64) must match the input format specification, or base calling accuracy will be degraded.
- The assembly algorithm follows the **Overlap-Layout-Consensus (OLC)** paradigm: it first finds pairwise overlaps between all reads, then builds an overlap graph, identifies seed sequences from high-coverage regions, greedily extends contigs, resolves repeat-induced ambiguities, and finally calls consensus bases with quality estimates.
- **Paired-end and mate-pair reads** provide link information used to build scaffolds and close gaps. Without proper pairing information or with incorrectly estimated library insert sizes, scaffolding will produce fragmented or incorrect layouts.
- Memory consumption scales with genome size and read depth. Large eukaryotic genomes require careful partitioning of reads into chunks and staged overlap detection to avoid exhausting available RAM.

## Pitfalls

- **Forgetting to initialize the AMOS bank before importing reads** causes all subsequent commands to fail with initialization errors. The bank must be created explicitly using companion tools before any data can be loaded or analyzed.
- **Specifying incorrect quality score offset** (mixing Phred+33 and Phred+64 encodings) corrupts quality values, leading to inflated or negative consensus quality scores and unreliable variant calls in the assembled contigs.
- **Insufficient overlap detection sensitivity** for short reads or high-error-rate reads results in missing genuine overlaps, causing fragmentation of contigs and decreased N50 statistics. Overlap parameters must be tuned to the read length and error rate of the dataset.
- **Using unpaired reads when mate-pair or paired-end libraries are available** wastes scaffold-building information, producing shorter scaffolds and larger gaps than the data actually supports.
- **Ignoring repeat handling for repetitive genomes** causes the assembler to resolve repeats incorrectly, collapsing paralogous sequences or creating chimeras that merge distinct genomic regions into single contigs.

## Examples

### Assemble a set of raw reads from a bacterial genome
**Args:** `assemble -D maxGap=500 -D minOverlap=30 -D errorRate=0.03 -o output.ace reads.fasta`
**Explanation:** Runs the overlap-layout-consensus assembler with a maximum gap size of 500 bp, minimum overlap of 30 bases, and 3% error tolerance, outputting assembled contigs in ACE format.

### Build an AMOS bank and populate it with paired-end reads
**Args:** `bank-transact -v -b mybank.afg -c "bank open mybank; read load reads_R1.fastq reads_R2.fastq libname=mylib"` 
**Explanation:** Opens or creates an AMOS bank and loads paired-end FASTQ reads with a library label, enabling the assembler to use pairing information during layout construction.

### Validate assembled contigs against reference sequences
**Args:** `validate -v -e ref_genome.fasta output.ace`
**Explanation:** Validates the ACE-format assembly by aligning contigs to the reference genome and reports structural discrepancies, coverage gaps, and misassemblies.

### Convert AMOS bank output to ACE format for downstream annotation
**Args:** `toAcedb -o assembly.ace mybank.afg`
**Explanation:** Exports the completed assembly from the AMOS bank to ACE format, which is compatible with tools like Consed and Phrap for manual curation and finishing.

### Generate consensus sequences with quality scores from a bank
**Args:** `bank2fasta -b mybank.afg -q -m -o consensus_seqs.fasta`
**Explanation:** Extracts consensus sequences from the assembly bank with per-base quality scores, producing a FASTA file suitable for variant calling and downstream informatics analysis.

### Close gaps in scaffolds using paired-end read linking information
**Args:** `bank-scaffold -b mybank.afg -i 500 -j 2000 -o scaffolds.scf`
**Explanation:** Constructs scaffolds from contigs using insert-size constraints of 500–2000 bp, resolving ordering and orientation of contigs and outputting scaffold files.

### Estimate genome coverage from raw read set before assembly
**Args:** `bank-stats -b mybank.afg`
**Explanation:** Prints statistics about the AMOS bank including total bases, number of reads, average read length, estimated coverage depth, and library distribution—useful for assessing whether sequencing depth is sufficient for assembly.

### Assemble reads using the HetASM variant for heterozygous diploid genomes
**Args:** `hetasm -D genomeSize=5e6 -D heterozygosity=0.01 -o hetasm_output.ace reads.fasta`
**Explanation:** Uses the HetASM assembler variant designed for diploid genomes with high heterozygosity, preventing collapsing of allelic variants into single consensus contigs.

### Merge multiple partial assemblies into a single coherent assembly
**Args:** `merge-amos -o merged_assembly.ace assembly1.ace assembly2.ace`
**Explanation:** Combines two or more independent ACE-format assemblies that cover overlapping regions, producing a unified assembly with resolved inconsistencies.

### Extract all read sequences from a bank for re-analysis
**Args:** `bank2fasta -b mybank.afg -trim -o trimmed_reads.fasta`
**Explanation:** Exports all reads from the bank with vector and quality trimming applied, generating a cleaned FASTA file for alternative assembly pipelines or validation.