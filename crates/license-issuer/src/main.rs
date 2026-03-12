//! Offline license-signing tool for oxo-call.
//!
//! This binary is intended for **Traitome maintainers only** and is NOT
//! distributed to end-users.  It holds (or reads) the private key that
//! corresponds to the public key embedded in the `oxo-call` binary.
//!
//! # Usage
//!
//! ## Generate a new Ed25519 key pair
//! ```
//! cargo run --bin license-issuer -- generate-keypair
//! ```
//! Store the printed private key securely (e.g. in a password manager or
//! offline vault).  Embed the public key in `src/license.rs`.
//!
//! ## Issue a license
//! ```
//! # Via environment variable (preferred):
//! export OXO_LICENSE_PRIVATE_KEY="<base64-encoded 32-byte seed>"
//! cargo run --bin license-issuer -- issue \
//!     --org "Acme University" \
//!     --email research@acme.edu \
//!     --type academic \
//!     --output license.oxo.json
//!
//! # Or via file:
//! cargo run --bin license-issuer -- issue \
//!     --private-key /path/to/private.key \
//!     --org "Example Corp" \
//!     --type commercial \
//!     --output license.oxo.json
//! ```

use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Local;
use clap::{Parser, Subcommand};
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "license-issuer",
    about = "Offline license-signing tool for oxo-call (maintainer use only)",
    long_about = "Signs oxo-call license files with the Traitome Ed25519 private key.\n\
                  Keep the private key secret — never commit it to the repository."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a fresh Ed25519 key pair (use once per deployment)
    GenerateKeypair {
        /// Write the private key seed to this file instead of stdout
        #[arg(long)]
        output: Option<std::path::PathBuf>,
    },

    /// Issue (sign) a new license file
    Issue {
        /// Path to private key file containing a Base64-encoded 32-byte seed.
        /// Alternatively set OXO_LICENSE_PRIVATE_KEY environment variable.
        #[arg(long)]
        private_key: Option<std::path::PathBuf>,

        /// Full legal name of the organization (or individual for academic)
        #[arg(long)]
        org: String,

        /// Contact e-mail address (optional)
        #[arg(long)]
        email: Option<String>,

        /// License type: "academic" or "commercial"
        #[arg(long, default_value = "commercial")]
        r#type: String,

        /// Issue date in YYYY-MM-DD format (defaults to today)
        #[arg(long)]
        issued_at: Option<String>,

        /// Write the signed license JSON to this file (defaults to stdout)
        #[arg(long, short)]
        output: Option<std::path::PathBuf>,
    },
}

// ── License structures (mirrors src/license.rs) ──────────────────────────────

const SCHEMA_VERSION: &str = "oxo-call-license-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum LicenseType {
    Academic,
    Commercial,
}

/// The payload that is signed — field order is the canonical wire format.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LicensePayload {
    schema: String,
    license_id: String,
    issued_to_org: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    contact_email: Option<String>,
    license_type: LicenseType,
    scope: String,
    perpetual: bool,
    issued_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LicenseFile {
    #[serde(flatten)]
    payload: LicensePayload,
    signature: String,
}

// ── Key helpers ──────────────────────────────────────────────────────────────

/// Load a signing key from (in priority order):
///   1. `--private-key <path>` file (first line, Base64)
///   2. `OXO_LICENSE_PRIVATE_KEY` environment variable (Base64)
fn load_signing_key(path: Option<&std::path::Path>) -> anyhow::Result<SigningKey> {
    let seed_b64 = if let Some(p) = path {
        std::fs::read_to_string(p)
            .map_err(|e| anyhow::anyhow!("Cannot read private key file '{}': {e}", p.display()))?
            .trim()
            .to_string()
    } else if let Ok(val) = std::env::var("OXO_LICENSE_PRIVATE_KEY") {
        val.trim().to_string()
    } else {
        anyhow::bail!(
            "No private key provided.\n\
             Use --private-key <file> or set OXO_LICENSE_PRIVATE_KEY=<base64-seed>"
        );
    };

    let seed_bytes = STANDARD
        .decode(&seed_b64)
        .map_err(|e| anyhow::anyhow!("Failed to Base64-decode private key: {e}"))?;
    let seed_array: [u8; 32] = seed_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Private key must be exactly 32 bytes"))?;
    Ok(SigningKey::from_bytes(&seed_array))
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateKeypair { output } => {
            let signing_key = SigningKey::generate(&mut OsRng);
            let private_b64 = STANDARD.encode(signing_key.as_bytes());
            let public_b64 = STANDARD.encode(signing_key.verifying_key().as_bytes());

            let text = format!(
                "PRIVATE_KEY_SEED={private_b64}\nPUBLIC_KEY={public_b64}\n"
            );

            eprintln!("─── Ed25519 key pair generated ───────────────────────────────────────");
            eprintln!("Public key  (embed in src/license.rs): {public_b64}");
            eprintln!("Private key (keep secret, never commit): {private_b64}");
            eprintln!("──────────────────────────────────────────────────────────────────────");

            if let Some(path) = output {
                std::fs::write(&path, &text)?;
                eprintln!("Keys written to '{}'", path.display());
            }
        }

        Commands::Issue {
            private_key,
            org,
            email,
            r#type,
            issued_at,
            output,
        } => {
            let signing_key = load_signing_key(private_key.as_deref())?;

            let license_type = match r#type.to_lowercase().as_str() {
                "academic" => LicenseType::Academic,
                "commercial" => LicenseType::Commercial,
                other => anyhow::bail!("Unknown license type '{}'. Use 'academic' or 'commercial'.", other),
            };

            let issued_at_str = issued_at
                .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());

            let payload = LicensePayload {
                schema: SCHEMA_VERSION.to_string(),
                license_id: Uuid::new_v4().to_string(),
                issued_to_org: org,
                contact_email: email,
                license_type,
                scope: "org".to_string(),
                perpetual: true,
                issued_at: issued_at_str,
            };

            // Canonical bytes for signing
            let payload_bytes = serde_json::to_vec(&payload)?;
            let signature = signing_key.sign(&payload_bytes);
            let signature_b64 = STANDARD.encode(signature.to_bytes());

            let license_file = LicenseFile {
                payload,
                signature: signature_b64,
            };

            let json = serde_json::to_string_pretty(&license_file)?;

            if let Some(path) = output {
                std::fs::write(&path, &json)?;
                eprintln!(
                    "✓ License written to '{}'\n  Issued to: {}\n  Type: {}",
                    path.display(),
                    license_file.payload.issued_to_org,
                    match license_file.payload.license_type {
                        LicenseType::Academic => "academic",
                        LicenseType::Commercial => "commercial",
                    }
                );
            } else {
                println!("{json}");
            }
        }
    }

    Ok(())
}
