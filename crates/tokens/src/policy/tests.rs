//! Tests for the accessibility policy.
//!
//! These pin the declarations independently of the token file they judge.

use crate::policy::{
    COLOR_SEPARATION_FLOOR, DEFAULT_THEME, REQUIRED_CONTRAST_PAIRINGS, REQUIRED_CURSOR_ROLES,
    REQUIRED_DISTINCT_PAIRS, REQUIRED_ENTITY_ROLES, REQUIRED_FONT_FAMILIES,
    REQUIRED_FOUNDATIONAL_COLOR_ROLES, REQUIRED_LABEL_IDS, REQUIRED_LAYOUT,
    REQUIRED_MEASUREMENT_UNITS, REQUIRED_PROGRESS_ROLES, REQUIRED_RADIUS_PX,
    REQUIRED_SELECTION_CONTRAST, REQUIRED_SEMANTIC_LABELS, REQUIRED_SEVERITY_ROLES,
    REQUIRED_SPACING_PX, REQUIRED_SPEC_VERSION, REQUIRED_STROKE_PX, REQUIRED_TEXT_CARET_WIDTH_PX,
    REQUIRED_TEXT_FLOWS, REQUIRED_TEXT_INPUT, REQUIRED_THEME_LABELS, REQUIRED_THEME_SIGNALS,
    REQUIRED_THEMES, REQUIRED_TOKEN_SET_VERSION, REQUIRED_TYPOGRAPHY_STYLES, WCAG_AA_NON_TEXT,
    WCAG_AA_NORMAL_TEXT, carries_meaning, required_color_roles, required_meaning_bearing_roles,
    threshold_for, vocabulary_requirement,
};

// Requirements: UI-008
//   The independent policy pins normal-text and non-text contrast to the published WCAG 2.2 AA floors rather than values supplied by the palette under audit
// Evidence: the_wcag_floors_are_the_published_values
#[test]
#[expect(
    clippy::assertions_on_constants,
    reason = "the constants are the subject: changing one must break a test that names its external criterion"
)]
fn the_wcag_floors_are_the_published_values() {
    assert!((WCAG_AA_NORMAL_TEXT - 4.5).abs() < f64::EPSILON);
    assert!((WCAG_AA_NON_TEXT - 3.0).abs() < f64::EPSILON);
    assert!(WCAG_AA_NORMAL_TEXT > WCAG_AA_NON_TEXT);
}

// Requirements: UI-007
//   The colour-separation smell-test floor is an explicit project decision above the collapse that prompted it, not a value the palette can lower
// Evidence: the_colour_separation_floor_is_recorded_as_a_project_decision
#[test]
#[expect(
    clippy::assertions_on_constants,
    reason = "the constant is the subject and cannot be lowerable without breaking its policy test"
)]
fn the_colour_separation_floor_is_recorded_as_a_project_decision() {
    assert!((COLOR_SEPARATION_FLOOR - 12.0).abs() < f64::EPSILON);
    assert!(COLOR_SEPARATION_FLOOR > 10.1);
}

// Requirements: UI-008
//   Pairing kinds map only to their exact external contrast criteria and unknown spellings fail closed instead of selecting a default
// Evidence: threshold_names_map_to_the_criterion_they_mean_and_nothing_else
#[test]
fn threshold_names_map_to_the_criterion_they_mean_and_nothing_else() {
    assert_eq!(threshold_for("text"), Some(WCAG_AA_NORMAL_TEXT));
    assert_eq!(threshold_for("ui"), Some(WCAG_AA_NON_TEXT));
    for unknown in ["", "TEXT", "large-text", "aaa", "whatever"] {
        assert_eq!(threshold_for(unknown), None);
    }
}

