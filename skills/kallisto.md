---
name: kallisto
category: rna-seq
description: Ultrafast pseudoalignment RNA-seq transcript quantification tool
tags: [rna-seq, quantification, pseudoalignment, transcript, expression, tpm, counts]
author: oxo-call built-in
source_url: "https://pachterlab.github.io/kallisto/"
---

## Concepts
- Kallisto uses pseudoalignment to quantify RNA-seq data; it is 20x faster than alignment-based tools with comparable accuracy.
- Two-step workflow: (1) kallisto index -i index.idx transcriptome.fa; (2) kallisto quant -i index.idx -o outdir R1.fq R2.fq.
- Kallisto requires transcriptome FASTA (cDNA), not genome FASTA.
- Output includes abundance.tsv (TPM and estimated counts), run_info.json (run statistics), and abundance.h5 (for sleuth).
- Use --bootstrap-samples 100 (-b 100) for bootstrap variance estimation; required for sleuth differential expression.
- Strandedness: --rf-stranded for reverse-strand (dUTP), --fr-stranded for forward-strand; default is unstranded.
- kb-python (kallisto|bustools) is the modern single-cell extension for scRNA-seq processing.
- Index options: -k sets k-mer size (default 31, odd only, max 63); --aa for amino acid sequences; --d-list for masking sequences.
- Quant options: --pseudobam outputs pseudoalignments as BAM; --genomebam projects to genome coordinates (requires -g GTF).
- Single-end mode: requires --single, -l (mean fragment length), and -s (standard deviation).
- BUS format: kallisto bus generates BUS files for single-cell data with -x for technology (10xv3, 10xv2, etc.).
- quant-tcc: quantifies from transcript-compatibility counts for long reads or pre-computed data.
- Thread control: -t/--threads for parallel processing; beneficial for index building and quantification.

## Pitfalls
- CRITICAL: Kallisto ARGS must start with a subcommand (index, quant, quant-tcc, bus, h5dump, inspect, version, cite) — never with flags like -i, -o, -b. The subcommand ALWAYS comes first.
- Kallisto requires transcriptome cDNA FASTA, NOT genome FASTA — indexing the genome produces wrong results.
- Without --bootstrap-samples, downstream differential expression with sleuth cannot estimate variance.
- For paired-end, both FASTQ files are passed as positional arguments (no -1/-2 flags) after -o output_dir.
- kallisto does not output BAM by default — use kallisto quant --pseudobam for BAM output (if needed).
- Forgetting strandedness option for stranded libraries leads to approximately half the reads assigned per strand.
- kallisto quant outputs to a directory — make sure the output directory does not already exist or use a fresh path.
- --genomebam requires both -g GTF and optionally -c chromosomes file for proper genome coordinate projection.
- k-mer size (-k) must be odd and ≤63; even values or values >63 will cause errors.
- Single-end quantification requires explicit --single, -l, and -s parameters; missing any will fail.
- The --d-list option masks sequences from quantification; useful for removing rRNA or other contaminants.
- For single-cell data, use kb-python (kb count) instead of direct kallisto bus for easier workflow.

## Examples

### build a kallisto index from a transcriptome FASTA
**Args:** `index -i transcriptome.idx transcriptome.fa`
**Explanation:** -i specifies the index output file; transcriptome.fa is the cDNA/transcript FASTA

### quantify paired-end RNA-seq reads
**Args:** `quant -i transcriptome.idx -o sample_output -b 100 --threads 8 R1.fastq.gz R2.fastq.gz`
**Explanation:** -o output directory; -b 100 bootstrap samples for variance; R1 and R2 are positional arguments

### quantify single-end RNA-seq reads with fragment length parameters
**Args:** `quant -i transcriptome.idx -o sample_output --single -l 200 -s 20 -b 100 --threads 8 reads.fastq.gz`
**Explanation:** --single for SE mode; -l mean fragment length; -s fragment length SD; required for SE mode

### quantify strand-specific reverse-strand paired-end RNA-seq
**Args:** `quant -i transcriptome.idx -o sample_output --rf-stranded -b 100 --threads 8 R1.fastq.gz R2.fastq.gz`
**Explanation:** --rf-stranded for dUTP/TruSeq reverse-strand libraries; use --fr-stranded for forward-strand

### quantify multiple samples in batch
**Args:** `quant -i transcriptome.idx -o sample1_out -b 50 --threads 4 sample1_R1.fq.gz sample1_R2.fq.gz`
**Explanation:** run kallisto quant once per sample; loop in shell for batch processing

### build index with custom k-mer size
**Args:** `index -k 21 -i transcriptome.idx transcriptome.fa`
**Explanation:** -k 21 sets k-mer size to 21 (must be odd); smaller k-mers improve sensitivity for short reads or divergent sequences

### build index masking rRNA sequences
**Args:** `index -i transcriptome.idx --d-list rRNA.fa transcriptome.fa`
**Explanation:** --d-list rRNA.fa masks rRNA sequences from quantification; reduces rRNA contamination in results

### generate pseudoalignments as BAM file
**Args:** `quant -i transcriptome.idx -o sample_output --pseudobam -b 100 --threads 8 R1.fastq.gz R2.fastq.gz`
**Explanation:** --pseudobam outputs pseudoalignments.bam; useful for visualization in IGV or downstream analysis

### project pseudoalignments to genome coordinates
**Args:** `quant -i transcriptome.idx -o sample_output --genomebam -g annotation.gtf -c chromosomes.txt -b 100 R1.fastq.gz R2.fastq.gz`
**Explanation:** --genomebam projects to genome coordinates; requires -g GTF and optionally -c chromosomes file

### generate BUS file for single-cell data
**Args:** `bus -i transcriptome.idx -o bus_output -x 10xv3 --threads 8 R1.fastq.gz R2.fastq.gz`
**Explanation:** bus command generates BUS format for single-cell; -x specifies technology (10xv3, 10xv2, etc.)

### list supported single-cell technologies
**Args:** `bus --list`
**Explanation:** lists all supported single-cell technologies with their barcode/UMI configurations

### convert HDF5 output to plaintext
**Args:** `h5dump abundance.h5 > abundance_dump.tsv`
**Explanation:** h5dump converts HDF5 abundance file to plaintext; useful when HDF5 libraries unavailable

### inspect index file information
**Args:** `inspect transcriptome.idx`
**Explanation:** inspect displays index statistics: number of targets, k-mer size, version information
