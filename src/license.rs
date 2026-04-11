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
///   Commercial: w_shixiang@163.com
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

/// Shared config consumed by `oxo-license`.
pub static OXO_CALL_CONFIG: oxo_license::LicenseConfig = oxo_license::LicenseConfig {
    schema_version: SCHEMA_VERSION,
    public_key_base64: EMBEDDED_PUBLIC_KEY_BASE64,
    license_env_var: "OXO_CALL_LICENSE",
    app_qualifier: "io",
    app_org: "traitome",
    app_name: "oxo-call",
    license_filename: "license.oxo.json",
};

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
         • Purchase a commercial license : w_shixiang@163.com\n\
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
        "Failed to parse license file as JSON: {0}\n\
         \n\
         Common causes:\n\
         • The file was modified or truncated — do not edit license files\n\
         • The file was saved with a UTF-8 BOM (some Windows editors add this automatically)\n\
         • The file was downloaded as an HTML page instead of raw JSON\n\
         \n\
         To fix:\n\
         • Download the public test license (raw JSON) directly from:\n\
           https://raw.githubusercontent.com/Traitome/oxo-call/main/docs/public-academic-test-license.oxo.json\n\
         • Apply for a personal academic license: https://github.com/Traitome/oxo-call#license\n\
         • Contact w_shixiang@163.com for a commercial license"
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
         Please contact w_shixiang@163.com to obtain a valid license."
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
    verify_license_with_key(license, OXO_CALL_CONFIG.public_key_base64)
}

/// Verify a [`LicenseFile`] against an arbitrary Base64-encoded public key.
///
/// This function is used by `verify_license` (with the embedded key) and by
/// unit tests (with an ephemeral test key).
pub fn verify_license_with_key(
    license: &LicenseFile,
    pubkey_base64: &str,
) -> Result<(), LicenseError> {
    let oxo_license = to_oxo_license_file(license);
    oxo_license::verify_license_with_key(&oxo_license, pubkey_base64, SCHEMA_VERSION)
        .map_err(map_oxo_error)
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
    #[cfg(not(target_arch = "wasm32"))]
    let projectdirs_path = directories::ProjectDirs::from("io", "traitome", "oxo-call")
        .map(|dirs| dirs.config_dir().join("license.oxo.json"));
    #[cfg(target_arch = "wasm32")]
    let projectdirs_path: Option<PathBuf> = None;
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
    oxo_license::find_license_path(cli_path, &OXO_CALL_CONFIG).or_else(|| {
        let candidates = default_license_candidates();
        candidates.into_iter().next()
    })
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

    // Strip UTF-8 BOM if present (some Windows editors prepend it automatically)
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);

    let license: LicenseFile = serde_json::from_str(content).map_err(LicenseError::ParseError)?;

    verify_license(&license)?;

    Ok(license)
}

fn to_oxo_license_file(license: &LicenseFile) -> oxo_license::LicenseFile {
    oxo_license::LicenseFile {
        payload: oxo_license::LicensePayload {
            schema: license.payload.schema.clone(),
            license_id: license.payload.license_id.clone(),
            issued_to_org: license.payload.issued_to_org.clone(),
            contact_email: license.payload.contact_email.clone(),
            license_type: license.payload.license_type.to_string(),
            scope: license.payload.scope.clone(),
            perpetual: license.payload.perpetual,
            issued_at: license.payload.issued_at.clone(),
        },
        signature: license.signature.clone(),
    }
}

