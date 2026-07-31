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

/// Read the full repository token document for strict-loader mutation tests.
fn repository_token_text() -> String {
    let path = repository_token_path();
    std::fs::read_to_string(&path).expect("read repository token JSON")
}

/// Parse the full repository token document for structural mutations.
fn repository_token_value() -> serde_json::Value {
    serde_json::from_str(&repository_token_text())
        .expect("repository token JSON is syntactically valid")
}

/// Load token JSON text through a uniquely named temporary file.
fn load_token_text(text: &str) -> Result<TokenSet, TokenError> {
    let path = temporary_token_file(text);
    let result = TokenSet::load(&path);
    let _ = std::fs::remove_file(path);
    result
}

/// Mutate a full repository token document, load it, and remove the fixture.
fn load_mutated_repository_tokens(
    mutate: impl FnOnce(&mut serde_json::Value),
) -> Result<TokenSet, TokenError> {
    let mut value = repository_token_value();
    mutate(&mut value);
    let text = serde_json::to_string_pretty(&value).expect("serialize mutated token JSON");
    load_token_text(&text)
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
    assert_eq!(set.token_set_version, "2.0.0");
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
    let error = load_mutated_repository_tokens(|value| {
        value["themes"]["dark"]["colors"]["text.primary"] =
            serde_json::Value::String("not-a-colour".to_owned());
    })
    .expect_err("a malformed colour must be refused");
    assert!(
        matches!(error, TokenError::Color { ref role, .. } if role == "text.primary"),
        "expected a colour error naming the role, got {error}"
    );
    assert!(error.to_string().contains("text.primary"));
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
    let error = load_mutated_repository_tokens(|value| {
        value
            .as_object_mut()
            .expect("token document is an object")
            .remove("nonColorChannels");
    })
    .expect_err("a truncated token file must be refused");
    assert!(matches!(error, TokenError::Parse { .. }));
    assert!(
        error.to_string().contains("nonColorChannels"),
        "the file omits only nonColorChannels, so that must be the parse failure: {error}"
    );
}

// Requirements: UI-001, UI-008
//   Every renderer-neutral measurement, theme, typography, layout, and cursor section is mandatory rather than defaulted to an unaudited value
// Evidence: token_loader_rejects_each_missing_renderer_neutral_contract_section
#[test]
fn token_loader_rejects_each_missing_renderer_neutral_contract_section() {
    for section in [
        "measurementUnits",
        "themeSignals",
        "typography",
        "layout",
        "cursors",
    ] {
        let error = load_mutated_repository_tokens(|value| {
            value
                .as_object_mut()
                .expect("token document is an object")
                .remove(section);
        })
        .expect_err("a required renderer-neutral section must be refused when absent");
        assert!(matches!(error, TokenError::Parse { .. }));
        assert!(
            error.to_string().contains(section),
            "the document omits only {section}, so that must be the parse failure: {error}"
        );
    }
}

