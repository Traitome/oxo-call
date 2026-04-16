---
name: eggnog-mapper
category: annotation
description: Fast functional annotation of proteins or genes via eggNOG ortholog database
tags: [annotation, ortholog, go-terms, kegg, cog, proteins, metagenomics, pfam, diamond, mmseqs2, hmmer]
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
- --target_orthologs controls ortholog type for annotation: one2one, many2one, one2many, many2many, or all (default).
- --go_evidence filters GO terms: experimental (experimental only), non-electronic (curated only), or all (default).
- --pfam_realign enables PFAM domain prediction: none (transfer from ortholog), realign (realign to query), or denovo (search against PFAM).
- --decorate_gff creates/decorates GFF files with emapper annotations for genome browsers.
- EGGNOG_DATA_DIR environment variable can replace --data_dir for specifying database location.

## Pitfalls

- Forgetting --data_dir when the eggNOG database is not in the default location causes a silent failure with no annotations.
- Using nucleotide input without --itype CDS or --itype genome will produce incorrect results — always specify input type explicitly.
- Very short proteins (<50 aa) are often not annotated due to DIAMOND sensitivity limits; consider --dmnd_ignore_warnings.
- --cpu 0 uses all available cores which can cause memory contention on shared nodes; set an explicit --cpu value.
- The output --output prefix must not already exist; existing files cause an error without --override.
- KEGG pathway annotations require --target_orthologs all (default) — restricting to one-to-one orthologs misses many pathway assignments.
- --pfam_realign realign or denovo requires PFAM database downloaded with download_eggnog_data.py -P flag.
- --decorate_gff yes with --resume can cause issues; GFF decoration is regenerated from scratch on resume.
- HMMER search mode (--usemem) requires hmmpgmd server; use --timeout_load_server to control startup attempts.
- MMseqs2 mode requires database index creation with mmseqs createindex before first use.

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

### annotate with PFAM domain realignment
**Args:** `-m diamond -i proteins.fasta --itype proteins --data_dir /data/eggnog_db -o results --cpu 16 --pfam_realign realign`
**Explanation:** --pfam_realign realign realigns PFAM domains to query sequences; requires PFAM database

### annotate with experimental GO terms only
**Args:** `-m diamond -i proteins.fasta --itype proteins --data_dir /data/eggnog_db -o results --cpu 16 --go_evidence experimental`
**Explanation:** --go_evidence experimental uses only experimentally validated GO terms for annotation

### annotate with one-to-one orthologs only
**Args:** `-m diamond -i proteins.fasta --itype proteins --data_dir /data/eggnog_db -o results --cpu 16 --target_orthologs one2one`
**Explanation:** --target_orthologs one2one uses only strict one-to-one orthologs for annotation (more conservative)

### create decorated GFF file with annotations
**Args:** `-m diamond -i proteins.fasta --itype proteins --data_dir /data/eggnog_db -o results --cpu 16 --decorate_gff yes`
**Explanation:** --decorate_gff yes creates a GFF file with emapper annotations for genome browser visualization

### download eggNOG database with PFAM
**Args:** `download_eggnog_data.py -P --data_dir /data/eggnog_db`
**Explanation:** -P flag downloads PFAM database; required for --pfam_realign realign or denovo options

### create custom taxonomic database
**Args:** `create_dbs.py -m diamond --dbname bacteria --taxa Bacteria --data_dir /data/eggnog_db`
**Explanation:** create_dbs.py creates taxon-specific Diamond database; faster searches for specific taxonomic groups

### annotate using custom Diamond database
**Args:** `-m diamond -i proteins.fasta --itype proteins --dmnd_db /data/eggnog_db/bacteria.dmnd -o results --cpu 16`
**Explanation:** --dmnd_db specifies custom Diamond database created with create_dbs.py for taxon-specific annotation
