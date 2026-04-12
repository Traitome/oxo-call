---
name: eggnog-mapper
category: annotation
description: Fast functional annotation of proteins or genes via eggNOG ortholog database
tags: [annotation, ortholog, go-terms, kegg, cog, proteins, metagenomics]
author: oxo-call built-in
source_url: "https://github.com/eggnogdb/eggnog-mapper/wiki"
---

## Concepts

- emapper.py assigns functional annotations (GO terms, KEGG pathways, COG categories, EC numbers) by mapping sequences to eggNOG orthologous groups.
- The default search mode uses diamond for protein queries (-m diamond); mmseqs2 (-m mmseqs) is faster for very large datasets; hmmer (-m hmmer) is most sensitive.
- Input can be protein FASTA (-i with default --itype proteins) or nucleotide FASTA (--itype CDS or --itype genome for gene prediction first).
- eggNOG databases must be pre-downloaded to a data directory; use download_eggnog_data.py to install them.
- The --tax_scope flag restricts annotation to a taxonomic level (e.g., 2=Bacteria, 33208=Metazoa); leaving it at auto uses the best OG at any level.
- Output files include a .annotations (main TSV), .hits (raw search hits), and .seed_orthologs; the .annotations file has all functional terms.

## Pitfalls

- Forgetting --data_dir when the eggNOG database is not in the default location causes a silent failure with no annotations.
- Using nucleotide input without --itype CDS or --itype genome will produce incorrect results — always specify input type explicitly.
- Very short proteins (<50 aa) are often not annotated due to DIAMOND sensitivity limits; consider --dmnd_ignore_warnings.
- --cpu 0 uses all available cores which can cause memory contention on shared nodes; set an explicit --cpu value.
- The output --output prefix must not already exist; existing files cause an error without --override.
- KEGG pathway annotations require --target_orthologs all (default) — restricting to one-to-one orthologs misses many pathway assignments.

## Examples

### annotate a protein FASTA file with eggNOG using diamond
**Args:** `-m diamond -i proteins.fasta --itype proteins --data_dir /data/eggnog_db -o results --cpu 16 --override`
**Explanation:** -m diamond is fast; --itype proteins specifies protein input; -o sets output prefix; --override replaces existing output

### annotate proteins and restrict to bacterial orthologs
**Args:** `-m diamond -i proteins.fasta --itype proteins --tax_scope 2 --data_dir /data/eggnog_db -o bact_results --cpu 16`
**Explanation:** --tax_scope 2 restricts annotation to Bacteria (NCBI taxid 2) for more specific functional assignments

### annotate a nucleotide CDS file
**Args:** `-m diamond -i genes.fna --itype CDS --translate --data_dir /data/eggnog_db -o cds_results --cpu 16`
**Explanation:** --itype CDS tells emapper the input is coding sequences; --translate converts to protein internally for diamond

### use mmseqs2 mode for fast annotation of large metagenomic protein set
**Args:** `-m mmseqs -i meta_proteins.fasta --itype proteins --data_dir /data/eggnog_db -o meta_results --cpu 32`
**Explanation:** -m mmseqs is significantly faster than diamond for millions of sequences, with slight sensitivity trade-off

### resume an interrupted annotation run
**Args:** `-m diamond -i proteins.fasta --itype proteins --data_dir /data/eggnog_db -o results --cpu 16 --resume`
**Explanation:** --resume continues from existing partial results instead of restarting, saving time on large jobs

### annotate a genome with gene prediction first
**Args:** `-m diamond -i genome.fna --itype genome --data_dir /data/eggnog_db -o genome_results --cpu 16 --genepred prodigal`
**Explanation:** --itype genome triggers internal gene prediction with prodigal before annotation
