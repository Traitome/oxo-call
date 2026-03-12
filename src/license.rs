/// Offline license verification for oxo-call.
///
/// oxo-call uses a **dual-license** model:
///
/// | Use case                             | License                           |
/// |--------------------------------------|-----------------------------------|
/// | Academic / research / education      | Free — requires a license file    |
/// | Personal non-commercial              | Free — requires a license file    |
/// | Commercial / production use          | Commercial license (per-org fee)  |
///
/// All users (academic and commercial) **must** obtain and place a signed
/// `license.oxo.json` file before the CLI will run core commands.
///
/// License file search order:
///   1. `--license <path>` CLI argument
///   2. `OXO_CALL_LICENSE` environment variable
///   3. Platform config directory from `directories::ProjectDirs`
///   4. Legacy Unix path: `~/.config/oxo-call/license.oxo.json`
///
/// To obtain a license:
///   Academic : <https://github.com/Traitome/oxo-call#license>
///   Commercial: license@traitome.com
use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The license schema identifier embedded in every license file.
pub const SCHEMA_VERSION: &str = "oxo-call-license-v1";

/// Ed25519 public key embedded in the binary (Base64-encoded 32 bytes).
///
/// Rotate this key only when you are intentionally changing the license trust
/// root (for example, after a private-key compromise or a planned key
/// rotation). When this value changes, regenerate any signed fixture licenses
/// used by tests so CI continues to verify against the embedded public key
/// without needing the private issuer key.
pub const EMBEDDED_PUBLIC_KEY_BASE64: &str = "SOTbyPWS8fSF+XS9dqEg9cFyag0wPO/YMA5LhI4PXw4=";

// ── Data structures ──────────────────────────────────────────────────────────

/// The license type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LicenseType {
    /// Free academic / educational / personal non-commercial license.
    Academic,
    /// Paid commercial license (per organization, perpetual).
    Commercial,
}

impl std::fmt::Display for LicenseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LicenseType::Academic => write!(f, "academic"),
            LicenseType::Commercial => write!(f, "commercial"),
        }
    }
}

/// The payload that is signed by the license issuer.
///
/// **Field order is part of the wire format** — do not reorder fields.
/// `serde_json::to_vec` is used for canonicalization; struct field order
/// determines JSON key order, which must be identical between issuer and CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePayload {
    /// License schema identifier — must be [`SCHEMA_VERSION`].
    pub schema: String,
    /// Unique license ID (UUID v4).
    pub license_id: String,
    /// Full legal name of the licensed organization (or individual for academic).
    pub issued_to_org: String,
    /// Optional contact e-mail for the licensee.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact_email: Option<String>,
    /// License type: `"academic"` or `"commercial"`.
    pub license_type: LicenseType,
    /// Authorization scope — always `"org"`.
    pub scope: String,
    /// Whether the license is perpetual — always `true`.
    pub perpetual: bool,
    /// Issue date in `YYYY-MM-DD` format.
    pub issued_at: String,
}

/// Full on-disk license file: payload fields + Ed25519 `signature`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseFile {
    /// All payload fields (flattened into the top-level JSON object).
    #[serde(flatten)]
    pub payload: LicensePayload,
    /// Base64-encoded Ed25519 signature over `serde_json::to_vec(&payload)`.
    pub signature: String,
}

// ── Error type ───────────────────────────────────────────────────────────────

/// Errors returned by the license verification subsystem.
#[derive(Debug, thiserror::Error)]
pub enum LicenseError {
    #[error(
        "No license file found.\n\
         Academic use is free but requires a signed license file.\n\
         \n\
         • Apply for an academic license : https://github.com/Traitome/oxo-call#license\n\
         • Purchase a commercial license : license@traitome.com\n\
         \n\
         Once you have a license file, place it at one of:\n\
         \t1. Pass --license <path> on the command line\n\
         \t2. Set OXO_CALL_LICENSE=<path> in your environment\n\
         \t3. Platform config dir (macOS example):\n\
         \t   ~/Library/Application Support/io.traitome.oxo-call/license.oxo.json\n\
         \t4. Legacy Unix fallback:\n\
         \t   ~/.config/oxo-call/license.oxo.json"
    )]
    NotFound,

    #[error("Failed to read license file '{path}': {source}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(
        "Failed to parse license file as JSON: {0}\n  Ensure the file is a valid oxo-call license."
    )]
    ParseError(serde_json::Error),

    #[error(
        "Invalid license schema: expected '{expected}', found '{found}'.\n\
         This license was issued for a different version of oxo-call."
    )]
    InvalidSchema { expected: String, found: String },

    #[error(
        "License signature is invalid.\n\
         The license file may have been tampered with or was not issued by Traitome.\n\
         Please contact license@traitome.com to obtain a valid license."
    )]
    InvalidSignature,

    #[error("Internal error — invalid embedded public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid signature encoding in license file: {0}")]
    InvalidSignatureEncoding(String),
}

