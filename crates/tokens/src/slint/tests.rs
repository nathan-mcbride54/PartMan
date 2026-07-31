//! Tests for deterministic generated Slint token source.

use std::collections::BTreeSet;

use super::{
    GenerationError, encoded_variant, pair_variant, render, repository_generated_slint_path,
    slint_pascal_case,
};
use crate::TokenSet;

fn repository_tokens() -> TokenSet {
    TokenSet::load_repository_tokens().expect("repository token contract loads")
}

fn decode_variant(encoded: &str) -> Vec<u8> {
    let payload = encoded
        .strip_prefix('v')
        .expect("generated variants always carry the v prefix");
    let bytes = payload.as_bytes();
    let mut output = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'z' {
            let digits =
                std::str::from_utf8(&bytes[index + 1..index + 3]).expect("hex escape is ASCII");
            output.push(u8::from_str_radix(digits, 16).expect("hex escape is valid"));
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    output
}

fn generated_enum_variants(output: &str, enum_name: &str) -> BTreeSet<String> {
    let start = format!("export enum {enum_name} {{\n");
    let body = output
        .split_once(&start)
        .unwrap_or_else(|| panic!("missing generated enum {enum_name}"))
        .1
        .split_once("}\n")
        .unwrap_or_else(|| panic!("unterminated generated enum {enum_name}"))
        .0;
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.trim_end_matches(',').to_owned())
        .collect()
}

fn encoded_values<'a>(values: impl IntoIterator<Item = &'a str>) -> BTreeSet<String> {
    values.into_iter().map(encoded_variant).collect()
}

fn assert_encoded_enum<'a>(
    output: &str,
    enum_name: &str,
    values: impl IntoIterator<Item = &'a str>,
) {
    assert_eq!(
        generated_enum_variants(output, enum_name),
        encoded_values(values),
        "generated enum {enum_name} does not exactly match its source roster"
    );
}

fn assert_semantic_enum_rosters(set: &TokenSet, output: &str) {
    assert_encoded_enum(
        output,
        "PartmanMeasurementUnit",
        [
            set.measurement_units.px.as_str(),
            set.measurement_units.letter_spacing_milli_px.as_str(),
            set.measurement_units.line_height_permille.as_str(),
        ],
    );
    assert_encoded_enum(
        output,
        "PartmanThemeId",
        set.themes.keys().map(String::as_str),
    );
    assert_encoded_enum(
        output,
        "PartmanColorRole",
        set.themes["dark"].colors.keys().map(String::as_str),
    );
    assert_encoded_enum(
        output,
        "PartmanSemanticRole",
        set.non_color_channels.roles.keys().map(String::as_str),
    );
    assert_encoded_enum(
        output,
        "PartmanMarkId",
        set.non_color_channels
            .roles
            .values()
            .map(|channels| channels.icon.as_str()),
    );
    assert_encoded_enum(
        output,
        "PartmanShapeId",
        set.non_color_channels
            .roles
            .values()
            .map(|channels| channels.shape.as_str()),
    );
    let label_ids = set
        .themes
        .values()
        .map(|theme| theme.label_id.as_str())
        .chain(std::iter::once(
            set.theme_signals.system_selection_label_id.as_str(),
        ))
        .chain(
            set.non_color_channels
                .roles
                .values()
                .map(|channels| channels.label_id.as_str()),
        );
    assert_encoded_enum(output, "PartmanLabelId", label_ids);
}

fn assert_renderer_enum_rosters(set: &TokenSet, output: &str) {
    assert_encoded_enum(
        output,
        "PartmanFontFamilyId",
        set.typography.families.keys().map(String::as_str),
    );
    assert_encoded_enum(output, "PartmanFontFamilyStrategy", ["platform-default"]);
    assert_encoded_enum(
        output,
        "PartmanTextStyleId",
        set.typography.styles.keys().map(String::as_str),
    );
    assert_encoded_enum(
        output,
        "PartmanTextFlowId",
        set.typography.flows.keys().map(String::as_str),
    );
    assert_encoded_enum(
        output,
        "PartmanSpacingId",
        set.layout.spacing_px.keys().map(String::as_str),
    );
    assert_encoded_enum(
        output,
        "PartmanRadiusId",
        set.layout.radius_px.keys().map(String::as_str),
    );
    assert_encoded_enum(
        output,
        "PartmanStrokeId",
        set.layout.stroke_px.keys().map(String::as_str),
    );
    assert_encoded_enum(
        output,
        "PartmanCursorRoleId",
        set.cursors.roles.keys().map(String::as_str),
    );
}

