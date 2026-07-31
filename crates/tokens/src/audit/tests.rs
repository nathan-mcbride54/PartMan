//! Mutation tests for the renderer-neutral visual-contract harness.
//!
//! The repository token file passing is necessary but weak evidence because it
//! was authored alongside the checks. Each policy family below therefore has a
//! hostile mutation that must produce a finding owned by the requirement that
//! makes the declaration mandatory.

use crate::audit::audit;
use crate::tokens::{
    Channels, CursorKind, Pairing, TextOverflow, TextWrap, ThemeId, TokenSet, VerticalAlignment,
};

fn repository_tokens() -> TokenSet {
    TokenSet::load_repository_tokens().expect("the repository token file loads")
}

// Requirements: UI-001, UI-003, UI-007, UI-008, UI-011, UI-013, PLAN-004, Section 12
//   The shipped v2 token contract passes a non-vacuous audit over every independently pinned policy family without coupling evidence to a check count
// Evidence: the_repository_token_set_satisfies_every_rule
#[test]
fn the_repository_token_set_satisfies_every_rule() {
    let report = audit(&repository_tokens());
    assert!(
        report.is_clean(),
        "shipped tokens must pass the harness:\n{}",
        report.summary()
    );
    assert!(
        report.checks > 0,
        "a clean audit that evaluated nothing would be the fake success Section 12 forbids"
    );
}

