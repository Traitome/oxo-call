---
name: chado-tools
category: Bioinformatics Database Management
description: Command-line utilities for managing Chado database schema and biological data. Provides tools for schema initialization, data loading, querying, migration, and export operations supporting the Generic Model Organism Database (GMOD) schema.
tags: [database, genomics, gmod, chado, sql, postgresql, biology, schema-management, data-loader]
author: AI-generated
source_url: https://gmod.org/wiki/CHADO
---

## Concepts

- **Chado Schema Structure**: Chado is a modular relational database schema implemented in PostgreSQL that organizes biological data across multiple modules (Sequence, Genetics, Phenotype, Publication, Organism, General). Tools interact with this normalized structure to store and retrieve genomic features, alleles, phenotypes, and related annotations.

- **Data Flow Model**: Inputs accept GFF3, FASTA, BED, and custom SQL dump formats for loading biological data; outputs generate query results as TSV, JSON, or SQL scripts. The tool maintains referential integrity by automatically managing foreign key relationships between features, organism, and analysis tables.

- **Key Modules**: Core modules include `seq` (sequences and features), `gen` (genetic data), `phenotype` (phenotypic annotations), `pub` (publications), `organism` (taxonomy), and `general` (controlled vocabularies). Each module has its own set of tables connected through foreign keys using Chado's feature relationship system.

- **Transaction and Rollback**: All data modification operations run within database transactions, enabling automatic rollback on failure. Use the `--dry-run` flag to preview changes without committing, which is essential for validating complex bulk loading operations before execution.

## Pitfalls

- **Missing Organism Records**: Loading sequence data without first inserting the corresponding organism record in the `organism` table causes foreign key constraint violations. Always ensure the organism exists (e.g., via `chado-tools organism add`) before loading features associated with that taxon.

- **Duplicate Feature Loading**: Attempting to load features that already exist in the database without using `--update` or `--replace` results in unique constraint errors. Use the `--update` flag to perform upsert operations, or pre-clean the database to avoid conflicts during bulk imports.

- **Incorrect Feature Type Mapping**: Specifying the wrong `type` parameter (e.g., using "gene" instead of the Chado-controlled vocabulary term like "gene") causes silent failures where features are inserted with incorrect types or rejected. Always verify CV terms match your Chado installation's controlled vocabulary.

- **Transaction Size Limits**: Loading extremely large GFF3 files in a single transaction can exhaust PostgreSQL's memory or hit `work_mem` limits. Chunk large files into smaller batches using the `--batch-size` parameter (recommended 1000-5000 features per batch) to prevent transaction failure.

- **Privilege Escalation Errors**: Running tools without sufficient database privileges (e.g., missing CREATE permission for staging tables) results in permission denied errors. Ensure the database user has appropriate roles: `chado_user` for data operations, `chado_admin` for schema modifications.

## Examples

### Initialize a new Chado database schema
**Args:** `init --host localhost --port 5432 --database chado_db --user postgres`
**Explanation:** Creates all required tables, indexes, and constraints for the Chado schema in an empty PostgreSQL database, establishing the foundation for subsequent data loading operations.

### Load GFF3 features into the database
**Args:** `load-gff3 features.gff3 --organism "Caenorhabditis elegans" --type gene --update`
**Explanation:** Parses a GFF3 file and inserts genetic features (genes, mRNAs, exons) into the Chado `feature` and `featureloc` tables, using the update flag to handle existing records.

### Add a new organism record
**Args:** `organism add --taxon "Homo sapiens" --common-name "human" --abbreviation "human"`
**Explanation:** Inserts a new organism row into the `organism` table with taxonomy information and abbreviations, which is a prerequisite for loading any species-specific genomic data.

### Export features to BED format
**Args:** `export --output features.bed --format bed --seqid "chr1" --type gene`
**Explanation:** Queries the Chado `feature` table for genes on chromosome 1 and exports them to BED format for use in genome browsers or downstream analysis tools like BEDTools.

### Run a custom SQL query against the database
**Args:** `query "SELECT f.uniquename, f.seqlen FROM feature f WHERE f.seqlen > 100000"`
**Explanation:** Executes a raw SQL statement against the Chado database to retrieve features exceeding a specific sequence length threshold, returning results in tabular format.

### Generate a feature summary report
**Args:** `summary --organism "Drosophila melanogaster" --module gen`
**Explanation:** Aggregates genetic module statistics for the specified organism, including feature counts by type, relationship tallies, and annotation coverage metrics.

### Migrate schema between databases
**Args:** `migrate --source-host db-prod --target-host db-staging --exclude phenote`
**Explanation:** Transfers the schema structure and optionally excludes specific modules (like phenotype data) when migrating between production and staging database environments.