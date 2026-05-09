---
name: aliceasm
category: sequence_assembly
description: A de novo genome assembler for assembling short reads into contigs and scaffolds. Supports multiple input formats and provides adjustable coverage thresholds for optimal assembly results.
tags: [genomics, assembly, de-novo, sequence-analysis]
author: AI-generated
source_url: https://github.com/aliceasm/aliceasm
---

## Concepts

- **Input Formats**: Accepts FASTA, FASTQ, and plain text formats for sequencing reads. Supports both single-end and paired-end read libraries. Multiple read files can be specified in a configuration file or via command-line arguments.
- **Coverage Threshold**: The `-c` flag sets the minimum coverage depth required for a k-mer to be included in the assembly graph. Higher values reduce chimeric assemblies but may fragment the output; lower values capture more low-coverage regions but increase errors.
- **K-mer Size Selection**: The `-k` flag specifies the k-mer length used during de Bruijn graph construction. Odd values between 17 and 127 are typical; larger k-mers improve specificity but reduce sensitivity for repeat resolution.
- **Output Structure**: Produces assembled contigs in FASTA format (default: `aliceasm-contigs.fasta`), an assembly graph file (`aliceasm-graph.bin`), and a summary statistics file (`aliceasm-stats.txt`).
- **Thread Control**: The `-t` flag controls the number of parallel threads; memory usage scales approximately linearly with thread count. Insufficient threads may cause assembly to hang on large datasets.

## Pitfalls

- **Specifying an Even K-mer Size**: Using an even number for the `-k` flag causes the assembler to fail with an "odd k-mer required" error. Always use odd integers (e.g., 21, 31, 45) to ensure correct de Bruijn graph construction.
- **Setting Coverage Below Read Depth**: Setting the `-c` threshold higher than the actual average read coverage removes valid k-mers, resulting in fragmented or empty output. Verify your library's coverage using tools like `samtools depth` or FastQC before assembly.
- **Memory Exhaustion on Large Genomes**: The `-m` flag (maximum memory in GB) that is too low causes the assembler to terminate with a memory allocation error. For a 100 Mb genome, at least 8 GB RAM is recommended; larger genomes require proportionally more.
- **Mismatched Read Orientations**: For paired-end libraries, failing to specify the correct library orientation (forward-reverse vs. reverse-forward) via `--library-type` leads to broken or incorrect scaffold connections.
- **Overwriting Output Without Warning**: Running with the `-o` flag pointing to an existing directory overwrites previous assemblies without prompting. Always verify the output directory is empty or specify a new path to prevent data loss.

## Examples

### Assemble a bacterial genome from single-end Illumina reads
**Args:** `-i reads.fq -k 31 -c 5 -t 8 -o assembly_output`
**Explanation:** Uses k-mer length 31 with minimum coverage 5 and 8 threads to assemble single-end reads from a bacterial dataset.

### Assemble paired-end reads with custom library orientation
**Args:** `-i left.fq -i right.fq --library-type fr -k 45 -c 10 -o paired_assembly`
**Explanation:** Specifies forward-reverse orientation for paired-end libraries, requiring a higher coverage threshold of 10 for better accuracy.

### Increase memory allocation for a large eukaryotic genome
**Args:** `-i large_reads.fq -k 61 -c 3 -m 32 -t 16 -o large_genome_out`
**Explanation:** Allocates 32 GB RAM and 16 threads to handle a large eukaryotic genome with deeper k-mer analysis, accepting lower coverage to preserve more regions.

### Resume a previous assembly from checkpoint
**Args:** --resume -o previous_output -o resumed_output`
**Explanation:** Uses the `--resume` flag to continue a previously interrupted assembly, using the checkpoint file in the output directory to avoid recomputing the graph.

### Specify a custom output filename for contigs
**Args:** `-i reads.fq -k 25 -c 4 --contigs-out custom_contigs.fasta`
**Explanation:** Changes the default contigs output filename from `aliceasm-contigs.fasta` to `custom_contigs.fasta` while using standard assembly parameters.

---