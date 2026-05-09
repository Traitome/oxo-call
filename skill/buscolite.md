---
name: buscolite
category: genomics/assembly_quality
description: Genome assembly quality assessment tool using single-copy ortholog benchmarking to evaluate completeness and contiguity.
tags:
- genomics
- assembly-quality
- busco
- annotation
- orthologs
- evaluation
- completeness
author: AI-generated
source_url: https://busco.ezlab.org
---

## Concepts

- **Input formats**: Accepts genomic sequences in FASTA format (genomes) or protein/CDS translations in FASTA format (annotations), plus a lineage-specific dataset (e.g., bacteria_odb10, eukaryota_odb10).
- **Output model**: Produces BUSCO summary files showing complete (C), fragmented (F), and missing (M) ortholog counts, plus a short summary table and optional detailed JSON/JSONL logs.
- **Run modes**: Supports genome mode (--mode genome), protein mode (--mode protein), and transcriptome mode (--mode transcriptome) to handle different input types.
- **Lineage datasets**: Uses pre-computed ortholog sets organized by taxonomic clade—running without specifying a valid lineage (--lineage) results in automated lineage detection or failure.
- **Parallelization**: Uses multiple CPU cores via -- Threads to speed up large genome assessments; omitting this defaults to single-threaded execution.

## Pitfalls

- **Wrong run mode**: Specifying --mode genome when input is a protein FASTA causes immediate failure; the mode must match the input type (genome/protein/transcriptome) to find orthologs correctly.
- **Missing lineage dataset**: Running without --lineage or with an incorrect/incomplete lineage path causes the tool to either auto-detect incorrectly or crash with a missing file error, leading to wasted computation.
- **Insufficient lineage coverage**: Using a too-broad lineage (e.g., eukaryota for a bacterial genome) produces misleadingly low completeness scores, giving a false assessment of assembly quality.
- **Out-of-memory on large genomes**: Processing chromosome-scale assemblies without specifying --limit or adequate RAM results in memory exhaustion; large eukaryotic genomes require explicit memory management.
- **Overwriting previous results**: Running buscolite in the same output directory without cleaning previous files silently appends or overwrites results, making traceability and comparison difficult.

## Examples

### Assess a bacterial genome assembly for completeness
**Args:** --in genome_assembly.fasta --out bacteria_busco --lineage bacteria_odb10 --mode genome --Threads 8
**Explanation:** Runs BUSCO in genome mode with the bacterial lineage dataset on 8 threads to evaluate how many universal bacterial single-copy orthologs are present in the assembly.

### Evaluate protein predictions from an annotated genome
**Args:** --in predicted_proteins.faa --out annotation_busco --lineage eukaryota_odb10 --mode protein
**Explanation:** Runs BUSCO in protein mode using the eukaryota lineage to determine the proportion of expected eukaryotic orthologs found in the predicted protein set.

### Analyze a transcriptome assembly
**Args:** --in transcriptome_assembly.fasta --out trans_busco --lineage metazoa_odb10 --mode transcriptome --Threads 4
**Explanation:** Uses transcriptome mode with the metazoan lineage on 4 threads to assess how many metazoan single-copy orthologs are recovered in the assembled transcripts.

### Generate machine-readable JSON output
**Args:** --in genome.fasta --out json_output --lineage fungi_odb10 --mode genome --format json
**Explanation:** Produces JSON-formatted results instead of the default text summary, making it easy to parse results in downstream automated pipelines.

### Clean and restart a run
**Args:** --in new_genome.fasta --out fresh_run --lineage protists_odb10 --mode genome --restart
**Explanation:** Clears any previous output in the specified directory before starting a fresh analysis, preventing mixed or corrupted results from previous partial runs.