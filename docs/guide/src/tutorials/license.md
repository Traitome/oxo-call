# License Setup

oxo-call uses a dual-license model with offline Ed25519 signature verification.

## License Types

| Type | Cost | Scope | For |
|------|------|-------|-----|
| **Academic** | Free | University, research institute | Students, researchers, academic staff |
| **Commercial** | Per-organization | Company, commercial entity | Industry R&D, commercial use |

## Obtaining a License

### Academic License (Free)

Academic licenses are free for researchers, students, and academic institutions:

1. Send an email to `w_shixiang@163.com` with:
   - Your name and role
   - Institution/university name
   - Brief description of your research
2. You will receive a `license.oxo.json` file

### Commercial License

Commercial licenses are **USD 200 per organization** — a single license covers all employees and contractors within your organization.

Contact `w_shixiang@163.com` to obtain a commercial license.

## Installing Your License

Place the `license.oxo.json` file in one of these locations (checked in order):

### 1. CLI Argument
```bash
oxo-call --license /path/to/license.oxo.json run samtools "..."
```

### 2. Environment Variable
```bash
export OXO_CALL_LICENSE=/path/to/license.oxo.json
```

### 3. Platform-Specific Default Path

| Platform | Path |
|----------|------|
| Linux | `~/.config/oxo-call/license.oxo.json` |
| macOS | `~/Library/Application Support/io.traitome.oxo-call/license.oxo.json` |
| Windows | `%APPDATA%\traitome\oxo-call\license.oxo.json` |

## Verifying Your License

```bash
oxo-call license verify
oxo-call license show
```

## License-Exempt Commands

The following commands work without a license:

- `oxo-call --help`
- `oxo-call --version`
- `oxo-call license verify`
- `oxo-call license show`

## Security

- **Offline verification**: No network calls required for license validation
- **Ed25519 signatures**: Cryptographically secure, tamper-proof
- **Public key embedded**: The verification key is compiled into the binary
