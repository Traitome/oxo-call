# How-to: Add Documentation for a New Tool

This guide shows you how to enrich oxo-call's documentation index for a tool, either manually or by pointing to external resources. Richer documentation means better LLM-generated commands.

---

## When do you need this?

Documentation is fetched automatically on first use — you often do not need to do anything manually. Use this guide when:

- The tool's `--help` output is too terse or missing important flags
- There is a detailed manual or tutorial page you want to include
- You want to add local documentation files for a custom or internal tool
- The automatic fetch failed and you want to provide docs manually

---

## Method 1: Automatic fetch (default)

On the first `run` or `dry-run` for a tool, oxo-call automatically runs:

```bash
<tool> --help
```

and caches the output. Nothing to do — just run your command.

To confirm it worked:

```bash
oxo-call docs list
# tool         sources   size
# samtools     help      18KB

oxo-call docs show samtools | head -20
```

---

## Method 2: Add a remote URL

For tools with detailed online documentation, adding the URL dramatically improves LLM accuracy:

```bash
# Add a documentation URL
oxo-call docs add bwa --url https://bio-bwa.sourceforge.net/bwa.shtml

# Multiple sources are combined automatically
oxo-call docs add samtools --url https://www.htslib.org/doc/samtools.html
```

oxo-call fetches the URL, strips HTML formatting, and merges it with the `--help` output. Duplicate content (80%+ overlap) is automatically deduplicated.

### Finding good documentation URLs

- **GitHub README**: `https://github.com/owner/tool#usage`
- **Tool manual page**: `https://tool.readthedocs.io/` or `https://tool.sourceforge.net/`
- **Bioconda recipe page**: links to the official docs
- **Bioinformatics tool databases**: [bio.tools](https://bio.tools), [toolshed.g2.bx.psu.edu](https://toolshed.g2.bx.psu.edu)

---

## Method 3: Add a local file

For tools with a manual page, PDF manual, or local documentation directory:

```bash
# Single file (markdown, plain text)
oxo-call docs add mytool --file /path/to/manual.md

# Directory of documentation files
oxo-call docs add mytool --dir /path/to/docs/

# Combine with --help and a URL
oxo-call docs add mytool \
  --file /path/to/manual.md \
  --url https://mytool.example.com/docs
```

### Converting a man page to text

```bash
man samtools | col -b > /tmp/samtools_man.txt
oxo-call docs add samtools --file /tmp/samtools_man.txt
```

### Converting a PDF manual

```bash
pdftotext tool_manual.pdf tool_manual.txt
oxo-call docs add mytool --file tool_manual.txt
```

---

## Updating Documentation

When a tool is updated and its flags change:

```bash
# Update a specific tool
oxo-call docs update samtools

# Update with a new URL
oxo-call docs update bwa --url https://new-docs.example.com

# Update all indexed tools
oxo-call docs update
```

---

## Removing Documentation

```bash
oxo-call docs remove mytool
```

Next time you run a command for that tool, the documentation will be automatically re-fetched.

---

## Checking Documentation Quality

View the full cached documentation for a tool:

```bash
oxo-call docs show samtools
```

Check the filesystem path:

```bash
oxo-call docs path samtools
# → ~/.local/share/oxo-call/docs/samtools.txt
```

View the raw file:

```bash
cat $(oxo-call docs path samtools)
```

---

## Troubleshooting

**Problem:** "No documentation found for tool X"

The `--help` output may have been too short (< 80 characters) to be useful. Try adding a URL:

```bash
oxo-call docs add X --url https://X.example.com/docs
```

**Problem:** Generated commands are wrong or use non-existent flags

The cached documentation may be outdated. Update it:

```bash
oxo-call docs update X
```

Then try adding remote documentation for additional context.

**Problem:** URL fetch fails

Check that the URL is accessible and returns text content (not a login page):

```bash
curl -s https://example.com/docs | head -20
```

oxo-call only accepts HTTP/HTTPS URLs and rejects paths that look like directory traversal.
