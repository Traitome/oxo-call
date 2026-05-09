---
name: Auspice
category: Phylogenetic Visualization
description: A web-based tool for visualizing interactive phylogenetic trees, geographic spread maps, and genomic epidemiologystyle frequency dynamics from Auspice JSON datasets.
tags: [phylogenetics, visualization, nextstrain, genomic-epidemiology, COVID-19, pathogen-evolution, interactive-plot, web-server]
author: AI-Generated
source_url: https://github.com/nextstrain/auspice
---

## Concepts

- **Auspice JSON Schema**: Auspice reads datasets in its own JSON format (distinct from older Nextstrain formats), where `tree.json` or `rooted-tree.json` contains the phylogeny with branch lengths, mutations, and trait data, and `meta.json` holds dataset-level metadata like strain names, authors, and display preferences. Using mismatched or outdated JSON schemas causes silent failures where panels render empty.
- **Multi-panel Visualization**: Auspice automatically renders multiple panels—an interactive tree with tip coloring, a geographic map showing transmissions, entropy plots for mutation rates, and frequency panels for clade dynamics. Panel behavior and colorings are controlled by `auspice_config.json` entries like `colorings`, `geo_resolutions`, and `display_defaults`; omitting these results in uncolored or flattened trees.
- **Local Web Server**: The `auspice view` command launches a local HTTP server (default port 4000) that serves the visualization in a browser. The server watches the dataset directory for changes and hot-reloads visualizations, enabling iterative refinement of trees and configurations without restarting the process.
- **Narrative Mode**: Auspice supports `--narrative` for time-resolved storytelling, where a `narrative.md` file links sequential Auspice JSON snapshots to narrative paragraphs, allowing users to scrub through evolutionary time like a timeline slider. This requires properly formatted Markdown with embedded YAML front-matter and correctly numbered JSON files.
- **Dataset Directory Structure**: Auspice expects a specific directory hierarchy where `auspice_config.json`, `tree.json` (or `rooted-tree.json`), `sequences.fasta`, and optional `tip-frequencies.json` files coexist. Renaming or relocating these files without updating the config breaks dataset loading entirely.

## Pitfalls

- **Wrong JSON Format**: Feeding Auspice a Nextstrain v1/v2 JSON dataset (e.g., produced by older `augur export` without the `--auspice` flag) produces no error message but renders a blank page with a console complaint about missing `meta` or `tree` keys. Always verify JSON files match the current Auspice JSON schema by checking for top-level keys like `data` (nodes array) and `metadata`.
- **Missing Auspice Config**: Running `auspice view` without a properly formatted `auspice_config.json` results in trees displayed in a single default color (gray) with no trait-based coloring, geographic panels show no country/region assignments, and panel visibility defaults to a minimal subset. Users often mistake this for a data issue when it is purely a configuration omission.
- **Hot-Reload Conflicts**: When running `auspice view` with `--dataset-prefix` pointing to a directory that is simultaneously being written to by an upstream pipeline (e.g., `augur export` finishes mid-serve), Auspice may load partial or corrupted JSON leading to crashes or "NaN" branch lengths displayed in the tree. Always complete pipeline exports before launching the viewer.
- **Port Busy / Firewall**: If default port 4000 is already in use, `auspice view` exits with a cryptic EADDRINUSE error and offers no alternative. Users on shared HPC systems frequently encounter this and assume the tool is broken. Explicitly specifying `--port` with an available port (e.g., `auspice view --port 5000`) resolves it.
- **Narrative YAML Front-matter Errors**: Narrative mode fails silently when the `narrative.md` front-matter is malformed (e.g., missing dashes, wrong YAML keys, or unquoted narrative URLs). Auspice renders the first narrative frame but refuses to scrub to others, displaying no error in the web UI.

## Examples

### View a pre-built Auspice dataset in the local browser
**Args:** `view --dataset-dir ./covid-build`
**Explanation:** Launches a local HTTP server on port 4000 and opens an interactive phylogenetic visualization from the specified directory containing `tree.json`, `meta.json`, and `auspice_config.json`.

### Serve a dataset on a specific port to avoid conflicts
**Args:** `view --dataset-dir ./zika-results --port 8765`
**Explanation:** Starts the Auspice viewer on port 8765 instead of the default, useful when running on shared systems or alongside other web services.

### View a dataset with verbose debug output
**Args:** `view --dataset-dir ./ebola-build --verbose`
**Explanation:** Enables verbose console logging during dataset loading and rendering, helpful for diagnosing silent failures or JSON parsing issues.

### Generate an Auspice-compatible dataset from an augur pipeline
**Args:** `build --input ./auspice_input.json --output ./dist/auspice`
**Explanation:** Converts augur-processed phylogenetic data into Auspice JSON format in the output directory, ready for `auspice view`. Requires the input JSON to already be in the Auspice schema.

### Open a narrative-driven visualization for time-resolved storytelling
**Args:** `view --narrative ./covid-narrative/narrative.md`
**Explanation:** Loads a narrative Markdown file with embedded Auspice JSON snapshots, enabling time-scrubbing through epidemic stages with accompanying explanatory text.

### Reload datasets from a different directory without restarting the server
**Args:** `view --dataset-dir ./new-flutreet-build --build-dir ./dist`
**Explanation:** Switches the active dataset to a new directory post-launch while preserving the running server process, useful for comparing related builds side-by-side by changing the URL path.

### Serve dataset with authentication token for restricted access
**Args:** `view --dataset-dir ./secret-path --build-url https://example.com/auspice`
**Explanation:** Configures Auspice to serve data from a base URL for relative path resolution in the web interface, useful when deploying behind a reverse proxy with token-based access control.

### Export dataset with custom URL base path for deployment
**Args:** `build --input ./combined.json --output ./public-auspice --build-url /ncov/`
**Explanation:** Builds the Auspice dataset with a base path prefix so that deployed URLs (e.g., `/ncov/`) resolve correctly in the browser when served behind nginx or Apache.