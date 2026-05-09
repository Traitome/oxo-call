---
name: chorus2-skill
category: Genomic Assembly
description: Long-read genome assembly tool for error correction and assembly of PacBio and Oxford Nanopore sequencing data into contigs.
tags: genomics, assembly, long-reads, nanopore, pacbio, bioinformatics, genome-assembly
author: AI-generated
source_url: https://github.com/marcelm/chorus2
---

## Concepts

- **Input Format:** chorus2 accepts raw long-read data in FASTA or FASTQ format. Reads should be uncompressed or gzipped (.gz). The tool performs error correction before assembly, making it suitable for high-error-rate long reads from third-generation sequencing platforms.
- **Two-Stage Process:** chorus2 operates in two stages — first correctingRead errors using overlapping read information, then assembling corrected reads into final contigs. This two-pass approach reduces spurious overlaps caused by sequencing errors.
- **Genome Size Parameter:** The `-gs` or `--genome-size` parameter estimates the expected genome size in base pairs (e.g., 5m for 5 megabases). This guides memory allocation and overlap detection sensitivity. Using an accurate estimate improves assembly quality.
- **Companion Binary:** chorus2-build constructs index files for faster overlap detection in subsequent runs. Building indices once and reusing them across multiple assemblies of the same dataset saves computational time.

## Pitfalls

- **Undersized Genome Estimate:** Setting the genome size parameter too small causes chorus2 to miss legitimate overlaps between reads, leading to fragmented assemblies with many small contigs and reduced coverage. Always use an overestimate when the exact genome size is unknown.
- **Excessive Error Tolerance:** Setting the error rate threshold (e.g., `--max-error-rate`) too высоко allows false overlaps between non-homologous regions to form, creating chimeric contigs that concatenate unrelated genomic segments. Keep error rates below 0.2 for typical long-read data.
- **Insufficient Thread Allocation:** Running chorus2 with too few threads causes slow overlap detection, especially for large datasets. Long-read assembly is computationally intensive — allocate at least 8 threads for datasets over 10 Gbases.
- **Mismatched Read Type:** Using chorus2 configured for PacBio data on Oxford Nanopore reads (or vice versa) without adjusting error rate parameters reducesassembly accuracy. Each platform has distinct error profiles; adjust threshold parameters accordingly.

## Examples

### Assemble a bacterial genome from PacBio raw reads
**Args:** `-gs 5m --pacbio input.fq.gz -t 16 -o assembly.fasta`
**Explanation:** The genome size is set to 5 megabases, PacBio mode is enabled, 16 threads are allocated for parallel processing, and output is written to assembly.fasta.

### Build an index for reuse across multiple assemblies
**Args:** `chorus2-build -i input.fq.gz -o read_index`
**Explanation:** The companion binary builds an index file from input reads for faster overlap detection in subsequent chorus2 runs, avoiding redundant computation.

### Assemble Nanopore reads with adjusted error tolerance
**Args:** `-gs 3g --nanopore --max-error-rate 0.15 reads.fq.gz -t 24 -o nanopore_assembly.fasta`
**Explanation:** Nanopore mode is specified with a slightly higher error rate (0.15) to account for the platform's error profile, and 24 threads handle the larger eukaryotic-sized dataset.

### Run with pre-built index to speed up assembly
**Args:** `-gs 5m --index read_index -t 16 -o fast_assembly.fasta`
**Explanation:** Using a previously built index skips the index construction step, significantly reducing runtime when re-assembling or testing different parameters on the same dataset.

### Correct reads only without final assembly
**Args:** `--stage correct -gs 5m input.fq.gz -o corrected_reads.fasta`
**Explanation:** The pipeline stops after the correction stage, outputting error-corrected reads that can be used as input for other assemblers or for downstream validation.

### Assemble with custom overlap sensitivity
**Args:** `-gs 6m --min-overlap 500 --min-identity 0.85 input.fa.gz -t 12 -o contigs.fasta`
**Explanation:** A minimum overlap of 500 bp and identity threshold of 0.85 filters low-quality overlaps, improving assembly specificity for repetitive genomes where spurious overlaps are common.