---
name: annot8r
category: annotation
description: GO term and pathway annotation of gene sequences using BLAST against curated databases
tags: [annotation, go-terms, pathway, kegg, cog, blast, functional-annotation]
author: oxo-call built-in
source_url: "https://www.hudsonalpha.org/labs/myers-lab/tools/"
---

## Concepts

- Functional annotation workflows often chain BLAST → GO/KEGG annotation.
- eggNOG-mapper is a widely used alternative for COG/GO/KEGG/KEGG pathway annotation.
- Common tool for GO annotation: InterProScan, eggNOG-mapper, Trinotate.
- eggNOG-mapper: emapper.py -i proteins.faa --output annotation_prefix -m diamond --cpu 8
- InterProScan: interproscan.sh -i proteins.faa -f tsv -o interpro.tsv --cpu 8
- Use BLAST2GO for comprehensive functional annotation with visualization.
- COG functional categories: J (translation), K (transcription), L (DNA replication), etc.

## Pitfalls

- GO annotation quality depends heavily on the reference database and identity threshold.
- InterProScan requires Java and significant disk space for databases.
- eggNOG-mapper requires the eggNOG database download before use.

## Examples

### annotate proteins with eggNOG-mapper for GO, KEGG, and COG terms
**Args:** `emapper.py -i proteins.faa --output eggnog_annotation --output_dir eggnog_results/ -m diamond --cpu 16 --go_evidence non-electronic`
**Explanation:** emapper.py; -i protein FASTA; -m diamond for fast search; --go_evidence non-electronic for high-quality GO

### run InterProScan for protein domain and GO annotation
**Args:** `interproscan.sh -i proteins.faa -f tsv -o interpro_results.tsv --cpu 8 --goterms --pathways`
**Explanation:** -f tsv tabular output; --goterms adds GO term annotations; --pathways adds MetaCyc/Reactome

### annotate with eggNOG-mapper using pre-downloaded database
**Args:** `emapper.py -i proteins.faa --data_dir /path/to/eggnog_db/ --output annotation --cpu 16 --override`
**Explanation:** --data_dir points to downloaded eggNOG database; --override overwrites existing output
