# Security Considerations

This page documents the security model, threat mitigations, and privacy considerations for oxo-call deployments.

---

## Threat Model

oxo-call generates and optionally executes shell commands using LLM output. The primary security concerns are:

1. **LLM output trust**: Generated commands are executed via shell — malicious or incorrect LLM output could cause harm
2. **API token exposure**: LLM API tokens stored in config files
3. **Data sent to LLM providers**: Task descriptions and tool documentation are sent to external APIs
4. **Supply chain**: Dependencies must be audited for vulnerabilities
5. **License tampering**: License files could be forged or modified

---

## Mitigations

### Input Validation

- **Tool name sanitization**: `validate_tool_name()` in `src/docs.rs` rejects path traversal attempts (`../`, `/`), empty names, and names with invalid characters
- **URL validation**: Remote documentation URLs are restricted to `http://` and `https://` schemes only
- **LLM response format**: Strict `ARGS:`/`EXPLANATION:` format validation with retry — malformed responses are rejected, not executed

### Data Anonymization

The `sanitize` module (`src/sanitize.rs`) provides two anonymization functions applied before sending data to LLM providers:

- **`redact_paths()`**: Replaces absolute file paths (e.g., `/home/user/data/patient.bam`) with `<PATH>`, preserving relative paths and filenames that have semantic value for command generation
- **`redact_env_tokens()`**: Redacts environment variable values containing `TOKEN=`, `KEY=`, or `SECRET=` patterns, replacing the value with `<REDACTED>` while preserving the variable name

### What Data Is Sent to the LLM

When you run `oxo-call run` or `dry-run`, the following is sent to the LLM provider:

| Sent | Not Sent |
|------|----------|
| Tool name (e.g., "samtools") | Actual file contents |
| Your task description (natural language) | License file |
| Tool `--help` output (cached) | Config file / API tokens for other providers |
| Skill content (concepts, pitfalls, examples) | Command execution output |
| System prompt rules | History entries |

**For maximum privacy**, use Ollama with local models — no data leaves your machine. See [Switch LLM Provider](../how-to/change-llm-provider.md).

### Dry-Run Mode

Always use `dry-run` to preview generated commands before execution:

```bash
# Preview without executing
oxo-call dry-run samtools "sort input.bam"

# Execute with confirmation prompt
oxo-call run --ask samtools "sort input.bam"
```

### API Token Security

- API tokens are stored in `config.toml` or passed via environment variables
- Tokens are never logged, included in history, or sent to other providers
- Use environment variables in shared or multi-user environments to avoid storing tokens in files

### License Security

- **Offline verification**: Ed25519 signature verification requires no network calls
- **Tamper-proof**: The public key is compiled into the binary — license files cannot be forged without the private signing key
- **No phone-home**: License verification is entirely local

### Supply Chain Security

- **`cargo audit`** runs in CI to detect known vulnerabilities in dependencies
- **SHA256 checksums** (`SHA256SUMS.txt`) are published with each release for binary integrity verification
- **Minimal dependencies**: The project uses well-audited crates (ed25519-dalek, reqwest, tokio)

---

## Deployment Recommendations

### Single-User Workstation

Default configuration is appropriate. Consider using Ollama for privacy-sensitive data.

### Shared HPC Cluster

- Use environment variables for API tokens (not config files)
- Set `OXO_CALL_LICENSE` to a shared license path
- Consider running Ollama as a shared service on a dedicated node

### Clinical / Regulated Environment

- Use Ollama exclusively (no external API calls)
- Use `--ask` flag for all commands (human-in-the-loop confirmation)
- Audit command history via `oxo-call history list`
- Keep `license.oxo.json` in a secure, access-controlled directory

---

## Related

- [License System](./license-system.md) — Ed25519 verification details
- [LLM Integration](./llm-integration.md) — provider configuration and prompt architecture
- [Configuration](../tutorials/configuration.md) — API token setup
