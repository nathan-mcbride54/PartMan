//! Tests for the accessibility harness.
//!
//! The repository's own token set passing proves very little on its own: it is
//! the input the checks were written alongside, so it passes by construction.
//! What the mutation table below establishes is that each check is *capable* of
//! failing, and names the requirement it would fail under. WP-020 learned this
//! the expensive way — a gate in `generate` was load-bearing on nothing, and
//! deleting it kept every test green, because every test fed it the real
//! catalogue.

use crate::audit::audit;
use crate::tokens::{Channels, TokenSet};

fn repository_tokens() -> TokenSet {
    TokenSet::load_repository_tokens().expect("the repository token file loads")
}

#[test]
fn the_repository_token_set_satisfies_every_rule() {
    let report = audit(&repository_tokens());
    assert!(
        report.is_clean(),
        "shipped tokens must pass the harness:\n{}",
        report.summary()
    );
    // A run that evaluated nothing would also report no findings, which is the
    // shape of fake success Section 12 forbids.
    assert!(
        report.checks > 100,
        "expected a substantial number of checks, ran {}",
        report.checks
    );
}

#[test]
fn every_theme_is_audited_not_just_the_default() {
    // If the harness only ever looked at `dark`, a high-contrast theme could
    // fail WCAG unnoticed -- the theme whose entire purpose is contrast.
    let mut set = repository_tokens();
    set.themes
        .get_mut("high-contrast")
        .expect("high-contrast theme")
        .colors
        .insert("text.primary".to_owned(), "#0A0A0A".to_owned());
    let report = audit(&set);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.detail.contains("high-contrast")),
        "a broken high-contrast theme must be reported:\n{}",
        report.summary()
    );
}

/// One way to defeat a check, and the requirement that must notice.
struct Mutation {
    name: &'static str,
    requirement: &'static str,
    apply: fn(&mut TokenSet),
}