// Requirements: UI-003, UI-011, PLAN-004
//   The independent vocabulary pins all eight storage entities, all five risk severities, and all eight progress states to their owning requirements
// Evidence: the_vocabulary_matches_the_specification_it_claims_to_come_from
#[test]
fn the_vocabulary_matches_the_specification_it_claims_to_come_from() {
    assert_eq!(
        REQUIRED_ENTITY_ROLES,
        [
            "entity.device",
            "entity.partition",
            "entity.container",
            "entity.volume",
            "entity.encryption",
            "entity.filesystem",
            "entity.mount",
            "entity.freeSpace",
        ]
    );
    assert_eq!(
        REQUIRED_SEVERITY_ROLES,
        [
            "severity.informational",
            "severity.reversible",
            "severity.disruptive",
            "severity.dataMoving",
            "severity.destructive",
        ]
    );
    assert_eq!(
        REQUIRED_PROGRESS_ROLES,
        [
            "progress.planning",
            "progress.awaitingAuthorization",
            "progress.executing",
            "progress.verifying",
            "progress.rebootPending",
            "progress.recovering",
            "progress.failed",
            "progress.complete",
        ]
    );

    for role in REQUIRED_ENTITY_ROLES {
        assert_eq!(vocabulary_requirement(role), Some("UI-003"));
    }
    for role in REQUIRED_SEVERITY_ROLES {
        assert_eq!(vocabulary_requirement(role), Some("PLAN-004"));
    }
    for role in REQUIRED_PROGRESS_ROLES {
        assert_eq!(vocabulary_requirement(role), Some("UI-011"));
    }
}

// Requirements: UI-003, UI-007, UI-011, PLAN-004
//   The required semantic roster contains no duplicate and agrees with the predicate that demands redundant non-colour channels
// Evidence: the_roster_has_no_duplicates_and_every_entry_carries_meaning
#[test]
fn the_roster_has_no_duplicates_and_every_entry_carries_meaning() {
    let roles: Vec<&str> = required_meaning_bearing_roles().collect();
    let unique: std::collections::BTreeSet<&&str> = roles.iter().collect();
    assert_eq!(roles.len(), 21);
    assert_eq!(roles.len(), unique.len());
    for role in &roles {
        assert!(carries_meaning(role));
    }
}

// Requirements: UI-007
//   Surfaces, text, borders, and focus decoration remain outside the meaning-bearing roster so redundant-channel rules target identity and state rather than visual primitives
// Evidence: surfaces_and_text_do_not_carry_meaning_under_ui_007
#[test]
fn surfaces_and_text_do_not_carry_meaning_under_ui_007() {
    for role in [
        "surface.sunken",
        "surface.base",
        "surface.raised",
        "surface.overlay",
        "text.primary",
        "text.secondary",
        "text.muted",
        "border.default",
        "border.strong",
        "focus.ring",
    ] {
        assert!(!carries_meaning(role));
    }
}

// Requirements: UI-008
//   Both the specification vocabulary version and breaking token-contract version are pinned literally outside the file being audited
// Evidence: both_versions_are_pinned_outside_the_token_file
#[test]
fn both_versions_are_pinned_outside_the_token_file() {
    assert_eq!(REQUIRED_SPEC_VERSION, "4.0.0");
    assert_eq!(REQUIRED_TOKEN_SET_VERSION, "2.0.0");
    assert_ne!(REQUIRED_TOKEN_SET_VERSION, REQUIRED_SPEC_VERSION);
}

// Requirements: UI-001, UI-013
//   The independent policy pins the dark default, separate high-contrast theme, system selection label, and exact system-colour-scheme mapping
// Evidence: theme_roster_labels_and_signals_are_exact
#[test]
fn theme_roster_labels_and_signals_are_exact() {
    assert_eq!(REQUIRED_THEMES, ["dark", "high-contrast", "light"]);
    assert_eq!(DEFAULT_THEME, "dark");
    assert_eq!(
        REQUIRED_THEME_LABELS.map(|binding| (binding.theme, binding.label_id)),
        [
            ("dark", "theme.dark"),
            ("high-contrast", "theme.highContrast"),
            ("light", "theme.light"),
        ]
    );
    assert_eq!(REQUIRED_THEME_SIGNALS.default_theme, "dark");
    assert_eq!(
        REQUIRED_THEME_SIGNALS.system_selection_label_id,
        "theme.system"
    );
    assert_eq!(REQUIRED_THEME_SIGNALS.unknown_color_scheme_theme, "dark");
    assert_eq!(REQUIRED_THEME_SIGNALS.dark_color_scheme_theme, "dark");
    assert_eq!(REQUIRED_THEME_SIGNALS.light_color_scheme_theme, "light");
    assert_eq!(REQUIRED_THEME_SIGNALS.high_contrast_theme, "high-contrast");
}

