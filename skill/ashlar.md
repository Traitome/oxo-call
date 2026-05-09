---
name: ashlar
category: Image Processing
description: A bioinformatics tool for aligning and stitching large histological image datasets, commonly used in digital pathology to register multi-channel and multi-round tissue images.
tags: [histology, image registration, stitching, pathology, microscopy, bioimaging]
author: AI-generated
source_url: https://github.com/labsyspharm/ashlar
---

## Concepts

- **Pyramidal TIFF input requirement:** ASHLAR operates on pyramidal multi-resolution TIFF files (e.g., .tif, .ndpi, .svs) where each file contains the original high-resolution image plus downsampled versions. Without pyramidal structure, processing will fail or produce incorrect alignments.

- **Pixel size specification is mandatory:** The `--pixel-size` flag must be specified in micrometers per pixel. If the metadata contains accurate pixel size information, you may use `--use-pixelsize-from-image`, otherwise the alignment will be incorrect and images will appear scaled improperly.

- **Channel and round alignment modes:** ASHLAR supports separate alignment modes for different imaging rounds using `--align-channel` (reference channel for alignment) and `--align-mode` options. Each imaged tissue section (round) can be aligned independently or to a common reference stack.

- **Edge handling for overlapping tiles:** The `--edge-mode` parameter controls how overlapping tile edges are blended. Options include `overlap` (blend overlapping regions), `tile` (use first tile's data), and `replace` (later tiles override earlier ones). Incorrect edge mode causes visible seams or data loss at tile boundaries.

- **Output file format:** By default, ASHLAR outputs a single stitched NDTIF (multi-dimensional TIFF) file containing all aligned planes. Use `--output-format` to specify alternative formats, and `--num-threads` to control parallel processing speed.

## Pitfalls

- **Forgetting to specify pixel size:** Omitting `--pixel-size` (or its equivalent) when metadata is unreliable causes images to be registered at wrong scales, leading to completely incorrect downstream analyses such as cell counting or spatial gene expression mapping.

- **Using non-pyramidal TIFF input:** Attempting to process single-resolution images or flattened JPEG/PNG files will either crash with an error or produce severely degraded results because ASHLAR relies on multi-resolution pyramids for efficient processing.

- **Misconfiguring the reference channel:** Specifying the wrong `--align-channel` (one with weak or no features) results in poor alignment quality across all channels, causing visible misalignment artifacts and inaccurate co-registration of molecular signals.

- **Ignoring filter size parameters:** The `--filter-sigma` parameter controls Gaussian smoothing before feature detection. A value too small produces noisy alignments, while an overly large value smooths away critical structural features needed for accurate registration.

- **Mismatched image dimensions across rounds:** When aligning multi-round data, using images with significantly different field-of-view sizes without adjusting `--maximum-shift` results in failed alignments or misplacement of entire image regions.

## Examples

### Align a single multi-resolution TIFF file to itself (self-alignment)
**Args:** `--pixel-size 0.65 input.tif`
**Explanation:** This initializes ASHLAR's internal reference coordinates using a single input file where the image serves as its own reference; useful for verifying image integrity before multi-image registration.

### Align multiple tile files from the same imaging round
**Args:** `--pixel-size 0.65 --align-channel 1 tile_01.tif tile_02.tif tile_03.tif`
**Explanation:** Aligns three overlapping tile images using channel 1 (typically DAPI or a nuclear stain) as the reference for feature-based alignment, producing a single stitched output.

### Register two imaging rounds from adjacent tissue sections
**Args:** `--pixel-size 0.65 --align-channel 1 --maximum-shift 500 round1.tif round2.tif`
**Explanation:** Registers a second imaging round to the first using a maximum allowed shift of 500 micrometers; critical for multi-round experiments like CODEX or indexed staining where tissue sections are aligned together.

### Process NDPI files from a pathology scanner
**Args:** `--pixel-size 0.65 --use-pixelsize-from-image sample.ndpi`
**Explanation:** Uses the pixel size embedded in the NDPI metadata rather than manually specifying it; many pathology scanners embed accurate scaling information in their proprietary formats.

### Align images with Gaussian smoothing for noisy input
**Args:** `--pixel-size 0.65 --filter-sigma 5.0 input.tif`
**Explanation:** Applies a Gaussian filter with sigma=5.0 pixels before feature detection to reduce noise-induced alignment errors; recommended when input images have high background noise or artifacts.

### Use overlap edge mode for seamless tile stitching
**Args:** `--pixel-size 0.65 --edge-mode overlap tile_*.tif`
**Explanation:** Uses alpha-blending in overlapping regions between tiles to create smooth transitions without visible seams; preferred for publication-quality composite images.

### Control parallel processing for large datasets
**Args:** `--pixel-size 0.65 --num-threads 8 large_image_1.tif large_image_2.tif`
**Explanation:** Allocates 8 CPU threads to speed up computation on multi-core systems; reduces wall-clock time significantly when processing high-resolution pathology images.