---
name: cenplot
category: Visualization
description: A bioinformatics tool for generating centroid-based plots from clustering analyses, supporting PCA, t-SNE, and other dimensional reduction visualizations with customizable appearance and export options.
tags: [visualization, clustering, plot, PCA, tsne, centroid, graphics]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/cenplot
---

## Concepts

- **Input Data Model**: Accepts tab-delimited or CSV files with numeric feature columns. Expects a header row and supports optional sample metadata columns for color coding. The tool automatically detects numeric columns for dimensionality reduction or direct centroid plotting.
- **Output Formats**: Generates publication-quality plots in PNG, PDF, SVG, or EPS formats. Supports both raster and vector outputs, with default DPI of 300 for PNG and vector formats (PDF/SVG/EPS) for scalable figures.
- **Centroid Computation**: Calculates cluster centroids as the mean of all points belonging to each cluster, optionally weighted by sample weights provided in the input. Centroids are marked with distinct symbols while individual points are plotted semi-transparently.
- **Interactive vs. Static**: By default generates static images, but can emit interactive HTML using Plotly.js for pan/zoom exploration. The HTML output includes hover labels showing sample identifiers and coordinates.

## Pitfalls

- **Forgetting to Specify Cluster Labels**: Omitting the `--cluster` flag when your input file contains cluster assignments will treat all samples as belonging to a single cluster, producing misleading centroid positions that represent the global mean rather than cluster-specific centroids.
- **Mismatched Column Indices**: Using `--x-col` or `--y-col` with indices that exceed the number of numeric columns causes the tool to crash with an unhelpful error. Always verify column counts using `head -1 yourfile.tsv | tr '\t' '\n' | cat -n` before plotting.
- **Non-Numeric Data in Feature Columns**: If any feature columns contain non-numeric values (NA, NaN, or text), the centroid computation produces NaN results and the plot renders incorrectly. Preprocess missing values with imputation or filter rows before running cenplot.

## Examples

### Generate a simple scatter plot from two numeric columns
**Args:** `--input data.tsv --x-col 1 --y-col 2 --output scatter.png`
**Explanation:** Plots column 1 on the X-axis and column 2 on the Y-axis directly without dimensionality reduction, suitable for viewing raw feature relationships.

### Add cluster centroids to a scatter plot
**Args:** `--input clustered_data.tsv --x-col 1 --y-col 2 --cluster 3 --output centroids.png`
**Explanation:** Groups points by the cluster ID in column 3 and overlays centroid markers at the mean position of each group, making cluster separation visually clear.

### Create a PDF with colored clusters and custom legend title
**Args:** `--inputExpr clustered_data.tsv --x-col 1 --y-col 2 --cluster 3 --color-by cluster --legend-title "Cell Type" --output clusters.pdf`
**Explanation:** Uses the cluster column values to assign colors and sets a custom legend title, producing vector output suitable for publication figures.

### Export an interactive HTML visualization
**Args:** `--input data.tsv --x-col 1 --y-col 2 --cluster 3 --format html --output interactive.html`
**Explanation:** Generates an HTML file with Plotly.js enabling mouse hover inspection of individual points and optional zoom, useful for exploratory data analysis.

### Compute centroids without plotting and save to file
**Args:** `--input clustered_data.tsv --x-col 1 --y-col 2 --cluster 3 --compute-centroids-only --output centroids.tsv`
**Explanation:** Outputs only the computed centroid coordinates for each cluster to a tab-delimited file, useful when you need numerical centroids for downstream analyses.

### Override default point transparency for dense clusters
**Args:** `--input dense_clusters.tsv --x-col 1 --y-col 2 --cluster 3 --alpha 0.1 --output dense.png`
**Explanation:** Lowers point transparency to 0.1 so overlapping points in dense regions remain visible, preventing the plot from appearing as a solid block of color.

### Add sample labels as hover text in interactive mode
**Args:** `--input data.tsv --x-col 1 --y-col 2 --labels --output hover_labels.html`
**Explanation:** Includes sample identifiers from the input in hover tooltips when generating HTML output, enabling rapid identification of outliers or specific points of interest.

### Scale point sizes by a numeric metadata column
**Args:** `--input data_with_size.tsv --x-col 1 --y-col 2 --size-col 4 --output sized.png`
**Explanation:** Varies point marker area proportionally to values in column 4, useful when you want to encode a quantitative attribute like expression level directly in the visualization.

### Generate plot with custom color palette
**Args:** `--input clustered.tsv --x-col 1 --y-col 2 --cluster 3 --palette "Set1,Set2" --output colored.png`
**Explanation:** Applies specific ColorBrewer palette strings to map clusters to distinct colors, ensuring accessibility compliance or matching figures to journal requirements.

### Set axis limits manually to exclude outlier samples
**Args:** `--input data.tsv --x-col 1 --y-col 2 --xlim 0,100 --ylim 0,100 --output cropped.png`
**Explanation:** Clips the plotting region to specified coordinate ranges, useful when a few extreme points compress the visible range of the majority of data.

