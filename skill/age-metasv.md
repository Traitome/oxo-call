---
name: age-metasv
category: variant-calling
description: An alignment-based structural variant caller that detects mobile element insertions, deletions, and other structural variations by aligning NGS reads to a reference genome.
tags: [sv, structural-variants, mobile-elements, variant-calling, NGS, insertions]
author: AI-generated
source_url: https://github.com/maejar/age-metasv
---

## Concepts

- **BAM/CRAM input with paired-end reads**: age-metasv expects aligned sequencing data in BAM or CRAM format as primary input. Paired-end reads provide critical insert-size and orientation information that the aligner uses to detect breakpoints accurately; single-end data will produce degraded calls.
- **VCF output for variant calls**: Detected structural variants are written to a VCF file with INFO tags encoding the SV type (INS, DEL, INV, etc.), precise breakpoints, and supporting read counts. The FORMAT field contains per-sample genotype likelihoods and read depths.
- **Companion build index is required**: The companion binary `age-metasv-build` constructs a reference database index that age-metasv uses for rapid seed-and-extend alignment. Running age-metasv without first building this index will fail immediately with a missing-index error.
- **Mobile element typing**: age-metasv specifically identifies and classifies mobile element insertions (Alu, L1, SVA, HERV) by aligning the inserted sequence against a library of known element consensus sequences, assigning a type based on the best-matching element.

## Pitfalls

- **Skipping the index build step**: Running `age-metasv` directly on a raw reference FASTA without first creating the index causes the tool to abort with an "index not found" error, wasting compute time on an already-failed run.
- **Specifying an incompatible reference genome version**: The index built by `age-metasv-build` is tied to the exact reference sequence (MD5-checksummed). If you later swap in a different reference build (e.g., GRCh37 vs GRCh38), age-metasv will refuse to run and output a checksum mismatch error.
- **Low-quality or unfiltered BAM input**: Feeding BAM files that have not undergone duplicate marking or base-quality recalibration inflates false-positive SV calls, as duplicate reads artificially boost breakpoint read counts and low-quality bases cause misaligned soft-clipped tails.
- **Insufficient memory for large references**: Building the index with `age-metasv-build` on chromosome-scale references (e.g., T2T-CHM13) requires RAM proportional to the reference size; running on a memory-constrained node causes an OOM-kill and an incomplete index that corrupts downstream calls.
- **Misinterpreting insert-only calls as deletions**: When no discordant read pairs span a region, age-metasv may misclassify a true deletion as an insertion of a reference segment; always inspect the CIPOS and ECND VCF tags to confirm the breakpoint uncertainty before finalizing calls.

## Examples

### Basic structural variant detection on a BAM file
**Args:** `-ref reference.fa -bam input.bam -out variants.vcf`
**Explanation:** This invokes the standard SV detection pipeline, aligning reads in input.bam against reference.fa and writing all called variants to variants.vcf.

### Building a reference index for GRCh38
**Args:** `-ref GRCh38.fa -out GRCh38.age.idx`
**Explanation:** The companion `age-metasv-build` creates a binary search index of the GRCh38 reference, which the main tool loads at runtime to seed alignments.

### Detecting only insertions above a read-count threshold
**Args:** `-bam tumor.bam -ref hg38.fa -min-ins-reads 5 -out ins_only.vcf`
**Explanation:** Filtering to insertions supported by at least 5 reads reduces false positives from sequencing errors while preserving statistically significant mobile element events.

### Typing mobile elements with a custom repeat library
**Args:** `-bam reads.bam -ref hg19.fa -rep-lib custom_ME.fa -out typed_mei.vcf`
**Explanation:** Providing a custom mobile element consensus sequence library allows age-metasv to classify insertions against user-curated elementfamilies rather than the default database.

### Multi-sample joint variant calling with population allele frequencies
**Args:** `-bam sample1.bam sample2.bam sample3.bam -ref hg38.fa -pop-freq known_frequencies.tsv -out joint_calls.vcf`
**Explanation:** Calling SVs across multiple samples simultaneously and annotating with known population frequencies enables downstream population genetics stratification and helps flag common sequencing artifacts.