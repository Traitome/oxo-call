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
- For trio phasing: use --ul for ultra-long ONT reads; -1/-2 for paternal/maternal k-mer databases.
- Hifiasm achieves near-complete T2T assemblies for many organisms with HiFi + Hi-C data.
- Use -l 0/1/2/3 to control duplication purging level (0=no purge, 3=aggressive).
- --primary outputs a primary assembly and an alternate assembly instead of haplotype-resolved assemblies.
- --hom-cov specifies homozygous read coverage manually; useful when auto-detection fails on low-coverage data.
- --ul integrates ultra-long ONT reads (>100kb) to improve contiguity and enable telomere-to-telomere assembly.
- --dual-scaf enables dual scaffolding mode for scaffolding with Hi-C and ultra-long reads simultaneously.
- -z INT trims INT bases from both ends of reads; useful for removing adapter contamination from older HiFi data.
- --telo-m specifies telomere motif for telomere identification; default CCCTAA for human (vertebrate telomere).

## Pitfalls

- Hifiasm outputs GFA format, not FASTA — convert with awk before using in downstream tools.
- HiFi reads below Q20 accuracy reduce assembly quality significantly.
- Without Hi-C or trio data, hifiasm produces a partially phased assembly (primary + alternate contigs).
- --primary flag produces a single primary assembly without haplotype separation.
- Hifiasm requires significant RAM for large genomes — human genome assembly needs ~300 GB RAM.
- For metagenomes, Flye or metaSPAdes are more appropriate than Hifiasm.
- --pat/--mat flags mentioned in old documentation are replaced by -1/-2 for paternal/maternal k-mer databases.
- Default -l 0 for trio assemblies disables purging; for HiFi-only assemblies default is -l 3 (aggressive).
- *.bin files are reused across runs; delete them when changing parameters or input data to avoid incorrect results.
- --hic*.bin files should be deleted when tuning Hi-C phasing parameters to ensure fresh alignment.
- -l 0 is recommended for inbred/homozygous genomes to disable purging of true homozygous sequence.
- --hom-cov auto-detection may fail on low-coverage or highly heterozygous samples; specify manually if results look incorrect.

## Examples

### assemble genome from PacBio HiFi reads
**Args:** `-o assembly -t 32 hifi_reads.fastq.gz`
**Explanation:** -o assembly output prefix; -t 32 threads; hifi_reads.fastq.gz input file; creates assembly.bp.hap1.p_ctg.gfa and assembly.bp.hap2.p_ctg.gfa haplotype-resolved assemblies

### haplotype-resolved assembly with Hi-C phasing data
**Args:** `-o phased_assembly -t 32 --h1 hic_R1.fastq.gz --h2 hic_R2.fastq.gz hifi_reads.fastq.gz`
**Explanation:** -o phased_assembly output prefix; -t 32 threads; --h1 hic_R1.fastq.gz --h2 hic_R2.fastq.gz provide Hi-C reads for haplotype phasing; hifi_reads.fastq.gz HiFi input

### assemble genome with custom number of haplotype rounds
**Args:** `-o assembly -t 32 --n-hap 4 hifi_reads.fastq.gz`
**Explanation:** -o assembly output prefix; -t 32 threads; --n-hap 4 sets expected ploidy level to 4; hifi_reads.fastq.gz input; default is 2 for diploid assemblies

### assemble with ultra-long ONT reads for improved scaffolding
**Args:** `-o assembly -t 32 --ul ultralong_reads.fastq.gz hifi_reads.fastq.gz`
**Explanation:** -o assembly output prefix; -t 32 threads; --ul ultralong_reads.fastq.gz adds ultra-long ONT reads (>100kb) for improved scaffold N50; hifi_reads.fastq.gz HiFi input; enables telomere-to-telomere assembly

### assemble with aggressive duplicate purging
**Args:** `-o assembly -t 32 -l 3 hifi_reads.fastq.gz`
**Explanation:** -o assembly output prefix; -t 32 threads; -l 3 aggressive purging level; hifi_reads.fastq.gz input; useful for highly heterozygous genomes with duplicate haplotigs

### generate primary/alternate assembly instead of haplotype-resolved
**Args:** `-o assembly -t 32 --primary hifi_reads.fastq.gz`
**Explanation:** -o assembly output prefix; -t 32 threads; --primary outputs primary assembly (p_ctg.gfa) and alternate assembly (a_ctg.gfa); hifi_reads.fastq.gz input; useful when haplotype phasing is not required

### trio binning assembly with parental k-mers
**Args:** `-o trio_assembly -t 32 -1 paternal.yak -2 maternal.yak hifi_reads.fastq.gz`
**Explanation:** -o trio_assembly output prefix; -t 32 threads; -1 paternal.yak -2 maternal.yak provide parental k-mer databases from yak count; hifi_reads.fastq.gz input; produces fully phased haplotypes without Hi-C

### trim adapter sequences from older HiFi reads
**Args:** `-o assembly -t 32 -z 20 hifi_reads.fastq.gz`
**Explanation:** -o assembly output prefix; -t 32 threads; -z 20 trims 20bp from both ends of reads; hifi_reads.fastq.gz input; removes adapter contamination common in older HiFi data

### specify homozygous coverage for low-coverage samples
**Args:** `-o assembly -t 32 --hom-cov 30 hifi_reads.fastq.gz`
**Explanation:** -o assembly output prefix; -t 32 threads; --hom-cov 30 manually sets homozygous coverage; hifi_reads.fastq.gz input; useful when auto-detection fails on low-coverage or aneuploid samples

### dual scaffolding with Hi-C and ultra-long reads
**Args:** `-o assembly -t 32 --h1 hic_R1.fq.gz --h2 hic_R2.fq.gz --ul ont_ul.fq.gz --dual-scaf hifi_reads.fastq.gz`
**Explanation:** -o assembly output prefix; -t 32 threads; --h1 hic_R1.fq.gz --h2 hic_R2.fq.gz Hi-C reads; --ul ont_ul.fq.gz ultra-long reads; --dual-scaf enables scaffolding with both; hifi_reads.fastq.gz input; produces more contiguous scaffolds

### identify telomeres during assembly
**Args:** `-o assembly -t 32 --telo-m CCCTAA hifi_reads.fastq.gz`
**Explanation:** -o assembly output prefix; -t 32 threads; --telo-m CCCTAA specifies telomere motif for vertebrates; hifi_reads.fastq.gz input; enables telomere identification and T2T assembly assessment

### assemble ONT simplex reads (beta mode)
**Args:** `-o assembly -t 32 --ont ont_reads.fastq.gz`
**Explanation:** -o assembly output prefix; -t 32 threads; --ont enables ONT simplex read assembly mode; ont_reads.fastq.gz nanopore input; experimental feature for nanopore-only assembly

### resume assembly from existing overlap files
**Args:** `-o assembly -t 32 -i hifi_reads.fastq.gz`
**Explanation:** -o assembly output prefix; -t 32 threads; -i ignores saved read correction and overlaps; hifi_reads.fastq.gz input; forces re-computation when previous steps failed or parameters changed
