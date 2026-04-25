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
**Explanation:** curl command; -L follows redirects; -O https://example.com/files/archive.tar.gz URL; saves with the remote filename (archive.tar.gz)

### download a file and save to a specific local filename
**Args:** `-L -o /data/dataset.csv https://example.com/dataset.csv`
**Explanation:** curl command; -L follows redirects; -o /data/dataset.csv specifies the local output filename; https://example.com/dataset.csv URL

### send a JSON POST request to an API
**Args:** `-X POST -H 'Content-Type: application/json' -d '{"name":"test","value":42}' https://api.example.com/endpoint`
**Explanation:** curl command; -X POST sets HTTP method; -H 'Content-Type: application/json' sets header; -d '{"name":"test","value":42}' JSON body data; https://api.example.com/endpoint URL

### authenticate with a Bearer token and call an API
**Args:** `-H 'Authorization: Bearer TOKEN' https://api.example.com/data`
**Explanation:** curl command; -H 'Authorization: Bearer TOKEN' adds Authorization header; https://api.example.com/data URL; replace TOKEN with actual token value

### resume an interrupted download
**Args:** `-L -C - -O https://example.com/large-file.iso`
**Explanation:** curl command; -L follows redirects; -C - resumes from where download stopped; -O https://example.com/large-file.iso URL; requires the partial file to already exist

### fetch only HTTP response headers
**Args:** `-I https://example.com`
**Explanation:** curl command; -I https://example.com URL; sends HEAD request and shows only response headers; no body is downloaded

### send a multipart form upload
**Args:** `-X POST -F 'file=@/local/path/data.txt' -F 'name=upload' https://api.example.com/upload`
**Explanation:** curl command; -X POST HTTP method; -F 'file=@/local/path/data.txt' multipart form file upload; -F 'name=upload' form field; https://api.example.com/upload URL; @filename reads file from disk

### download with progress bar and follow redirects silently
**Args:** `-L --progress-bar -o output.zip https://example.com/file.zip`
**Explanation:** curl command; -L follows redirects; --progress-bar shows simpler progress bar; -o output.zip output filename; https://example.com/file.zip URL

### set a connection timeout and retry on failure
**Args:** `--connect-timeout 10 --retry 3 --retry-delay 5 -L -O https://example.com/file.tar.gz`
**Explanation:** curl command; --connect-timeout 10 fails after 10s; --retry 3 retries up to 3 times; --retry-delay 5 waits 5s between retries; -L follows redirects; -O https://example.com/file.tar.gz URL

### pass basic authentication credentials
**Args:** `-u alice:password123 https://protected.example.com/api`
**Explanation:** curl command; -u alice:password123 HTTP basic auth credentials; https://protected.example.com/api URL; use -u alice to have curl prompt for password instead

### download multiple files in parallel
**Args:** `-Z -O https://example.com/file1.zip -O https://example.com/file2.zip`
**Explanation:** curl command; -Z enables parallel transfers; -O https://example.com/file1.zip -O https://example.com/file2.zip multiple URLs; downloads each URL to its remote filename concurrently

### fail on HTTP errors (404, 500)
**Args:** `-f -L -O https://example.com/file.zip`
**Explanation:** curl command; -f makes curl exit with error code on HTTP 4xx/5xx; -L follows redirects; -O https://example.com/file.zip URL; without -f curl returns success even on 404

### limit download speed
**Args:** `--limit-rate 1M -L -O https://example.com/large-file.iso`
**Explanation:** curl command; --limit-rate 1M caps download speed at 1 MB/s; -L follows redirects; -O https://example.com/large-file.iso URL; useful for not overwhelming network bandwidth

### set total operation timeout
**Args:** `--max-time 300 -L -O https://example.com/file.tar.gz`
**Explanation:** curl command; --max-time 300 aborts if entire operation exceeds 300 seconds; -L follows redirects; -O https://example.com/file.tar.gz URL; protects against hanging transfers

### retry on connection refused
**Args:** `--retry 5 --retry-connrefused --retry-delay 2 -L -O https://example.com/file.zip`
**Explanation:** curl command; --retry 5 retries up to 5 times; --retry-connrefused retries even when server refuses connection; --retry-delay 2 waits 2s between retries; -L follows redirects; -O https://example.com/file.zip URL; useful for services starting up

### upload file with PUT
**Args:** `-T localfile.txt https://api.example.com/upload/destination.txt`
**Explanation:** curl command; -T localfile.txt file upload via PUT request; https://api.example.com/upload/destination.txt URL; different from -d which sends file contents as POST body

### download with specific User-Agent
**Args:** `-A "Mozilla/5.0" -L -O https://example.com/file.zip`
**Explanation:** curl command; -A "Mozilla/5.0" sets User-Agent header; -L follows redirects; -O https://example.com/file.zip URL; some servers block default curl user-agent