fn assert_contrast_enum_rosters(set: &TokenSet, output: &str) {
    let expected_text_pairs = set
        .contrast_rules
        .pairings
        .iter()
        .filter(|pair| pair.kind == "text")
        .map(|pair| pair_variant(&pair.foreground, &pair.background))
        .collect::<BTreeSet<_>>();
    let expected_ui_pairs = set
        .contrast_rules
        .pairings
        .iter()
        .filter(|pair| pair.kind == "ui")
        .map(|pair| pair_variant(&pair.foreground, &pair.background))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        generated_enum_variants(output, "PartmanTextContrastPairId"),
        expected_text_pairs
    );
    assert_eq!(
        generated_enum_variants(output, "PartmanUiContrastPairId"),
        expected_ui_pairs
    );
    assert_eq!(
        output.matches("id: PartmanTextContrastPairId.").count(),
        expected_text_pairs.len()
    );
    assert_eq!(
        output.matches("id: PartmanUiContrastPairId.").count(),
        expected_ui_pairs.len()
    );
}

// Requirements: UI-008, UI-013
//   Data-derived Slint identifiers round-trip every UTF-8 byte and remain distinct under Slint 1.17.1 normalization and Rust enum case conversion
// Evidence: identifier_encoding_is_reversible_and_collision_safe
#[test]
fn identifier_encoding_is_reversible_and_collision_safe() {
    let hostile = [
        "foo_bar",
        "foo-bar",
        "HelloHello",
        "hello-hello",
        "9leading",
        "-leading",
        "z",
        "global",
        "line\nfeed",
        "bidi\u{202e}control",
        "é",
        "emoji-💽",
    ];
    let mut slint_names = BTreeSet::new();
    let mut rust_names = BTreeSet::new();
    for source in hostile {
        let encoded = encoded_variant(source);
        assert!(encoded.starts_with('v'));
        assert!(
            encoded.bytes().all(|byte| byte.is_ascii_alphanumeric()),
            "{encoded}"
        );
        assert_eq!(decode_variant(&encoded), source.as_bytes(), "{source:?}");
        assert!(slint_names.insert(encoded.clone()), "{encoded}");
        assert!(rust_names.insert(slint_pascal_case(&encoded)), "{encoded}");
    }
    assert_ne!(encoded_variant("foo_bar"), encoded_variant("foo-bar"));
    assert_ne!(
        slint_pascal_case(&encoded_variant("HelloHello")),
        slint_pascal_case(&encoded_variant("hello-hello"))
    );
}

// Requirements: UI-008
//   Oriented pair identifiers retain both source roles without depending on a delimiter that either encoded atom can contain
// Evidence: pair_identifier_encoding_preserves_orientation
#[test]
fn pair_identifier_encoding_preserves_orientation() {
    let forward = pair_variant("text.primary", "surface.base");
    let reverse = pair_variant("surface.base", "text.primary");
    assert_ne!(forward, reverse);
    let (foreground, background) = forward
        .split_once("-to-")
        .expect("pair identifier has one fixed delimiter");
    assert_eq!(decode_variant(foreground), b"text.primary");
    assert_eq!(decode_variant(background), b"surface.base");
}

// Requirements: UI-001, UI-003, UI-007, UI-008, UI-011, UI-013, PLAN-004
//   Rendering starts only after the independent token audit accepts the complete versioned contract
// Evidence: generation_refuses_a_token_set_that_fails_policy
#[test]
fn generation_refuses_a_token_set_that_fails_policy() {
    let mut set = repository_tokens();
    set.token_set_version = "forged".to_owned();
    let error = render(&set).expect_err("failed independent policy must stop generation");
    assert!(matches!(error, GenerationError::Audit(_)));
}

// Requirements: UI-008, UI-013
//   The generated source is deterministic UTF-8 text with LF-only line endings, no BOM, and one terminal newline
// Evidence: rendering_is_byte_deterministic_and_platform_neutral
#[test]
fn rendering_is_byte_deterministic_and_platform_neutral() {
    let set = repository_tokens();
    let first = render(&set).expect("clean contract renders");
    let second = render(&set).expect("same clean contract renders again");
    assert_eq!(first.as_bytes(), second.as_bytes());
    assert!(!first.starts_with('\u{feff}'));
    assert!(!first.contains('\r'));
    assert!(first.ends_with('\n'));
    assert!(!first.ends_with("\n\n"));
}

// Requirements: UI-008, UI-013
//   The committed Slint token boundary is exactly the deterministic output of the audited repository schema
// Evidence: committed_slint_contract_has_no_generation_drift
#[test]
fn committed_slint_contract_has_no_generation_drift() {
    let expected = render(&repository_tokens()).expect("clean contract renders");
    let path = repository_generated_slint_path();
    let actual = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    assert_eq!(actual.as_bytes(), expected.as_bytes(), "{}", path.display());
}

