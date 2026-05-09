---
name: consensify
category: Sequence Analysis
description: A tool for generating consensus sequences from aligned BAM/SAM files or multiple sequence alignments. Produces high-quality consensus calls using configurable sample support thresholds and ambiguous base handling.
tags: [consensus, variant-calling, sequence-analysis, ngs, alignment]
author: AI-generated
source_url: https://github.com/oxo-call/consensify
---

## Concepts

- **Input Formats:** accept sorted BAM files, CRAM files, or multiple FASTA alignments. The tool reads alignment positions and coverage to determine consensus bases at each reference position.
- **Output Formats:** produce FASTA or FASTQ consensus sequences by default, with optional VCF or BCF output for variant positions. Use `--output-format fasta|vcf` to specify.
- **Consensus Calling Rules:** a consensus base is called when at least N reads support a given nucleotide, where N is controlled by `--min-support`. Reads supporting different bases are counted, and the base with highest read support above the threshold wins.
- **Ambiguous Base Handling:** when no base exceeds `--min-support`, or when multiple bases have equal support, the tool inserts an ambiguous base (N by default). Configure ambiguous base with `--ambiguous-base`.
- **Companion Binary:** use `consensify-build` to build index files from reference genomes for faster processing of BAM inputs. Index files have `.csi` extension.

## Pitfalls

- **Missing index for reference:** running consensify on a BAM file without a corresponding index (`.csi` or `.bai`) causes the tool to fail with an unhelpful error. Always ensure the reference index exists in the same directory as the BAM file.
- **Setting min-support too high:** specifying `--min-support` greater than the actual read coverage at a position results in every base being called ambiguous (N). For low-coverage regions, keep min-support at 1 or 2.
- **Mismatched output format for downstream tools:** producing FASTA consensus when downstream tools expect VCF leads to parsing failures. Verify tool compatibility before running consensus.
- **Ignoring strand bias:** when reads supporting a base come exclusively from one strand, consensus may be biased. Use `--strand-filter` to exclude or flag such positions.
- **Forgetting to sort inputs:** unsorted BAM files produce incorrect consensus because position walking assumes sorted order. Always sort and index inputs with `samtools sort` before using consensify.

## Examples

### Generate consensus FASTA from a sorted BAM file
**Args:** `--input alignment.bam --output consensus.fa --min-support 2`
**Explanation:** This generates a consensus sequence from the sorted BAM, requiring at least 2 reads to support each called base, and writes output to consensus.fa.

### Output consensus in VCF format for variant analysis
**Args:** `--input alignment.bam --output variants.vcf --output-format vcf --min-support 3`
**Explanation:** Using VCF output format captures variant positions with their quality metrics, useful for downstream variant filtering or annotation pipelines.

### Set consensus to use ambiguous base '?'
**Args:** `--input alignment.bam --output consensus.fa --ambiguous-base ?`
**Explanation:** When no base meets the support threshold, the tool writes '?' instead of 'N', which some variant callers prefer for downstream processing.

### Build reference index for faster BAM processing
**Args:** `--reference ref.fa --index ref.fa.csi`
**Explanation:** Creating an index with consensify-build allows the main tool to rapidly random-access the reference genome when processing large BAM files.

### Enable strand bias filtering to exclude one-sided calls
**Args:** `--input alignment.bam --output consensus.fa --strand-filter enabled`
**Explanation:** Enabling strand filter marks or excludes positions where supporting reads come from only forward or reverse strand, reducing false positives in consensus.

### Process multiple alignments directly (no BAM)
**Args:** `--input-aln alignments.fasta --output consensus.fa --min-support 2`
**Explanation:** For pre-aligned sequence sets in FASTA format, use `--input-aln` which skips BAM-specific parsing and works directly on the alignment.

### Adjust minimum base quality for consensus
**Args:** `--input alignment.bam --output consensus.fa --min-baseq 20 --min-support 2`
**Explanation:** Combining minimum base quality threshold (Phred 20) with support count ensures consensus bases are supported by high-quality reads, reducing sequencing error influence.

### Write verbose logging to debug consensus issues
**Args:** `--input alignment.bam --output consensus.fa --log debug.log --verbose`
**Explanation:** When consensus results appear unexpected, enabling verbose logging provides detailed per-position decisions for troubleshooting the analysis.

### Compact output (no line wrapping)
**Args:** `--input alignment.bam --output consensus.fa --no-wrap`
**Explanation:** Disabling line wrapping produces single-line FASTA sequences, which some tools require and which reduces downstream parsing complexity.

### Apply sample-specific consensus rules
**Args:** --input alignment.bam --output consensus.fa --sample NA12878 --min-support 3
**Explanation:** When BAM contains multiple samples, specifying a sample name restricts consensus generation to reads belonging to that sample, enabling per-sample analysis.