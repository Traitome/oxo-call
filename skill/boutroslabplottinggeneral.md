---
name: boutroslabplottinggeneral
category: Visualization
description: R package for generating publication-ready plots for bioinformatics and biological research. Supports scatter plots, heatmaps, box plots, line plots, and complex multi-panel figures with customizable aesthetics and annotation features.
tags: [plotting, visualization, bioinformatics, R, graphics, publication]
author: AI-generated
source_url: https://bioconductor.org/packages/release/bioc/html/boutroslab.plotting.general.html
---

## Concepts

- The package operates as an R library loaded via `library(boutroslab.plotting.general)` and provides high-level plotting functions like `create.scatterplot()`, `create.heatmap()`, and `create.boxplot()` that accept data frames or matrices as input.
- Plot output formats are controlled by the `filename` parameter—specifying extensions like `.pdf`, `.png`, or `.tiff` determines the graphics device, with PDF recommended for publication-quality vector graphics.
- The package uses a consistent argument structure across plotting functions: `data` for the input data, `x` and `y` specifying column names for coordinates, `filename` for output, and `key` parameters for legends and annotations.
- Aesthetic customization is handled through separate style objects (e.g., `boutroslab.plotting.general.style()`) that define theme settings globally, ensuring visual consistency across multiple plots in a manuscript.
- Grid layout for multi-panel figures is managed via the `layout` parameter or by specifying `mar` (margins) and `omi` (outer margins) in inches, allowing precise control over panel arrangement.

## Pitfalls

- Failing to specify the `filename` extension explicitly results in the default device (usually quartz or windows) being used, which produces screen-resolution raster output unsuitable for publication print requirements.
- Mismatching `x` or `y` column names with actual data frame column names triggers errors since the package performs strict name matching; ensure column names in your data exactly match the strings passed to x/y parameters.
- Using `pdf()` device outside the package functions while expecting boutroslab styling causes incompatibility—the package style objects must be applied within the create.* functions to affect axes, labels, and legend formatting.
- Setting excessively large `cex` (character expansion) values without adjusting `mar` or `omi` causes plot elements to be clipped at figure edges, particularly problematic when adding axis labels orTitles.
- Attempting to overlay multiple plot types (e.g., combining scatter and density) in a single panel without using the `add` parameter results in each subsequent plot overwriting the previous one rather than compositing.

## Examples

### Generate a scatter plot with confidence ellipses
**Args:** `create.scatterplot( formula = gene_expression ~ mutation_status, data = mydata, filename = "scatter_ellipse.pdf", type = "ci", conf.level = 0.95 )`
**Explanation:** The `type = "ci"` argument adds 95% confidence ellipses around each group defined by mutation_status, useful for visualizing group-specific variance in gene expression studies.

### Create a heatmap with row and column clustering
**Args:** `create.heatmap( x = colnames(expression_matrix), y = rownames(expression_matrix), filename = "heatmap_clustered.pdf", clustering = "both", as.is = TRUE )`
**Explanation:** Setting `clustering = "both"` performs hierarchical clustering on rows and columns simultaneously, while `as.is = TRUE` preserves the matrix values for color mapping.

### Generate side-by-side box plots for multiple conditions
**Args:** `create.boxplot( formula = response ~ treatment + genotype, data = clinical_data, filename = "boxplot_paired.pdf", con红旗 = TRUE )`
**Explanation:** The `formula` syntax with two variables creates grouped box plots with treatment on x-axis and genotype as grouping factor, enabling visual comparison across experimental conditions.

### Add a legend and customize point styles
**Args:** `create.scatterplot( x = timepoint, y = abundance, data = timeseries, filename = "timeseries.pdf", key = list( points = list( col = c("red","blue"), pch = c(16,17) ), text = c("Control","Treated") ) )`
**Explanation:** The `key` parameter creates a custom legend mapping point colors and symbols to treatment groups, essential when plotting multiple experimental conditions in one figure.

### Export plot as high-resolution PNG for web use
**Args:** `create.scatterplot( x = variable1, y = variable2, data = df, filename = "plot_300dpi.png", resolution = 300, width = 10, height = 8 )`
**Explanation:** Explicitly setting `resolution = 300` and dimensions in inches produces a print-resolution PNG suitable for web display or digital publication where file size matters.

### Customize axis labels and font sizes
**Args:** `create.scatterplot( x = log2_fold_change, y = p_value, data = de_results, filename = "volcano.pdf", xaxis.label = "log2 Fold Change", yaxis.label = "-log10(p-value)", cex.axis = 1.2, cex.lab = 1.5 )`
**Explanation:** Custom axis labels with increased font sizes (`cex.axis`, `cex.lab`) improve readability in volcano plots commonly used in differential expression analysis.

### Create a multi-panel figure using layout
**Args:** `create.scatterplot( x = numeric_col1, y = numeric_col2, data = df, filename = "multipanel.pdf", layout = c(2,2), par.mar = c(3,3,2,1) )`
**Explanation:** The `layout = c(2,2)` parameter arranges successive plots in a 2×2 grid, and `par.mar` sets consistent margins across all four panels for aligned figures.

### Add reference lines at thresholds
**Args:** `create.scatterplot( x = expression, y = survival_time, data = patient_data, filename = "survival_scatter.pdf", add.grid = TRUE, horizontal.lines = c(0, -log10(0.05)), vertical.lines = c(-1, 1) )`
**Explanation:** Adding reference lines at common thresholds (e.g., p=0.05 significance, fold-change cutoffs) creates scatter plots with decision boundaries for classification tasks.