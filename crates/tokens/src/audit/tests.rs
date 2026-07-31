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

// Requirements: UI-001, UI-003, UI-007, UI-008, UI-011, PLAN-004
//   The shipped token set passes a non-vacuous audit over the required themes, semantic roles, redundant channels, contrast pairings, and risk vocabulary
// Evidence: the_repository_token_set_satisfies_every_rule
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

// Requirements: UI-001, UI-008
//   The harness evaluates every supported theme, including high contrast, instead of proving only that the default dark palette passes
// Evidence: every_theme_is_audited_not_just_the_default
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
        // ------------------------------------------------------------------
        // Everything below was demonstrated as a *live bypass* by the
        // 2026-07-29 project audit. Each one passed the entire Tier-1 gate
        // before the policy was moved out of the audited file.
        // ------------------------------------------------------------------
        Mutation {
            name: "the file lowers the WCAG text floor to 3.0 so a dim colour fits under it",
            requirement: "UI-008",
            apply: |set| {
                // The audit's exact reproduction: this pairing plus a dimmed
                // colour passed all 160 tests, reporting 3.33:1 on normal text.
                set.contrast_rules.thresholds.insert("text".to_owned(), 3.0);
                set.themes
                    .get_mut("light")
                    .expect("light")
                    .colors
                    .insert("text.secondary".to_owned(), "#7F8899".to_owned());
            },
        },
        Mutation {
            name: "the file lowers only the threshold, leaving every colour compliant",
            requirement: "UI-008",
            apply: |set| {
                // Weakening the standard is a finding even when nothing
                // currently violates it, because the next palette edit would
                // land against the lowered bar.
                set.contrast_rules.thresholds.insert("ui".to_owned(), 1.5);
            },
        },
        Mutation {
            name: "the file stops restating a WCAG threshold entirely",
            requirement: "UI-008",
            apply: |set| {
                set.contrast_rules.thresholds.remove("text");
            },
        },
        Mutation {
            name: "the file lowers the colour-separation floor instead of fixing the palette",
            requirement: "UI-007",
            apply: |set| {
                set.color_vision_separation.minimum_delta_e = 1.0;
            },
        },
        Mutation {
            name: "a required entity role is deleted from every theme, pairing and channel table",
            requirement: "UI-003",
            apply: |set| {
                // The audit's second reproduction. Consistent deletion made a
                // coordinated omission indistinguishable from a smaller
                // product: 234 checks became 228 and the gate stayed green.
                for theme in set.themes.values_mut() {
                    theme.colors.remove("entity.container");
                }
                set.contrast_rules.pairings.retain(|pairing| {
                    pairing.foreground != "entity.container"
                        && pairing.background != "entity.container"
                });
                set.non_color_channels.roles.remove("entity.container");
            },
        },
        Mutation {
            name: "a PLAN-004 severity is deleted from the vocabulary",
            requirement: "PLAN-004",
            apply: |set| {
                for theme in set.themes.values_mut() {
                    theme.colors.remove("severity.dataMoving");
                }
                set.contrast_rules.pairings.retain(|pairing| {
                    pairing.foreground != "severity.dataMoving"
                        && pairing.background != "severity.dataMoving"
                });
                set.non_color_channels.roles.remove("severity.dataMoving");
                set.color_vision_separation
                    .must_remain_distinct
                    .retain(|pair| {
                        pair[0] != "severity.dataMoving" && pair[1] != "severity.dataMoving"
                    });
            },
        },
        Mutation {
            name: "a UI-011 progress state is deleted from the vocabulary",
            requirement: "UI-011",
            apply: |set| {
                for theme in set.themes.values_mut() {
                    theme.colors.remove("progress.recovering");
                }
                set.contrast_rules.pairings.retain(|pairing| {
                    pairing.foreground != "progress.recovering"
                        && pairing.background != "progress.recovering"
                });
                set.non_color_channels.roles.remove("progress.recovering");
                set.color_vision_separation
                    .must_remain_distinct
                    .retain(|pair| {
                        pair[0] != "progress.recovering" && pair[1] != "progress.recovering"
                    });
            },
        },
        Mutation {
            name: "a role keeps its colour but is dropped from every contrast pairing",
            requirement: "UI-008",
            apply: |set| {
                // Present in the file, checked by nothing. Before the roster
                // contract this was invisible, because coverage was defined by
                // whatever the pairing list happened to contain.
                set.contrast_rules.pairings.retain(|pairing| {
                    pairing.foreground != "severity.destructive"
                        && pairing.background != "severity.destructive"
                });
            },
        },
        Mutation {
            name: "the reversible/destructive risk pair is quietly removed from the distinct list",
            requirement: "UI-007",
            apply: |set| {
                // The single most important pair in the product: "fully
                // undoable" against "data is intentionally destroyed".
                set.color_vision_separation
                    .must_remain_distinct
                    .retain(|pair| {
                        !(pair.contains(&"severity.reversible".to_owned())
                            && pair.contains(&"severity.destructive".to_owned()))
                    });
            },
        },
        Mutation {
            name: "the whole distinct-pair list is emptied",
            requirement: "UI-007",
            apply: |set| {
                set.color_vision_separation.must_remain_distinct.clear();
            },
        },
        Mutation {
            name: "a meaningful role is invented without going through the specification",
            requirement: "PLAN-004",
            apply: |set| {
                for theme in set.themes.values_mut() {
                    theme
                        .colors
                        .insert("severity.catastrophic".to_owned(), "#FF0000".to_owned());
                }
            },
        },
        Mutation {
            name: "the token set claims a specification version the vocabulary was not derived from",
            requirement: "UI-008",
            apply: |set| {
                set.spec_version = "5.0.0".to_owned();
            },
        },
        Mutation {
            name: "the token set loses its own version",
            requirement: "UI-008",
            apply: |set| {
                set.token_set_version = String::new();
            },
        },
        Mutation {
            // The 2026-07-29 follow-up audit's exact reproduction. The old
            // check only required a non-empty string, so this passed while
            // WP-030 described parsing as "versioned".
            name: "tokenSetVersion is a non-empty string that is not a version",
            requirement: "UI-008",
            apply: |set| {
                set.token_set_version = "not-a-version".to_owned();
            },
        },
        Mutation {
            name: "tokenSetVersion is a plausible but unsupported vocabulary",
            requirement: "UI-008",
            apply: |set| {
                set.token_set_version = "2.0.0".to_owned();
            },
        },
    ]
}

// Requirements: UI-001, UI-003, UI-007, UI-008, UI-011, PLAN-004, Section 12
//   Twenty-seven named hostile mutations prove the static policy checks can fail for theme, vocabulary, channel, contrast, version, and progress-state regressions
// Evidence: every_check_rejects_a_mutation_that_defeats_it
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

// Requirements: UI-007, UI-008
//   A passing audit reports its tightest contrast and colour-vision margins instead of hiding proximity to either policy floor
// Evidence: the_summary_reports_the_tightest_pairing_it_saw
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

// Requirements: UI-008
//   Audit output names the keyboard, screen-reader, zoom, and other rendered behavior that a static token harness does not establish
// Evidence: the_caveats_are_carried_into_the_output
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
