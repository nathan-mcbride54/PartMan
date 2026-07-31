//! Tests for reading the token file.

use std::io::Write as _;

use crate::tokens::{TokenError, TokenSet, repository_token_path};

/// Write `contents` to a uniquely named temporary file and return its path.
///
/// Named per process and per call for the reason `crates/fixtures` records:
/// two concurrent `cargo test` runs of the same crate must not choose the same
/// path and delete each other's inputs.
fn temporary_token_file(contents: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "partman-tokens-{}-{}.json",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = std::fs::File::create(&path).expect("create temporary token file");
    file.write_all(contents.as_bytes())
        .expect("write temporary token file");
    path
}

// Requirements: UI-001
//   The crate resolves the one repository token source from its manifest directory rather than from a caller-controlled working directory
// Evidence: the_repository_token_file_is_where_the_crate_expects_it
#[test]
fn the_repository_token_file_is_where_the_crate_expects_it() {
    let path = repository_token_path();
    assert!(
        path.is_file(),
        "expected the token file at {}",
        path.display()
    );
    // Resolved from CARGO_MANIFEST_DIR rather than the working directory, so a
    // test invoked from the workspace root and one invoked from the crate
    // directory find the same file.
    assert!(
        path.ends_with("schemas/design-tokens.json")
            || path.ends_with("schemas\\design-tokens.json")
    );
}

// Requirements: UI-001, UI-008
//   The shipped token file loads with its pinned specification and token-set versions and includes the default dark theme
// Evidence: loading_the_repository_token_file_succeeds_and_carries_its_versions
#[test]
fn loading_the_repository_token_file_succeeds_and_carries_its_versions() {
    let set = TokenSet::load_repository_tokens().expect("repository tokens load");
    assert!(!set.token_set_version.is_empty());
    assert_eq!(
        set.spec_version, "4.0.0",
        "the token set records which specification version it was written against"
    );
    assert!(set.themes.contains_key("dark"));
}

// Requirements: UI-008
//   A malformed colour in any declared role is rejected while loading the whole token set rather than deferred until first use
// Evidence: a_malformed_colour_is_refused_at_load_rather_than_at_first_use
#[test]
fn a_malformed_colour_is_refused_at_load_rather_than_at_first_use() {
    // The whole file is validated up front. A bad colour in a role that no
    // pairing currently references would otherwise sit undetected until the
    // day something referenced it, which is the worst possible moment.
    // `r##"..."##`, not `r#"..."#`: the JSON below contains `"#16181C"`, and the
    // `"#` inside it would otherwise close the raw string early.
    let path = temporary_token_file(
        r##"{
          "tokenSetVersion": "1.0.0",
          "specVersion": "4.0.0",
          "themes": {
            "dark": {
              "label": "Dark",
              "requirement": "UI-001",
              "colors": { "surface.base": "#16181C", "text.primary": "not-a-colour" }
            }
          },
          "contrastRules": { "thresholds": { "text": 4.5 }, "pairings": [] },
          "nonColorChannels": { "roles": {} },
          "colorVisionSeparation": { "minimumDeltaE": 12.0, "mustRemainDistinct": [] }
        }"##,
    );
    let error = TokenSet::load(&path).expect_err("a malformed colour must be refused");
    assert!(
        matches!(error, TokenError::Color { ref role, .. } if role == "text.primary"),
        "expected a colour error naming the role, got {error}"
    );
    assert!(error.to_string().contains("text.primary"));
    let _ = std::fs::remove_file(&path);
}

// Requirements: UI-008
//   Malformed JSON fails closed with source-path context instead of becoming an empty or partial accessibility policy
// Evidence: malformed_json_is_refused_with_the_path_in_the_message
#[test]
fn malformed_json_is_refused_with_the_path_in_the_message() {
    let path = temporary_token_file("{ this is not json");
    let error = TokenSet::load(&path).expect_err("malformed JSON must be refused");
    assert!(matches!(error, TokenError::Parse { .. }));
    assert!(error.to_string().contains("design") || error.to_string().contains("partman-tokens"));
    let _ = std::fs::remove_file(&path);
}

// Requirements: UI-001, UI-008
//   A missing token source is a read error rather than a default palette or a vacuously clean audit
// Evidence: a_missing_file_is_refused_rather_than_treated_as_an_empty_token_set
#[test]
fn a_missing_file_is_refused_rather_than_treated_as_an_empty_token_set() {
    let missing = std::env::temp_dir().join("partman-tokens-does-not-exist-9f3a.json");
    let error = TokenSet::load(&missing).expect_err("a missing file must be refused");
    assert!(matches!(error, TokenError::Read { .. }));
}

// Requirements: UI-007, UI-008
//   Omitting a required policy section is rejected instead of defaulted, so absent non-colour channels cannot pass the accessibility gate
// Evidence: a_file_missing_a_required_section_is_refused_not_defaulted
#[test]
fn a_file_missing_a_required_section_is_refused_not_defaulted() {
    // Defaulting an absent `nonColorChannels` to empty would turn UI-007 into a
    // rule that passes precisely when nobody wrote it down.
    let path = temporary_token_file(
        r#"{
          "tokenSetVersion": "1.0.0",
          "specVersion": "4.0.0",
          "themes": {},
          "contrastRules": { "thresholds": {}, "pairings": [] },
          "colorVisionSeparation": {
            "minimumDeltaE": 12.0,
            "mustRemainDistinct": []
          }
        }"#,
    );
    let error = TokenSet::load(&path).expect_err("a truncated token file must be refused");
    assert!(matches!(error, TokenError::Parse { .. }));
    assert!(
        error.to_string().contains("nonColorChannels"),
        "the file omits only nonColorChannels, so that must be the parse failure: {error}"
    );
    let _ = std::fs::remove_file(&path);
}

// Requirements: UI-001, UI-003
//   Theme-and-role lookup resolves only declared values and returns no invented fallback for unknown themes or semantic roles
// Evidence: colour_lookup_resolves_through_theme_and_role
#[test]
fn colour_lookup_resolves_through_theme_and_role() {
    let set = TokenSet::load_repository_tokens().expect("repository tokens load");
    assert!(set.color("dark", "surface.base").is_some());
    assert!(set.color("dark", "role.that.does.not.exist").is_none());
    assert!(
        set.color("theme-that-does-not-exist", "surface.base")
            .is_none()
    );
}
