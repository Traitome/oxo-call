---
name: banner
category: text-rendering
description: A Unix utility that renders ASCII text as large block-letter banners, commonly used in bioinformatics pipelines and scripts to create visual section dividers, headers, and log separators.
tags: [ascii-art, text-rendering, logging, unix-tool, visualization]
author: AI-generated
source_url: https://man7.org/linux/man-pages/man6/banner.6.html
---

## Concepts

- Banner converts text input into large ASCII block characters using fixed-width character cells, with each letter constructed from hash marks (#) or other characters across multiple text rows. Input longer than the configured width wraps automatically or truncates depending on the tool version.
- The output is purely stdout-based and purely textual, making it ideal for adding visual separators in log files, pipeline reports, and documentation generated within bioinformatics workflows. Output can be redirected to files or piped to other text processing tools.
- Banner accepts single-word inputs by default; multi-word or multi-line banners require appropriate quoting or multiple invocations. The `-w` flag controls the output width in characters, and different implementations may support alternative fill characters via `-c` or environment variables.

## Pitfalls

- Passing strings with special shell characters (spaces, pipes, wildcards) without quoting causes silent truncation or errors. Running `banner my report` produces only "my" as the banner because the shell splits on the space. Always quote arguments: `banner "my report"`.
- Banner output has no newline management—each invocation prints multiple newlines, which can create excessive whitespace in log files when used repeatedly. Bioinformatics scripts that call banner in loops may generate excessively long log files, wasting disk space and reducing readability.
- The maximum width varies by implementation; specifying `-w` values larger than supported causes layout distortion or ignored width settings on some systems. Using extremely wide widths (e.g., `-w 500`) on Linux banner implementations produces broken or no output.

## Examples

### Print a simple single-word banner header
**Args:** `Alignment Report`
**Explanation:** Prints large block letters spelling "ALIGNMENT REPORT" to stdout, creating a visual section header in pipeline output logs.

### Set custom output width for narrow terminals
**Args:** `-w 40 Section Complete`
**Explanation:** Constrains the banner to 40 characters wide so the output fits in narrower terminal windows or log viewers with constrained display width.

### Create a banner with a custom fill character
**Args:** `-c '@' WARNING`
**Explanation:** Some banner implementations support `-c` to replace the default hash-fill with an alternate character, useful for matching documentation branding or color-coded logging schemes.

### Capture banner output to a log file
**Args:** `Run Complete > pipeline.log`
**Explanation:** Redirects the banner output into a log file rather than printing to stdout, allowing programmatic log assembly where banners serve as section delimiters in pipeline execution records.

### Use banner output in a shell pipeline for further processing
**Args:** `Sample_Ready | grep -v '^$'`
**Explanation:** Pipes banner output through text filters (removing blank lines here) to create compact, non-whitespace-heavy log entries for bioinformatics audit trails.