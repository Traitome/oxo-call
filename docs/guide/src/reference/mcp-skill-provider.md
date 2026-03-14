# MCP Skill Provider

oxo-call supports the **Model Context Protocol (MCP)** as a skill source.
Any MCP-compatible server that exposes bioinformatics skill resources can act
as an organisational or project-scoped skill library, queried automatically
when oxo-call looks for a skill.

---

## What is MCP?

The [Model Context Protocol](https://modelcontextprotocol.io) is an open
standard (originally from Anthropic, now widely adopted) that lets AI clients
connect to external data and tool servers using a uniform JSON-RPC interface.
oxo-call uses MCP's **resources** API to discover and retrieve skill content
from remote servers — no new dependencies required (HTTP + JSON are already
used by the LLM client).

---

## Load Priority

When oxo-call looks for a skill it checks sources in this order (highest wins):

| Priority | Source | Location |
|----------|--------|----------|
| 1 | **User-defined** | `~/.config/oxo-call/skills/<tool>.md` |
| 2 | **Community-installed** | `~/.local/share/oxo-call/skills/<tool>.md` |
| 3 | **MCP servers** | Registered in `config.toml` under `[mcp]` |
| 4 | **Built-in** | Compiled into the binary |

MCP skills are only queried when no higher-priority skill is found for the
requested tool, keeping latency impact minimal for well-covered tools.

---

## MCP Server Contract

An MCP server acting as a skill provider must implement three JSON-RPC methods:

### `initialize`

Standard MCP handshake. oxo-call sends its client info and accepts any
response conforming to the MCP 2024-11-05 protocol version.

### `resources/list`

Lists available skill resources. oxo-call recognises skill resources in two ways:

1. **Preferred — `skill://` URI scheme**
   ```json
   {
     "uri": "skill://samtools",
     "name": "samtools",
     "description": "Suite of programs for SAM/BAM/CRAM handling",
     "mimeType": "text/markdown"
   }
   ```

2. **Fallback — `text/markdown` MIME type**
   Any resource with `"mimeType": "text/markdown"` is treated as a skill.
   The resource `"name"` field is used as the tool name.

### `resources/read`

Returns the Markdown content for a given URI.

```json
// Request
{ "jsonrpc": "2.0", "id": 3, "method": "resources/read",
  "params": { "uri": "skill://samtools" } }

// Response
{ "jsonrpc": "2.0", "id": 3,
  "result": { "contents": [{ "uri": "skill://samtools",
                              "text": "---\nname: samtools\n...",
                              "mimeType": "text/markdown" }] } }
```

The `text` field must contain valid oxo-call skill Markdown (YAML front-matter
followed by `## Concepts`, `## Pitfalls`, and `## Examples` sections).
Legacy TOML format is also accepted as a fallback.

---

## Transport

oxo-call uses **stateless HTTP POST** (no SSE session required):

```
POST <server-url>/mcp HTTP/1.1
Content-Type: application/json
Accept: application/json
MCP-Protocol-Version: 2024-11-05
Authorization: Bearer <api-key>   # only if api_key is configured

{"jsonrpc":"2.0","id":1,"method":"initialize","params":{...}}
```

If the server URL already ends in `/mcp`, it is used as-is; otherwise `/mcp`
is appended automatically.

**Timeout**: 5 seconds per request. Network errors are silently ignored (the
next source in the priority chain is tried).

---

## Configuration

Register MCP servers in `~/.config/oxo-call/config.toml`:

```toml
[[mcp.servers]]
url  = "http://localhost:3000"
name = "local-skills"

[[mcp.servers]]
url     = "https://skills.example.org"
name    = "org-skills"
api_key = "your-secret-token"
```

Or manage them with the CLI:

```bash
# Register a server
oxo-call skill mcp add http://localhost:3000 --name local-skills

# Register an authenticated server
oxo-call skill mcp add https://skills.example.org \
    --name org-skills --api-key your-secret-token

# List registered servers
oxo-call skill mcp list

# Test connectivity
oxo-call skill mcp ping

# Remove a server
oxo-call skill mcp remove local-skills
```

---

## Building an MCP Skill Server

Any HTTP server that implements the three JSON-RPC methods above qualifies.
Below is a minimal example in Python using the
[`mcp`](https://pypi.org/project/mcp/) SDK:

```python
from mcp.server.fastmcp import FastMCP
from pathlib import Path

mcp = FastMCP("my-skills")
SKILLS_DIR = Path("./skills")   # directory of .md files

@mcp.resource("skill://{tool}")
def get_skill(tool: str) -> str:
    """Serve a skill Markdown file."""
    path = SKILLS_DIR / f"{tool}.md"
    if not path.exists():
        raise FileNotFoundError(f"No skill for '{tool}'")
    return path.read_text()

if __name__ == "__main__":
    mcp.run(transport="streamable-http", port=3000)
```

Skills served must follow the [oxo-call skill format](./skill-system.md).

---

## Verification

After registering a server:

```bash
# Show all skills including MCP sources
oxo-call skill list

# Show a skill from an MCP server
oxo-call skill show mytool

# Ping all registered servers
oxo-call skill mcp ping
```

`skill list` shows MCP-sourced skills with a yellow `mcp:<server-name>` label.
`skill mcp ping` reports each server's name, version, and skill count.

---

## Security Considerations

- **api_key** is stored in plain text in `config.toml`.  Restrict file
  permissions (`chmod 600 ~/.config/oxo-call/config.toml`).
- Only `http://` and `https://` URLs are supported.
- Skill content from MCP servers is parsed and validated before use; invalid
  content is silently skipped.
- oxo-call does not execute any code returned by MCP servers.  Skill content
  is only injected into LLM prompts.

See [Security Considerations](./security.md) for the full security model.