// Requirements: UI-001, UI-008, UI-013, Section 12
//   Every map-shaped token namespace rejects duplicate JSON member names instead of silently choosing one declaration last-wins
// Evidence: token_loader_rejects_duplicate_keys_in_every_map_namespace
#[test]
fn token_loader_rejects_duplicate_keys_in_every_map_namespace() {
    let family = r#"      "platform-ui": {
        "strategy": "platform-default"
      }"#;
    let duplicate_family = r#"      "platform-ui": {
        "strategy": "platform-default"
      },
      "platform-ui": {
        "strategy": "platform-default"
      }"#;
    let cases = [
        ("themes", "    \"high-contrast\": {", "    \"dark\": {"),
        (
            "theme colors",
            "        \"surface.base\": \"#16181C\",",
            "        \"surface.sunken\": \"#16181C\",",
        ),
        ("font families", family, duplicate_family),
        (
            "text styles",
            "      \"body-small\": {",
            "      \"body\": {",
        ),
        (
            "text flows",
            "      \"multi-line\": {",
            "      \"single-line\": {",
        ),
        ("spacing tokens", "      \"xs\": 4,", "      \"none\": 4,"),
        ("radius tokens", "      \"lg\": 14,", "      \"md\": 14,"),
        (
            "stroke tokens",
            "      \"strong\": 2,",
            "      \"hairline\": 2,",
        ),
        (
            "cursor roles",
            "      \"action\": \"pointer\",",
            "      \"default\": \"pointer\",",
        ),
        (
            "contrast thresholds",
            "\"thresholds\": { \"text\": 4.5, \"ui\": 3.0 },",
            "\"thresholds\": { \"text\": 4.5, \"text\": 3.0 },",
        ),
        (
            "semantic channels",
            "      \"entity.partition\": { \"icon\": \"slice\"",
            "      \"entity.device\": { \"icon\": \"slice\"",
        ),
    ];

    let repository = repository_token_text();
    for (namespace, original, duplicate) in cases {
        assert_eq!(
            repository.matches(original).count(),
            1,
            "the duplicate-key fixture for {namespace} must identify one canonical member"
        );
        let mutated = repository.replacen(original, duplicate, 1);
        let error = load_token_text(&mutated)
            .expect_err("a duplicate key must fail before the token audit runs");
        assert!(matches!(error, TokenError::Parse { .. }));
        assert!(
            error.to_string().contains("duplicate map key"),
            "{namespace} did not fail as an explicit duplicate: {error}"
        );
    }
}

// Requirements: UI-013
//   Theme and semantic-channel labels are localization identifiers, and legacy embedded display-label fields are rejected both as replacements and as extra aliases
// Evidence: token_loader_rejects_legacy_theme_and_semantic_channel_label_fields
#[test]
fn token_loader_rejects_legacy_theme_and_semantic_channel_label_fields() {
    let theme_error = load_mutated_repository_tokens(|value| {
        let theme = value["themes"]["dark"]
            .as_object_mut()
            .expect("dark theme is an object");
        let label_id = theme.remove("labelId").expect("dark theme has a labelId");
        theme.insert("label".to_owned(), label_id);
    })
    .expect_err("legacy theme label must be refused");
    assert!(matches!(theme_error, TokenError::Parse { .. }));
    assert!(theme_error.to_string().contains("label"));

    let channel_error = load_mutated_repository_tokens(|value| {
        let channel = value["nonColorChannels"]["roles"]["entity.device"]
            .as_object_mut()
            .expect("device channel is an object");
        let label_id = channel
            .remove("labelId")
            .expect("device channel has a labelId");
        channel.insert("label".to_owned(), label_id);
    })
    .expect_err("legacy semantic-channel label must be refused");
    assert!(matches!(channel_error, TokenError::Parse { .. }));
    assert!(channel_error.to_string().contains("label"));

    let extra_theme_label = load_mutated_repository_tokens(|value| {
        value["themes"]["dark"]
            .as_object_mut()
            .expect("dark theme is an object")
            .insert("label".to_owned(), serde_json::json!("Dark"));
    })
    .expect_err("legacy theme label must be refused even beside labelId");
    assert!(matches!(extra_theme_label, TokenError::Parse { .. }));
    assert!(extra_theme_label.to_string().contains("unknown field"));

    let extra_channel_label = load_mutated_repository_tokens(|value| {
        value["nonColorChannels"]["roles"]["entity.device"]
            .as_object_mut()
            .expect("device channel is an object")
            .insert("label".to_owned(), serde_json::json!("Device"));
    })
    .expect_err("legacy semantic label must be refused even beside labelId");
    assert!(matches!(extra_channel_label, TokenError::Parse { .. }));
    assert!(extra_channel_label.to_string().contains("unknown field"));
}

