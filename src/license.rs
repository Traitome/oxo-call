/// License information and enforcement logic for oxo-call.
///
/// oxo-call uses a **dual-license** model:
///
/// | Use case                             | License                   |
/// |--------------------------------------|---------------------------|
/// | Academic / research / education      | Free (BUSL-1.1 §2)        |
/// | Personal non-commercial              | Free (BUSL-1.1 §2)        |
/// | Commercial / production use          | Commercial license needed |
///
/// The Business Source License 1.1 (BUSL-1.1) is the canonical legal document.
/// This module provides runtime license notices and a soft-check for commercial
/// deployments.  After the **Change Date** (4 years from each version's release),
/// the code automatically converts to the MIT License.
///
/// To obtain a commercial license: <https://github.com/Traitome/oxo-call#licensing>
/// One-time license notice printed on first run (stored in config to avoid repeat).
pub const FIRST_RUN_NOTICE: &str = r#"
  ╔══════════════════════════════════════════════════════════════════╗
  ║                    oxo-call License Notice                       ║
  ║                                                                  ║
  ║  Free for academic, research, and personal non-commercial use.   ║
  ║  Commercial use requires a commercial license.                   ║
  ║                                                                  ║
  ║  Details: https://github.com/Traitome/oxo-call#licensing         ║
  ╚══════════════════════════════════════════════════════════════════╝
"#;

/// Full license information text shown by `oxo-call license`.
pub const LICENSE_INFO: &str = r#"
oxo-call License Information
═════════════════════════════

License:  Business Source License 1.1 (BUSL-1.1)
Licensor: Traitome (https://github.com/Traitome)
Product:  oxo-call

PERMITTED USES (no license key required)
─────────────────────────────────────────
  ✓  Academic research and education
  ✓  Personal non-commercial projects
  ✓  Open-source software development
  ✓  Evaluation and testing

RESTRICTED USES (commercial license required)
──────────────────────────────────────────────
  ✗  Integration into commercial products or SaaS platforms
  ✗  Use in for-profit production pipelines serving third parties
  ✗  Offering oxo-call's functionality as a paid service

CHANGE DATE & LICENSE
──────────────────────
  Four years after each version's release date, that version's source
  code automatically converts to the MIT License.

COMMERCIAL LICENSING
─────────────────────
  To obtain a commercial license, contact:
    • Email : license@traitome.com
    • Web   : https://github.com/Traitome/oxo-call#licensing

COMMUNITY CONTRIBUTIONS
────────────────────────
  Skill files contributed to the oxo-call-skills registry are released
  under CC-BY-4.0 to remain freely usable by all.

Full license text: https://github.com/Traitome/oxo-call/blob/main/LICENSE
"#;

/// Check the configured license type and return a status string for `config show`.
pub fn license_status(license_key: Option<&str>) -> &'static str {
    match license_key {
        Some(k) if !k.is_empty() && is_valid_license_key(k) => "commercial (valid)",
        Some(k) if !k.is_empty() => "commercial (key present — not verified)",
        _ => "academic / non-commercial (free)",
    }
}

/// Very lightweight license-key heuristic: a non-empty string starting with "OXO-".
/// Real validation happens server-side when the user registers the key.
/// This is intentionally soft — legal protection is in the LICENSE file.
pub fn is_valid_license_key(key: &str) -> bool {
    key.starts_with("OXO-") && key.len() >= 16
}
