---
name: verkko
category: assembly
description: Graph-based telomere-to-telomere genome assembler combining HiFi and Oxford Nanopore reads
tags: [assembly, hifi, ont, pacbio, long-read, genome, graph, telomere-to-telomere]
author: oxo-call built-in
source_url: "https://github.com/marbl/verkko"
---

## Concepts

- Verkko builds a consensus De Bruijn graph from HiFi reads, then uses long ONT reads to resolve tangles and phase haplotypes.
- Both --hifi and --ont inputs are optional individually, but combining them yields the most complete assemblies.
- Haplotype-resolved (phased) assembly requires trio binning data (--hap-kmers maternal.meryl paternal.meryl) or Hi-C reads.
- Verkko is a Snakemake workflow internally; use --snakeopts to pass Snakemake flags like --cores or cluster submission options.
- Output is written to the directory specified with -d; key outputs are assembly.fasta (unphased) or haplotype1.fasta / haplotype2.fasta (phased).
- Verkko requires substantial RAM; a human genome assembly typically needs 256-512 GB RAM and many CPU hours.

## Pitfalls

- Running without --threads causes Verkko to use only 1 thread; always set --threads to the number of available cores.
- Mixing HiFi reads below 99% accuracy with Verkko degrades the initial graph quality — filter low-quality reads beforehand.
- The -d output directory must not contain a previous partial run unless --resume is used; stale intermediate files cause errors.
- ONT reads shorter than 10 kb contribute little to tangle resolution; pre-filter with NanoFilt or equivalent if median length is low.
- Trio-binning requires Meryl k-mer databases from parental Illumina reads; using raw FASTQs directly in --hap-kmers will fail.
- Verkko writes large intermediate graph files; ensure the output filesystem has at least 10x the input data size free.

## Examples

### assemble a genome using only HiFi reads
**Args:** `--hifi hifi_reads.fastq.gz -d assembly_out --threads 64`
**Explanation:** HiFi-only assembly; -d sets output directory; --threads should match available CPUs

### assemble a genome with both HiFi and ONT reads for maximum continuity
**Args:** `--hifi hifi_reads.fastq.gz --ont ont_reads.fastq.gz -d hybrid_assembly --threads 64`
**Explanation:** combined HiFi+ONT mode; ONT reads resolve complex repeats and improve contig length

### perform haplotype-resolved assembly with trio binning
**Args:** `--hifi hifi_reads.fastq.gz --ont ont_reads.fastq.gz --hap-kmers maternal.meryl paternal.meryl -d trio_assembly --threads 64`
**Explanation:** --hap-kmers takes Meryl databases built from parental short reads for phasing into hap1/hap2

### run Verkko on a cluster using Slurm via Snakemake
**Args:** `--hifi hifi_reads.fastq.gz --ont ont_reads.fastq.gz -d assembly_out --threads 4 --snakeopts "--cluster 'sbatch -c {threads} --mem {resources.mem_gb}G' --jobs 50"`
**Explanation:** --snakeopts passes Snakemake arguments for cluster execution; --threads here sets the local thread count

### resume an interrupted Verkko assembly
**Args:** `--hifi hifi_reads.fastq.gz --ont ont_reads.fastq.gz -d assembly_out --threads 64 --resume`
**Explanation:** --resume continues from the last completed Snakemake checkpoint; reuses existing intermediate files

### assemble with ONT reads only (no HiFi)
**Args:** `--ont ont_reads.fastq.gz -d ont_assembly --threads 64`
**Explanation:** ONT-only mode uses a longer k-mer graph; quality is lower than HiFi+ONT but works without PacBio data

### run Verkko with Hi-C data for phasing
**Args:** `--hifi hifi_reads.fastq.gz --hic1 hic_R1.fastq.gz --hic2 hic_R2.fastq.gz -d hic_assembly --threads 64`
**Explanation:** uses Hi-C reads for haplotype phasing; alternative to trio binning

### generate assembly statistics
**Args:** `stats assembly.fasta`
**Explanation:** outputs assembly statistics including N50, contig count, and total size

### evaluate assembly completeness with BUSCO
**Args:** `busco -i assembly.fasta -l eukaryota_odb10 -o busco_out -m genome --cpu 16`
**Explanation:** runs BUSCO on Verkko assembly; assesses gene set completeness

### align reads to assembly for quality check
**Args:** `minimap2 -ax map-hifi assembly.fasta hifi_reads.fastq.gz | samtools sort -o aligned.bam`
**Explanation:** aligns HiFi reads back to assembly; check coverage uniformity

### run Verkko with custom k-mer size
**Args:** `--hifi hifi_reads.fastq.gz --k-mer-size 31 -d custom_k_assembly --threads 64`
**Explanation:** --k-mer-size adjusts De Bruijn graph k-mer; default is 21 for HiFi
