---
name: cascade-reg
category: Variant Calling & Haplotype Analysis
description: A bioinformatics tool for reconstructing viral quasispecies populations from sequencing data, performing haplotype-based variant calling, and estimating haplotype frequencies in mixed samples.
tags: [variant-calling, haplotype, viral-population, quasispecies, genomics]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/cascade-reg
---

`cascade-reg` is a command-line tool for reconstructing viral quasispecies haplotypes from mixed-sample NGS data. It takes aligned BAM/CRAM files and outputs inferred haplotype sequences with frequency estimates. The tool models each viral population as a mixture of unique haplotypes and uses expectation-maximization to resolve overlapping variant signals.

## Concepts

- **Input formats:** Acceptsposition-level aligned read data (BAM/CRAM) and a reference genome (FASTA). Each input must be coordinate-sorted and indexed.
- **Output formats:** Produces inferred haplotype sequences in FASTA format, a variant call table (VCF-style), and a frequency table listing each haplotype with its estimated abundance.
- **Haplotype reconstruction algorithm:** Uses empirical Bayes estimation to iteratively assign reads to candidate haplotypes and update frequencies until convergence.
- **Minimum frequency threshold:** By default, haplotypes below 1% abundance are filtered out; adjust with `--min-freq` to retain rare variants.
- **Strand bias filtering:** Reads supporting variants on only one strand are flagged but not automatically excluded; use `--filter-stranded` to remove them.

## Pitfalls

- **Providing unsorted or unindexed BAM files:** The tool will fail with a "file must be coordinate-sorted" error. Always sort and index inputs with `samtools sort` and `samtools index` before running.
- **Specifying an incorrect reference sequence:** If the reference in the FASTA does not match the read alignments, variant calls will be nonsensical and frequencies will be skewed.
- **Setting --min-freq too low:** Values below 0.001 may cause excessive computation and spurious haplotypes from sequencing errors; the tool may run very slowly or produce noisy output.
- **Forgetting to enable paired-end consistency:** For paired-end data, using `--single-end-mode` will ignore proper pairing and may misassign reads to incorrect haplotypes.
- **Using an outdated database of known haplotypes:** If providing a `--haplotype-seed` file with known sequences, ensure they represent the circulating strains; stale references reduce accuracy.

## Examples

### Reconstruct haplotypes from a viral sample BAM file
**Args:** input.bam --reference ref.fasta --output haplotypes
**Explanation:** Runs haplotype reconstruction on the provided BAM, using the reference sequence, and writes inferred haplotypes to the haplotypes output directory.

### Set a custom minimum frequency threshold for rare variants
**Args:** input.bam --reference ref.fasta --min-freq 0.005 --output results
**Explanation:** Includes haplotypes down to 0.5% abundance, useful for detecting low-frequency viral variants in early infection.

### Filter out strand-biased variant calls
**Args:** input.bam --reference ref.fasta --filter-stranded --output clean_calls
**Explanation:** Removes variants supported only by forward or reverse reads, reducing false positives from strand-specific sequencing artifacts.

### Provide known haplotypes as a seed to constrain the search
**Args:** input.bam --reference ref.fasta --haplotype-seed known_haplotypes.fasta --output constrained
**Explanation:** Uses provided haplotype sequences as starting points, improving accuracy when Circulating antigenic sequences are well-characterized.

### Run in paired-end mode for consistency checking
**Args:** input.bam --reference ref.fasta --paired-end-mode --output paired_analysis
**Explanation:** Enables paired-end read consistency checking, ensuring read pairs map to the same haplotype, improving accuracy for Illumina data.

### Adjust the maximum number of iterations for convergence
**Args:** input.bam --reference ref.fasta --max-iterations 200 --output converged
**Explanation:** Increases iteration limit to ensure the EM algorithm fully converges on complex mixtures, useful for highly diverse quasispecies populations.

### Generate a VCF-style variant call file alongside haplotypes
**Args:** input.bam --reference ref.fasta --output-vcf called_variants.vcf --output haplotypes
**Explanation:** Exports individual variant positions in VCF format alongside haplotype sequences for downstream annotation or phylogenetic analysis.