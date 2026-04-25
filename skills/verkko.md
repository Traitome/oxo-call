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
**Explanation:** verkko command; --hifi hifi_reads.fastq.gz HiFi reads input; -d assembly_out output directory; --threads 64 parallelism

### assemble a genome with both HiFi and ONT reads for maximum continuity
**Args:** `--hifi hifi_reads.fastq.gz --ont ont_reads.fastq.gz -d hybrid_assembly --threads 64`
**Explanation:** verkko command; --hifi hifi_reads.fastq.gz HiFi reads; --ont ont_reads.fastq.gz ONT reads; -d hybrid_assembly output directory; --threads 64 parallelism; combined mode resolves complex repeats

### perform haplotype-resolved assembly with trio binning
**Args:** `--hifi hifi_reads.fastq.gz --ont ont_reads.fastq.gz --hap-kmers maternal.meryl paternal.meryl -d trio_assembly --threads 64`
**Explanation:** verkko command; --hifi hifi_reads.fastq.gz HiFi reads; --ont ont_reads.fastq.gz ONT reads; --hap-kmers maternal.meryl paternal.meryl Meryl databases from parental reads; -d trio_assembly output directory; --threads 64 parallelism; phasing into hap1/hap2

### run Verkko on a cluster using Slurm via Snakemake
**Args:** `--hifi hifi_reads.fastq.gz --ont ont_reads.fastq.gz -d assembly_out --threads 4 --snakeopts "--cluster 'sbatch -c {threads} --mem {resources.mem_gb}G' --jobs 50"`
**Explanation:** verkko command; --hifi hifi_reads.fastq.gz HiFi reads; --ont ont_reads.fastq.gz ONT reads; -d assembly_out output directory; --threads 4 local threads; --snakeopts Snakemake cluster arguments; --cluster 'sbatch...' Slurm submission; --jobs 50 parallel jobs

### resume an interrupted Verkko assembly
**Args:** `--hifi hifi_reads.fastq.gz --ont ont_reads.fastq.gz -d assembly_out --threads 64 --resume`
**Explanation:** verkko command; --hifi hifi_reads.fastq.gz HiFi reads; --ont ont_reads.fastq.gz ONT reads; -d assembly_out output directory; --threads 64 parallelism; --resume continues from last checkpoint

### assemble with ONT reads only (no HiFi)
**Args:** `--ont ont_reads.fastq.gz -d ont_assembly --threads 64`
**Explanation:** verkko command; --ont ont_reads.fastq.gz ONT reads only; -d ont_assembly output directory; --threads 64 parallelism; uses longer k-mer graph; quality lower than HiFi+ONT

### run Verkko with Hi-C data for phasing
**Args:** `--hifi hifi_reads.fastq.gz --hic1 hic_R1.fastq.gz --hic2 hic_R2.fastq.gz -d hic_assembly --threads 64`
**Explanation:** verkko command; --hifi hifi_reads.fastq.gz HiFi reads; --hic1 hic_R1.fastq.gz --hic2 hic_R2.fastq.gz Hi-C reads; -d hic_assembly output directory; --threads 64 parallelism; Hi-C for haplotype phasing

### generate assembly statistics
**Args:** `stats assembly.fasta`
**Explanation:** verkko stats subcommand; assembly.fasta input assembly; outputs N50, contig count, total size

### evaluate assembly completeness with BUSCO
**Args:** `busco -i assembly.fasta -l eukaryota_odb10 -o busco_out -m genome --cpu 16`
**Explanation:** busco command; -i assembly.fasta input assembly; -l eukaryota_odb10 lineage database; -o busco_out output directory; -m genome mode; --cpu 16 threads; assesses gene set completeness

### align reads to assembly for quality check
**Args:** `minimap2 -ax map-hifi assembly.fasta hifi_reads.fastq.gz | samtools sort -o aligned.bam`
**Explanation:** minimap2 command; -ax map-hifi HiFi alignment preset; assembly.fasta reference; hifi_reads.fastq.gz reads; | samtools sort pipes to samtools; -o aligned.bam sorted BAM output

### run Verkko with custom k-mer size
**Args:** `--hifi hifi_reads.fastq.gz --k-mer-size 31 -d custom_k_assembly --threads 64`
**Explanation:** verkko command; --hifi hifi_reads.fastq.gz HiFi reads; --k-mer-size 31 custom k-mer; -d custom_k_assembly output directory; --threads 64 parallelism; default is 21 for HiFi