// Requirements: UI-001, UI-008
//   The harness evaluates every supported theme, including high contrast, instead of proving only that the default dark palette passes
// Evidence: every_theme_is_audited_not_just_the_default
#[test]
fn every_theme_is_audited_not_just_the_default() {
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

fn mutation(name: &'static str, requirement: &'static str, apply: fn(&mut TokenSet)) -> Mutation {
    Mutation {
        name,
        requirement,
        apply,
    }
}

fn selected_text_pairing(set: &mut TokenSet) -> &mut Pairing {
    set.contrast_rules
        .pairings
        .iter_mut()
        .find(|pairing| {
            pairing.foreground == "surface.sunken" && pairing.background == "focus.ring"
        })
        .expect("repository tokens declare the selected-text pairing")
}

fn contrast_mutations() -> Vec<Mutation> {
    vec![
        mutation("body text drops below AA", "UI-008", |set| {
            set.themes
                .get_mut("dark")
                .expect("dark")
                .colors
                .insert("text.secondary".to_owned(), "#3A3F47".to_owned());
        }),
        mutation("a UI-component colour falls under 3:1", "UI-008", |set| {
            set.themes
                .get_mut("dark")
                .expect("dark")
                .colors
                .insert("border.default".to_owned(), "#1A1C21".to_owned());
        }),
        mutation("a pairing names an unknown threshold", "UI-008", |set| {
            set.contrast_rules.pairings[0].kind = "whatever".to_owned();
        }),
        mutation("a pairing names an unknown role", "UI-008", |set| {
            set.contrast_rules.pairings[0].foreground = "text.invented".to_owned();
        }),
        mutation("the file lowers the WCAG text floor", "UI-008", |set| {
            set.contrast_rules.thresholds.insert("text".to_owned(), 3.0);
        }),
        mutation("the file lowers the WCAG UI floor", "UI-008", |set| {
            set.contrast_rules.thresholds.insert("ui".to_owned(), 1.5);
        }),
        mutation("the file removes a WCAG threshold", "UI-008", |set| {
            set.contrast_rules.thresholds.remove("text");
        }),
        mutation(
            "normal text is reclassified as a UI component while its colour is dimmed",
            "UI-008",
            |set| {
                for pairing in &mut set.contrast_rules.pairings {
                    if pairing.foreground == "text.secondary" {
                        pairing.kind = "ui".to_owned();
                    }
                }
                set.themes
                    .get_mut("dark")
                    .expect("dark")
                    .colors
                    .insert("text.secondary".to_owned(), "#6B6B6B".to_owned());
            },
        ),
        mutation(
            "a non-selection contrast pairing is duplicated",
            "UI-008",
            |set| {
                let duplicate = set
                    .contrast_rules
                    .pairings
                    .iter()
                    .find(|pairing| {
                        pairing.foreground == "text.primary" && pairing.background == "surface.base"
                    })
                    .expect("canonical primary-text pairing")
                    .clone();
                set.contrast_rules.pairings.push(duplicate);
            },
        ),
        mutation(
            "a non-selection contrast pairing has a conflicting class",
            "UI-008",
            |set| {
                set.contrast_rules.pairings.push(Pairing {
                    foreground: "text.primary".to_owned(),
                    background: "surface.base".to_owned(),
                    kind: "ui".to_owned(),
                });
            },
        ),
    ]
}

fn color_vision_mutations() -> Vec<Mutation> {
    vec![
        mutation(
            "reversible and destructive share a colour",
            "UI-007",
            |set| {
                let destructive = set.themes["dark"].colors["severity.destructive"].clone();
                set.themes
                    .get_mut("dark")
                    .expect("dark")
                    .colors
                    .insert("severity.reversible".to_owned(), destructive);
            },
        ),
        mutation("complete and failed converge under CVD", "UI-007", |set| {
            let dark = set.themes.get_mut("dark").expect("dark");
            dark.colors
                .insert("progress.complete".to_owned(), "#4CAF50".to_owned());
            dark.colors
                .insert("progress.failed".to_owned(), "#B8860B".to_owned());
        }),
        mutation(
            "the file lowers the colour-separation floor",
            "UI-007",
            |set| {
                set.color_vision_separation.minimum_delta_e = 1.0;
            },
        ),
        mutation("the critical risk pair is removed", "UI-007", |set| {
            set.color_vision_separation
                .must_remain_distinct
                .retain(|pair| {
                    !(pair.contains(&"severity.reversible".to_owned())
                        && pair.contains(&"severity.destructive".to_owned()))
                });
        }),
        mutation("the distinct-pair list is emptied", "UI-007", |set| {
            set.color_vision_separation.must_remain_distinct.clear();
        }),
        mutation(
            "an unsupported distinct pair is invented",
            "UI-007",
            |set| {
                set.color_vision_separation.must_remain_distinct.push([
                    "severity.informational".to_owned(),
                    "severity.reversible".to_owned(),
                ]);
            },
        ),
        mutation(
            "a reversed distinct pair duplicates an existing pair",
            "UI-007",
            |set| {
                set.color_vision_separation.must_remain_distinct.push([
                    "severity.destructive".to_owned(),
                    "severity.reversible".to_owned(),
                ]);
            },
        ),
    ]
}

fn measurement_unit_mutations() -> Vec<Mutation> {
    vec![
        mutation("Px stops meaning logical pixels", "UI-008", |set| {
            set.measurement_units.px = "device-pixel".to_owned();
        }),
        mutation(
            "letter-spacing scale loses its logical-pixel base",
            "UI-008",
            |set| {
                set.measurement_units.letter_spacing_milli_px =
                    "thousandths-of-device-pixel".to_owned();
            },
        ),
        mutation("line height loses its font-size base", "UI-008", |set| {
            set.measurement_units.line_height_permille = "thousandths-of-logical-pixel".to_owned();
        }),
    ]
}

fn color_roster_mutations() -> Vec<Mutation> {
    vec![
        mutation(
            "a foundational colour is removed everywhere",
            "UI-008",
            |set| {
                for theme in set.themes.values_mut() {
                    theme.colors.remove("surface.overlay");
                }
                set.contrast_rules.pairings.retain(|pairing| {
                    pairing.foreground != "surface.overlay"
                        && pairing.background != "surface.overlay"
                });
            },
        ),
        mutation(
            "an unsupported foundational colour is invented",
            "UI-008",
            |set| {
                for theme in set.themes.values_mut() {
                    theme
                        .colors
                        .insert("surface.highlight".to_owned(), "#123456".to_owned());
                }
            },
        ),
        mutation("an entity role is removed everywhere", "UI-003", |set| {
            remove_semantic_role(set, "entity.container");
        }),
        mutation("a severity role is removed everywhere", "PLAN-004", |set| {
            remove_semantic_role(set, "severity.dataMoving");
        }),
        mutation("a progress role is removed everywhere", "UI-011", |set| {
            remove_semantic_role(set, "progress.recovering");
        }),
        mutation("an unsupported severity is invented", "PLAN-004", |set| {
            for theme in set.themes.values_mut() {
                theme
                    .colors
                    .insert("severity.catastrophic".to_owned(), "#FF0000".to_owned());
            }
        }),
        mutation(
            "a semantic role loses all contrast coverage",
            "UI-008",
            |set| {
                set.contrast_rules.pairings.retain(|pairing| {
                    pairing.foreground != "severity.destructive"
                        && pairing.background != "severity.destructive"
                });
            },
        ),
    ]
}

fn remove_semantic_role(set: &mut TokenSet, role: &str) {
    for theme in set.themes.values_mut() {
        theme.colors.remove(role);
    }
    set.contrast_rules
        .pairings
        .retain(|pairing| pairing.foreground != role && pairing.background != role);
    set.non_color_channels.roles.remove(role);
    set.color_vision_separation
        .must_remain_distinct
        .retain(|pair| pair[0] != role && pair[1] != role);
}

fn channel_mutations() -> Vec<Mutation> {
    vec![
        mutation("a semantic role loses its channel entry", "UI-007", |set| {
            set.non_color_channels.roles.remove("severity.destructive");
        }),
        mutation("a channel icon is blank", "UI-007", |set| {
            set.non_color_channels
                .roles
                .get_mut("severity.destructive")
                .expect("severity.destructive")
                .icon = "   ".to_owned();
        }),
        mutation("two roles share icon and labelId", "UI-007", |set| {
            let reversible = set.non_color_channels.roles["severity.reversible"].clone();
            set.non_color_channels
                .roles
                .insert("severity.destructive".to_owned(), reversible);
        }),
        mutation("channels name an unsupported role", "UI-007", |set| {
            set.non_color_channels.roles.insert(
                "severity.catastrophic".to_owned(),
                Channels {
                    icon: "skull".to_owned(),
                    label_id: "meaning.severity.catastrophic".to_owned(),
                    shape: "triangle".to_owned(),
                },
            );
        }),
    ]
}

fn theme_mutations() -> Vec<Mutation> {
    vec![
        mutation("the high-contrast theme loses a colour", "UI-001", |set| {
            set.themes
                .get_mut("high-contrast")
                .expect("high-contrast")
                .colors
                .remove("severity.destructive");
        }),
        mutation("the high-contrast theme is removed", "UI-001", |set| {
            set.themes.remove("high-contrast");
        }),
        mutation("an extra theme is invented", "UI-001", |set| {
            let extra = set.themes["dark"].clone();
            set.themes.insert("sepia".to_owned(), extra);
        }),
        mutation("default theme signal drifts", "UI-001", |set| {
            set.theme_signals.default_theme = ThemeId::Light;
        }),
        mutation("system selection label mapping drifts", "UI-001", |set| {
            set.theme_signals.system_selection_label_id = "theme.auto".to_owned();
        }),
        mutation(
            "unknown system scheme stops falling back dark",
            "UI-001",
            |set| {
                set.theme_signals.system_color_scheme.unknown = ThemeId::Light;
            },
        ),
        mutation("dark system scheme maps to light", "UI-001", |set| {
            set.theme_signals.system_color_scheme.dark = ThemeId::Light;
        }),
        mutation("light system scheme maps to dark", "UI-001", |set| {
            set.theme_signals.system_color_scheme.light = ThemeId::Dark;
        }),
        mutation(
            "high contrast stops using a separate theme",
            "UI-001",
            |set| {
                set.theme_signals.high_contrast_theme = ThemeId::Dark;
            },
        ),
    ]
}

fn label_mutations() -> Vec<Mutation> {
    vec![
        mutation("a theme labelId is blank", "UI-013", |set| {
            set.themes.get_mut("dark").expect("dark").label_id = "   ".to_owned();
        }),
        mutation("a theme embeds an English label", "UI-013", |set| {
            set.themes.get_mut("light").expect("light").label_id = "Light".to_owned();
        }),
        mutation("two themes reuse a labelId", "UI-013", |set| {
            set.themes
                .get_mut("high-contrast")
                .expect("high-contrast")
                .label_id = "theme.dark".to_owned();
        }),
        mutation("a semantic labelId is wrong", "UI-013", |set| {
            set.non_color_channels
                .roles
                .get_mut("entity.device")
                .expect("entity.device")
                .label_id = "meaning.entity.disk".to_owned();
        }),
        mutation("a semantic labelId is blank", "UI-013", |set| {
            set.non_color_channels
                .roles
                .get_mut("progress.failed")
                .expect("progress.failed")
                .label_id = String::new();
        }),
        mutation("a semantic channel embeds English", "UI-013", |set| {
            set.non_color_channels
                .roles
                .get_mut("severity.destructive")
                .expect("severity.destructive")
                .label_id = "Destructive".to_owned();
        }),
        mutation("two semantic roles reuse a labelId", "UI-013", |set| {
            set.non_color_channels
                .roles
                .get_mut("progress.failed")
                .expect("progress.failed")
                .label_id = "meaning.progress.complete".to_owned();
        }),
    ]
}

fn typography_roster_mutations() -> Vec<Mutation> {
    vec![
        mutation("the platform font family is removed", "UI-008", |set| {
            set.typography.families.remove("platform-ui");
        }),
        mutation("an extra font family is invented", "UI-008", |set| {
            let family = set.typography.families["platform-ui"].clone();
            set.typography.families.insert("brand".to_owned(), family);
        }),
        mutation("a required text style is removed", "UI-008", |set| {
            set.typography.styles.remove("exact-value");
        }),
        mutation("an extra text style is invented", "UI-008", |set| {
            let style = set.typography.styles["body"].clone();
            set.typography.styles.insert("marketing".to_owned(), style);
        }),
        mutation(
            "a text style references an unknown family",
            "UI-008",
            |set| {
                set.typography.styles.get_mut("body").expect("body").family = "missing".to_owned();
            },
        ),
    ]
}

fn typography_value_mutations() -> Vec<Mutation> {
    vec![
        mutation("body size drifts", "UI-008", |set| {
            set.typography.styles.get_mut("body").expect("body").size_px = 15;
        }),
        mutation("heading weight drifts", "UI-008", |set| {
            set.typography
                .styles
                .get_mut("heading")
                .expect("heading")
                .weight = 600;
        }),
        mutation("caption italic flag drifts", "UI-008", |set| {
            set.typography
                .styles
                .get_mut("caption")
                .expect("caption")
                .italic = true;
        }),
        mutation("title letter spacing drifts", "UI-008", |set| {
            set.typography
                .styles
                .get_mut("title")
                .expect("title")
                .letter_spacing_milli_px = 0;
        }),
        mutation("eyebrow line height drifts", "UI-008", |set| {
            set.typography
                .styles
                .get_mut("eyebrow")
                .expect("eyebrow")
                .line_height_permille = 1_500;
        }),
    ]
}

fn text_flow_mutations() -> Vec<Mutation> {
    vec![
        mutation("a required text flow is removed", "UI-008", |set| {
            set.typography.flows.remove("multi-line");
        }),
        mutation("an extra text flow is invented", "UI-008", |set| {
            let flow = set.typography.flows["single-line"].clone();
            set.typography.flows.insert("ticker".to_owned(), flow);
        }),
        mutation("single-line wrapping drifts", "UI-008", |set| {
            set.typography
                .flows
                .get_mut("single-line")
                .expect("single-line")
                .wrap = TextWrap::WordWrap;
        }),
        mutation("single-line overflow drifts", "UI-008", |set| {
            set.typography
                .flows
                .get_mut("single-line")
                .expect("single-line")
                .overflow = TextOverflow::Clip;
        }),
        mutation("single-line vertical alignment drifts", "UI-008", |set| {
            set.typography
                .flows
                .get_mut("single-line")
                .expect("single-line")
                .vertical_alignment = VerticalAlignment::Top;
        }),
        mutation("text input selects the wrong style", "UI-008", |set| {
            set.typography.text_input.style = "caption".to_owned();
        }),
        mutation("text input selects the wrong flow", "UI-008", |set| {
            set.typography.text_input.flow = "multi-line".to_owned();
        }),
        mutation("text input references an unknown style", "UI-008", |set| {
            set.typography.text_input.style = "missing".to_owned();
        }),
        mutation("text input references an unknown flow", "UI-008", |set| {
            set.typography.text_input.flow = "missing".to_owned();
        }),
    ]
}

fn selection_mutations() -> Vec<Mutation> {
    vec![
        mutation(
            "the text-input selection roles are reversed",
            "UI-008",
            |set| {
                set.typography.text_input.selection_pair.foreground = "focus.ring".to_owned();
                set.typography.text_input.selection_pair.background = "surface.sunken".to_owned();
            },
        ),
        mutation(
            "the selected-text contrast pair is removed",
            "UI-008",
            |set| {
                set.contrast_rules.pairings.retain(|pairing| {
                    !(pairing.foreground == "surface.sunken" && pairing.background == "focus.ring")
                });
            },
        ),
        mutation(
            "the selected-text pair uses the UI floor",
            "UI-008",
            |set| {
                selected_text_pairing(set).kind = "ui".to_owned();
            },
        ),
        mutation(
            "the selected-text contrast orientation reverses",
            "UI-008",
            |set| {
                let pairing = selected_text_pairing(set);
                std::mem::swap(&mut pairing.foreground, &mut pairing.background);
            },
        ),
        mutation(
            "the selected-text contrast pair is duplicated",
            "UI-008",
            |set| {
                let pairing = selected_text_pairing(set).clone();
                set.contrast_rules.pairings.push(pairing);
            },
        ),
    ]
}

fn spacing_and_radius_mutations() -> Vec<Mutation> {
    vec![
        mutation("a spacing token is removed", "UI-008", |set| {
            set.layout.spacing_px.remove("xxl");
        }),
        mutation("an extra spacing token is invented", "UI-008", |set| {
            set.layout.spacing_px.insert("huge".to_owned(), 64);
        }),
        mutation("a spacing value drifts", "UI-008", |set| {
            set.layout.spacing_px.insert("md".to_owned(), 13);
        }),
        mutation("a radius token is removed", "UI-008", |set| {
            set.layout.radius_px.remove("pill");
        }),
        mutation("an extra radius token is invented", "UI-008", |set| {
            set.layout.radius_px.insert("round".to_owned(), 20);
        }),
        mutation("a radius value drifts", "UI-008", |set| {
            set.layout.radius_px.insert("lg".to_owned(), 12);
        }),
    ]
}

fn stroke_and_layout_mutations() -> Vec<Mutation> {
    vec![
        mutation("a stroke token is removed", "UI-008", |set| {
            set.layout.stroke_px.remove("focus");
        }),
        mutation("an extra stroke token is invented", "UI-008", |set| {
            set.layout.stroke_px.insert("heavy".to_owned(), 4);
        }),
        mutation("a stroke value drifts", "UI-008", |set| {
            set.layout.stroke_px.insert("strong".to_owned(), 3);
        }),
        mutation("default padding selects the wrong token", "UI-008", |set| {
            set.layout.default_layout_padding = "lg".to_owned();
        }),
        mutation("default spacing reference is unresolved", "UI-008", |set| {
            set.layout.default_layout_spacing = "missing".to_owned();
        }),
        mutation("focus-ring offset drifts", "UI-008", |set| {
            set.layout.focus_ring_offset_px = 0;
        }),
        mutation("minimum target size shrinks", "UI-008", |set| {
            set.layout.minimum_target_size_px = 43;
        }),
    ]
}

fn cursor_and_version_mutations() -> Vec<Mutation> {
    vec![
        mutation("a cursor role is removed", "UI-008", |set| {
            set.cursors.roles.remove("disabled");
        }),
        mutation("an extra cursor role is invented", "UI-008", |set| {
            set.cursors
                .roles
                .insert("help".to_owned(), CursorKind::Pointer);
        }),
        mutation("the action cursor mapping drifts", "UI-008", |set| {
            set.cursors
                .roles
                .insert("action".to_owned(), CursorKind::Default);
        }),
        mutation("the text caret width drifts", "UI-008", |set| {
            set.cursors.text_caret_width_px = 1;
        }),
        mutation("the specification version drifts", "UI-008", |set| {
            set.spec_version = "5.0.0".to_owned();
        }),
        mutation("the token version is blank", "UI-008", |set| {
            set.token_set_version = String::new();
        }),
        mutation("the token version is not a version", "UI-008", |set| {
            set.token_set_version = "not-a-version".to_owned();
        }),
        mutation("the token version is unsupported v3", "UI-008", |set| {
            set.token_set_version = "3.0.0".to_owned();
        }),
    ]
}

fn mutations() -> Vec<Mutation> {
    [
        contrast_mutations(),
        color_vision_mutations(),
        measurement_unit_mutations(),
        color_roster_mutations(),
        channel_mutations(),
        theme_mutations(),
        label_mutations(),
        typography_roster_mutations(),
        typography_value_mutations(),
        text_flow_mutations(),
        selection_mutations(),
        spacing_and_radius_mutations(),
        stroke_and_layout_mutations(),
        cursor_and_version_mutations(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

// Requirements: UI-001, UI-003, UI-007, UI-008, UI-011, UI-013, PLAN-004, Section 12
//   Named hostile mutations prove every independently pinned v2 policy family can fail while the evidence remains independent of the table's changing row count
// Evidence: every_policy_family_rejects_a_hostile_mutation
#[test]
fn every_policy_family_rejects_a_hostile_mutation() {
    let mutations = mutations();
    assert!(
        !mutations.is_empty(),
        "mutation evidence must be non-vacuous under Section 12"
    );
    for mutation in mutations {
        let mut set = repository_tokens();
        (mutation.apply)(&mut set);
        let report = audit(&set);
        assert!(
            report.checks > 0,
            "mutation {:?} reached a vacuous audit",
            mutation.name
        );
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

// Requirements: UI-001, UI-007, UI-008, UI-013
//   Audit output names the operating-system, rendering, non-colour selection, absent health-state vocabulary, catalogue, and wider accessibility behavior that static tokens do not establish
// Evidence: the_caveats_are_carried_into_the_output
#[test]
fn the_caveats_are_carried_into_the_output() {
    let caveats = crate::audit::Report::caveats();
    for required_boundary in [
        "operating-system",
        "minimum-target",
        "without colour",
        "health-state",
        "catalogue",
        "screen-reader",
    ] {
        assert!(
            caveats
                .iter()
                .any(|caveat| caveat.contains(required_boundary)),
            "caveats must name the boundary {required_boundary:?}"
        );
    }
}
