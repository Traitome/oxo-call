---
name: bio_assembly_refinement
category: Genome Assembly
description: A tool for refining, polishing, and improving the quality of genome assemblies by correcting errors, resolving misjoins, and optimizing contig boundaries.
tags: [assembly, refinement, polishing, genome, contigs]
author: AI-generated
source_url: https://github.com/bio-assemblies/bio-assembly-refinement
---

## Concepts

- The tool operates on FASTA-formatted assembly files and produces a refined FASTA output, processing assemblies as linear sequences of nucleotides organized by sequence identifier (header lines starting with `>`).
- The primary data model treats each sequence record as an independent contig, enabling targeted refinement of individual contigs without requiring a full database reload for each operation.
- Key behaviors include consensus polishing (adjusting base calls based on read alignment quality scores), misjoin detection (identifying breakpoints where adjacent contigs may be misassembled), and boundary optimization (extending or trimming contig ends to maximize quality metrics).
- Output is written to stdout by default when a single input file is processed, allowing seamless piping to other tools in assembly pipelines; explicit output file specification via `--output` is required for file-based redirection.
- The companion binary `bio_assembly_refinement-build` constructs index files (`.bai` format) that accelerate repeated operations on large assemblies, which `bio_assembly_refinement` automatically detects and uses when present in the same directory as the input file.

## Pitfalls

- Specifying an output filename that matches the input filename causes silent data loss because the tool overwrites files without confirmation; always use distinct input and output paths.
- Running refinement without first building an index file on assemblies larger than 50 Mb results in dramatically slower processing (often 10-50x slower) because the tool must scan the entire assembly for each operation.
- Misinterpreting the `--min-contig-length` threshold: contigs shorter than the specified value are silently discarded from the output, not just flagged, which can lead to missing essential sequences in downstream analyses.
- Providing quality score files in the wrong encoding (e.g., Sanger vs. Illumina quality formats) produces incorrect consensus calls, leading to polished assemblies with artifactual homopolymer errors.
- Failing to specify `--num-threads` on multi-core systems leaves computational resources unused; the tool defaults to single-threaded execution, causing unnecessarily long runtimes for large genomes.

## Examples

### Polish a FASTA assembly using PacBio HiFi reads
**Args:** `input.fasta --reads hifi_reads.fastq --algorithm arrow --output polished.fasta`
**Explanation:** This aligns PacBio HiFi reads to the assembly using the Arrow algorithm and writes consensus-corrected sequences to the output file.

### Build an index file for accelerated processing
**Args:** `bio_assembly_refinement-build assembly.fasta --index assembly.bai`
**Explanation:** The companion binary constructs an index that enables fast random access during subsequent refinement operations on the assembly.

### Refine an assembly with misjoin correction enabled
**Args:** `--input draft_assembly.fasta --correct-misjoins --min-breakpoint-score 15 --output refined.fasta`
**Explanation:** This detects and resolves assembly misjoins where breakpoint alignment scores exceed the threshold, outputting corrected contigs.

### Filter out short contigs before polishing
**Args:** `--input raw_assembly.fasta --min-contig-length 500 --polish-only --output filtered_polished.fasta`
**Explanation:** Contigs shorter than 500 bp are removed before polishing begins, reducing processing time on low-complexity or spurious contigs.

### Process a large assembly using multiple threads
**Args:** `--input sorghum_assembly.fasta --num-threads 16 --output refined.fasta`
**Explanation:** This allocates 16 CPU threads to parallelize alignment and consensus computation, significantly accelerating processing of large genomes.