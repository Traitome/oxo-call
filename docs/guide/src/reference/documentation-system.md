# Documentation System

## Overview

The documentation system in oxo-call serves a critical role: it provides grounding context for the LLM, preventing hallucination by anchoring generated commands in real tool documentation.

## Documentation Sources

Documentation is merged from multiple sources in this order:

1. **Cached `--help` output** — Automatically captured on first use
2. **Live `--help` output** — Fresh capture if cache is stale
3. **Local documentation files** — User-provided `.md`, `.txt`, `.rst`, `.html` files
4. **Remote URLs** — Fetched from HTTP/HTTPS sources

## Storage

- **Index**: JSON metadata file tracking all indexed tools
- **Cache**: Per-tool text files with combined documentation
- **Location**: Platform-specific data directory

## Validation & Security

- **Tool names**: Must be alphanumeric with hyphens, dots, and underscores only
- **URLs**: Only HTTP/HTTPS accepted (no file://, ftp://, etc.)
- **Help text**: Validated to be 80–16,000 characters
- **Deduplication**: 80% overlap detection prevents redundant content
- **Path traversal**: Tool names are sanitized to prevent directory traversal attacks

## API

The documentation system is accessed through two main modules:

- `src/docs.rs` — Documentation resolver (fetch, merge, validate)
- `src/index.rs` — Index manager (add, remove, update, list, metadata)
