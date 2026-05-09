---
name: abpoa
category: consensus-assembly
description: Adaptive Banded Partial Order Alignment tool for generating consensus sequences and calling variants from multiple sequence alignments. Optimized for speed and memory efficiency through dynamic banding techniques.
tags: [consensus, variant-calling, alignment, multiple-sequence-alignment, assembly]
author: AI-generated
source_url: https://github.com/zhangzhangyi/ABPOA
---

## Concepts

- **Input flexibility**: abpoa accepts various input formats including FASTA, FASTQ, SAM, BAM, and pre-aligned multiple sequence alignment (MSA) files. For MSA inputs, sequences must be aligned to a reference with gaps represented by '-' or '.' characters.
- **Partial order alignment algorithm**: The tool uses an adaptive banding technique that dynamically adjusts the alignment band width based on sequence divergence. This allows faster execution on similar sequences while maintaining accuracy on divergent regions.
- **Output modes**: abpoa can produce consensus sequences (-o flag), variant calls (-v flag), or normalized alignments (-n flag). When multiple sequences are provided, it generates a consensus by majority voting with configurable threshold (-t flag for minimum frequency).
- **Reference handling**: When input sequences are not pre-aligned, a reference sequence can be provided (-r flag) to guide the alignment. The reference must be in FASTA format.

## Pitfalls

- **Mismatched input format**: Not specifying the correct input format (-f flag) when using non-standard file extensions causes parsing errors and alignment failures. Always verify your input format matches the file content.
- **Insufficient sequence coverage**: Setting the consensus threshold too high (-t 1.0) with low-coverage alignments produces no consensus calls. For haploid consensus, use -t 0.5; for diploid, consider -t 0.3 to capture heterozygous positions.
- **Band width trade-off misconfiguration**: Using an excessively narrow band width (-b 1) on highly divergent sequences results in missed alignment boundaries and fragmented consensus. Increase band width for more divergent inputs.
- **Reference sequence contamination**: Providing a reference (-r) that is not the true source sequence leads to alignments against the wrong baseline, producing invalid consensus and variant calls. Always verify your reference matches the input sample source.

## Examples

### Generate consensus from a multiple sequence alignment file in FASTA format

**Args:** -i input_aligned.fa -o consensus.fa -f fasta_msa
**Explanation:** Reads an MSA file in FASTA format and outputs consensus sequences. The -f flag explicitly specifies the input format as aligned FASTA sequences.

### Call variants from aligned reads against a reference sequence

**Args:** -r reference.fa -i aligned_reads.sam -v variants.vcf -f sam
**Explanation:** Aligns input reads to the reference using SAM format and outputs variant calls in VCF format for downstream analysis.

### Generate consensus with custom minimum frequency threshold

**Args:** -i reads.fa -o consensus.fa -r ref.fa -t 0.6
**Explanation:** Uses a 60% minimum frequency threshold for consensus basecalls, requiring majority support for each called base.

### Output normalized alignment with specified band width

**Args:** -i input.fa -n normalized.fa -b 30 -f fasta
**Explanation:** Produces a normalized alignment with band width of 30, suitable for moderately divergent sequences, improving alignment completeness.

### Process BAM file and output consensus to stdout

**Args:** -i reads.bam -o - -f bam -t 0.5
**Explanation:** Reads a BAM file, generates haploid consensus (50% threshold), and outputs to stdout. The -o - flag writes to standard output.

### Generate consensus with gzip compressed output

**Args:** -i input_msa.fa -o consensus.fa.gz -f fasta_msa -z
**Explanation:** Reads aligned FASTA sequences and writes gzip-compressed consensus output. The -z flag enables automatic compression of the output file.