// Requirements: UI-001
//   Theme requirement traceability is independent policy rather than unaudited prose that the canonical token file can contradict
// Evidence: token_loader_rejects_unaudited_theme_requirement_metadata
#[test]
fn token_loader_rejects_unaudited_theme_requirement_metadata() {
    let error = load_mutated_repository_tokens(|value| {
        value["themes"]["dark"]
            .as_object_mut()
            .expect("dark theme is an object")
            .insert(
                "requirement".to_owned(),
                serde_json::json!("UI-001 default dark charcoal"),
            );
    })
    .expect_err("theme requirement prose must not become unaudited token metadata");
    assert!(matches!(error, TokenError::Parse { .. }));
    assert!(error.to_string().contains("requirement"));
}

// Requirements: UI-008
//   Every numeric token field uses its declared bounded integer representation, including signed negative letter spacing, instead of accepting fractional or out-of-range values
// Evidence: token_loader_enforces_integer_units_and_their_signedness
#[test]
fn token_loader_enforces_integer_units_and_their_signedness() {
    let unsigned_fields = [
        "/typography/styles/body/sizePx",
        "/typography/styles/body/weight",
        "/typography/styles/body/lineHeightPermille",
        "/layout/spacingPx/md",
        "/layout/radiusPx/md",
        "/layout/strokePx/strong",
        "/layout/focusRingOffsetPx",
        "/layout/minimumTargetSizePx",
        "/cursors/textCaretWidthPx",
    ];

    for pointer in unsigned_fields {
        for invalid in [
            serde_json::json!(1.5),
            serde_json::json!(-1),
            serde_json::json!(65_536),
        ] {
            let error = load_mutated_repository_tokens(|value| {
                *value
                    .pointer_mut(pointer)
                    .expect("numeric token pointer exists") = invalid;
            })
            .expect_err("fractional, negative, and overflowing unsigned units must be refused");
            assert!(
                matches!(error, TokenError::Parse { .. }),
                "{pointer} accepted an invalid integer representation"
            );
        }
    }

    for invalid in [
        serde_json::json!(0.5),
        serde_json::json!(-32_769),
        serde_json::json!(32_768),
    ] {
        let error = load_mutated_repository_tokens(|value| {
            value["typography"]["styles"]["title"]["letterSpacingMilliPx"] = invalid;
        })
        .expect_err("fractional and overflowing signed letter spacing must be refused");
        assert!(matches!(error, TokenError::Parse { .. }));
    }

    let signed = load_mutated_repository_tokens(|value| {
        value["typography"]["styles"]["title"]["letterSpacingMilliPx"] = serde_json::json!(-1);
    })
    .expect("negative integral letter spacing is part of the contract");
    assert_eq!(
        signed.typography.styles["title"].letter_spacing_milli_px,
        -1
    );
}

// Requirements: UI-008
//   Closed renderer-neutral vocabularies reject cursor values that the audited contract does not define
// Evidence: token_loader_rejects_unknown_closed_cursor_enum_value
#[test]
fn token_loader_rejects_unknown_closed_cursor_enum_value() {
    let error = load_mutated_repository_tokens(|value| {
        value["cursors"]["roles"]["action"] = serde_json::Value::String("grab".to_owned());
    })
    .expect_err("unknown cursor value must be refused");
    assert!(matches!(error, TokenError::Parse { .. }));
    assert!(error.to_string().contains("grab"));
}

// Requirements: UI-008
//   Unknown nested typography fields fail closed so a misspelled or renderer-private override cannot silently escape the shared contract
// Evidence: token_loader_rejects_unknown_nested_typography_field
#[test]
fn token_loader_rejects_unknown_nested_typography_field() {
    let error = load_mutated_repository_tokens(|value| {
        value["typography"]["flows"]["single-line"]
            .as_object_mut()
            .expect("single-line flow is an object")
            .insert("rendererOverride".to_owned(), serde_json::json!(true));
    })
    .expect_err("unknown nested field must be refused");
    assert!(matches!(error, TokenError::Parse { .. }));
    assert!(error.to_string().contains("rendererOverride"));
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