// Requirements: UI-008
//   Logical-pixel and permille suffixes have exact renderer-neutral meanings outside the canonical file they qualify
// Evidence: measurement_unit_policy_is_exact
#[test]
fn measurement_unit_policy_is_exact() {
    assert_eq!(REQUIRED_MEASUREMENT_UNITS.px, "logical-pixel");
    assert_eq!(
        REQUIRED_MEASUREMENT_UNITS.letter_spacing_milli_px,
        "thousandths-of-logical-pixel"
    );
    assert_eq!(
        REQUIRED_MEASUREMENT_UNITS.line_height_permille,
        "thousandths-of-font-size"
    );
}

// Requirements: UI-008
//   Every current oriented foreground/background use and its text-or-UI contrast class is pinned independently of the token file
// Evidence: contrast_pairing_policy_is_exact
#[test]
fn contrast_pairing_policy_is_exact() {
    assert_eq!(REQUIRED_CONTRAST_PAIRINGS.len(), 35);
    assert_eq!(
        REQUIRED_CONTRAST_PAIRINGS.map(|pairing| (
            pairing.foreground,
            pairing.background,
            pairing.kind
        )),
        [
            ("text.primary", "surface.base", "text"),
            ("text.primary", "surface.raised", "text"),
            ("text.primary", "surface.overlay", "text"),
            ("text.primary", "surface.sunken", "text"),
            ("text.secondary", "surface.base", "text"),
            ("text.secondary", "surface.raised", "text"),
            ("text.muted", "surface.base", "text"),
            ("text.muted", "surface.raised", "text"),
            ("border.default", "surface.base", "ui"),
            ("border.strong", "surface.base", "ui"),
            ("focus.ring", "surface.base", "ui"),
            ("focus.ring", "surface.raised", "ui"),
            ("surface.sunken", "focus.ring", "text"),
            ("entity.device", "surface.base", "text"),
            ("entity.partition", "surface.base", "text"),
            ("entity.container", "surface.base", "text"),
            ("entity.volume", "surface.base", "text"),
            ("entity.encryption", "surface.base", "text"),
            ("entity.filesystem", "surface.base", "text"),
            ("entity.mount", "surface.base", "text"),
            ("entity.freeSpace", "surface.base", "ui"),
            ("severity.informational", "surface.base", "text"),
            ("severity.reversible", "surface.base", "text"),
            ("severity.disruptive", "surface.base", "text"),
            ("severity.dataMoving", "surface.base", "text"),
            ("severity.destructive", "surface.base", "text"),
            ("severity.destructive", "surface.raised", "text"),
            ("progress.planning", "surface.base", "text"),
            ("progress.awaitingAuthorization", "surface.base", "text"),
            ("progress.executing", "surface.base", "text"),
            ("progress.verifying", "surface.base", "text"),
            ("progress.rebootPending", "surface.base", "text"),
            ("progress.recovering", "surface.base", "text"),
            ("progress.failed", "surface.base", "text"),
            ("progress.complete", "surface.base", "text"),
        ]
    );
}

// Requirements: UI-003, UI-007, UI-008, UI-011, PLAN-004
//   The full visual roster is exactly ten foundational roles followed by the 21 independently pinned semantic roles, with no duplicate
// Evidence: the_full_visual_roster_is_exactly_thirty_one_roles
#[test]
fn the_full_visual_roster_is_exactly_thirty_one_roles() {
    assert_eq!(
        REQUIRED_FOUNDATIONAL_COLOR_ROLES,
        [
            "surface.sunken",
            "surface.base",
            "surface.raised",
            "surface.overlay",
            "text.primary",
            "text.secondary",
            "text.muted",
            "border.default",
            "border.strong",
            "focus.ring",
        ]
    );
    assert_eq!(
        required_color_roles().collect::<Vec<_>>(),
        [
            "surface.sunken",
            "surface.base",
            "surface.raised",
            "surface.overlay",
            "text.primary",
            "text.secondary",
            "text.muted",
            "border.default",
            "border.strong",
            "focus.ring",
            "entity.device",
            "entity.partition",
            "entity.container",
            "entity.volume",
            "entity.encryption",
            "entity.filesystem",
            "entity.mount",
            "entity.freeSpace",
            "severity.informational",
            "severity.reversible",
            "severity.disruptive",
            "severity.dataMoving",
            "severity.destructive",
            "progress.planning",
            "progress.awaitingAuthorization",
            "progress.executing",
            "progress.verifying",
            "progress.rebootPending",
            "progress.recovering",
            "progress.failed",
            "progress.complete",
        ]
    );
    let roles = required_color_roles().collect::<Vec<_>>();
    assert_eq!(roles.len(), 31);
    assert_eq!(
        roles
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        31
    );
}

