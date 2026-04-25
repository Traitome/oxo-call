---
name: bakta
category: annotation
description: Fast and standardized annotation of bacterial genomes and plasmids in GFF3 and GenBank format
tags: [annotation, bacteria, genome, gff, genbank, plasmid, ncbi, insdc, prokaryote, mag]
author: oxo-call built-in
source_url: "https://github.com/oschwengers/bakta"
---

## Concepts

- Bakta is a modern prokaryotic annotation tool that produces INSDC/NCBI-compliant annotation files. Recommended over Prokka for NCBI submissions.
- Use `--db` to specify the Bakta database path. Database can also be set via `BAKTA_DB` environment variable. Download with `bakta_db download --output /path/to/db`.
- Input: genome FASTA file as positional argument (supports gzipped .fasta.gz). Use `--min-contig-length` to filter short contigs (default: 1; 200 in compliant mode).
- Output files: GFF3, GenBank/EMBL, TSV table, FAA (protein FASTA), FNA (nucleotide FASTA), log files, and circular genome plots.
- Organism metadata: `--genus`, `--species`, `--strain` for taxonomy-aware annotations; `--plasmid` for plasmid name; `--gram` for Gram stain (+/-/?) for signal peptide prediction.
- Use `--compliant` for INSDC-compliant sequence IDs and `--locus-tag` for gene name prefixes — both required for GenBank/EMBL submission.
- Use `--complete` to indicate all sequences are complete replicons (chromosome/plasmids). This enables circular topology handling.
- Provide trusted proteins with `--proteins` (FASTA) or custom HMMs with `--hmms` (HMMER format) to improve CDS annotation accuracy.
- Pre-annotated regions: `--regions` accepts GFF3 or GenBank files with structural annotations (no functional annotations).
- Metagenome mode: `--meta` adjusts CDS prediction for metagenome-assembled genomes (MAGs).
- Translation table: `--translation-table` selects 11 (default, Bacteria/Archaea), 4 (Mycoplasma), or 25 (Candidate Division SR1/Gracilibacteria).
- Skip specific feature types: `--skip-trna`, `--skip-rrna`, `--skip-crispr`, `--skip-cds`, etc. to speed up re-runs or skip unwanted features.
- Companion tools: `bakta_db` (database management), `bakta_proteins` (annotate protein FASTA directly), `bakta_plot` (visualization), `bakta_io` (format conversion).

## Pitfalls

- Bakta's main command takes the genome FASTA as a positional argument (not a flag). ARGS starts with flags like `--db`, `--threads`, and ends with the FASTA path.
- Bakta database must be downloaded before running — use `bakta_db download` first. Without it, Bakta fails immediately.
- `--db` must point to the full database directory, not just a file within it. The directory must contain the complete db structure.
- The output directory must not already exist unless `--force` is used. Bakta creates it fresh by default.
- For NCBI submission, always use `--compliant` and set `--locus-tag`. Without these, the output will not pass NCBI validation.
- Bakta requires sequences ≥200 bp in compliant mode — very short contigs are automatically excluded.
- Without `--gram`, signal peptide prediction defaults to `?` (both orientations), which is less specific. Use `--gram +` or `--gram -` when known.
- `--prodigal-tf` allows using a pre-trained Prodigal training file for better CDS prediction on closely related species.
- `--replicons` accepts a TSV/CSV table mapping sequence IDs to replicon types (chromosome/plasmid) and names — required for multi-replicon submissions.

## Examples

### annotate a bacterial genome with Bakta
**Args:** `--db /path/to/bakta_db/ --threads 8 --output annotation/ --prefix genome_annotation genome.fasta`
**Explanation:** --db database path; --threads 8 parallel processing; --output directory; --prefix file naming prefix; genome.fasta is the input

### annotate genome for NCBI submission
**Args:** `--db /path/to/bakta_db/ --compliant --locus-tag MYORG --genus Escherichia --species coli --strain K12 --threads 8 --output ncbi_annotation/ --prefix ecoli_K12 genome.fasta`
**Explanation:** --db database path; --compliant INSDC-compliant format; --locus-tag MYORG gene name prefix; --genus Escherichia/--species coli/--strain K12 for taxonomy-aware annotation; --threads 8 parallel processing; --output ncbi_annotation/ output directory; --prefix ecoli_K12 file naming prefix; genome.fasta input; required for NCBI submission

