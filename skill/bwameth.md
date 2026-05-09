---
name: bwameth
category: sequence_alignment
description: A fast aligner for bisulfite-sequencing (BS-Seq) data, optimized for methylome analysis. Built on the BWA algorithm but with modifications to handle reduced cytosine bases in bisulfite-converted reads.
tags: bisulfite-seq, methylation, alignment, epigenomics, wgbs, dna-methylation
author: AI-generated
source_url: https://github.com/brentp/bwa-meth
---

## Concepts

- **Bisulfite Conversion Handling**: bwameth treats cytosines in reads as matching either C or T in the reference, accounting for the fact that unmodified cytosines are converted to uracil (read as T) during bisulfite treatment while methylated cytosines (5mC) remain as C.
- **Two-Reference Strategy**: The algorithm uses two separate alignment passes—one against a C-to-T converted reference and another against a G-to-A converted reverse complement—to correctly align reads from both bisulfite-converted strands.
- **Input/Output Formats**: Accepts FASTQ or FASTA input files and produces SAM/BAM output; supports standard SAM format headers and flags for downstream methylation calling tools like MethylDackel or Bismark.
- **Index Requirement**: Unlike standard BWA, bwameth requires a specialized index built with `bwameth-build` that is optimized for bisulfite-aligned reference sequences.

## Pitfalls

- **Using Standard BWA Index**: Attempting to align with a standard BWA index (created by `bwa index`) will produce incorrect mappings because the algorithm expects bisulfite-converted reference coordinates. This leads to misalignment and false methylation calls.
- **Forgetting to Preserve Read Group Information**: Omitting the `@RG` read group tags in alignments causes metadata loss, which is critical for downstream pooling of samples in large methylome studies. Many tools fail to merge samples correctly without proper read groups.
- **Specifying Wrong Read Encoding**: If input FASTQ files use Illumina 1.8+ phred+33 quality encoding but the aligner is configured for phred+64 (older Illumina), quality scores are misinterpreted, leading to overly stringent or lenient base quality filtering.
- **Ignoring Multimap Defaults**: Not setting the `-a` flag for all hits causes multimapping reads to be silently discarded, biasing methylation estimates in repetitive genomic regions where many reads legitimately map to multiple loci.

## Examples

### Build a bisulfite-sequencing reference index from a FASTA file

**Args:** `hg38.fa hg38.fa.bwameth`

**Explanation:** Creates a specialized bwameth index from the hg38 reference genome, required before aligning any BS-Seq reads. The index stores both C-to-T and G-to-A converted versions of the reference.

### Align single-end bisulfite-seq reads to a reference index

**Args:** `-t 16 hg38.fa.bwameth reads.fq.gz > aligned.sam`

**Explanation:** Aligns single-end FASTQ reads against the bwameth index using 16 threads, outputting SAM format for downstream methylation analysis.

### Align paired-end bisulfite-seq reads with standard parameters

**Args:** `-t 8 -M hg38.fa.bwameth read1.fq.gz read2.fq.gz > paired.sam`

**Explanation:** Aligns paired-end reads with 8 threads and marks shorter split alignments as secondary (standard `-M` flag behavior) to maintain compatibility with Picard and GATK pipelines.

### Align and output directly to a compressed BAM file

**Args:** `-t 12 hg38.fa.bwameth reads.fq.gz | samtools view -bS - > aligned.bam`

**Explanation:** Uses pipe to convert SAM output directly to BAM for reduced disk usage; samtools handles the conversion with `-bS` (BAM output, SAM input).

### Align with explicit maximum insert size for paired-end data

**Args:** `-t 8 -i 500 hg38.fa.bwameth read1.fq.gz read2.fq.gz | samtools sort -o sorted.bam -`

**Explanation:** Sets maximum insert size to 500bp, preventing spurious alignments from incorrectly paired fragments, and pipes directly to a sorted BAM output.

### Align with read group information for downstream sample pooling

**Args:** `-t 8 -R '@RG\tID:sample1\tSM:experiment1\tPL:ILLUMINA' hg38.fa.bwameth reads.fq.gz > rg_tagged.sam`

**Explanation:** Adds read group metadata during alignment, essential for correctly pooling multiple samples in methylome analysis and required by most epigenomics pipelines.

### Output all alignments including multimapping reads

**Args:** `-t 8 -a hg38.fa.bwameth reads.fq.gz > all_hits.sam`

**Explanation:** Reports all valid alignments for reads mapping to multiple locations using the `-a` flag, necessary for unbiased methylation calling in repetitive regions.