// Requirements: UI-003, UI-007, UI-011, UI-013, PLAN-004
//   Every semantic role maps to its exact stable meaning label and the complete 25-label roster includes concrete themes plus system selection
// Evidence: label_ids_are_stable_literal_contract_values
#[test]
fn label_ids_are_stable_literal_contract_values() {
    assert_eq!(
        REQUIRED_SEMANTIC_LABELS.map(|binding| (binding.role, binding.label_id)),
        [
            ("entity.device", "meaning.entity.device"),
            ("entity.partition", "meaning.entity.partition"),
            ("entity.container", "meaning.entity.container"),
            ("entity.volume", "meaning.entity.volume"),
            ("entity.encryption", "meaning.entity.encryption"),
            ("entity.filesystem", "meaning.entity.filesystem"),
            ("entity.mount", "meaning.entity.mount"),
            ("entity.freeSpace", "meaning.entity.freeSpace"),
            ("severity.informational", "meaning.severity.informational",),
            ("severity.reversible", "meaning.severity.reversible"),
            ("severity.disruptive", "meaning.severity.disruptive"),
            ("severity.dataMoving", "meaning.severity.dataMoving"),
            ("severity.destructive", "meaning.severity.destructive"),
            ("progress.planning", "meaning.progress.planning"),
            (
                "progress.awaitingAuthorization",
                "meaning.progress.awaitingAuthorization",
            ),
            ("progress.executing", "meaning.progress.executing"),
            ("progress.verifying", "meaning.progress.verifying"),
            ("progress.rebootPending", "meaning.progress.rebootPending"),
            ("progress.recovering", "meaning.progress.recovering"),
            ("progress.failed", "meaning.progress.failed"),
            ("progress.complete", "meaning.progress.complete"),
        ]
    );
    assert_eq!(
        REQUIRED_LABEL_IDS,
        [
            "theme.dark",
            "theme.highContrast",
            "theme.light",
            "theme.system",
            "meaning.entity.device",
            "meaning.entity.partition",
            "meaning.entity.container",
            "meaning.entity.volume",
            "meaning.entity.encryption",
            "meaning.entity.filesystem",
            "meaning.entity.mount",
            "meaning.entity.freeSpace",
            "meaning.severity.informational",
            "meaning.severity.reversible",
            "meaning.severity.disruptive",
            "meaning.severity.dataMoving",
            "meaning.severity.destructive",
            "meaning.progress.planning",
            "meaning.progress.awaitingAuthorization",
            "meaning.progress.executing",
            "meaning.progress.verifying",
            "meaning.progress.rebootPending",
            "meaning.progress.recovering",
            "meaning.progress.failed",
            "meaning.progress.complete",
        ]
    );
    assert_eq!(
        REQUIRED_LABEL_IDS
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        25
    );
}

// Requirements: UI-008
//   The platform font strategy and all seven typography styles are exact independent policy values rather than schema-owned configuration
// Evidence: typography_policy_is_exact
#[test]
fn typography_policy_is_exact() {
    assert_eq!(
        REQUIRED_FONT_FAMILIES.map(|family| (family.id, family.strategy)),
        [("platform-ui", "platform-default")]
    );
    assert_eq!(
        REQUIRED_TYPOGRAPHY_STYLES.map(|style| (
            style.id,
            style.family,
            style.size_px,
            style.weight,
            style.italic,
            style.letter_spacing_milli_px,
            style.line_height_permille,
        )),
        [
            ("body", "platform-ui", 16, 400, false, 0, 1_500),
            ("body-small", "platform-ui", 14, 400, false, 0, 1_500),
            ("caption", "platform-ui", 12, 400, false, 0, 1_500),
            ("heading", "platform-ui", 16, 700, false, 0, 1_250),
            ("title", "platform-ui", 18, 700, false, -360, 1_200),
            ("eyebrow", "platform-ui", 11, 700, false, 1_000, 1_200),
            ("exact-value", "platform-ui", 12, 400, false, 0, 1_500),
        ]
    );
}