// ── Core verification ────────────────────────────────────────────────────────

/// Verify a [`LicenseFile`] against the binary's embedded public key.
///
/// Returns `Ok(())` when the license is valid, or a descriptive [`LicenseError`].
pub fn verify_license(license: &LicenseFile) -> Result<(), LicenseError> {
    verify_license_with_key(license, EMBEDDED_PUBLIC_KEY_BASE64)
}

/// Verify a [`LicenseFile`] against an arbitrary Base64-encoded public key.
///
/// This function is used by `verify_license` (with the embedded key) and by
/// unit tests (with an ephemeral test key).
pub fn verify_license_with_key(
    license: &LicenseFile,
    pubkey_base64: &str,
) -> Result<(), LicenseError> {
    // 1. Schema check
    if license.payload.schema != SCHEMA_VERSION {
        return Err(LicenseError::InvalidSchema {
            expected: SCHEMA_VERSION.to_string(),
            found: license.payload.schema.clone(),
        });
    }

    // 2. Decode the embedded public key
    let pubkey_bytes = STANDARD
        .decode(pubkey_base64)
        .map_err(|e| LicenseError::InvalidPublicKey(e.to_string()))?;
    let pubkey_array: [u8; 32] = pubkey_bytes
        .try_into()
        .map_err(|_| LicenseError::InvalidPublicKey("expected exactly 32 bytes".to_string()))?;
    let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
        .map_err(|e| LicenseError::InvalidPublicKey(e.to_string()))?;

    // 3. Decode the signature from the license file
    let sig_bytes = STANDARD
        .decode(&license.signature)
        .map_err(|e| LicenseError::InvalidSignatureEncoding(e.to_string()))?;
    let sig_array: [u8; 64] = sig_bytes.try_into().map_err(|_| {
        LicenseError::InvalidSignatureEncoding("expected exactly 64 bytes".to_string())
    })?;
    let signature = Signature::from_bytes(&sig_array);

    // 4. Canonical payload bytes (field order defined by LicensePayload declaration)
    let payload_bytes = serde_json::to_vec(&license.payload).map_err(LicenseError::ParseError)?;

    // 5. Verify
    verifying_key
        .verify(&payload_bytes, &signature)
        .map_err(|_| LicenseError::InvalidSignature)?;

    Ok(())
}

// ── Discovery ────────────────────────────────────────────────────────────────

fn legacy_unix_license_path_from_home(home_dir: Option<PathBuf>) -> Option<PathBuf> {
    home_dir.map(|home| home.join(".config/oxo-call/license.oxo.json"))
}

fn default_license_candidates_from(
    projectdirs_path: Option<PathBuf>,
    home_dir: Option<PathBuf>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(path) = projectdirs_path {
        candidates.push(path);
    }

    if let Some(path) = legacy_unix_license_path_from_home(home_dir)
        && !candidates.contains(&path)
    {
        candidates.push(path);
    }

    candidates
}

fn default_license_candidates() -> Vec<PathBuf> {
    let projectdirs_path = directories::ProjectDirs::from("io", "traitome", "oxo-call")
        .map(|dirs| dirs.config_dir().join("license.oxo.json"));
    let home_dir = std::env::var_os("HOME").map(PathBuf::from);
    default_license_candidates_from(projectdirs_path, home_dir)
}

/// Return the first candidate license file path, following this priority:
///
/// 1. `cli_path` (from `--license <path>`)
/// 2. `OXO_CALL_LICENSE` environment variable
/// 3. First existing path among the platform config dir and the legacy Unix
///    fallback `~/.config/oxo-call/license.oxo.json`
pub fn find_license_path(cli_path: Option<&Path>) -> Option<PathBuf> {
    if let Some(p) = cli_path {
        return Some(p.to_path_buf());
    }
    if let Ok(p) = std::env::var("OXO_CALL_LICENSE") {
        return Some(PathBuf::from(p));
    }
    let candidates = default_license_candidates();
    candidates
        .iter()
        .find(|path| path.exists())
        .cloned()
        .or_else(|| candidates.into_iter().next())
}

