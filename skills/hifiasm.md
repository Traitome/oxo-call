---
name: hifiasm
category: assembly
description: Haplotype-resolved de novo assembler for PacBio HiFi reads with trio and Hi-C phasing support
tags: [assembly, hifi, pacbio, haplotype, phasing, de-novo, long-read, diploid]
author: oxo-call built-in
source_url: "https://github.com/chhylp123/hifiasm"
---

## Concepts

- Hifiasm produces highly accurate haplotype-resolved assemblies from PacBio HiFi (CCS) reads.
- Default output: two haplotype-resolved assemblies (hap1, hap2) in GFA format + primary assembly.
- Use -o for output prefix; -t for threads; reads are positional argument at the end.
- GFA output must be converted to FASTA: awk '/^S/ {print ">"$2; print $3}' output.bp.hap1.p_ctg.gfa > hap1.fasta.
- For Hi-C phasing: use --h1 and --h2 for Hi-C R1 and R2 reads alongside HiFi reads.
- For trio phasing: use --ul for ultra-long ONT reads; --pat/--mat for paternal/maternal k-mer databases.
- Hifiasm achieves near-complete T2T assemblies for many organisms with HiFi + Hi-C data.
- Use -l 0/1/2/3 to control duplication purging level (0=no purge, 3=aggressive).

## Pitfalls

- Hifiasm outputs GFA format, not FASTA — convert with awk before using in downstream tools.
- HiFi reads below Q20 accuracy reduce assembly quality significantly.
- Without Hi-C or trio data, hifiasm produces a partially phased assembly (primary + alternate contigs).
- --primary flag produces a single primary assembly without haplotype separation.
- Hifiasm requires significant RAM for large genomes — human genome assembly needs ~300 GB RAM.
- For metagenomes, Flye or metaSPAdes are more appropriate than Hifiasm.

## Examples

### assemble genome from PacBio HiFi reads
**Args:** `-o assembly -t 32 hifi_reads.fastq.gz`
**Explanation:** -o output prefix; -t 32 threads; creates assembly.bp.hap1.p_ctg.gfa and assembly.bp.hap2.p_ctg.gfa

### haplotype-resolved assembly with Hi-C phasing data
**Args:** `-o phased_assembly -t 32 --h1 hic_R1.fastq.gz --h2 hic_R2.fastq.gz hifi_reads.fastq.gz`
**Explanation:** --h1/--h2 provide Hi-C reads for haplotype phasing alongside HiFi reads

### convert hifiasm GFA output to FASTA
**Args:** `/^S/ {print ">"$2; print $3}`
**Explanation:** awk command: awk '/^S/ {print ">"$2; print $3}' assembly.bp.hap1.p_ctg.gfa > hap1.fasta

### assemble with ultra-long ONT reads for improved scaffolding
**Args:** `-o assembly -t 32 --ul ultralong_reads.fastq.gz hifi_reads.fastq.gz`
**Explanation:** --ul adds ultra-long ONT reads (>100kb) to improve scaffold N50 and telomere-to-telomere assembly

### assemble with aggressive duplicate purging
**Args:** `-o assembly -t 32 -l 3 hifi_reads.fastq.gz`
**Explanation:** -l 3 aggressive purging level; useful for highly heterozygous genomes with duplicate haplotigs
