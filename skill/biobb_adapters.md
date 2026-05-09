I'll create a comprehensive skill file for biobb_adapters based on my knowledge of this bioinformatics tool collection.

---
name: biobb_adapters
category: bioinformatics_workflows
description: Python library providing CLI wrappers and adapters for popular bioinformatics tools including BLAST, Bowtie2, HMMER, ClustalO, MUSCLE, and others. Enables unified programmatic access to sequence alignment, homology search, and analysis tools.
tags: [bioinformatics, wrappers, sequence-analysis, alignment, ngs, homology]
author: AI-generated
source_url: https://github.com/bioexcel/biobb_adapters
---

## Concepts

- **Unified Tool Wrappers**: biobb_adapters provides Python-based wrappers for over 20 bioinformatics tools, each with consistent input/output handling and configuration management through YAML or dictionary-based parameters.
- **Tool-Specific Adapter Architecture**: Each tool (e.g., blastp, bowtie2_build, muscle, hmmer_hmmsearch) has its own adapter class requiring specific mandatory arguments like input_path, output_path, and tool-specific flags such as -query, -subject, -db, or -in.
- **Output Path Requirements**: Most adapters mandate an explicit output_path argument; omitting it causes the adapter to fail with a configuration error rather than generating a default output file.
- **I/O Format Flexibility**: Adapters accept multiple file formats (FASTA, GenBank, PDB, SDF) and automatically handle format-specific parsing, though some tools require exact format matches (e.g., bowtie2 requires FASTA/FASTQ for building indexes).

## Pitfalls

- **Confusing Python API with CLI Commands**: Users often attempt to call biobb_adapters functions directly from shell using the tool name (e.g., `biobb_adapters blastp`), but these are Python class imports requiring proper module syntax, leading to "module not found" errors.
- **Misunderstanding Wrapper vs. Companion Binaries**: Some tools in the collection (like bowtie2-build) are separate companion binaries that must be installed independently; the wrapper only provides the interface to call them, not the tool itself.
- **Parameter Naming Mismatch**: Passing flags using the underlying tool's CLI syntax (e.g., `-out`) instead of the wrapper's expected parameter name (e.g., `output_path`) causes configuration validation failures.
- **Missing Mandatory Arguments**: Many adapters have required arguments beyond input/output paths; omitting tool-specific required flags (such as -evalue for blastp or -genome for bowtie2_build) results in runtime errors from the underlying tool binary.

## Examples

### Run BLASTP protein sequence alignment
**Args:** --input_path protein.fasta --output_path blast_results.xml --db_path swissprot --evalue 0.001 --max_hits 10
**Explanation:** Executes BLASTP alignment against a protein database, outputting XML results with an e-value threshold of 0.001 and limiting to 10 alignments.

### Build Bowtie2 index from reference genome
**Args:** --input_path genome.fa --output_path genome_idx --threads 4
**Explanation:** Creates a Bowtie2 index from a FASTA reference genome using 4 threads for parallel processing, outputting index files with the base name genome_idx.

### Search sequence with HMMER profile
**Args:** --input_path sequences.fa --output_path hmmer_results.txt --hmm_profile profile.hmm --noalias --cpu 2
**Explanation:** Runs hmmsearch using a hidden Markov model profile against input sequences, outputting text results with 2 CPU threads.

### Multiple sequence alignment with MUSCLE
**Args:** --input_path sequences.fasta --output_path aligned.fasta --diagonalbreak 16
**Explanation:** Performs multiple sequence alignment using MUSCLE with a diagonal breaking parameter of 16 for improved speed on divergent sequences.

### ClustalO clustering and alignment
**Args:** --input_path unaligned.fa --output_path clustalo_out.fasta --tree --tree_output guide_tree.nwk
**Explanation:** Generates both a multiple sequence alignment and a guide tree in Newick format for phylogenetic analysis downstream.

### HMMER hmmbuild model creation
**Args:** --input_path aligned_seed.sto --output_path new_model.hmm --seed 42
**Explanation:** Builds a new HMM profile from a seed alignment in Stockholm format using a random seed of 42 for reproducibility.

