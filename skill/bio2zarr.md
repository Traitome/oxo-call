---
name: bio2zarr
category: bioinformatics-data-conversion
description: A command-line tool that converts biological data formats such as HDF5, NetCDF, and N5 into Zarr format for efficient chunk-based storage, compression, and cloud-native access.
tags:
  - zarr
  - hdf5
  - netcdf
  - data-conversion
  - chunked-storage
  - hpc
  - cloud-native
author: AI-Generated
source_url: https://github.com/fomightez/bio2zarr
---

## Concepts

- **Zarr as the target format**: bio2zarr converts HDF5, NetCDF-4, and N5 datasets into Zarr stores, which use chunked arrays with per-chunk compression (defaulting to blosc) and support for coordinate variables. This means the output is splittable, making it ideal for parallel I/O on HPC systems and cloud storage backends.
- **Chunking control via `--chunks`**: You must specify chunk sizes per dimension using a comma-separated list (e.g., `--chunks 100,100,10`). Chunk shape directly impacts read performance — chunks that are too small cause overhead from many reads, while chunks that are too large waste memory during partial access.
- **Dtype and scaling awareness**: If a dataset uses an integer dtype with an attached scale or offset attribute (common in imaging formats like microscopy HDF5), bio2zarr preserves these as Zarr array attributes. Failing to specify `--scale` / `--offset` when converting scaled data will produce raw integer values rather than physically meaningful floats.
- **Group and dataset hierarchy**: Source formats like HDF5 and NetCDF use hierarchical groups; bio2zarr maps these into the Zarr group structure faithfully. The output path must be specified with `--output` or `-o`, and intermediate directories are created automatically.
- **Transcoding with `--transcode`**: When the output Zarr already exists, bio2zarr will refuse to overwrite it by default. Using `--transcode` forces reconversion of an existing Zarr, useful when you change chunking or compression parameters.

## Pitfalls

- **Mismatched chunk dimensionality**: Specifying `--chunks` with the wrong number of dimensions for the input array (e.g., passing two chunk sizes for a 3D array) causes bio2zarr to either raise an error or produce incorrectly shaped chunks, breaking downstream tools that expect uniformly chunked data.
- **Forgetting `--dimension-separator` for HDF5 paths**: HDF5 datasets may use external links or soft links whose paths contain slashes. By default, bio2zarr maps these to Zarr group paths using the standard slash separator. If your downstream tool uses a Zarr consumer that does not handle `/` in array names (e.g., some numcodecs-based pipelines), the conversion silently fails or produces unreadable output.
- **Ignoring the fill-value contract**: Integer arrays in HDF5 often use a fill value (e.g., `-32767` for signed 16-bit) to mark missing data. Without the `--fill-value` flag, bio2zarr may preserve the raw fill value in the Zarr output, and subsequent analysis that does not check the fill-value metadata will treat these pixels as real data.
- **Converting large HDF5 files without `--shard`**: Very large single-array HDF5 files (tens of GBs) converted with default per-chunk blosc compression can produce fragmented Zarr stores. Not using `--shard` or not selecting an appropriate sharding strategy results in excessive per-chunk metadata overhead, inflating storage size and degrading read performance.
- **Omitting `-- compressor` for read-heavy workflows**: If no compressor is specified, bio2zarr defaults to blosc with reasonable defaults, but for read-heavy access patterns the default compression level may be suboptimal. Skipping explicit `--compressor` when you need specific compression (e.g., `zstd:5`) means accepting the default, which may yield slower decompression than a lighter codec like `zlib:1` for certain data types.

## Examples

### Convert a 2D HDF5 dataset to a Zarr array with explicit chunking

**Args:** `--input data.h5 --path /images/stack --output data.zarr --chunks 512,512`
**Explanation:** The `--chunks 512,512` flag instructs bio2zarr to split the 2D image array into 512×512 pixel tiles, enabling partial reads that load only the needed tiles into memory during downstream visualization.

### Convert a 3D NetCDF-4 volume with Zstd compression

**Args:** `--input volume.nc --path /temperature --output volume.zarr --chunks 100,100,50 --compressor zstd:5 --dtype float32`
**Explanation:** The `--compressor zstd:5` flag applies Zstandard compression at level 5 to each chunk, reducing the Zarr store size significantly for float32 volume data while maintaining fast decompression for repeated read access.

### Preserve scale and offset attributes from a microscopy HDF5 file

**Args:** `--input microscopy.h5 --path /raw --output processed.zarr --chunks 256,256,1 --scale 0.5 --offset -128`
**Explanation:** The `--scale 0.5 --offset -128` flags ensure that integer pixel values in the source HDF5 are converted to physically meaningful float values (intensity = scale × value + offset) in the Zarr output, which is required for quantitative imaging analysis.

### Force reconversion of an existing Zarr store with new chunk sizes

**Args:** `--input large.h5 --path /data --output large.zarr --chunks 200,200,20 --transcode`
**Explanation:** The `--transcode` flag forces bio2zarr to overwrite the existing Zarr store at `large.zarr`, applying the new `--chunks 200,200,20` specification so that downstream tools that require different chunking can access the data without re-converting from the source HDF5.

### Convert a multi-group HDF5 file preserving the full hierarchy as Zarr groups

**Args:** `--input experiment.h5 --output experiment.zarr --chunks 128,128,128 --recursive`
**Explanation:** The `--recursive` flag instructs bio2zarr to traverse all groups in the HDF5 file and map each one to a corresponding Zarr group, preserving the original dataset hierarchy so that tools reading the Zarr can navigate `/experiment/conditionA/signal` paths identically.

### Convert an N5 dataset with a custom dimension separator

**Args:** `--input scan.n5 --path /s0 --output scan.zarr --chunks 64,64,64 --compressor blosc --dimension-separator /`
**Explanation:** The `--dimension-separator /` flag specifies that each chunk boundary in the Zarr output should use `/` as the subfolder separator, matching the N5 format convention and ensuring compatibility with N5-compatible readers like Python's `python-bioimage-series`.

### Convert a large HDF5 file using sharded storage for reduced metadata overhead

**Args:** `--input huge.h5 --path /volume --output huge.zarr --chunks 500,500,500 --shard --compressor zstd:3`
**Explanation:** The `--shard` flag causes bio2zarr to package multiple chunks into shard files with a single shared metadata header, dramatically reducing the number of files and metadata operations when reading the resulting Zarr from object stores like S3, which is critical for performance at scale.