// Requirements: UI-008
//   Text-flow enum spellings, text-input style and flow, and selected-text contrast roles are pinned independently and literally
// Evidence: text_flow_and_input_policy_are_exact
#[test]
fn text_flow_and_input_policy_are_exact() {
    assert_eq!(
        REQUIRED_TEXT_FLOWS.map(|flow| (
            flow.id,
            flow.wrap,
            flow.overflow,
            flow.horizontal_alignment,
            flow.vertical_alignment,
        )),
        [
            ("single-line", "no-wrap", "elide", "left", "center"),
            ("multi-line", "word-wrap", "clip", "left", "top"),
        ]
    );
    assert_eq!(REQUIRED_TEXT_INPUT.style, "body");
    assert_eq!(REQUIRED_TEXT_INPUT.flow, "single-line");
    assert_eq!(
        REQUIRED_TEXT_INPUT.selection_foreground_role,
        "surface.sunken"
    );
    assert_eq!(REQUIRED_TEXT_INPUT.selection_background_role, "focus.ring");
    assert_eq!(
        REQUIRED_SELECTION_CONTRAST.foreground_role,
        "surface.sunken"
    );
    assert_eq!(REQUIRED_SELECTION_CONTRAST.background_role, "focus.ring");
    assert_eq!(REQUIRED_SELECTION_CONTRAST.threshold, "text");
}

// Requirements: UI-008
//   Spacing, radius, stroke, focus-offset, padding, layout-spacing, and minimum-target values are exact independent policy declarations
// Evidence: layout_policy_is_exact
#[test]
fn layout_policy_is_exact() {
    assert_eq!(
        REQUIRED_SPACING_PX.map(|token| (token.id, token.value_px)),
        [
            ("none", 0),
            ("xs", 4),
            ("sm", 8),
            ("md", 12),
            ("lg", 16),
            ("xl", 24),
            ("xxl", 32),
        ]
    );
    assert_eq!(
        REQUIRED_RADIUS_PX.map(|token| (token.id, token.value_px)),
        [("none", 0), ("sm", 4), ("md", 8), ("lg", 14), ("pill", 999)]
    );
    assert_eq!(
        REQUIRED_STROKE_PX.map(|token| (token.id, token.value_px)),
        [("hairline", 1), ("strong", 2), ("focus", 3)]
    );
    assert_eq!(REQUIRED_LAYOUT.default_layout_padding, "md");
    assert_eq!(REQUIRED_LAYOUT.default_layout_spacing, "sm");
    assert_eq!(REQUIRED_LAYOUT.focus_ring_offset_px, 3);
    assert_eq!(REQUIRED_LAYOUT.minimum_target_size_px, 44);
}

// Requirements: UI-008
//   Text-caret width and all four semantic cursor mappings are pinned to their exact closed spellings
// Evidence: cursor_policy_is_exact
#[test]
fn cursor_policy_is_exact() {
    assert_eq!(REQUIRED_TEXT_CARET_WIDTH_PX, 2);
    assert_eq!(
        REQUIRED_CURSOR_ROLES.map(|binding| (binding.role, binding.cursor)),
        [
            ("default", "default"),
            ("action", "pointer"),
            ("disabled", "not-allowed"),
            ("text", "text"),
        ]
    );
}

// Requirements: UI-007, UI-011, PLAN-004
//   Risk and progress role pairs whose confusion would mislead remain an exact independent roster
// Evidence: required_distinct_pairs_are_exact
#[test]
fn required_distinct_pairs_are_exact() {
    assert_eq!(
        REQUIRED_DISTINCT_PAIRS,
        [
            ("severity.reversible", "severity.destructive"),
            ("severity.reversible", "severity.dataMoving"),
            ("severity.disruptive", "severity.destructive"),
            ("severity.informational", "severity.destructive"),
            ("progress.complete", "progress.failed"),
            ("progress.executing", "progress.failed"),
            ("progress.complete", "progress.recovering"),
        ]
    );
}
