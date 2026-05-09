---
name: centreseq
category: Sequence Analysis
description: A bioinformatics tool for generating consensus sequences from multiple sequence alignments or assemblies, extracting representative central sequences from sequence clusters, and performing center-of-mass sequence computations for genomic analyses.
tags: [consensus, sequence, alignment, assembly, central-sequence, bioinformatics]
author: AI-generated
source_url: https://github.com/centreseq/centreseq
---

## Concepts

- **Input formats**: Accepts FASTA, FASTQ, and multi-FASTA files containing multiple sequences for consensus or central sequence computation. The tool treats each sequence record as an independent observation for computing the statistical center.

- **Output modes**: Produces consensus sequences in FASTA format, with optional quality scores and per-position confidence values. Results include both the central/consensus sequence and an optional statistic summary file showing positional base distribution.

- **Alignment-based computation**: When operating in consensus mode, centreseq performs positional nucleotide frequency analysis across all input sequences, reporting the majority base at each position as the consensus. Supports IUPAC ambiguity codes for heterozygous positions.

- **Statistical weighting**: The tool can weight sequences by quality scores (from FASTQ) or custom abundance values, giving higher influence to higher-quality or more prevalent sequences in the final consensus computation.

- **Companion binary**: Use `centreseq-build` to pre-compute index files for large multi-sequence databases, accelerating repeated queries against the same sequence collection.

## Pitfalls

- **Mixing incompatible sequence lengths**: Providing sequences of substantially different lengths without specifying alignment handling causes undefined consensus output. The tool requires either pre-aligned sequences or explicit gap-parameter settings.

- **Ignoring quality score encoding**: Running on FASTQ files without specifying quality-score weighting uses Phred+33 encoding by default. Using Phred+64 encoded older Illumina 1.8 files produces systematically wrong consensus calls at low-quality positions.

- **Output file overwrites**: Running centreseq multiple times with the same output filename will silently overwrite existing consensus files. Always specify unique output names or use the append flag when accumulating results.

- **Memory limits with massive datasets**: Processing more than available RAM with whole-genome-scale datasets (millions of sequences) causes the tool to crash mid-computation. Use the chunked-processing flag for very large multi-sequence inputs.

- **Ambiguous IUPAC handling**: Without explicit ambiguity-resolution settings, centreseq reports ambiguity codes in the consensus output. Downstream tools expecting strict A/T/G/C sequences will fail or produce errors.

## Examples

### Generate a consensus sequence from a multiple sequence alignment FASTA file

**Args:** `-i alignment.fasta -o consensus.fasta --method majority`

**Explanation:** Reads all sequences from the alignment file, computes the majority base at each position, and writes the resulting consensus sequence to the output file using the majority-vote method.

### Compute a quality-weighted consensus from FASTQ reads

**Args:** `-i reads.fastq -o consensus.fastq --method weighted --weight-quality --phred+33`

**Explanation:** Uses per-base quality scores from the FASTQ input to weight each base call when computing the consensus, giving more influence to higher-quality base calls at each position.

### Extract the central sequence from a cluster using geometric median

**Args:** `-i cluster.fasta -o central.fasta --method geometric --distance hamming`

**Explanation:** Computes the geometric median sequence (the sequence minimizing total Hamming distance to all others) and outputs it as the representative central sequence for the cluster.

### Generate consensus with confidence scores for each position

**Args:** `-i alignment.fasta -o consensus.fasta --method majority --output-scores confidence.txt`

**Explanation:** Writes the consensus sequence to the primary output file while also producing a confidence score file showing the percentage support for each consensus base at every position.

### Process sequences in chunks to handle large datasets within memory limits

**Args:** `-i large_sequences.fasta -o consensus.fasta --method majority --chunk-size 50000 --out-chunk-dir chunks/`

**Explanation:** Splits the input into chunks of 50,000 sequences each, processes each chunk separately, and merges the partial results to produce the final consensus without exhausting available memory.