/// Each row: a mutation, and the requirement whose check must reject it.
///
/// Every entry was confirmed to fail *before* being added, by deleting the
/// check it targets and observing this table go red. A mutation the harness
/// accepts is a check that does not exist.
#[expect(
    clippy::too_many_lines,
    reason = "an exhaustive data table, not control flow. Splitting it across \
              functions would hide that it is one list, which is the property \
              that makes it reviewable."
)]
fn mutations() -> Vec<Mutation> {
    vec![
        Mutation {
            name: "body text drops below AA against its own surface",
            requirement: "UI-008",
            apply: |set| {
                set.themes
                    .get_mut("dark")
                    .expect("dark")
                    .colors
                    .insert("text.secondary".to_owned(), "#3A3F47".to_owned());
            },
        },
        Mutation {
            name: "a UI-component colour falls under 3:1",
            requirement: "UI-008",
            apply: |set| {
                set.themes
                    .get_mut("dark")
                    .expect("dark")
                    .colors
                    .insert("border.default".to_owned(), "#1A1C21".to_owned());
            },
        },
        Mutation {
            name: "the high-contrast theme loses a role the dark theme defines",
            requirement: "UI-001",
            apply: |set| {
                set.themes
                    .get_mut("high-contrast")
                    .expect("high-contrast")
                    .colors
                    .remove("severity.destructive");
            },
        },
        Mutation {
            name: "the accessible high-contrast theme is removed entirely",
            requirement: "UI-001",
            apply: |set| {
                set.themes.remove("high-contrast");
            },
        },
        Mutation {
            name: "a risk-bearing role loses its non-colour channel",
            requirement: "UI-007",
            apply: |set| {
                set.non_color_channels.roles.remove("severity.destructive");
            },
        },
        Mutation {
            name: "a role keeps its entry but empties the visible label",
            requirement: "UI-007",
            apply: |set| {
                set.non_color_channels.roles.insert(
                    "severity.destructive".to_owned(),
                    Channels {
                        icon: "triangle-exclamation".to_owned(),
                        label: "   ".to_owned(),
                        shape: "triangle".to_owned(),
                    },
                );
            },
        },
        Mutation {
            name: "two roles share an icon and a label, making redundancy non-redundant",
            requirement: "UI-007",
            apply: |set| {
                let reversible = set
                    .non_color_channels
                    .roles
                    .get("severity.reversible")
                    .expect("severity.reversible")
                    .clone();
                set.non_color_channels
                    .roles
                    .insert("severity.destructive".to_owned(), reversible);
            },
        },
        Mutation {
            name: "channels are declared for a role that does not exist",
            requirement: "UI-007",
            apply: |set| {
                set.non_color_channels.roles.insert(
                    "severity.catastrophic".to_owned(),
                    Channels {
                        icon: "skull".to_owned(),
                        label: "Catastrophic".to_owned(),
                        shape: "triangle".to_owned(),
                    },
                );
            },
        },
        Mutation {
            name: "'reversible' and 'destructive' become the same colour",
            requirement: "UI-007",
            apply: |set| {
                let destructive = set
                    .themes
                    .get("dark")
                    .expect("dark")
                    .colors
                    .get("severity.destructive")
                    .expect("severity.destructive")
                    .clone();
                set.themes
                    .get_mut("dark")
                    .expect("dark")
                    .colors
                    .insert("severity.reversible".to_owned(), destructive);
            },
        },
        Mutation {
            name: "'complete' and 'failed' converge under colour-vision deficiency",
            requirement: "UI-007",
            apply: |set| {
                // Distinct in sRGB, but a red/green pair chosen so protanopia
                // and deuteranopia collapse them. This is the mutation that
                // makes the simulation worth running at all: no contrast check
                // and no channel check would notice it.
                let dark = set.themes.get_mut("dark").expect("dark");
                dark.colors
                    .insert("progress.complete".to_owned(), "#4CAF50".to_owned());
                dark.colors
                    .insert("progress.failed".to_owned(), "#B8860B".to_owned());
            },
        },
        Mutation {
            name: "a pairing names a threshold that does not exist",
            requirement: "UI-008",
            apply: |set| {
                set.contrast_rules.pairings[0].kind = "whatever".to_owned();
            },
        },
        Mutation {
            name: "a pairing names a role no theme defines",
            requirement: "UI-008",
            apply: |set| {
                set.contrast_rules.pairings[0].foreground = "text.invented".to_owned();
            },
        },
    ]
}

#[test]
fn every_check_rejects_a_mutation_that_defeats_it() {
    for mutation in mutations() {
        let mut set = repository_tokens();
        (mutation.apply)(&mut set);
        let report = audit(&set);
        assert!(
            !report.is_clean(),
            "mutation {:?} was accepted; the {} check does not exist",
            mutation.name,
            mutation.requirement
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.requirement == mutation.requirement),
            "mutation {:?} was caught, but not by {}; got {:?}",
            mutation.name,
            mutation.requirement,
            report
                .findings
                .iter()
                .map(|finding| finding.requirement)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn the_summary_reports_the_tightest_pairing_it_saw() {
    // Reporting how close a passing set came to failing is the difference
    // between "green" and "green with 0.03 to spare". The latter is actionable.
    let report = audit(&repository_tokens());
    let (ratio, described) = report
        .tightest_contrast
        .as_ref()
        .expect("the audit measured at least one pairing");
    assert!(*ratio >= 3.0, "tightest pairing {described} is {ratio}");
    assert!(!described.is_empty());

    let (difference, _) = report
        .closest_separation
        .as_ref()
        .expect("the audit measured at least one colour-vision pair");
    assert!(*difference > 0.0);
}

#[test]
fn the_caveats_are_carried_into_the_output() {
    // A green harness that does not say what it failed to check invites being
    // read as an accessibility guarantee. UI-008 is far wider than contrast.
    let caveats = crate::audit::Report::caveats();
    assert!(caveats.len() >= 3);
    assert!(
        caveats
            .iter()
            .any(|caveat| caveat.contains("screen-reader") || caveat.contains("zoom")),
        "the caveats must name the parts of UI-008 this harness cannot see"
    );
}
