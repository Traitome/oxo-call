---
name: bbmix
category: Metagenomics / Abundance Estimation
description: A BBMap suite tool that estimates the mixture fraction (relative abundance) of each reference sequence in a mixed read population by modeling coverage depth and read assignment. It takes a set of reference sequences and a SAM/BAM of reads mapped to those references, then reports the proportional representation of each reference in the sample.
tags:
  - bbmap
  - metagenomics
  - abundance-estimation
  - coverage
  - read-ratio
  - community-profiling
author: AI-generated
source_url: https://sourceforge.net/projects/bbmap/
---

## Concepts

- **Input requirements**: Bbmix requires two primary inputs — a reference FASTA file (containing all candidate sequences) and a SAM/BAM file of reads pre-aligned to that reference. Reads must be mapped before running bbmix; the tool does not perform alignment itself.
- **Abundance estimation model**: Bbmix computes relative abundance for each reference sequence by analyzing the depth of coverage and the number of reads assigned to each sequence, then normalizes across all references to produce proportional fractions that sum to 1.0.
- **Output format**: The primary output is a plain-text table listing each reference sequence identifier, the estimated read count or coverage, and the computed mixture fraction (decimal proportion). An optional coverage histogram can be written for downstream visualization.
- **Handling ambiguous mappings**: When a read maps equally well to multiple references (multi-mapped reads), bbmix distributes the read weight proportionally across those references rather than arbitrarily assigning it to one, which is critical for accurate estimation in closely related organisms.
- **Reference indexing convention**: Reference files passed to bbmix should use standard BBMap suffixes (.fasta, .fa, .fastq, .fq). The companion script `bbmerge.sh` is unrelated; read merging must be done separately before alignment if needed.

## Pitfalls

- **Running bbmix on unmapped or unaligned reads**: Passing a FASTQ file directly to bbmix will produce no meaningful output because bbmix expects mapped read coordinates. Users must first align reads to the reference using `bbmap.sh` or another aligner, and then pass the resulting SAM/BAM file.
- **Reference sequences not present in the alignment input**: If a reference sequence in the FASTA file has zero mapped reads, bbmix will assign it a mixture fraction of 0 (or omit it), which may be correct but can also indicate that the alignment step used a tolerance threshold that excluded those reads.
- **Inconsistent reference naming between FASTA and SAM**: If sequence headers in the reference FASTA differ from the RNAME field in the SAM file (e.g., due to added prefixes or different whitespace parsing), bbmix will fail to match reads to references and will report zero abundance for all entries.
- **Overwriting output files without warning**: If the specified output file already exists, bbmix will silently overwrite it, potentially losing previous results — especially problematic in batch pipelines processing multiple samples.
- **Ignoring multi-mapped read distribution method**: In samples with highly conserved regions (e.g., ribosomal genes shared across species), how multi-mapped reads are distributed can shift mixture estimates by several percentage points. Failing to check or configure this parameter can yield misleading relative abundances.

## Examples

### Estimate mixture fractions for a two-species viral sample
**Args:** `ref=viral_references.fasta in=sample_mapped.sam out=viral_mixture.txt`
**Explanation:** This runs bbmix with the reference set and a SAM of pre-aligned reads, writing the proportional abundances to a plain-text results file.

### Add column headers and specify a sample name in the output
**Args:** `ref=references.fasta in=mapped.bam out=abundance.tsv header=t samples=SampleA`
**Explanation:** Including a header row and sample name label makes the output easier to parse when concatenating results across multiple samples for comparative analysis.

### Request a coverage histogram alongside the fraction table
**Args:** `ref=ref_db.fasta in=mapped.sam out=fractions.txt covhist=coverage_profile.txt`
**Explanation:** The additional coverage histogram file enables downstream inspection of per-base depth, which can reveal systematic biases such as hypervariable regions skewing abundance estimates.

### Use a strict ambiguity threshold to downweight multi-mapped reads
**Args:** `ref=ref_genomes.fasta in=aln.sam out=strict_abund.txt ambiguity=0.5`
**Explanation:** Setting an ambiguity threshold discards reads that map to more than two references, producing more conservative estimates when the reference set contains paralogous sequences.

### Process multiple samples in batch using a list file
**Args:** `ref=ref.fasta in=sample_list.txt out=batch_results/ ambiguous=0.8`
**Explanation:** Passing a file containing one input path per line allows bbmix to iterate over many samples automatically, which is more efficient than individual command invocations in large metagenomic surveys.

### Force overwrite and suppress status messages for scripted pipelines
**Args:** `ref=references.fasta in=sample1.sam out=result.txt overwrite=t 2>/dev/null`
**Explanation:** The overwrite flag permits re-running the same sample without manual deletion, and redirecting stderr to /dev/null keeps log files clean in automated workflows.

### Verify read-to-reference assignment before estimating fractions
**Args:** `ref=ref.fasta in=mapped.sam out=verify.txt stats=t`
**Explanation:** Enabling the stats flag makes bbmix emit a diagnostic summary of total reads, mapped reads, and per-reference read counts, which helps identify mismatched headers or low alignment rates before trusting the mixture fractions.