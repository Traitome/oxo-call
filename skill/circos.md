---
name: circos
category: Visualization
description: A bioinformatics tool for generating circular generic plots and genomic visualizations such as ideograms, links, heatmaps, and scatter plots.
tags: bioinformatics, genomics, visualization, circular-plot, chromosomes, SVG, ideogram
author: AI-generated
source_url: http://circos.ca/
---

## Concepts

- Circos operates entirely from configuration files (YAML-like syntax) that define all plot elements including karyotype (chromosome organization), tracks (heatmaps, scatter, line), links (arcs), and global image parameters. The main execution command requires a `-conf` flag pointing to this configuration file.
- Data inputs require tab-delimited or comma-separated files for links, regions, histogram data, scatter plots, and other data tracks. Files must use consistent delimiters and include chromosome identifiers matching the karyotype definition.
- The tool generates output in SVG (scalable vector graphics) by default, which can be converted to PNG using the `--cd` flag. The SVG output is preferred for publication-quality images as it maintains resolution independence.
- Circos uses a "karyotype" file that defines chromosome names, lengths, and optionally color/staining patterns. This file serves as the foundation for all positioning in the circular layout.
- Multiple data tracks are rendered in order from outermost to innermost ring of the circle, with each track type having specific formatting rules defined in the configuration.

## Pitfalls

- **Misconfigured karyotype paths**: If the karyotype file path referenced in the configuration does not exist or contains chromosome identifiers that don't match your data file identifiers, Circos produces no output and error messages can be cryptic.
- **Track ordering conflicts**: Defining overlapping tracks without adjusting their radial spacing or z-depth causes elements to obscure each other, resulting in data being hidden or rendered incorrectly.
- **Image dimension overflow**: Setting image radii too large for the defined image size causes track elements to be clipped at the image boundaries, resulting in lost data visualization.
- **Delimiter mismatches in data files**: Using tabs in configuration but commas in data files (or vice versa) causes parsing failures that prevent track rendering without clear error feedback.
- **Missing required parameters**: Omitting mandatory track parameters (like `r0` and `r1` for radial positions) causes immediate termination without generating output files.

## Examples

### Run Circos with a configuration file
**Args:** `-conf etc/circos.conf`
**Explanation:** Executes the Circos visualization using the specified configuration file that defines all plot parameters, data files, and output settings.

### Generate output with a specific file name
**Args:** `-conf etc/circos.conf -outputfile mygenome.png`
**Explanation:** Creates the visualization and saves the output directly to `mygenome.png` instead of using the default filename from the configuration.

### Enable SVG debugging mode
**Args:** `-conf etc/circos.conf -svgdebug`
**Explanation:** Dumps SVG positioning information and intermediate coordinates to aid in troubleshooting layout issues with tracks or links.

### Display Circos version information
**Args:** `-version`
**Explanation:** Prints the installed Circos version number which is essential for reproducibility and when reporting bugs or seeking support.

### Display command-line help
**Args:** `-help`
**Explanation:** Shows all available command-line options and their descriptions, useful for learning new flags or verifying installation.