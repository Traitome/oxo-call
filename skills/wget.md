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
**Explanation:** wget command; https://example.com/files/data.tar.gz URL to download; saves as data.tar.gz in current directory; follows redirects automatically

### download a file with a custom local filename
**Args:** `-O /data/myfile.csv https://example.com/export/data.csv`
**Explanation:** wget command; -O /data/myfile.csv custom output filename; https://example.com/export/data.csv URL to download; -O - streams to stdout

### resume an interrupted download
**Args:** `-c https://example.com/large-file.iso`
**Explanation:** wget command; -c continues/resumes partial download; https://example.com/large-file.iso URL to download; continues from where download stopped

### download in the background with logging
**Args:** `-b -q https://example.com/large-dataset.tar.gz`
**Explanation:** wget command; -b runs in background; -q suppresses console output; https://example.com/large-dataset.tar.gz URL; log goes to wget-log

### download with retry and timeout settings
**Args:** `--tries=5 --timeout=30 --wait=2 https://example.com/file.tar.gz`
**Explanation:** wget command; --tries=5 retries up to 5 times; --timeout=30 connection timeout in seconds; --wait=2 seconds between retries; https://example.com/file.tar.gz URL

### mirror a website section without going to parent directories
**Args:** `-r -l 2 -np -P ./mirror https://example.com/docs/`
**Explanation:** wget command; -r recursive download; -l 2 depth limit; -np no-parent (don't ascend); -P ./mirror save directory; https://example.com/docs/ URL

### download only PDF files recursively from a site
**Args:** `-r -l 3 -np -A '.pdf' https://example.com/papers/`
**Explanation:** wget command; -r recursive; -l 3 depth limit; -np no-parent; -A '.pdf' accept only PDFs; https://example.com/papers/ URL

### download a list of URLs from a file
**Args:** `-i urls.txt -P downloads/`
**Explanation:** wget command; -i urls.txt reads URLs from file (one per line); -P downloads/ save directory

### send a POST request and download the response
**Args:** `--post-data='query=search+term' -O result.html https://example.com/search`
**Explanation:** wget command; --post-data='query=search+term' URL-encoded POST body; -O result.html output filename; https://example.com/search URL

### download with a custom User-Agent header
**Args:** `--user-agent='Mozilla/5.0' -O page.html https://example.com/page`
**Explanation:** wget command; --user-agent='Mozilla/5.0' custom User-Agent; -O page.html output filename; https://example.com/page URL; some servers reject default wget UA

### download with authentication credentials
**Args:** `--user=username --password=pass -O file.zip https://example.com/protected/file.zip`
**Explanation:** wget command; --user=username --password=pass HTTP basic authentication; -O file.zip output filename; https://example.com/protected/file.zip URL

### download through a proxy server
**Args:** `--proxy=on --proxy-user=user --proxy-password=pass -O file.zip https://example.com/file.zip`
**Explanation:** wget command; --proxy=on enable proxy; --proxy-user=user --proxy-password=pass proxy credentials; -O file.zip output filename; https://example.com/file.zip URL

### download and follow relative links only
**Args:** `-r -l 2 --relative -P ./local https://example.com/docs/`
**Explanation:** wget command; -r recursive; -l 2 depth limit; --relative follow only relative links; -P ./local save directory; https://example.com/docs/ URL

### download with bandwidth limiting
**Args:** `--limit-rate=500k -O largefile.iso https://example.com/largefile.iso`
**Explanation:** wget command; --limit-rate=500k limits speed to 500 KB/s; -O largefile.iso output filename; https://example.com/largefile.iso URL

### download and convert links for offline viewing
**Args:** `-r -l 2 -np -k -P ./offline https://example.com/docs/`
**Explanation:** wget command; -r recursive; -l 2 depth limit; -np no-parent; -k convert links to local; -P ./offline save directory; https://example.com/docs/ URL; creates browsable offline copy

### download with timestamping (only if newer)
**Args:** `-N https://example.com/updated-data.txt`
**Explanation:** wget command; -N timestamping; https://example.com/updated-data.txt URL; downloads only if remote file is newer than local

### download with certificate checking disabled
**Args:** `--no-check-certificate -O file.zip https://example.com/file.zip`
**Explanation:** wget command; --no-check-certificate disables SSL validation; -O file.zip output filename; https://example.com/file.zip URL; use with caution
