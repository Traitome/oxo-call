---
name: bamsurgeon
category: variant-simulation
description: A tool for adding simulated somatic mutations (SNVs and indels) to existing BAM files for testing variant callers and pipelines.
tags: bam, mutation, simulation, variant-calling, spike-in, snv, indel, bioinformatics
author: AI-generated
source_url: https://github.com/adamewing/bamsurgeon
---

## Concepts

- **Input Requirements:** bamsurgeon requires a coordinate-sorted and indexed BAM file aligned to a reference FASTA genome, along with a variant list in tab-separated format (chromosome, position, reference allele, alternate allele).
- **Variant Inclusion Logic:** Only reads that cross the variant position and meet the minimum base quality (--minbq) and minimum mapping quality (--minmq) thresholds are considered for mutation; reads are modified using an in-silico mutagenesis approach.
- **Output Generation:** The tool outputs a new BAM file with variants incorporated into the appropriate reads, along with a VCF file documenting the spiked-in variants for truth set comparison.
- **Strand-Specificity:** By default, bamsurgeon adds variants to both strands (--strandprop option controls this), allowing simulation of artifacts specific to forward or reverse strand sequencing.
- **Indel Handling:** For insertions and deletions, the tool adjusts read CIGAR strings and adds soft-clipping as needed; the --maxclip parameter controls the maximum soft-clip length allowed.

## Pitfalls

- **Unpaired Reads Being Ignored:** If reads are marked as unpaired (0x1 flag not set), bamsurgeon may skip them by default; using --include-unpaired ensures these reads are also modified, which is important for simulate true single-end data.
- **Reference Mismatches Due to Genome Version:** Using a reference genome version that differs from the one the original BAM was aligned to will cause failures or incorrect variant insertion; always verify the exact reference FASTA used for alignment.
- **Insufficient Read Depth for Rare Variants:** When the input BAM lacks enough reads covering a variant position, the spiked-in variant will have lower allele frequency than intended; increase --maxreads or select regions with sufficient coverage.
- **Memory Exhaustion with Large Files:** Processing whole-genome BAM files consumes significant RAM; using the --region option to limit processing to specific chromosomal regions mitigates this.
- **VCF Output Not Matching Expected Format:** Some downstream tools require specific VCF annotations; bamsurgeon's output VCF may need custom filtering or annotation addition for compatibility.

## Examples

### Add a single SNV to a specific genomic position

**Args:** --ref reference.fa --bam input.bam --vars snv.tsv --out output.bam --force

**Explanation:** This adds the SNV defined in the variants file to reads overlapping that position, overwriting any existing output BAM file.

### Add multiple variants spanning a chromosomal region

**Args:** --ref genome.fa --bam input.bam --vars variants.tsv --out output.bam --region chr1:1000000-2000000

**Explanation:** This limits the mutation addition to variants within the specified chromosomal region, reducing memory usage for large BAM files.

### Add indels with soft-clipping adjustment

**Args:** --ref genome.fa --bam input.bam --vars indels.tsv --out output.bam --maxclip 10 --force

**Explanation:** This allows soft-clipping of up to 10 bases around indels to maintain proper read structure and ensure alignment integrity.

### Spike-in rare somatic variants at 10% allele frequency

**Args:** --ref genome.fa --bam input.bam --vars somatic.tsv --out output.bam --prop 0.1

**Explanation:** This introduces variants at approximately 10% frequency by only modifying a subset of qualifying reads, simulating low-allele-frequency somatic mutations.

### Increase sensitivity for low-quality reads

**Args:** --ref genome.fa --bam input.bam --vars variants.tsv --out output.bam --minbq 10 --minmq 15

**Explanation:** This lowers the minimum base quality and mapping quality thresholds to include more reads for variant insertion, useful for low-coverage regions.

### Process a specific list of reads by read name

**Args:** --ref genome.fa --bam input.bam --vars variants.tsv --out output.bam --readlist reads.txt

**Explanation:** This restricts mutation addition to only the specified reads listed in the readlist file, enabling targeted spike-in experiments.

### Force overwrite existing output files

**Args:** --ref genome.fa --bam input.bam --vars variants.tsv --out output.bam --force

**Explanation:** This enables overwriting of the output BAM if it already exists, which is necessary for rerunning pipelines without manual cleanup.