# Installation

oxo-call can be installed through multiple channels depending on your needs.

## From crates.io (Recommended)

The simplest way to install oxo-call is via Rust's package manager:

```bash
cargo install oxo-call
```

This downloads, compiles, and installs the latest published version. Requires [Rust](https://rustup.rs/) to be installed.

## From GitHub Releases (Pre-built Binaries)

Pre-built binaries are available for every tagged release on GitHub:

1. Visit the [Releases page](https://github.com/Traitome/oxo-call/releases)
2. Download the archive for your platform:

| Platform | Architecture | File |
|----------|-------------|------|
| Linux | x86_64 | `oxo-call-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | aarch64 | `oxo-call-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz` |
| Linux (musl) | x86_64 | `oxo-call-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz` |
| Linux (musl) | aarch64 | `oxo-call-vX.Y.Z-aarch64-unknown-linux-musl.tar.gz` |
| macOS | x86_64 (Intel) | `oxo-call-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| macOS | aarch64 (Apple Silicon) | `oxo-call-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| Windows | x86_64 | `oxo-call-vX.Y.Z-x86_64-pc-windows-msvc.zip` |
| Windows | aarch64 | `oxo-call-vX.Y.Z-aarch64-pc-windows-msvc.zip` |
| WebAssembly | wasm32-wasip1 | `oxo-call-vX.Y.Z-wasm32-wasip1.tar.gz` (advanced) |

3. Extract and move to your PATH:

```bash
# Linux / macOS
tar xzf oxo-call-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz
sudo mv oxo-call /usr/local/bin/

# Or add to your user bin directory
mv oxo-call ~/.local/bin/
```

## From Source (Git Clone)

For the latest development version or to contribute:

```bash
git clone https://github.com/Traitome/oxo-call.git
cd oxo-call
cargo install --path .
```

To build a release-optimized binary:

```bash
cargo build --release
# Binary is at target/release/oxo-call
```

## Verifying Installation

After installation, verify it works:

```bash
oxo-call --version
oxo-call --help
```

## Updating

```bash
# From crates.io
cargo install oxo-call

# From source
cd oxo-call
git pull
cargo install --path .
```

## System Requirements

- **Operating System**: Linux, macOS, or Windows
- **Rust**: 2024 edition (if building from source)
- **LLM Access**: A valid API token for at least one supported LLM provider
- **License**: A valid `license.oxo.json` file (free for academic use)