### Combine multiple input files into one multi-panel figure
**Args:** `--input A.tsv --input B.tsv --x-col 1 --y-col 2 --output combined.png --layout 2x1`
**Explanation:** Stacks two input files vertically in a single image, enabling direct visual comparison of clustering results across experimental conditions.

### Export centroids with confidence interval ellipses
**Args:** `--input clustered_data.tsv --x-col 1 --y-col 2 --cluster 3 --ellipses --output with_ellipses.png`
**Explanation:** Overlays 95% confidence ellipses around each cluster centroid, visually representing the spread and orientation of points within each group.

### Generate a blank plot template for external annotations
**Args:** `--input empty_template.tsv --x-col 1 --y-col 2 --no-points --output template.png`
**Explanation:** Creates an axes and labels only without plotting actual points, useful when you need to add annotations or overlays in illustration software.

### Run in batch mode processing multiple CSV files
**Args:** `--input "*.csv" --x-col 1 --y-col 2 --cluster 3 --output "out_" --format png`
**Explanation:** Processes all CSV files matching the glob pattern, generating separate output files with the specified prefix for large-scale visualization workflows.

### Apply log transformation to feature columns before plotting
**Args:** `--input data.tsv --x-col 1 --y-col 2 --log-transform --output log_plot.png`
**Explanation:** Applies log10 transformation to both X and Y columns prior to computation and display, handling skewed distributions common in gene expression or count data.

### Save plot statistics to a companion TSV report
**Args:** `--input clustered.tsv --x-col 1 --y-col 2 --cluster 3 --stats-output report.tsv --output plot.png`
**Explanation:** Writes summary statistics (cluster sizes, centroid coordinates, within-cluster variance) alongside the visual output, supporting reproducible research workflows.

### Use custom point marker shapes for each cluster
**Args:** "--input clustered.tsv --x-col 1 --y-col 2 --cluster 3 --shapes circle,square,triangle --output shapes.png"
**Explanation:** Assigns distinct geometric shapes to different clusters rather than relying only on color, improving accessibility for grayscale or colorblind-friendly publications.

### Generate plot with axis labels extracted from header comments
**Args:** `--input data.tsv --x-col 1 --y-col 2 --use-headers --output labeled.png`
**Explanation:** Parses commented header lines in the input file to populate axis labels and titles automatically, ensuring metadata is preserved in the generated figure.

### Adjust point jitter to reduce overlap in categorical plotting
**Args:** `--input data.tsv --x-col 1 --y-col 2 --jitter 0.1,0.1 --output jittered.png`
**Explanation:** Applies small random displacement to point coordinates to reveal hidden overplotting, useful when plotting discrete or low-variance data.

### Rotate plot output to landscape orientation
**Args:** `--input data.tsv --x-col 1 --y-col 2 --orientation landscape --output landscape.png`
**Explanation:** Changes page orientation to landscape for wider plots, maximizing horizontal space when you have many samples or wide spread in X coordinates.

### Include a scale bar for distance reference
**Args:** "--input data.tsv --x-col 1 --y-col 2 --scalebar 10 --output with_scale.png"
**Explanation:** Adds a scale bar representing 10 units in data coordinates, essential for spatial datasets where true distances matter.

### Export both plot image and centroid coordinates for reproducibility
**Args:** `--input clustered.tsv --x-col 1 --y-col 2 --cluster 3 --save-state --output full_output`
**Explanation:** Generates both the visual PNG and a JSON state file capturing all parameters and computed centroids, enabling exact recreation of the plot later.

### Compute principal components then plot first two PCs
**Args:** `--input data.tsv --pca --pc-x 1 --pc-y 2 --output pca_plot.png`
**Explanation:** Performs PCA on all numeric columns internally and plots the resulting first and second principal components, reducing high-dimensional data to 2D for visualization.

### Plot first two t-SNE dimensions from pre-computed coordinates
**Args:** `--input tsne_results.tsv --x-col tsne1 --y-col tsne2 --cluster 3 --output tsne.png`
**Explanation:** Reads externally computed t-SNE coordinates from specific columns and visualizes them with cluster coloring, when t-SNE was run as a preprocessing step.

### Overlay trajectory arrows from pseudotime ordering
**Args:** `--input pseudotime.tsv --x-col 1 --y-col 2 --pseudotime 3 --trajectory --output trajectory.png`
**Explanation:** Draws arrows connecting points in pseudotime order to visualize developmental trajectories or differentiation paths within the data.

### Generate a plot with transparent background
**Args:** `--input data.tsv --x-col 1 --y-col 2 --transparent-bg --output transparent.png`
**Explanation:** Sets the plot background to fully transparent (alpha 0), useful when embedding figures in presentations or documents with varying background colors.

### Limit maximum points plotted to manage large datasets
**Args:** `--input huge_data.tsv --x-col 1 --y-col 2 --max-points 10000 --output downsampled.png`
**Explanation:** Randomly subsamples to at most 10000 points when input exceeds this threshold, preventing memory issues and slow rendering on very large datasets.

### Customize axis tick formatting for scientific notation
**Args:** "--input data.tsv --x-col 1 --y-col 2 --x-format sci --y-format sci --output scientific.png"
**Explanation:** Formats axis tick labels in scientific notation (e.g., 1e+3), helpful when coordinate values span multiple orders of magnitude.