### annotate plasmid sequence
**Args:** `--db /path/to/bakta_db/ --plasmid pMYPLASMID --complete --threads 4 --output plasmid_annotation/ --prefix plasmid plasmid.fasta`
**Explanation:** --db database path; --plasmid pMYPLASMID names the plasmid; --complete indicates circular topology; --threads 4 parallel processing; --output plasmid_annotation/ output directory; --prefix plasmid file naming prefix; plasmid.fasta input

### download the Bakta database
**Args:** `bakta_db download --output /path/to/bakta_db/`
**Explanation:** downloads the latest Bakta database to the specified directory; required before first annotation run

### annotate with trusted proteins and custom HMMs
**Args:** `--db /path/to/bakta_db/ --proteins trusted_proteins.faa --hmms custom_models.hmm --threads 8 --output annotation/ --prefix custom genome.fasta`
**Explanation:** --db database path; --proteins trusted_proteins.faa provides known protein sequences for improved CDS annotation; --hmms custom_models.hmm provides custom HMM profiles for specific gene families; --threads 8 parallel processing; --output annotation/ output directory; --prefix custom file naming prefix; genome.fasta input

### annotate a metagenome-assembled genome (MAG)
**Args:** `--db /path/to/bakta_db/ --meta --translation-table 11 --threads 8 --output mag_annotation/ --prefix mag mag_contigs.fasta`
**Explanation:** --db database path; --meta enables metagenome mode for CDS prediction; --translation-table 11 bacterial genetic code; --threads 8 parallel processing; --output mag_annotation/ output directory; --prefix mag file naming prefix; mag_contigs.fasta input; better suited for fragmented/lower-quality assemblies

### annotate with pre-annotated regions
**Args:** `--db /path/to/bakta_db/ --regions existing_regions.gff3 --threads 8 --output annotation/ --prefix with_regions genome.fasta`
**Explanation:** --db database path; --regions existing_regions.gff3 imports structural annotations from a GFF3/GenBank file; --threads 8 parallel processing; --output annotation/ output directory; --prefix with_regions file naming prefix; genome.fasta input; functional annotations are added by Bakta

### annotate with Gram-positive signal peptide prediction
**Args:** `--db /path/to/bakta_db/ --gram + --genus Bacillus --species subtilis --threads 8 --output annotation/ --prefix bsub genome.fasta`
**Explanation:** --db database path; --gram + optimizes signal peptide prediction for Gram-positive bacteria; --genus Bacillus/--species subtilis for taxonomy-aware annotation; --threads 8 parallel processing; --output annotation/ output directory; --prefix bsub file naming prefix; genome.fasta input

### annotate only specific feature types (skip CRISPR and sORF)
**Args:** `--db /path/to/bakta_db/ --skip-crispr --skip-sorf --threads 8 --output annotation/ --prefix minimal genome.fasta`
**Explanation:** --db database path; --skip-crispr and --skip-sorf speed up annotation by skipping CRISPR array and small ORF detection; --threads 8 parallel processing; --output annotation/ output directory; --prefix minimal file naming prefix; genome.fasta input; useful for re-runs

### annotate proteins directly from a FASTA file
**Args:** `bakta_proteins --db /path/to/bakta_db/ --threads 8 --output protein_annotation/ proteins.faa`
**Explanation:** bakta_proteins companion tool annotates protein sequences directly without genome context; --db database path; --threads 8 parallel processing; --output protein_annotation/ output directory; proteins.faa input protein FASTA; useful for annotating translated CDS

### force overwrite of existing output directory
**Args:** `--db /path/to/bakta_db/ --force --threads 8 --output existing_dir/ --prefix rerun genome.fasta`
**Explanation:** --db database path; --force allows overwriting an existing output directory; --threads 8 parallel processing; --output existing_dir/ output directory; --prefix rerun file naming prefix; genome.fasta input; useful for re-running with different parameters
