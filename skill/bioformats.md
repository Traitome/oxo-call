---
name: bioformats
category: microscopy-image-io
description: Bio-Formats is a Java library and command-line toolkit for reading, writing, and converting life sciences microscopy image formats, supporting over 100 proprietary and open file formats. It provides tools for inspecting metadata, converting between formats, and extracting imaging parameters from microscopy experiments.
tags:
  - microscopy
  - imaging
  - bioimaging
  - format-conversion
  - metadata
  - OME-TIFF
  - life-sciences
  - Java
author: AI-generated
source_url: https://www.openmicroscopy.org/bioformats/
---

## Concepts

- Bio-Formats command-line tools (bftools) are invoked via companion binaries such as `showinf`, `bfconvert`, and `bfinfo` — the tool name `bioformats` itself is NOT a CLI entry point. Always call the specific companion binary for the desired operation.

- The `showinf` tool reads a microscopy file and prints all embedded metadata (dimensions, pixel type, channel names, physical calibration, timestamps, ROI data) to stdout. Use it as the primary inspection tool before any conversion or analysis pipeline.

- The `bfconvert` tool converts between microscopy formats; output format is determined by the file extension or the `--out-type` flag. It supports batch processing with glob patterns and can write OME-TIFF (`.ome.tiff`), which embeds full OME-XML metadata for long-term preservation.

- Bio-Formats automatically detects the file format even without an extension, but explicit format specification via `--format` or `formatlist` can resolve ambiguous cases. Always verify the detected format matches your expectation using `bfinfo` before downstream processing.

- Memory consumption scales with the number of series (Z-stack, time-points, channels) and tile size. Use the `--series` flag to restrict processing to a single series when working with multi-series HCS (High-Content Screening) plates to avoid OutOfMemory errors.

## Pitfalls

- Calling `bioformats` directly without a companion binary will fail because no executable named `bioformats` exists in the bftools distribution. Always use `showinf`, `bfconvert`, or `bfinfo` as the command.

- Converting large multi-series files (e.g., Leica .lif, Nikon .nd2) without specifying `--series` may cause the JVM to exhaust heap memory. Always set `--series` explicitly or increase heap with `-Xmx` to match the largest series dimensions.

- OME-TIFF output requires that the filename ends with `.ome.tiff` or the `--out-type ome.tiff` flag is set. Outputting with a plain `.tif` extension will write a raw TIFF without embedded OME-XML metadata, defeating the purpose of format standardization.

- Using the `--quiet` flag suppresses error messages, making it appear that conversion succeeded even when individual planes failed. Always remove `--quiet` during troubleshooting to see per-plane failure messages.

- Bio-Formats does not guarantee lossless conversion for all proprietary formats; some codecs (e.g., JPEG 2000 in Aperio .svs) are re-encoded during export, potentially altering pixel values. Always compare checksum or histogram after conversion to verify data integrity.

## Examples

### Display all metadata from a microscopy file

**Args:** `showinf /path/to/image.nd2`
**Explanation:** The `showinf` companion binary reads the specified file and prints pixel dimensions, channel colors, physical pixel size, timestamps, and full OME-XML metadata to the terminal for inspection.

### Convert a microscopy file to OME-TIFF for archiving

**Args:** `bfconvert --out-type ome.tiff /path/to/image.czi /path/to/output.ome.tiff`
**Explanation:** Specifying `--out-type ome.tiff` ensures the output embeds OME-XML metadata for reproducible downstream analysis, and the `.ome.tiff` extension confirms the format choice.

### Convert only the first series from a multi-series plate file

**Args:** `bfconvert --series 1 /path/to/HCS_plate.lif /path/to/series1.ome.tiff`
**Explanation:** Using `--series 1` restricts conversion to the first Z-stack/time-point/channel group, preventing memory exhaustion and producing a single-series output file for targeted analysis.

### Verify the automatically detected format before conversion

**Args:** `bfinfo /path/to/mystery.afs`
**Explanation:** Running `bfinfo` prints the file header, detected format name, and series count without modifying the file, allowing you to confirm the format is correctly identified before committing to conversion.

### Convert a directory of TIFF files to OME-TIFF using a glob pattern

**Args:** `bfconvert --out-type ome.tiff --glob "/path/to/dir/*.tif" /path/to/output_dir`
**Explanation:** The `--glob` flag processes all matching input files and writes corresponding `.ome.tiff` outputs into the specified output directory, enabling batch format standardization for an entire dataset.

### Extract the TIFF comment/tag from a proprietary file

**Args:** `showinf -tiffcomment /path/to/image.qptiff`
**Explanation:** The `-tiffcomment` flag prints only the embedded TIFF comment (ImageDescription tag), which in QuickTime肺 images contains the format-specific metadata header, useful for debugging format detection issues.

### Convert with custom chunk size to reduce memory footprint

**Args:** `bfconvert --tilex 512 --tiley 512 /path/to/large_image.nd2 /path/to/output.ome.tiff`
**Explanation:** Specifying `--tilex` and `--tiley` divides the image into 512×512 pixel tiles during conversion, allowing processing of very large files on systems with limited RAM by streaming tiles rather than loading the full image into memory.