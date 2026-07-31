//! Tests for the accessibility policy.
//!
//! These check that the policy says what the specification and WCAG say, not
//! that it matches the token file — the token file is the thing being judged.

use crate::policy::{
    COLOR_SEPARATION_FLOOR, DEFAULT_THEME, REQUIRED_ENTITY_ROLES, REQUIRED_PROGRESS_ROLES,
    REQUIRED_SEVERITY_ROLES, REQUIRED_THEMES, WCAG_AA_NON_TEXT, WCAG_AA_NORMAL_TEXT,
    carries_meaning, required_meaning_bearing_roles, threshold_for, vocabulary_requirement,
};

// Requirements: UI-008
//   The independent policy pins normal-text and non-text contrast to the published WCAG 2.2 AA floors rather than values supplied by the palette under audit
// Evidence: the_wcag_floors_are_the_published_values
#[test]
#[expect(
    clippy::assertions_on_constants,
    reason = "asserting on the constants is the entire point: these tests pin them to \
              externally published values, so changing a constant must break a test that \
              names the success criterion being changed"
)]
fn the_wcag_floors_are_the_published_values() {
    // Written out rather than referenced so that changing either constant has
    // to change a test that names the success criterion it would be breaking.
    assert!(
        (WCAG_AA_NORMAL_TEXT - 4.5).abs() < f64::EPSILON,
        "SC 1.4.3 Contrast (Minimum) at AA is 4.5:1 for normal text"
    );
    assert!(
        (WCAG_AA_NON_TEXT - 3.0).abs() < f64::EPSILON,
        "SC 1.4.11 Non-text Contrast at AA is 3:1"
    );
    assert!(
        WCAG_AA_NORMAL_TEXT > WCAG_AA_NON_TEXT,
        "text is held to a stricter floor than interface components"
    );
}

// Requirements: UI-007
//   The colour-separation smell-test floor is an explicit project decision above the collapse that prompted it, not a value the palette can lower
// Evidence: the_colour_separation_floor_is_recorded_as_a_project_decision
#[test]
#[expect(
    clippy::assertions_on_constants,
    reason = "as above: the constant is the subject under test, and it must not be \
              lowerable without breaking a test that states why the floor exists"
)]
fn the_colour_separation_floor_is_recorded_as_a_project_decision() {
    // Not a standard. Fixed here so that lowering it is a reviewed code change
    // rather than a palette edit, which is exactly how the audit defeated the
    // previous arrangement.
    assert!((COLOR_SEPARATION_FLOOR - 12.0).abs() < f64::EPSILON);
    assert!(
        COLOR_SEPARATION_FLOOR > 10.1,
        "the floor must sit above the delta-E 10.1 collapse WP-030 increment 1 found between \
         severity.reversible and severity.destructive under deuteranopia"
    );
}

// Requirements: UI-008
//   Pairing kinds map only to their exact external contrast criteria and unknown spellings fail closed instead of selecting a default
// Evidence: threshold_names_map_to_the_criterion_they_mean_and_nothing_else
#[test]
fn threshold_names_map_to_the_criterion_they_mean_and_nothing_else() {
    assert_eq!(threshold_for("text"), Some(WCAG_AA_NORMAL_TEXT));
    assert_eq!(threshold_for("ui"), Some(WCAG_AA_NON_TEXT));
    for unknown in ["", "TEXT", "large-text", "aaa", "whatever"] {
        assert_eq!(
            threshold_for(unknown),
            None,
            "{unknown:?} must be refused rather than defaulted"
        );
    }
}

// Requirements: UI-003, UI-011, PLAN-004
//   The independent vocabulary pins all eight storage entities, all five risk severities, and all eight progress states to their owning requirements
// Evidence: the_vocabulary_matches_the_specification_it_claims_to_come_from
#[test]
fn the_vocabulary_matches_the_specification_it_claims_to_come_from() {
    // UI-003 names eight things to distinguish; PLAN-004 defines exactly five
    // ordinal severities; UI-011 names eight progress states. Pin the names,
    // not just the lengths and prefixes: otherwise a coordinated typo in the
    // policy and token JSON could repeat the self-consistency failure this
    // independent policy exists to prevent.
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
        ],
        "UI-003 entity types"
    );
    assert_eq!(
        REQUIRED_SEVERITY_ROLES,
        [
            "severity.informational",
            "severity.reversible",
            "severity.disruptive",
            "severity.dataMoving",
            "severity.destructive",
        ],
        "PLAN-004 severities 0..=4"
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
        ],
        "UI-011 progress states"
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
    assert_eq!(
        roles.len(),
        unique.len(),
        "a duplicated role would be checked twice and removed once"
    );
    for role in &roles {
        assert!(
            carries_meaning(role),
            "{role} is in the roster but the predicate does not recognise it, so the roster \
             check and the UI-007 channel check would disagree about it"
        );
    }
}

// Requirements: UI-007
//   Surfaces, text, borders, and focus decoration remain outside the meaning-bearing roster so redundant-channel rules target identity and state rather than visual primitives
// Evidence: surfaces_and_text_do_not_carry_meaning_under_ui_007
#[test]
fn surfaces_and_text_do_not_carry_meaning_under_ui_007() {
    // If these were "meaningful" the harness would demand an icon for a
    // background colour, and a rule that produces noise is a rule people learn
    // to ignore.
    for role in [
        "surface.base",
        "surface.raised",
        "text.primary",
        "text.muted",
        "border.default",
        "focus.ring",
    ] {
        assert!(!carries_meaning(role), "{role} should not require an icon");
    }
}

// Requirements: UI-008
//   Both the specification vocabulary version and token-set version are pinned outside the file being audited and cannot be conflated
// Evidence: both_versions_are_pinned_outside_the_token_file
#[test]
fn both_versions_are_pinned_outside_the_token_file() {
    // A version the audited file supplies and nothing compares against is
    // documentation. Both are compared; neither is empty.
    use crate::policy::{REQUIRED_SPEC_VERSION, REQUIRED_TOKEN_SET_VERSION};
    assert!(!REQUIRED_TOKEN_SET_VERSION.is_empty());
    assert!(!REQUIRED_SPEC_VERSION.is_empty());
    assert_ne!(
        REQUIRED_TOKEN_SET_VERSION, REQUIRED_SPEC_VERSION,
        "these version two different things and should not be conflated"
    );
}

// Requirements: UI-001
//   Independent policy requires the default dark theme, accessible high-contrast theme, and system light theme
// Evidence: the_required_themes_include_the_default_and_the_accessible_one
#[test]
fn the_required_themes_include_the_default_and_the_accessible_one() {
    assert!(REQUIRED_THEMES.contains(&DEFAULT_THEME));
    assert!(
        REQUIRED_THEMES.contains(&"high-contrast"),
        "UI-001 requires an accessible high-contrast theme"
    );
    assert!(
        REQUIRED_THEMES.contains(&"light"),
        "UI-001 requires system theme support"
    );
}
