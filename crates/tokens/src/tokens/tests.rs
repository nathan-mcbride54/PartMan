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

#[test]
fn malformed_json_is_refused_with_the_path_in_the_message() {
    let path = temporary_token_file("{ this is not json");
    let error = TokenSet::load(&path).expect_err("malformed JSON must be refused");
    assert!(matches!(error, TokenError::Parse { .. }));
    assert!(error.to_string().contains("design") || error.to_string().contains("partman-tokens"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_missing_file_is_refused_rather_than_treated_as_an_empty_token_set() {
    let missing = std::env::temp_dir().join("partman-tokens-does-not-exist-9f3a.json");
    let error = TokenSet::load(&missing).expect_err("a missing file must be refused");
    assert!(matches!(error, TokenError::Read { .. }));
}

#[test]
fn a_file_missing_a_required_section_is_refused_not_defaulted() {
    // Defaulting an absent `nonColorChannels` to empty would turn UI-007 into a
    // rule that passes precisely when nobody wrote it down.
    let path = temporary_token_file(
        r#"{
          "tokenSetVersion": "1.0.0",
          "specVersion": "4.0.0",
          "themes": {},
          "contrastRules": { "thresholds": {}, "pairings": [] }
        }"#,
    );
    let error = TokenSet::load(&path).expect_err("a truncated token file must be refused");
    assert!(matches!(error, TokenError::Parse { .. }));
    let _ = std::fs::remove_file(&path);
}

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
