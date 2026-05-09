---
name: biobb_remote
category: Data Transfer / Remote Operations
description: A BioBB (BioExcel Building Blocks) tool for handling remote data transfer operations including download, upload, and synchronization between local filesystems and remote servers using various protocols such as HTTP, FTP, and SFTP.
tags: [data-transfer, remote-access, file-upload, file-download, bioinformatics-workflow, biobb]
author: AI-generated
source_url: https://biobb.readthedocs.io/en/latest/reference/data_transfer/Remote.html
---

## Concepts

- **Remote Data Operations**: biobb_remote supports three primary operations — `download` retrieves files from remote servers to local paths, `upload` sends local files to remote destinations, and `sync` maintains bidirectional consistency between directories.

- **Supported Protocols**: The tool natively handles HTTP/HTTPS, FTP, and SFTP connections; protocol selection is determined by the remote URL scheme (e.g., `https://`, `ftp://`, `sftp://`), and authentication credentials are passed via standard parameters like `--user`, `--password`, or `--key_file`.

- **Output as Properties Object**: All biobb_remote commands return a `properties` dictionary containing keys like `remote_path`, `local_path`, `unique_tmp`, and `archive` — these outputs are designed to be consumed as inputs (`input_new`) by subsequent steps in a BioBB workflow chain.

- **Compression Handling**: When working with compressed archives (GZIP, ZIP, TAR), use the `--extracted_path` parameter to specify where the decompressed contents should be placed; the tool automatically detects archive formats by file extension or magic bytes.

## Pitfalls

- **Protocol Mismatch Causes Connection Failures**: Specifying an incorrect protocol (e.g., using `sftp://` for an HTTP-only endpoint) will result in a connection error; always verify the remote server supports your intended protocol before execution.

- **Overwriting Without Backup**: Running upload or sync operations without setting `--overwrite false` will silently replace existing remote files, potentially losing data that was not backed up; the default behavior permits overwriting.

- **Missing Credentials for Protected Endpoints**: Attempting to access password-protected remote resources without providing `--password` or `--key_file` results in authentication failures; anonymous access must be explicitly enabled on the remote server.

- **Incorrect Path Formatting**: Mixing Windows-style backslashes (`\`) with Unix-style forward slashes (`/`) in remote paths when targeting Unix servers can cause file-not-found errors; always normalize paths to forward slashes for remote operations.

- **Timeout Ignored Without Proper Units**: The `--timeout` parameter expects seconds as an integer; passing values like "30s" or "1m" as strings will be silently coerced or cause parameter validation errors.

## Examples

### Download a single file from an HTTP server
**Args:** `download --url https://example.com/data/file.txt --output_path ./local_data/file.txt --overwrite true`
**Explanation:** This downloads `file.txt` from the HTTP server and saves it to the local `./local_data/` directory, permitting overwriting of any existing local copy.

### Upload a local file to an SFTP server with password authentication
**Args:** `upload --input_path ./results/output.csv --remote_url sftp://server.example.com/data/ --user myuser --password secret123 --overwrite false`
**Explanation:** This uploads the local file to the specified SFTP directory using password authentication, preventing accidental overwriting of existing remote files.

### Download and extract a GZIP archive to a specific directory
**Args:** `download --url https://example.com/dataset.tar.gz --output_path ./archive.tar.gz --extract true --extracted_path ./datasets/`
**Explanation:** This downloads the compressed archive and automatically decompresses its contents into the `./datasets/` directory after extraction.

### Synchronize a local directory with a remote FTP location
**Args:** `sync --input_path ./local_dir --remote_url ftp://ftp.example.com/backups/ --user anonymous --mode upload --sync_strict true`
**Explanation:** This synchronizes the local directory contents to the remote FTP server, using strict mode to ensure all remote files match local counterparts after the operation.

### Download a file with connection timeout and retry settings
**Args:** `download --url https://example.com/large_file.bam --output_path ./data/large_file.bam --timeout 120 --retry_attempts 3 --retry_delay 10`
**Explanation:** This downloads a large binary file with a 120-second connection timeout and automatic retry up to 3 times with 10-second delays between attempts, improving reliability for unstable connections.