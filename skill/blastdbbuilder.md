---
name: blastdbbuilder
category: Bioinformatics
description: A tool for creating BLAST (Basic Local Alignment Search Tool) databases from sequence data. Converts FASTA or GenBank format sequences into binary database files optimized for fast similarity searches.
tags:
  - blast
  - ncbi
  - database
  - sequence-alignment
  - genomics
  - database-creation
author: AI-generated
source_url: https://blast.ncbi.nlm.nih.gov/Blast.cgi?CMD=Web&PAGE_TYPE=BlastDocs&DOC_TYPE=Download
---

## Concepts

- **Input Formats:** Accepts FASTA (`.fna`, `.fa`, `.fasta`), GenBank (`.gbk`, `.gb`), and ASN.1 binary formats. Each record must have a unique identifier in the definition line (header after the `>` symbol) for proper database indexing.
- **Database Structure:** Creates multiple binary files (`.ndb`, `.nhr`, `.nin`, `.nos`, `.nsq`, `.ntx`, `.ntt`) that together form a searchable BLAST database. The metadata file (`.db`) stores gi numbers, titles, and taxonomy information.
- **Taxonomy Integration:** When sequence headers contain taxonomy IDs (e.g., `[taxid:9606]`), the tool automatically builds a taxonomy-aware database enabling taxonomic filtering during BLAST searches.
- **Masking Options:** Supports database-side masking using dustmasker or segmasker to ignore low-complexity regions during searches, improving the biological relevance of alignment results.

## Pitfalls

- **Duplicate Sequence Identifiers:** Providing FASTA files with duplicate sequence IDs causes database creation to fail or produce corrupted databases, as the index cannot uniquely map multiple sequences.
- **Missing Read Permissions:** Attempting to create a database in a directory without write permissions silently fails to produce output files, leading to "no database found" errors during subsequent BLAST searches.
- **Incorrect Input File Path:** Specifying a non-existent or misspelled input FASTA file path produces an obscure error message without clearly indicating the file was not found.
- **Insufficient Disk Space:** Building large BLAST databases (multi-gigabyte genomes) without adequate free disk space results in partial database files that are unusable by BLAST search programs.

## Examples

### Create a basic protein BLAST database from a FASTA file

**Args:** `-in proteins.fasta -dbtype prot -title "MyProteinDB" -out my_protein_db`

**Explanation:** Converts a FASTA file containing protein sequences into a searchable protein BLAST database with the specified title for identification.

### Create a nucleotide BLAST database with taxonomic identifiers

**Args:** `-in sequences.fna -dbtype nucl -title "NucleotideDB" -taxid_map taxonomy_ids.txt -out nucl_db`

**Explanation:** Builds a nucleotide database and integrates taxonomy information from a mapping file, enabling taxonomic filtering in BLAST queries.

### Build a protein database with dust masking enabled

**Args:** `-in protein_seqs.faa -dbtype prot -mask_id yes -out masked_protein_db`

**Explanation:** Applies low-complexity region masking to the database so BLAST searches automatically filter out low-quality alignments in repetitive regions.

### Create a BLAST database in a custom output directory

**Args:** `-in input.fasta -dbtype prot -out /path/to/output/db_name -title "CustomDB"`

**Explanation:** Specifies an absolute path for the output database files, useful for organizing multiple databases in dedicated directories.

### Build a database with sequence length limits for parsing

**Args:** `-in large_genome.fasta -dbtype nucl -parse_seqids -max_file_size 4000000000 -out large_db`

**Explanation:** Creates a database with explicit sequence ID parsing and splits output into multiple files limited to 4GB each, suitable for filesystem constraints.