fn map_oxo_error(error: oxo_license::LicenseError) -> LicenseError {
    match error {
        oxo_license::LicenseError::NotFound => LicenseError::NotFound,
        oxo_license::LicenseError::ReadError { path, source } => {
            LicenseError::ReadError { path, source }
        }
        oxo_license::LicenseError::ParseError(err) => LicenseError::ParseError(err),
        oxo_license::LicenseError::InvalidSchema { expected, found } => {
            LicenseError::InvalidSchema { expected, found }
        }
        oxo_license::LicenseError::InvalidSignature => LicenseError::InvalidSignature,
        oxo_license::LicenseError::InvalidPublicKey(err) => LicenseError::InvalidPublicKey(err),
        oxo_license::LicenseError::InvalidSignatureEncoding(err) => {
            LicenseError::InvalidSignatureEncoding(err)
        }
    }
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
  • Commercial licenses are per-organization, one-time fee; contact: w_shixiang@163.com

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
    use crate::ENV_LOCK;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
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

    // ─── LicenseType::Display ─────────────────────────────────────────────────

    #[test]
    fn test_license_type_display_academic() {
        let lt = LicenseType::Academic;
        assert_eq!(lt.to_string(), "academic");
    }

    #[test]
    fn test_license_type_display_commercial() {
        let lt = LicenseType::Commercial;
        assert_eq!(lt.to_string(), "commercial");
    }

    // ─── find_license_path: OXO_CALL_LICENSE env var ─────────────────────────

    #[test]
    fn test_find_license_path_from_env_var() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        unsafe {
            std::env::set_var("OXO_CALL_LICENSE", path.to_str().unwrap());
        }
        let found = find_license_path(None);
        // Should use the env var path
        assert_eq!(found.as_deref(), Some(path.as_path()));
        unsafe {
            std::env::remove_var("OXO_CALL_LICENSE");
        }
    }

    #[test]
    fn test_find_license_path_from_cli_arg_takes_precedence_over_env() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let cli_path = PathBuf::from("/tmp/cli-license.json");
        let env_path = PathBuf::from("/tmp/env-license.json");
        unsafe {
            std::env::set_var("OXO_CALL_LICENSE", env_path.to_str().unwrap());
        }
        let found = find_license_path(Some(&cli_path));
        assert_eq!(found.as_deref(), Some(cli_path.as_path()));
        unsafe {
            std::env::remove_var("OXO_CALL_LICENSE");
        }
    }

    // ─── load_and_verify: valid license from temp file ────────────────────────

    #[test]
    fn test_load_and_verify_valid_license() {
        let (key, pubkey_b64) = make_test_keypair();
        let license = make_license(&key, LicenseType::Academic);

        // Write the license to a temp file
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let json = serde_json::to_string_pretty(&license).unwrap();
        std::fs::write(tmp.path(), json.as_bytes()).unwrap();

        // load_and_verify uses the EMBEDDED key, so this verifies with the real key —
        // but we can't use our test key for load_and_verify.
        // Instead verify that load_and_verify reads and parses without I/O error.
        // We test verify_license_with_key separately for correctness.
        let result = load_and_verify(Some(tmp.path()));
        // This will fail with InvalidSignature because we used a test key,
        // not the embedded key. Just verify it doesn't fail with I/O error.
        match result {
            Err(LicenseError::InvalidSignature) => {} // expected with test key
            Err(LicenseError::InvalidSchema { .. }) => {} // also acceptable
            Ok(_) => {} // would pass if test key == embedded key (impossible by design)
            Err(e) => panic!("unexpected error type: {e}"),
        }
        // Verify the key-specific check works
        assert!(verify_license_with_key(&license, &pubkey_b64).is_ok());
    }

    // ─── verify_license: with embedded key (smoke test) ───────────────────────

    #[test]
    fn test_verify_license_embedded_key_wrong_schema_fails() {
        // A license signed with the test key but with wrong schema
        let (key, _) = make_test_keypair();
        let mut payload = LicensePayload {
            schema: "wrong-schema".to_string(),
            license_id: "00000000-0000-0000-0000-000000000099".to_string(),
            issued_to_org: "Test".to_string(),
            contact_email: None,
            license_type: LicenseType::Academic,
            scope: "org".to_string(),
            perpetual: true,
            issued_at: "2025-01-01".to_string(),
        };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let sig = key.sign(&payload_bytes);
        let license = LicenseFile {
            payload: payload.clone(),
            signature: STANDARD.encode(sig.to_bytes()),
        };
        // verify_license uses EMBEDDED_PUBLIC_KEY_BASE64 — should fail with InvalidSchema
        let result = verify_license(&license);
        assert!(
            matches!(result, Err(LicenseError::InvalidSchema { .. })),
            "expected InvalidSchema, got: {result:?}"
        );
        payload.schema = SCHEMA_VERSION.to_string();
        let _ = payload;
    }

    // ─── LicenseError: invalid public key ─────────────────────────────────────

    #[test]
    fn test_invalid_public_key_base64_fails() {
        let (key, _) = make_test_keypair();
        let license = make_license(&key, LicenseType::Academic);
        let result = verify_license_with_key(&license, "!!!not-valid-base64!!!");
        assert!(
            matches!(result, Err(LicenseError::InvalidPublicKey(_))),
            "expected InvalidPublicKey, got: {result:?}"
        );
    }

    #[test]
    fn test_invalid_public_key_wrong_length_fails() {
        let (key, _) = make_test_keypair();
        let license = make_license(&key, LicenseType::Academic);
        // Valid base64 but only 16 bytes, not 32
        let short_key = STANDARD.encode([0u8; 16]);
        let result = verify_license_with_key(&license, &short_key);
        assert!(
            matches!(result, Err(LicenseError::InvalidPublicKey(_))),
            "expected InvalidPublicKey, got: {result:?}"
        );
    }

    // ─── LicenseError: invalid signature encoding ─────────────────────────────

    #[test]
    fn test_invalid_signature_encoding_fails() {
        let (key, pubkey_b64) = make_test_keypair();
        let mut license = make_license(&key, LicenseType::Academic);
        license.signature = "!!!not-valid-base64!!!".to_string();
        let result = verify_license_with_key(&license, &pubkey_b64);
        assert!(
            matches!(result, Err(LicenseError::InvalidSignatureEncoding(_))),
            "expected InvalidSignatureEncoding, got: {result:?}"
        );
    }

    #[test]
    fn test_invalid_signature_wrong_length_fails() {
        let (key, pubkey_b64) = make_test_keypair();
        let mut license = make_license(&key, LicenseType::Academic);
        // Valid base64 but 32 bytes (not 64)
        license.signature = STANDARD.encode([0u8; 32]);
        let result = verify_license_with_key(&license, &pubkey_b64);
        assert!(
            matches!(result, Err(LicenseError::InvalidSignatureEncoding(_))),
            "expected InvalidSignatureEncoding, got: {result:?}"
        );
    }

    // ─── legacy_unix_license_path_from_home ──────────────────────────────────

    #[test]
    fn test_legacy_unix_license_path_some() {
        let home = PathBuf::from("/home/alice");
        let path = legacy_unix_license_path_from_home(Some(home));
        assert_eq!(
            path,
            Some(PathBuf::from(
                "/home/alice/.config/oxo-call/license.oxo.json"
            ))
        );
    }

    #[test]
    fn test_legacy_unix_license_path_none() {
        let path = legacy_unix_license_path_from_home(None);
        assert!(path.is_none());
    }
}
