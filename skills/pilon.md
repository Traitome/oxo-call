---
name: pilon
category: assembly
description: Whole-genome assembly improvement and variant detection using short-read alignments
tags: [polishing, assembly, illumina, variant-calling, indel, snp, genome-improvement]
author: oxo-call built-in
source_url: "https://github.com/broadinstitute/pilon/wiki"
---

## Concepts

- Pilon takes a draft genome assembly and Illumina reads aligned to it (BAM) to correct SNPs, indels, and local misassemblies.
- Input BAM files must be sorted and indexed with samtools; Pilon reads alignment evidence to propose corrections.
- Multiple BAM types can be combined: --frags (paired-end), --jumps (mate-pair), --bam (unpaired/mixed); using all available data improves results.
- Pilon runs as a Java application; heap size must be set with -Xmx to avoid out-of-memory errors on large genomes.
- --changes flag writes a file listing every correction made; review it to assess assembly quality improvement.
- Multiple rounds of Pilon polishing (3-4) can improve accuracy; each round aligns reads to the latest corrected assembly.

## Pitfalls

- Not setting Java heap size (-Xmx) causes OOM errors on genomes larger than a few hundred Mb; use -Xmx64g or larger.
- Input BAMs must have read groups (@RG tags); Pilon may fail or give poor results without them.
- Pilon does not handle circular chromosomes correctly — split circular sequences at origin before polishing if needed.
- Polishing with very low coverage (<10x) can introduce errors; Pilon needs sufficient depth to distinguish errors from variants.
- The --fix all option changes SNPs, indels, gaps, and local; use --fix bases for SNP/indel only to avoid large structural changes.
- Running Pilon on an already-polished Nanopore assembly with Illumina reads can introduce reference bias if coverage is uneven.

## Examples

### polish a draft assembly with paired-end Illumina reads
**Args:** `-Xmx64g -jar pilon.jar --genome draft.fasta --frags aligned.sorted.bam --output polished --changes --threads 16`
**Explanation:** -Xmx64g sets 64 GB Java heap; --frags for paired-end BAM; --changes logs every correction

### polish with mate-pair and paired-end libraries combined
**Args:** `-Xmx128g -jar pilon.jar --genome draft.fasta --frags pe.sorted.bam --jumps mp.sorted.bam --output polished_v2 --threads 16`
**Explanation:** --frags for paired-end, --jumps for mate-pair; combining both library types improves large-scale error correction

### run Pilon fixing only SNPs and small indels (not structural)
**Args:** `-Xmx64g -jar pilon.jar --genome draft.fasta --frags aligned.sorted.bam --output polished --fix bases --threads 16`
**Explanation:** --fix bases restricts corrections to SNPs and indels; avoids aggressive structural changes from --fix all

### generate a VCF of variants found in the assembly
**Args:** `-Xmx64g -jar pilon.jar --genome draft.fasta --frags aligned.sorted.bam --output variants --variant --threads 16`
**Explanation:** --variant mode outputs a VCF of all positions where reads differ from the assembly, useful for quality assessment

### polish a specific set of sequences (e.g., unplaced contigs only)
**Args:** `-Xmx32g -jar pilon.jar --genome contigs.fasta --frags aligned.sorted.bam --output polished_contigs --targets contig_list.txt --threads 8`
**Explanation:** --targets restricts polishing to sequences listed in contig_list.txt; useful for targeted correction

### second round of polishing after re-aligning reads to first round output
**Args:** `-Xmx64g -jar pilon.jar --genome polished.fasta --frags re_aligned.sorted.bam --output polished_r2 --changes --threads 16`
**Explanation:** iterative polishing: re-align reads to polished.fasta with BWA, then run Pilon again; repeat 3-4 times
