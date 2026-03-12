# License System

## Technical Details

### Cryptographic Verification

- **Algorithm**: Ed25519 (RFC 8032)
- **Implementation**: `ed25519-dalek` crate
- **Verification**: Offline (no network required)
- **Public key**: Embedded in the binary at compile time

### License Payload Schema

```json
{
  "schema": "oxo-call-license-v1",
  "license_id": "uuid-v4",
  "issued_to_org": "Organization Name",
  "contact_email": "user@example.com",
  "license_type": "academic|commercial",
  "scope": "full",
  "issued_at": "2024-01-01T00:00:00Z",
  "signature": "base64-encoded-ed25519-signature"
}
```

### License Resolution

oxo-call searches for a license in this order:

1. **CLI argument**: `--license /path/to/license.oxo.json`
2. **Environment variable**: `OXO_CALL_LICENSE`
3. **Platform config directory**: `<config_dir>/license.oxo.json`
4. **Legacy Unix path**: `~/.oxo-call/license.oxo.json`

### License Gate Enforcement

The license gate is enforced in `src/main.rs` before command dispatch. Only these are exempt:
- `license` subcommands
- `--help` (handled by Clap before `run()`)
- `--version` (handled by Clap before `run()`)

## Maintainer Tools

The `crates/license-issuer` crate provides offline signing tools:

```bash
# Generate a new Ed25519 keypair
cargo run -p license-issuer -- generate-keypair

# Issue a license
cargo run -p license-issuer -- issue \
  --org "Example University" \
  --type academic \
  --output license.oxo.json
```

> **Important**: `license.rs` and `license-issuer/src/main.rs` must be kept in sync — they share the same payload schema.