// Requirements: UI-008
//   Normal-text and UI-only pair rosters have different Slint ID, declaration, result, and resolver types rather than a lower-threshold runtime kind flag
// Evidence: text_and_ui_contrast_boundaries_are_statically_distinct
#[test]
fn text_and_ui_contrast_boundaries_are_statically_distinct() {
    let output = render(&repository_tokens()).expect("clean contract renders");
    assert!(output.contains("export enum PartmanTextContrastPairId"));
    assert!(output.contains("export enum PartmanUiContrastPairId"));
    assert!(output.contains("export struct PartmanTextContrastPair"));
    assert!(output.contains("export struct PartmanUiContrastPair"));
    assert!(output.contains("-> PartmanTextContrastColors"));
    assert!(output.contains("-> PartmanUiContrastColors"));
    assert!(!output.contains("PartmanContrastKind"));
}

// Requirements: UI-008
//   Every generated lookup intended for another global or a future wrapper is explicitly public and pure because Slint 1.17.1 functions default to private
// Evidence: generated_function_api_is_explicitly_public_and_pure
#[test]
fn generated_function_api_is_explicitly_public_and_pure() {
    let set = repository_tokens();
    let output = render(&set).expect("clean contract renders");
    let expected = set.themes["dark"].colors.len() + 3 + 3 + 2;
    assert_eq!(output.matches("public pure function ").count(), expected);
    assert!(output.lines().all(|line| {
        let declaration = line.trim_start();
        !declaration.starts_with("pure function ") && !declaration.starts_with("function ")
    }));
}

// Requirements: UI-001, UI-003, UI-007, UI-008, UI-011, UI-013, PLAN-004
//   The generated contract carries every version-2 token family but no widget palette, style metric, asset, translation, or English display-label channel
// Evidence: generated_contract_is_complete_but_renderer_and_catalogue_neutral
#[test]
fn generated_contract_is_complete_but_renderer_and_catalogue_neutral() {
    let output = render(&repository_tokens()).expect("clean contract renders");
    for declaration in [
        "PartmanMeasurementUnits",
        "PartmanThemeSignals",
        "PartmanSemanticChannels",
        "PartmanFontFamilySpec",
        "PartmanTextStyleSpec",
        "PartmanTextFlowSpec",
        "PartmanTextInputSpec",
        "PartmanLayoutSpec",
        "PartmanCursorSpec",
        "PartmanDistinctRolePair",
        "PartmanRawGeneratedPalette",
        "PartmanGeneratedMetrics",
        "PartmanGeneratedContrast",
    ] {
        assert!(output.contains(declaration), "missing {declaration}");
    }
    for forbidden in [
        "StyleMetrics",
        "std-widgets.slint",
        "@image-url",
        "@font-face",
        "@tr(",
        "import {",
    ] {
        assert!(!output.contains(forbidden), "found forbidden {forbidden}");
    }
    assert!(
        output.match_indices("Palette.").all(|(index, _)| {
            index > 0 && output.as_bytes()[index - 1].is_ascii_alphanumeric()
        }),
        "a bare upstream Palette member access is forbidden"
    );
    let quotes = output.matches('"').count();
    assert_eq!(
        quotes, 4,
        "only the two version metadata strings are allowed"
    );
}

// Requirements: UI-001, UI-003, UI-007, UI-008, UI-011, UI-013, PLAN-004
//   Every generated enum inventory exactly matches its audited source roster, including labels, renderer metrics, cursors, and oriented text-versus-UI contrast IDs
// Evidence: generated_identifier_vocabulary_matches_the_audited_schema
#[test]
fn generated_identifier_vocabulary_matches_the_audited_schema() {
    let set = repository_tokens();
    let output = render(&set).expect("clean contract renders");
    assert_semantic_enum_rosters(&set, &output);
    assert_renderer_enum_rosters(&set, &output);
    assert_contrast_enum_rosters(&set, &output);
}

// Requirements: UI-008
//   Integer token measurements render without binary floating-point conversion or host-dependent formatting
// Evidence: integer_measurements_have_exact_slint_literals
#[test]
fn integer_measurements_have_exact_slint_literals() {
    let output = render(&repository_tokens()).expect("clean contract renders");
    assert!(output.contains("letter-spacing: -0.36px"));
    assert!(output.contains("letter-spacing: 1px"));
    assert!(output.contains("minimum-target-size: 44px"));
    assert!(output.contains("line-height-permille: 1200"));
}