### Add gridlines at major axis intervals
**Args:** `--input data.tsv --x-col 1 --y-col 2 --grid --output gridded.png`
**Explanation:** Overlays gridlines aligned with major tick marks, making it easier for readers to estimate point coordinates from the figure.

### Combine multiple datasets with different cluster naming
**Args:** `--input A.tsv --input B.tsv --x-col 1 --y-col 2 --cluster 3 --normalize-clusters --output normalized.png`
**Explanation:** Merges inputs and remaps cluster IDs to a unified numbering scheme when combining datasets that used different original cluster label conventions.

### Export plot to SVG for editing in vector graphics software
**Args:** `--input data.tsv --x-col 1 --y-col 2 --output editable.svg --format svg`
**Explanation:** Generates vector SVG output that can be opened in Inkscape or Illustrator for fine-tuning labels, arrows, or styling after generation.

### Set plot resolution explicitly for high-DPI printing
**Args:** `--input data.tsv --x-col 1 --y-col 2 --dpi 600 --output highres.png`
**Explanation:**Overrides the default 300 DPI to 600 for print publications requiring 600 DPI figures, at the cost of larger file sizes.

### Color points by continuous gradient rather than discrete clusters
**Args:** `--input data.tsv --x-col 1 --y-col 2 --color-col 4 --gradient viridis --output gradient.png`
**Explanation:** Applies a continuous color gradient from column 4 to map values visually, not cluster membership, for expression-level visualizations.

### Remove axis ticks and labels for clean embedding
**Args:** "--input data.tsv --x-col 1 --y-col 2 --hide-axis --output clean.png"
**Explanation:** Suppresses all axis markings for use when embedding plots as panels in larger composite figures, reducing redundant visual elements.

### Title the plot with custom text
**Args:** `--input data.tsv --x-col 1 --y-col 2 --title "Gene Expression by Cluster" --output titled.png`
**Explanation:** Adds a centered title above the plotting area with the specified string, helpful for identifying figure panels in manuscripts.

### Reverse the default color mapping order
**Args:** "--input data.tsv --x-col 1 --y-col 2 --cluster 3 --reverse-palette --output reversed.png"
**Explanation:** Flips the order of colors in the selected palette so the first cluster uses the last color rather than the first, useful for matching legends to other figures.

### Suppress legend entirely from output
**Args:** `--input data.tsv --x-col 1 --y-col 2 --cluster 3 --no-legend --output no_legend.png`
**Explanation:** Hides the legend box entirely when including it would obscure data points or clash with the figure layout, common in multi-panel composites.

### Use a subset of rows by filtering on a metadata column
**Args:** `--input data.tsv --x-col 1 --y-col 2 --filter "cell_type==T" --output filtered.png`
**Explanation:** Includes only rows where the specified metadata column matches the given value, allowing targeted visualization of specific sample subsets.

### Increase font size for all text elements
**Args:** `--input data.tsv --x-col 1 --y-col 2 --font-size 14 --output large_text.png`
**Explanation:** Scales all text (axis labels, title, legend) to 14pt, making figures more readable when projected during presentations.

### Add confidence interval error bars to centroid points
**Args:** `--input clustered.tsv --x-col 1 --y-col 2 --cluster 3 --error-bars --output with_errors.png`
**Explanation:** Adds symmetric error bars representing standard error around each centroid, communicating uncertainty in the cluster position estimates.

### Save a minimal reproducible plot script
**Args:** `--input data.tsv --x-col 1 --y-col 2 --cluster 3 --script-output reproduce.sh --output plot.png`
**Explanation:** Writes a standalone shell script containing all flags and parameters used, enabling exact re-generation of the plot without storing intermediate state.

### Embed the plot directly into a multi-page PDF document
**Args:** `--input data.tsv --x-col 1 --y-col 2 --output document.pdf --pdf-append --page-label "Fig S1"`
**Explanation:** Appends the generated plot as a new page to an existing PDF rather than generating a standalone image, useful for automated report generation.

### Use a dark theme for visualizations
**Args:** `--input data.tsv --x-col 1 --y-col 2 --theme dark --output dark_mode.png`
**Explanation:** Inverts the plot background and text colors to a dark theme suitable for presentation software with light text, reducing eye strain in dark rooms.

### Set point shape to only label centroids for cleaner visualization
**Args:** `--input clustered.tsv --x-col 1 --y-col 2 --cluster 3 --centroids-only --output clean_centroids.png`
**Explanation:** Displays only centroid markers without individual points when clusters are well-separated, reducing visual clutter while preserving the main message.

### Write a configuration file documenting all plot settings
**Args:** `--input data.tsv --x-col 1 --y-col 2 --cluster 3 --dump-config config.json --output plot.png`
**Explanation:** Emits a JSON file with all applied settings alongside the image, facilitating review, reuse, and modification of plot parameters.

### Annotate specific points by row index
**Args:** "--input data.tsv --x-col 1 --y-col 2 --annotate 10