---
name: biobb_adapters
category: bioinformatics_workflows
description: Python library providing CLI wrappers and adapters for popular bioinformatics tools including BLAST, Bowtie2, HMMER, ClustalO, MUSCLE, and others. Enables unified programmatic access to sequence alignment, homology search, and analysis tools.
tags: [bioinformatics, wrappers, sequence-analysis, alignment, ngs, homology]
author: AI-generated
source_url: https://github.com/bioexcel/biobb_adapters
---

## Concepts

- **Unified Tool Wrappers**: biobb_adapters provides Python-based wrappers for over 20 bioinformatics tools, each with consistent input/output handling and configuration management through YAML or dictionary-based parameters.
- **Tool-Specific Adapter Architecture**: Each tool (e.g., blastp, bowtie2_build, muscle, hmmer_hmmsearch) has its own adapter class requiring specific mandatory arguments like input_path, output_path, and tool-specific flags such as -query, -subject, -db, or -in.
- **Output Path Requirements**: Most adapters mandate an explicit output_path argument; omitting it causes the adapter to fail with a configuration error rather than generating a default output file.
- **I/O Format Flexibility**: Adapters accept multiple file formats (FASTA, GenBank, PDB, SDF) and automatically handle format-specific parsing, though some tools require exact format matches (e.g., bowtie2 requires FASTA/FASTQ for building indexes).

## Pitfalls

- **Confusing Python API with CLI Commands**: Users often attempt to call biobb_adapters functions directly from shell using the tool name (e.g., `biobb_adapters blastp`), but these are Python class imports requiring proper module syntax, leading to "module not found" errors.
- **Misunderstanding Wrapper vs. Companion Binaries**: Some tools in the collection (like bowtie2-build) are separate companion binaries that must be installed independently; the wrapper only provides the interface to call them, not the tool itself.
- **Parameter Naming Mismatch**: Passing flags using the underlying tool's CLI syntax (e.g., `-out`) instead of the wrapper's expected parameter name (e.g., `output_path`) causes configuration validation failures.
- **Missing Mandatory Arguments**: Many adapters have required arguments beyond input/output paths; omitting tool-specific required flags (such as -evalue for blastp or -genome for bowtie2_build) results in runtime errors from the underlying tool binary.

## Examples

### Run BLASTP protein sequence alignment
**Args:** --input_path protein.fasta --output_path blast_results.xml --db_path swissprot --evalue 0.001 --max_hits 10
**Explanation:** Executes BLASTP alignment against a protein database, outputting XML results with an e-value threshold of 0.001 and limiting to 10 alignments.

### Build Bowtie2 index from reference genome
**Args:** --input_path genome.fa --output_path genome_idx --threads 4
**Explanation:** Creates a Bowtie2 index from a FASTA reference genome using 4 threads for parallel processing, outputting index files with the base name genome_idx.

### Search sequence with HMMER profile
**Args:** --input_path sequences.fa --output_path hmmer_results.txt --hmm_profile profile.hmm --noalias --cpu 2
**Explanation:** Runs hmmsearch using a hidden Markov model profile against input sequences, outputting text results with 2 CPU threads.

### Multiple sequence alignment with MUSCLE
**Args:** --input_path sequences.fasta --output_path aligned.fasta --diagonalbreak 16
**Explanation:** Performs multiple sequence alignment using MUSCLE with a diagonal breaking parameter of 16 for improved speed on divergent sequences.

### ClustalO clustering and alignment
**Args:** --input_path unaligned.fa --output_path clustalo_out.fasta --tree --tree_output guide_tree.nwk
**Explanation:** Generates both a multiple sequence alignment and a guide tree in Newick format for phylogenetic analysis downstream.

### HMMER hmmbuild model creation
**Args:** --input_path aligned_seed.sto --output_path new_model.hmm --seed 42
**Explanation:** Builds a new HMM profile from a seed alignment in Stockholm format using a random seed of 42 for reproducibility.