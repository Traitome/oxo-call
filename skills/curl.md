---
name: curl
category: networking
description: Transfer data to/from servers supporting HTTP, HTTPS, FTP, and many other protocols
tags: [curl, http, https, download, api, rest, networking, upload, request, ftp, sftp, wget]
author: oxo-call built-in
source_url: "https://curl.se/docs/manpage.html"
---

## Concepts

- curl transfers data using URLs. Basic syntax: 'curl [options] URL'. By default, output goes to stdout — use -o FILE to save to a file or -O to use the remote filename.
- Key download flags: -o <file> (save to named file), -O (save with remote filename), -L (follow HTTP redirects, important for most download links), -C - (resume interrupted download).
- HTTP methods and data: -X POST (set method), -d 'data' (send POST body, sets Content-Type to application/x-www-form-urlencoded), -H 'header: value' (add header), --data-raw / --data-binary for raw body.
- Authentication: -u user:password (basic auth), -H 'Authorization: Bearer TOKEN' (token auth). Never pass passwords in command history; use -u user (curl will prompt) or environment variables.
- SSL/TLS: by default curl verifies certificates. Use -k / --insecure to skip verification (insecure, development only). Specify a CA bundle with --cacert.
- Output and debugging: -v shows request/response headers (verbose); -s silences progress but keeps output; -S shows errors even with -s; -I fetches only the HTTP response headers (HEAD request).
- --max-time sets total operation timeout; --connect-timeout sets only connection phase timeout.
- --retry retries on transient errors; --retry-connrefused also retries on connection refused.
- --limit-rate throttles bandwidth; useful for not overwhelming shared networks.
- -Z / --parallel enables parallel transfers for multiple URLs.
- -f / --fail makes curl exit with error on HTTP 4xx/5xx responses (by default curl returns 0 even on 404).

## Pitfalls

- Without -L, curl does NOT follow HTTP redirects. Many download URLs (GitHub releases, CDNs) redirect, so always add -L when downloading files.
- Without -o or -O, curl dumps response body to stdout. When piping (curl url | tar xz), this is intended, but it means the file is not saved unless explicitly specified.
- Large POST data: using -d @filename reads data from a file; -d 'raw string' inline data — do not confuse the two. For binary data always use --data-binary @file.
- curl -X DELETE or -X PUT without -d may send an empty body. Some APIs require an empty JSON body '{}' even for delete/update requests; add -H 'Content-Type: application/json' -d '{}'.
- The -k / --insecure flag should never be used in production scripts — it disables certificate verification and exposes you to MITM attacks.
- When using -o with multiple URLs, each URL needs its own -o flag in the correct order, or use -O for each URL.
- By default curl exits 0 even on HTTP errors (404, 500); use -f / --fail to make it exit non-zero on HTTP errors.
- --connect-timeout only affects connection phase; use --max-time for total operation timeout.
- --retry-delay waits between retries; --retry-max-time limits total retry duration.
- -d @file sends file contents; -T file uploads the file (different semantics for PUT vs POST).

## Examples

### download a file and save with its original filename
**Args:** `-L -O https://example.com/files/archive.tar.gz`
**Explanation:** -L follows redirects; -O saves with the remote filename (archive.tar.gz)

### download a file and save to a specific local filename
**Args:** `-L -o /data/dataset.csv https://example.com/dataset.csv`
**Explanation:** -o specifies the local output filename; -L follows redirects

### send a JSON POST request to an API
**Args:** `-X POST -H 'Content-Type: application/json' -d '{"name":"test","value":42}' https://api.example.com/endpoint`
**Explanation:** -X POST sets the method; -H sets Content-Type header; -d provides the JSON body

### authenticate with a Bearer token and call an API
**Args:** `-H 'Authorization: Bearer TOKEN' https://api.example.com/data`
**Explanation:** -H adds the Authorization header; replace TOKEN with the actual token value

### resume an interrupted download
**Args:** `-L -C - -O https://example.com/large-file.iso`
**Explanation:** -C - resumes from where the download stopped; requires the partial file to already exist

### fetch only HTTP response headers
**Args:** `-I https://example.com`
**Explanation:** -I sends a HEAD request and shows only the response headers; no body is downloaded

### send a multipart form upload
**Args:** `-X POST -F 'file=@/local/path/data.txt' -F 'name=upload' https://api.example.com/upload`
**Explanation:** -F sends a multipart/form-data request; @filename tells curl to read the file from disk

### download with progress bar and follow redirects silently
**Args:** `-L --progress-bar -o output.zip https://example.com/file.zip`
**Explanation:** --progress-bar shows a simpler progress bar instead of the default stats block

### set a connection timeout and retry on failure
**Args:** `--connect-timeout 10 --retry 3 --retry-delay 5 -L -O https://example.com/file.tar.gz`
**Explanation:** --connect-timeout fails after 10s; --retry retries up to 3 times with 5s delay

### pass basic authentication credentials
**Args:** `-u alice:password123 https://protected.example.com/api`
**Explanation:** -u user:pass sends HTTP basic auth; use -u alice to have curl prompt for password instead

### download multiple files in parallel
**Args:** `-Z -O https://example.com/file1.zip -O https://example.com/file2.zip`
**Explanation:** -Z enables parallel transfers; multiple -O flags download each URL to its remote filename concurrently

### fail on HTTP errors (404, 500)
**Args:** `-f -L -O https://example.com/file.zip`
**Explanation:** -f makes curl exit with error code on HTTP 4xx/5xx; without -f curl returns success even on 404

### limit download speed
**Args:** `--limit-rate 1M -L -O https://example.com/large-file.iso`
**Explanation:** --limit-rate 1M caps download speed at 1 MB/s; useful for not overwhelming network bandwidth

### set total operation timeout
**Args:** `--max-time 300 -L -O https://example.com/file.tar.gz`
**Explanation:** --max-time 300 aborts if entire operation exceeds 300 seconds; protects against hanging transfers

### retry on connection refused
**Args:** `--retry 5 --retry-connrefused --retry-delay 2 -L -O https://example.com/file.zip`
**Explanation:** --retry-connrefused retries even when server refuses connection; useful for services starting up

### upload file with PUT
**Args:** `-T localfile.txt https://api.example.com/upload/destination.txt`
**Explanation:** -T uploads file via PUT request; different from -d which sends file contents as POST body

### download with specific User-Agent
**Args:** `-A "Mozilla/5.0" -L -O https://example.com/file.zip`
**Explanation:** -A sets User-Agent header; some servers block default curl user-agent
