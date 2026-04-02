---
name: annot8r
category: annotation
description: Automated BLAST-based functional annotation of gene sequences with GO, EC, and KEGG terms
tags: [annotation, go-terms, pathway, kegg, ec-number, blast, functional-annotation]
author: oxo-call built-in
source_url: "https://sourceforge.net/projects/annot8r/"
---

## Concepts

- annot8r automates BLAST-based functional annotation by searching sequences against curated databases (UniProt entries with GO, EC, and KEGG annotations).
- The workflow is: (1) prepare databases with annot8r_blast2go.pl, (2) run BLAST searches, (3) map hits to GO/EC/KEGG terms.
- annot8r uses BLAST e-value and identity thresholds to filter annotation transfers.
- Output includes tab-delimited files with sequence IDs mapped to GO terms, EC numbers, and KEGG pathways.
- For modern functional annotation, consider eggNOG-mapper (emapper.py) or InterProScan as alternatives.

## Pitfalls

- annot8r requires pre-formatted BLAST databases built from UniProt with GO/EC/KEGG mappings.
- Annotation quality depends on the BLAST e-value threshold — too permissive values transfer incorrect annotations.
- annot8r is a legacy Perl tool; ensure BioPerl and BLAST+ are installed before running.
- The tool is no longer actively maintained — eggNOG-mapper or InterProScan are recommended for new projects.

## Examples

### run BLAST search for GO term annotation of protein sequences
**Args:** `-blast blastp -in proteins.faa -db annot8r_db/go_db -evalue 1e-10 -out go_annotations.txt`
**Explanation:** -blast blastp for protein search; -db points to pre-built GO annotation database; -evalue filters weak hits

### annotate nucleotide sequences with KEGG pathway terms
**Args:** `-blast blastx -in contigs.fasta -db annot8r_db/kegg_db -evalue 1e-5 -out kegg_annotations.txt`
**Explanation:** -blast blastx translates nucleotide queries; -db KEGG annotation database; maps to KEGG pathways

### run GO annotation with strict identity threshold
**Args:** `-blast blastp -in proteins.faa -db annot8r_db/go_db -evalue 1e-20 -identity 70 -out stringent_go.txt`
**Explanation:** -identity 70 requires at least 70% identity for annotation transfer; reduces false annotations

### annotate with EC number database for enzyme classification
**Args:** `-blast blastp -in proteins.faa -db annot8r_db/ec_db -evalue 1e-10 -out ec_annotations.txt`
**Explanation:** EC number annotation maps proteins to enzyme commission classes; useful for metabolic reconstruction

### batch annotate multiple FASTA files with GO terms
**Args:** `-blast blastp -in all_proteins.faa -db annot8r_db/go_db -evalue 1e-10 -cpu 8 -out batch_go_annotations.txt`
**Explanation:** -cpu 8 uses 8 threads for BLAST search; processes all sequences in the input FASTA file
