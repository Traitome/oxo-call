---
name: wget
category: networking
description: Non-interactive network downloader for HTTP, HTTPS, and FTP with resume and recursion support
tags: [wget, download, http, ftp, networking, mirror, recursive]
author: oxo-call built-in
source_url: "https://www.gnu.org/software/wget/manual/wget.html"
---

## Concepts

- wget downloads files from the web non-interactively. Basic syntax: 'wget [options] URL'. By default, it saves to a file named after the URL basename in the current directory.
- Key output flags: -O <file> (save to named file, use '-' for stdout), -P <dir> (save into directory), -q (quiet), -v (verbose, default), --progress=bar (show progress bar).
- Resume and retry: -c (continue/resume partial download), --tries=N (retry N times, 0=infinite), --timeout=N (connection timeout in seconds), --wait=N (seconds between downloads).
- wget follows HTTP redirects automatically by default (unlike curl which needs -L). It also handles cookies with --load-cookies and --save-cookies.
- Recursive download: -r (recursive), -l N (depth limit, default 5), -np (no-parent, don't ascend to parent directories), -A '.pdf' (accept pattern), -R '.html' (reject pattern).
- Background download: -b runs wget in background; log goes to wget-log. Use 'tail -f wget-log' to monitor progress.

## Pitfalls

- wget -O '' with an empty filename string will fail silently. Always provide a valid filename or use -O - to explicitly pipe to stdout.
- wget -r without --no-parent (-np) will recurse into parent directories, potentially downloading the entire website. Always add -np for scoped recursive downloads.
- Without -c, re-running wget on a partial download will overwrite (and restart) the partial file. Use -c to resume.
- wget -r creates a directory structure mirroring the remote URL. The actual file will be at hostname/path/file, not in the current directory.
- --tries=0 means infinite retries. Use a reasonable limit like --tries=5 to avoid infinite loops on permanently unavailable URLs.
- wget does not support HTTP methods other than GET/POST. For PUT, DELETE, or custom methods, use curl instead.

## Examples

### download a file and save with its remote filename
**Args:** `https://example.com/files/data.tar.gz`
**Explanation:** saves as data.tar.gz in the current directory; wget follows redirects automatically

### download a file with a custom local filename
**Args:** `-O /data/myfile.csv https://example.com/export/data.csv`
**Explanation:** -O specifies the local output filename; -O - streams to stdout

### resume an interrupted download
**Args:** `-c https://example.com/large-file.iso`
**Explanation:** -c continues from where the download stopped if the partial file exists

### download in the background with logging
**Args:** `-b -q https://example.com/large-dataset.tar.gz`
**Explanation:** -b runs in background; output logged to wget-log; -q suppresses console output

### download with retry and timeout settings
**Args:** `--tries=5 --timeout=30 --wait=2 https://example.com/file.tar.gz`
**Explanation:** --tries retries up to 5 times; --timeout limits connection time; --wait pauses between retries

### mirror a website section without going to parent directories
**Args:** `-r -l 2 -np -P ./mirror https://example.com/docs/`
**Explanation:** -r recursive; -l 2 depth limit; -np no-parent; -P saves to ./mirror directory

### download only PDF files recursively from a site
**Args:** `-r -l 3 -np -A '.pdf' https://example.com/papers/`
**Explanation:** -A '.pdf' accepts only PDF files; -r -l 3 -np limits recursive scope

### download a list of URLs from a file
**Args:** `-i urls.txt -P downloads/`
**Explanation:** -i reads URLs from a file (one per line); -P saves all files into downloads/

### send a POST request and download the response
**Args:** `--post-data='query=search+term' -O result.html https://example.com/search`
**Explanation:** --post-data sends a URL-encoded POST body; -O saves the response

### download with a custom User-Agent header
**Args:** `--user-agent='Mozilla/5.0' -O page.html https://example.com/page`
**Explanation:** --user-agent sets the User-Agent header; some servers reject default wget User-Agent

### download with authentication credentials
**Args:** `--user=username --password=pass -O file.zip https://example.com/protected/file.zip`
**Explanation:** --user and --password provide HTTP basic authentication credentials

### download through a proxy server
**Args:** `--proxy=on --proxy-user=user --proxy-password=pass -O file.zip https://example.com/file.zip`
**Explanation:** uses proxy settings from environment or command line; --proxy-user for proxy authentication

### download and follow relative links only
**Args:** `-r -l 2 --relative -P ./local https://example.com/docs/`
**Explanation:** --relative follows only relative links; prevents downloading external sites

### download with bandwidth limiting
**Args:** `--limit-rate=500k -O largefile.iso https://example.com/largefile.iso`
**Explanation:** --limit-rate limits download speed; useful for not saturating network

### download and convert links for offline viewing
**Args:** `-r -l 2 -np -k -P ./offline https://example.com/docs/`
**Explanation:** -k converts links to local references; creates browsable offline copy

### download with timestamping (only if newer)
**Args:** `-N https://example.com/updated-data.txt`
**Explanation:** -N (timestamping) downloads only if remote file is newer than local

### download with certificate checking disabled
**Args:** `--no-check-certificate -O file.zip https://example.com/file.zip`
**Explanation:** --no-check-certificate disables SSL certificate validation; use with caution