/// Load the license file from `cli_path` (or the default search path) and
/// verify its Ed25519 signature.  Returns the parsed [`LicenseFile`] on
/// success, or a descriptive [`LicenseError`] on failure.
pub fn load_and_verify(cli_path: Option<&Path>) -> Result<LicenseFile, LicenseError> {
    let path = find_license_path(cli_path).ok_or(LicenseError::NotFound)?;

    if !path.exists() {
        return Err(LicenseError::NotFound);
    }

    let content = std::fs::read_to_string(&path).map_err(|e| LicenseError::ReadError {
        path: path.clone(),
        source: e,
    })?;

    let license: LicenseFile = serde_json::from_str(&content).map_err(LicenseError::ParseError)?;

    verify_license(&license)?;

    Ok(license)
}

// ── Legacy display helpers (kept for `oxo-call license`) ─────────────────────

/// Full license information text shown by `oxo-call license`.
pub const LICENSE_INFO: &str = r#"
oxo-call License Information
═════════════════════════════

License model: Dual license (Academic free / Commercial per-org)
Licensor:      Traitome (https://github.com/Traitome)
Product:       oxo-call

PERMITTED USES
──────────────
  Academic / research / education     — free, requires a signed academic license file
  Personal non-commercial             — free, requires a signed academic license file
  Commercial / production use         — requires a purchased commercial license (per org)

REQUIREMENTS FOR ALL USERS
───────────────────────────
  • A valid signed license file must be present before running any core commands.
  • License files are issued by Traitome and verified offline using Ed25519 signatures.
  • Academic licenses are free; apply at: https://github.com/Traitome/oxo-call#license
  • Commercial licenses are per-organization, one-time fee; contact: license@traitome.com

HOW TO PLACE YOUR LICENSE FILE
────────────────────────────────
  Option 1 — CLI flag:          oxo-call --license /path/to/license.oxo.json <command>
  Option 2 — Environment var:   export OXO_CALL_LICENSE=/path/to/license.oxo.json
  Option 3 — Default location:
    macOS default: ~/Library/Application Support/io.traitome.oxo-call/license.oxo.json
    Legacy Unix:   ~/.config/oxo-call/license.oxo.json
    Windows:       %APPDATA%\oxo-call\license.oxo.json

LICENSE VERIFICATION
─────────────────────
  Run:  oxo-call license verify
  This prints the license holder, type, and issue date without running any tool.

Full license texts: LICENSE-ACADEMIC  |  LICENSE-COMMERCIAL
"#;

// ── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
pub mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    /// Generate an ephemeral signing key and return (signing_key, public_key_base64).
    pub fn make_test_keypair() -> (SigningKey, String) {
        // Use a fixed seed for deterministic tests
        let seed: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let signing_key = SigningKey::from_bytes(&seed);
        let pubkey_b64 = STANDARD.encode(signing_key.verifying_key().as_bytes());
        (signing_key, pubkey_b64)
    }

    /// Build a minimal valid [`LicenseFile`] signed with the given key.
    pub fn make_license(signing_key: &SigningKey, license_type: LicenseType) -> LicenseFile {
        let payload = LicensePayload {
            schema: SCHEMA_VERSION.to_string(),
            license_id: "00000000-0000-0000-0000-000000000001".to_string(),
            issued_to_org: "Test University".to_string(),
            contact_email: None,
            license_type,
            scope: "org".to_string(),
            perpetual: true,
            issued_at: "2025-01-01".to_string(),
        };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let signature = signing_key.sign(&payload_bytes);
        let signature_b64 = STANDARD.encode(signature.to_bytes());
        LicenseFile {
            payload,
            signature: signature_b64,
        }
    }

    #[test]
    fn test_valid_academic_license_passes() {
        let (key, pubkey_b64) = make_test_keypair();
        let license = make_license(&key, LicenseType::Academic);
        assert!(verify_license_with_key(&license, &pubkey_b64).is_ok());
    }

    #[test]
    fn test_valid_commercial_license_passes() {
        let (key, pubkey_b64) = make_test_keypair();
        let license = make_license(&key, LicenseType::Commercial);
        assert!(verify_license_with_key(&license, &pubkey_b64).is_ok());
    }

    #[test]
    fn test_tampered_signature_fails() {
        let (key, pubkey_b64) = make_test_keypair();
        let mut license = make_license(&key, LicenseType::Academic);
        // Corrupt the signature
        license.signature = STANDARD.encode([0u8; 64]);
        let err = verify_license_with_key(&license, &pubkey_b64).unwrap_err();
        assert!(
            matches!(err, LicenseError::InvalidSignature),
            "expected InvalidSignature, got: {err}"
        );
    }

    #[test]
    fn test_tampered_field_fails() {
        let (key, pubkey_b64) = make_test_keypair();
        let mut license = make_license(&key, LicenseType::Academic);
        // Tamper with a payload field after signing
        license.payload.issued_to_org = "Attacker Corp".to_string();
        let err = verify_license_with_key(&license, &pubkey_b64).unwrap_err();
        assert!(
            matches!(err, LicenseError::InvalidSignature),
            "expected InvalidSignature, got: {err}"
        );
    }

    #[test]
    fn test_wrong_schema_fails() {
        let (key, pubkey_b64) = make_test_keypair();
        let mut license = make_license(&key, LicenseType::Academic);
        license.payload.schema = "oxo-call-license-v0".to_string();
        // Re-sign with the tampered schema so signature is valid but schema wrong
        let payload_bytes = serde_json::to_vec(&license.payload).unwrap();
        let signature = key.sign(&payload_bytes);
        license.signature = STANDARD.encode(signature.to_bytes());

        let err = verify_license_with_key(&license, &pubkey_b64).unwrap_err();
        assert!(
            matches!(err, LicenseError::InvalidSchema { .. }),
            "expected InvalidSchema, got: {err}"
        );
    }

    #[test]
    fn test_no_license_path_returns_not_found() {
        // Use a path that doesn't exist
        let path = Path::new("/tmp/oxo-call-nonexistent-license-test.json");
        let err = load_and_verify(Some(path)).unwrap_err();
        assert!(
            matches!(err, LicenseError::NotFound),
            "expected NotFound, got: {err}"
        );
    }

    #[test]
    fn test_invalid_json_returns_parse_error() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(b"not valid json {{{").unwrap();
        let path = f.path().to_path_buf();
        let err = load_and_verify(Some(&path)).unwrap_err();
        assert!(
            matches!(err, LicenseError::ParseError(_)),
            "expected ParseError, got: {err}"
        );
    }

    #[test]
    fn test_roundtrip_json_serialization() {
        let (key, _) = make_test_keypair();
        let license = make_license(&key, LicenseType::Commercial);
        let json = serde_json::to_string_pretty(&license).unwrap();
        let parsed: LicenseFile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.payload.license_id, license.payload.license_id);
        assert_eq!(parsed.payload.license_type, license.payload.license_type);
        assert_eq!(parsed.signature, license.signature);
    }

    #[test]
    fn test_contact_email_optional_in_signing() {
        let (key, pubkey_b64) = make_test_keypair();
        // License without contact_email
        let mut payload = LicensePayload {
            schema: SCHEMA_VERSION.to_string(),
            license_id: "00000000-0000-0000-0000-000000000002".to_string(),
            issued_to_org: "Acme Corp".to_string(),
            contact_email: None,
            license_type: LicenseType::Commercial,
            scope: "org".to_string(),
            perpetual: true,
            issued_at: "2025-06-01".to_string(),
        };
        let sig1 = {
            let bytes = serde_json::to_vec(&payload).unwrap();
            key.sign(&bytes)
        };
        let lic_no_email = LicenseFile {
            payload: payload.clone(),
            signature: STANDARD.encode(sig1.to_bytes()),
        };
        assert!(verify_license_with_key(&lic_no_email, &pubkey_b64).is_ok());

        // Same license but with email — different payload, different signature
        payload.contact_email = Some("admin@acme.com".to_string());
        let sig2 = {
            let bytes = serde_json::to_vec(&payload).unwrap();
            key.sign(&bytes)
        };
        let lic_with_email = LicenseFile {
            payload,
            signature: STANDARD.encode(sig2.to_bytes()),
        };
        assert!(verify_license_with_key(&lic_with_email, &pubkey_b64).is_ok());

        // Cross-verify: email license with no-email signature should fail
        let lic_tampered = LicenseFile {
            payload: lic_with_email.payload.clone(),
            signature: lic_no_email.signature.clone(),
        };
        assert!(verify_license_with_key(&lic_tampered, &pubkey_b64).is_err());
    }

    #[test]
    fn test_default_license_candidates_include_legacy_unix_fallback() {
        let candidates = default_license_candidates_from(
            Some(PathBuf::from(
                "/Users/example/Library/Application Support/io.traitome.oxo-call/license.oxo.json",
            )),
            Some(PathBuf::from("/Users/example")),
        );

        assert_eq!(
            candidates,
            vec![
                PathBuf::from(
                    "/Users/example/Library/Application Support/io.traitome.oxo-call/license.oxo.json",
                ),
                PathBuf::from("/Users/example/.config/oxo-call/license.oxo.json"),
            ]
        );
    }

    #[test]
    fn test_default_license_candidates_deduplicate_same_path() {
        let path = PathBuf::from("/home/example/.config/oxo-call/license.oxo.json");
        let candidates = default_license_candidates_from(
            Some(path.clone()),
            Some(PathBuf::from("/home/example")),
        );

        assert_eq!(candidates, vec![path]);
    }
}
