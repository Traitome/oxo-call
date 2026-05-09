---
name: cellxgene
category: Single-Cell Genomics Visualization
description: Interactive visualization and exploration tool for single-cell gene expression data. Supports AnnData format, enables dynamic filtering, gene expression heatmaps, cell metadata browsing, and collaborative data sharing via local web server or packaged archives.
tags:
  - single-cell
  - visualization
  - gene-expression
  - anndata
  - interactive
  - exploration
  - heatmap
  - scatter-plot
author: AI-Generated
source_url: https://cellxgene.readthedocs.io/
---

## Concepts

- **AnnData Data Model**: cellxgene operates on AnnData objects with three core components: `X` (expression matrix, shape cells × genes), `obs` (cell metadata DataFrame), and `var` (gene metadata DataFrame). You must ensure `obs` contains cell-level annotations (cluster labels, sample ID, tissue origin) before launching visualization, as these become filterable axes in interactive plots.

- **Multiple Expression Layers**: AnnData supports multiple expression matrices stored in `layers` dict (e.g., raw counts, normalized CPM, log-transformed). cellxgene defaults to displaying the last layer or the default `.X`, so explicitly specify which layer to visualize using the `--backed` mode or layer selection in the UI to avoid showing unnormalized counts as if they were interpretable expression values.

- **Interactive Filtering and Gene Sets**: cellxgene enables dynamic filtering on any `obs` column (e.g., cell type, batch, QC metrics) and supports gene set enrichment visualization. The `--annotations` flag loads a TSV/CSV gene set file that highlights specified genes in the UI, allowing you to explore expression of curated pathways across cell populations without manual gene list entry.

- **Launch Modes**: You can serve cellxgene in local server mode (`cellxgene launch`) for interactive exploration on a remote machine accessed via browser, or export a self-contained HTML archive (`cellxgene export`) that embeds the data for sharing without a running server. The `--port` and `--address` flags control network binding for multi-user access scenarios.

- **Data Indexing and Performance**: Large AnnData files (>1M cells) benefit from preprocessing with `cellxgene prepare` which reorders the backing file for faster chunked access. The prepare command also validates schema conformance, checks for required metadata columns, and compresses the archive into `.cxg` format for improved load times in multi-user deployments.

---

## Pitfalls

- **Missing Cell Metadata (obs)**: Failing to include informative `obs` columns (cell type labels, cluster IDs, donor metadata) before launching results in a visualization where only expression values are visible with no filtering axes. The UI becomes a static heatmap rather than an explorable dataset, severely limiting biological insight.

- **Using Unnormalized Expression Values**: Displaying raw count data in `.X` without normalization produces misleading visualizations where highly abundant genes (e.g., mitochondrial, housekeeping) dominate the color scale. Cells appear stratified by sequencing depth rather than biological variation, leading to incorrect conclusions about cell populations.

- **Large File Not Preprocessed**: Launching cellxgene on unprepared largeAnnData files (>500K cells) causes slow initial load times and sluggish interactive performance. The server may timeout or appear unresponsive during the first minutes, and panning/zooming in scatter plots becomes laggy, degrading the exploratory experience for collaborators.

- **Incompatible H5AD Schema**: cellxgene expects specific AnnData schema conventions including string dtype for categorical obs columns and proper index types. H5AD files created with older AnnData versions may have incompatible encoding, causing silent failures where some obs columns appear missing or filterable categories are empty despite data existing.

- **Export Without Obs Columns**: Exporting a packaged archive (`cellxgene export`) from a dataset with no `obs` metadata produces a static visualization that cannot be filtered post-export. The resulting HTML file shows only the default view without interactive annotation or cell subsetting capabilities, reducing the archive to a static image rather than an exploratory tool.

---

## Examples

### Visualize a prepared single-cell dataset with interactive filtering
**Args:** `launch data/processed.h5ad --open --port 5005`
**Explanation:** Launches cellxgene on the specified AnnData file, auto-opens the browser, and serves on port 5005, enabling immediate interactive exploration of cell populations with whatever obs metadata exists in the file.

### Prepare a large dataset and create an optimized archive
**Args:** `prepare data/raw_1million_cells.h5ad --output data/optimized.cxg --skip-missing`
**Explanation:** Converts the input AnnData to cellxgene archive format with chunked indexing for faster load times, skipping missing obs columns to avoid validation errors, suitable for datasets with >500K cells where interactivity matters.

### Export a self-contained HTML visualization for sharing
**Args:** `export data/processed.h5ad --embeddings --output cellxgene_archive.html`
**Explanation:** Exports a static HTML file containing embedded expression data and UMAP/PCA coordinates, allowing collaborators to view the visualization without installing cellxgene or running a server, though filtering capabilities depend on obs metadata presence.

### Load a dataset with a custom gene set annotation file
**Args:** `launch data/processed.h5ad --annotations gene_sets/pathway_genes.tsv --annotations-output gene_sets/annotated.h5ad`
**Explanation:** Loads predefined gene lists from a TSV file and highlights those genes in the UI, then saves the enriched dataset with gene set annotations embedded in the H5AD for future sessions, enabling pathway-focused exploration.

### Serve cellxgene on a specific host for remote collaborators
**Args:** `launch data/processed.h5ad --host 0.0.0.0 --port 8080 --max-census-size 20GB`
**Explanation:** Binds the cellxgene web server to all network interfaces and specifies memory allocation, allowing collaborators on other machines to access the visualization via browser at the remote host IP address with controlled resource usage.

### Prepare and relaunch after fixing metadata issues
**Args:** `prepare data/fix_metadata.h5ad --obs-columns cell_type,batch,tissue --skip-validation --output data/prepared.h5ad`
**Explanation:** Validates and prepares an AnnData file with explicit column declarations, skipping problematic validation checks for known non-critical schema deviations, producing a cellxgene-ready file that launches without metadata warnings.

### Index a dataset for fast gene expression queries
**Args:** `index data/processed.h5ad --output data/indexed.h5ad --overwrite`
**Explanation:** Creates a backing index optimized for cellxgene's query patterns, overwriting the original file to enable faster filter operations and scatter plot rendering when exploring large expression datasets interactively.