---
name: cooler
category: genomics/hic-analysis
description: Tool for storing, querying, and manipulating Hi-C contact matrices using the HDF5-based Cooler format. Provides subcommands for creating coolers from raw contact pairs, merging files, extracting genomic regions, and applying balancing/normalization.
tags:
  - hi-c
  - chromatin-interaction
  - contact-matrix
  - hdf5
  - genomics
  - 3d-genome
  - chromatin-conformation
author: AI-generated
source_url: https://github.com/open2c/cooler
---

## Concepts

- **Cooler File Structure**: Cooler files are HDF5 archives organized into four core tables: `chroms` (chromosome names and lengths), `bins` (genomic bin boundaries), `pixels` (contact records with row/bin1_id, col/bin2_id, count), and `index` (offset pointers enabling fast region queries). Each bin has a unique integer ID encoding its genomic position.

- **Bin Size and Resolution**: The `resolution` of a cooler is determined by the fixed bin size used during creation (e.g., 10000 for 10kb resolution). All contact records reference bins from the same size-stratified bin table. Higher resolution means smaller bins and larger file sizes; cooler cannot mix bin sizes within a single file.

- **Cooler Creation Workflow**: Raw Hi-C contact pairs (from paired-end alignment) are first sorted by read ID, then pairs are assigned to genomic bins using their mapping positions, and finally a contacts file (or BEDPE) is passed to `cooler load` with `--assembly` and resolution parameters to populate the pixels table.

- **Balancing and Normalization**: Cooler files can store iterative correction weights in the `weight` column of the bins table. Balancing is performed by `cooler balance`, which computes symmetric correction factors. Balanced contact frequencies are calculated as `count / (weight_i * weight_j)`. Unbalanced coolers still function for most operations but will not produce normalized contact frequencies.

- **Zoomify for Multi-Resolution**: The `cooler zoomify` subcommand generates a multi-resolution cooler containing progressively merged versions at powers-of-two bin size multipliers (e.g., 10kb, 20kb, 40kb, 80kb). This enables efficient multi-scale queries without storing separate files for each resolution.

## Pitfalls

- **Mismatched Chromosome Naming**: If chromosome names in the input contacts file do not exactly match those in the `--assembly` genome metadata, `cooler load` silently skips or misassigns records. Always verify chromosome naming conventions (e.g., chr1 vs 1) between your alignment BEDPE and the genome annotation file.

- **Memory Exhaustion During Merging**: `cooler merge` loads all input coolers into memory when constructing the output. Merging many large high-resolution coolers can exceed available RAM. Use `--chunk-size` to limit per-chromosome batch processing and consider merging at lower resolution if memory constraints are severe.

- **Silent Data Loss from Pre-Filtering**: If you pre-filter contacts before loading (e.g., removing duplicates, filtering by MAPQ), ensure the resulting contact count is accurately documented. Balancing coefficients computed on incomplete data will be biased and unrepresentative of the true interaction landscape.

- **Incorrect Resolution in Queries**: Queries to a cooler with resolution X require specifying genomic coordinates in base pairs. Specifying coordinates in bin units or using an incorrect multiple will return empty results without warning. Always specify `--region` coordinates in base pairs matching the cooler's resolution (e.g., `chr1:0-1000000` for a 1Mb region in a 10kb-resolution cooler yields 100 bins).

- **Overwriting Balanced Weights Accidentally**: Re-loading contacts into an already-balanced cooler file will reset the bins table and erase previously computed `weight` values. Store balanced versions separately or use `--joined` mode to preserve corrections if re-loading is necessary.

## Examples

### Create a cooler file from a BEDPE contacts file
**Args:** `load --assembly hg19 --format bgzip --n-threads 4 hg19.chrom.sizes:10000 contacts.bedpe.gz experiment.cool`
**Explanation:** Loads a sorted BEDPE file of Hi-C contacts into a new 10kb-resolution cooler file using hg19 chromosome sizes, utilizing 4 threads for parallel compression.

### Query a genomic region and output as BEDPE
**Args:** `dump --region chr1:10000000-20000000 --balanced -o contacts.txt experiment.cool`
**Explanation:** Extracts all contacts within the specified 10Mb region of chr1 from a balanced cooler, outputting normalized contact frequencies to a text file.

### Merge multiple coolers into a single file
**Args:** `merge -o merged.cool sample1.cool sample2.cool sample3.cool`
**Explanation:** Combines contact records from three replicate coolers (must have identical bin tables/resolutions) into a single merged cooler file for consolidated analysis.

### Generate a multi-resolution zoomed cooler
**Args:** `zoomify -p 8 -o multires.cool single_res.cool`
**Explanation:** Creates a pyramid of progressively lower-resolution coolers inside a single HDF5 file using 8 threads, enabling efficient multi-scale visualization and analysis.

### Compute and store iterative correction weights (balancing)
**Args:** `balance --ignore-diags 2 --max-iter 200 single_res.cool`
**Explanation:** Computes iterative correction weights for matrix balancing, ignoring the first 2 diagonal bins and performing up to 200 iterations, storing the weights in the bins table for downstream normalization.

### Load contacts from a tabix-indexed contact list
**Args:** `cload tabix --assembly hg38 pairs.gz@chromsizes.tsv sample.cool`
**Explanation:** Uses the tabix-indexed contact list format (with genomic coordinates encoded in the paired records) to efficiently load contacts into a cooler, referencing chromosome sizes from the provided file.

### Validate the integrity of a cooler file
**Args:** `validate experiment.cool`
**Explanation:** Checks that all required HDF5 tables and datasets are present and correctly structured, verifying the cooler file is not corrupted and meets specification requirements.