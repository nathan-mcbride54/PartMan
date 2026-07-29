//! The WP-030 accessibility harness.
//!
//! Three checks, each computed from the token set rather than recorded beside
//! it, and each reported with the number it produced so a reader can see how
//! close a passing pairing came to failing:
//!
//! 1. **UI-008 contrast.** Every declared pairing, in every theme, meets its
//!    WCAG 2.2 AA threshold.
//! 2. **UI-007 redundant channels.** Every role whose meaning is risk, state,
//!    or identity carries an icon, a label, and a shape, so colour is never the
//!    only carrier. This is the guarantee.
//! 3. **Colour-vision separation.** Roles whose confusion would mislead a user
//!    about risk stay apart under simulated protanopia, deuteranopia and
//!    tritanopia. This is a smell test, and [`Report::caveats`] says so in the
//!    output rather than leaving a reader to assume otherwise.
//!
//! A check that cannot fail is worthless, so every one of these is paired with
//! a mutation in the tests that it must reject.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::color::{Deficiency, Srgb, contrast_ratio, delta_e_76, simulate};
use crate::policy;
use crate::tokens::TokenSet;

/// One thing the harness objected to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    /// Requirement identifier, so a failure names the rule it broke.
    pub requirement: &'static str,
    /// Human-readable detail, including the computed figure.
    pub detail: String,
}

/// The result of auditing a token set.
#[derive(Debug, Clone, Default)]
pub struct Report {
    /// Everything that failed.
    pub findings: Vec<Finding>,
    /// How many individual assertions were evaluated.
    pub checks: usize,
    /// The tightest contrast pairing seen, as `(ratio, description)`.
    pub tightest_contrast: Option<(f64, String)>,
    /// The closest colour-vision pair seen, as `(delta_e, description)`.
    pub closest_separation: Option<(f64, String)>,
}

impl Report {
    /// Whether the token set satisfies every rule.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }

    /// What this report does *not* establish.
    ///
    /// Printed alongside a pass so that a green harness is never mistaken for
    /// an accessibility guarantee it cannot give.
    #[must_use]
    pub fn caveats() -> &'static [&'static str] {
        &[
            "Contrast is computed for the exact colour pairs the token file declares. \
             A surface the front end invents, or a pairing it makes that is not listed, \
             is not covered.",
            "Colour-vision separation uses a model (Machado 2009) and the crudest \
             delta-E formula (CIE76). Passing is not evidence that two colours are \
             distinguishable in practice; UI-007's redundant channels are the guarantee.",
            "Nothing here renders anything. WCAG 2.2 AA also requires focus order, \
             screen-reader semantics, 200% zoom and reduced motion (UI-008), none of \
             which a token file can satisfy and none of which this harness inspects.",
        ]
    }

    /// A human-readable summary for the task runner.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut text = String::new();
        let _ = write!(
            text,
            "tokens: {} check(s) evaluated, {} finding(s)",
            self.checks,
            self.findings.len()
        );
        if let Some((ratio, what)) = &self.tightest_contrast {
            let _ = write!(text, "\n  tightest contrast: {ratio:.2}:1 ({what})");
        }
        if let Some((difference, what)) = &self.closest_separation {
            let _ = write!(
                text,
                "\n  closest colour-vision pair: delta-E {difference:.1} ({what})"
            );
        }
        for finding in &self.findings {
            let _ = write!(text, "\n  {}: {}", finding.requirement, finding.detail);
        }
        text
    }
}

/// Audit a token set against UI-001, UI-007 and UI-008.
#[must_use]
pub fn audit(set: &TokenSet) -> Report {
    let mut report = Report::default();
    check_declared_policy_agrees(set, &mut report);
    check_themes_present(set, &mut report);
    check_required_roster(set, &mut report);
    check_contrast(set, &mut report);
    check_non_color_channels(set, &mut report);
    check_color_vision(set, &mut report);
    report.findings.sort();
    report
}

/// The token file may restate the policy for a front end to read. It may not
/// *decide* it.
///
/// Found by the 2026-07-29 audit: lowering the file's own `text` threshold to
/// 3.0 let normal text pass at 3.33:1 through the whole Tier-1 gate. The floors
/// now come from [`crate::policy`], and the file's copies are required to agree
/// with them exactly. A disagreement is a finding, not a new setting.
fn check_declared_policy_agrees(set: &TokenSet, report: &mut Report) {
    report.checks += 1;
    if set.token_set_version != policy::REQUIRED_TOKEN_SET_VERSION {
        report.findings.push(Finding {
            requirement: "UI-008",
            detail: format!(
                "token set declares tokenSetVersion {:?}, but this harness understands the \
                 vocabulary of {:?}. A non-empty string is not a version: re-derive the roster \
                 against crates/tokens/src/policy.rs before changing it.",
                set.token_set_version,
                policy::REQUIRED_TOKEN_SET_VERSION
            ),
        });
    }

    report.checks += 1;
    if set.spec_version != policy::REQUIRED_SPEC_VERSION {
        report.findings.push(Finding {
            requirement: "UI-008",
            detail: format!(
                "token set declares specVersion {:?}, but this harness encodes the vocabulary of {:?}; \
                 re-derive the roles before changing it",
                set.spec_version,
                policy::REQUIRED_SPEC_VERSION
            ),
        });
    }

    for (kind, declared) in &set.contrast_rules.thresholds {
        report.checks += 1;
        match policy::threshold_for(kind) {
            None => report.findings.push(Finding {
                requirement: "UI-008",
                detail: format!(
                    "token set declares threshold {kind:?}, which is not a WCAG category this \
                     harness recognises"
                ),
            }),
            Some(required) => {
                if (declared - required).abs() > f64::EPSILON {
                    report.findings.push(Finding {
                        requirement: "UI-008",
                        detail: format!(
                            "token set declares the {kind:?} floor as {declared}, but WCAG 2.2 AA \
                             requires {required}. The file may restate the policy; it may not \
                             lower it."
                        ),
                    });
                }
            }
        }
    }

    // Both categories must be restated, so a front end reading this file cannot
    // silently inherit a missing one as "no constraint".
    for kind in ["text", "ui"] {
        report.checks += 1;
        if !set.contrast_rules.thresholds.contains_key(kind) {
            report.findings.push(Finding {
                requirement: "UI-008",
                detail: format!("token set does not restate the {kind:?} WCAG threshold"),
            });
        }
    }

    report.checks += 1;
    let declared = set.color_vision_separation.minimum_delta_e;
    if (declared - policy::COLOR_SEPARATION_FLOOR).abs() > f64::EPSILON {
        report.findings.push(Finding {
            requirement: "UI-007",
            detail: format!(
                "token set declares a colour-separation floor of {declared}, but the project's \
                 recorded floor is {}. Changing it is an ADR, not a palette edit.",
                policy::COLOR_SEPARATION_FLOOR
            ),
        });
    }
}

/// The product vocabulary is fixed by the specification, not by the palette.
///
/// Found by the 2026-07-29 audit: deleting `entity.container` from every theme,
/// every pairing and the channel table left the gate green and simply evaluated
/// six fewer checks, so a coordinated omission was indistinguishable from a
/// smaller product. UI-003 names the entity types, PLAN-004 the severities and
/// UI-011 the progress states; membership is now exact in both directions.
fn check_required_roster(set: &TokenSet, report: &mut Report) {
    let Some(reference) = set.themes.get(policy::DEFAULT_THEME) else {
        return;
    };

    for required in policy::required_meaning_bearing_roles() {
        report.checks += 1;
        if !reference.colors.contains_key(required) {
            report.findings.push(Finding {
                requirement: "UI-003",
                detail: format!(
                    "role {required:?} is required by the specification's vocabulary but is not \
                     defined in the {:?} theme",
                    policy::DEFAULT_THEME
                ),
            });
        }
    }

    // The reverse direction: a meaning-bearing role the contract does not know
    // about is a vocabulary change that has not been through the specification.
    for role in reference
        .colors
        .keys()
        .filter(|role| policy::carries_meaning(role))
    {
        report.checks += 1;
        if !policy::required_meaning_bearing_roles().any(|required| required == role) {
            report.findings.push(Finding {
                requirement: "UI-003",
                detail: format!(
                    "role {role:?} carries meaning but is not in the specification-derived \
                     vocabulary; add it to crates/tokens/src/policy.rs with its requirement, or \
                     remove it"
                ),
            });
        }
    }

    // Every meaning-bearing role must actually be contrast-checked somewhere.
    // Without this, a role could exist, declare its channels, and never appear
    // in a pairing -- present in the file and covered by nothing.
    for required in policy::required_meaning_bearing_roles() {
        report.checks += 1;
        let paired = set
            .contrast_rules
            .pairings
            .iter()
            .any(|pairing| pairing.foreground == required || pairing.background == required);
        if !paired {
            report.findings.push(Finding {
                requirement: "UI-008",
                detail: format!(
                    "role {required:?} appears in no contrast pairing, so nothing checks whether \
                     it is legible"
                ),
            });
        }
    }

    // And every risk pair the project decided must stay distinct has to be
    // present, so the list cannot be shortened to make a palette pass.
    for (one, other) in policy::REQUIRED_DISTINCT_PAIRS {
        report.checks += 1;
        let present = set
            .color_vision_separation
            .must_remain_distinct
            .iter()
            .any(|pair| {
                (pair[0] == one && pair[1] == other) || (pair[0] == other && pair[1] == one)
            });
        if !present {
            report.findings.push(Finding {
                requirement: "UI-007",
                detail: format!(
                    "the pair ({one:?}, {other:?}) must remain distinguishable under colour-vision \
                     deficiency, but the token set no longer declares it"
                ),
            });
        }
    }
}

/// UI-001 requires a dark default, a system theme, and an accessible
/// high-contrast theme. A token set missing one of them cannot satisfy it, and
/// the absence is easier to introduce than to notice.
fn check_themes_present(set: &TokenSet, report: &mut Report) {
    for required in policy::REQUIRED_THEMES {
        report.checks += 1;
        if !set.themes.contains_key(required) {
            report.findings.push(Finding {
                requirement: "UI-001",
                detail: format!("no {required:?} theme is defined"),
            });
        }
    }

    // Every theme must define the same roles. A role present in one theme and
    // absent from another is a component that renders in the default theme and
    // falls back to nothing in high contrast -- exactly where it matters most.
    let Some(reference) = set.themes.get(policy::DEFAULT_THEME) else {
        return;
    };
    let expected: BTreeSet<&String> = reference.colors.keys().collect();
    for (name, theme) in &set.themes {
        report.checks += 1;
        let present: BTreeSet<&String> = theme.colors.keys().collect();
        let missing: Vec<&&String> = expected.difference(&present).collect();
        let extra: Vec<&&String> = present.difference(&expected).collect();
        if !missing.is_empty() || !extra.is_empty() {
            report.findings.push(Finding {
                requirement: "UI-001",
                detail: format!(
                    "theme {name:?} does not define the same roles as {:?}: missing {missing:?}, unexpected {extra:?}",
                    "dark"
                ),
            });
        }
    }
}

/// UI-008: every declared pairing meets its WCAG 2.2 AA threshold, in every
/// theme. Computed, never recorded.
fn check_contrast(set: &TokenSet, report: &mut Report) {
    for (theme_name, theme) in &set.themes {
        for pairing in &set.contrast_rules.pairings {
            report.checks += 1;
            // From `policy`, never from `set`. The file being audited does not
            // get a vote on the standard it is audited against.
            let Some(threshold) = policy::threshold_for(&pairing.kind) else {
                report.findings.push(Finding {
                    requirement: "UI-008",
                    detail: format!(
                        "pairing {}/{} names unknown threshold {:?}",
                        pairing.foreground, pairing.background, pairing.kind
                    ),
                });
                continue;
            };
            let (Some(foreground), Some(background)) = (
                theme.colors.get(&pairing.foreground),
                theme.colors.get(&pairing.background),
            ) else {
                report.findings.push(Finding {
                    requirement: "UI-008",
                    detail: format!(
                        "theme {theme_name:?} does not define {:?} or {:?}",
                        pairing.foreground, pairing.background
                    ),
                });
                continue;
            };
            let (Ok(foreground), Ok(background)) =
                (Srgb::parse(foreground), Srgb::parse(background))
            else {
                // `TokenSet::load` parses every colour, so reaching here means
                // the set was built by hand in a test rather than loaded.
                report.findings.push(Finding {
                    requirement: "UI-008",
                    detail: format!(
                        "theme {theme_name:?} pairing {}/{} has an unparseable colour",
                        pairing.foreground, pairing.background
                    ),
                });
                continue;
            };
            let ratio = contrast_ratio(foreground, background);
            let described = format!(
                "{theme_name}: {} on {}",
                pairing.foreground, pairing.background
            );
            if report
                .tightest_contrast
                .as_ref()
                .is_none_or(|(seen, _)| ratio < *seen)
            {
                report.tightest_contrast = Some((ratio, described.clone()));
            }
            if ratio < threshold {
                report.findings.push(Finding {
                    requirement: "UI-008",
                    detail: format!(
                        "{described} is {ratio:.2}:1, below the {threshold}:1 required for {:?}",
                        pairing.kind
                    ),
                });
            }
        }
    }
}

/// UI-007: colour is never the only carrier. Every entity, severity and
/// progress role must declare an icon, a label and a shape, and every declared
/// channel set must correspond to a real role.
fn check_non_color_channels(set: &TokenSet, report: &mut Report) {
    let Some(reference) = set.themes.get("dark") else {
        return;
    };

    // Which roles carry meaning that UI-007 names: identity, selection, file
    // system, health, risk. Surfaces and text do not, so requiring an icon for
    // `surface.base` would be noise that trains a reader to ignore the rule.
    // The predicate lives in `policy` so this loop and `check_required_roster`
    // cannot drift apart about what "meaningful" means.
    for role in reference
        .colors
        .keys()
        .filter(|role| policy::carries_meaning(role))
    {
        report.checks += 1;
        match set.non_color_channels.roles.get(role) {
            None => report.findings.push(Finding {
                requirement: "UI-007",
                detail: format!("role {role:?} carries meaning but declares no non-colour channel"),
            }),
            Some(channels) => {
                for (name, value) in [
                    ("icon", &channels.icon),
                    ("label", &channels.label),
                    ("shape", &channels.shape),
                ] {
                    if value.trim().is_empty() {
                        report.findings.push(Finding {
                            requirement: "UI-007",
                            detail: format!("role {role:?} has an empty {name}"),
                        });
                    }
                }
            }
        }
    }

    // The reverse direction. A channel set for a role that no longer exists is
    // a rename that was only half applied, and it would otherwise sit in the
    // file looking like coverage.
    for role in set.non_color_channels.roles.keys() {
        report.checks += 1;
        if !reference.colors.contains_key(role) {
            report.findings.push(Finding {
                requirement: "UI-007",
                detail: format!("non-colour channels declared for unknown role {role:?}"),
            });
        }
    }

    // Two roles sharing an icon *and* a label would make the redundant channel
    // non-redundant, which is the one way this table can be filled in
    // completely and still fail to do its job.
    let mut seen: BTreeSet<(&str, &str)> = BTreeSet::new();
    for (role, channels) in &set.non_color_channels.roles {
        report.checks += 1;
        let key = (channels.icon.as_str(), channels.label.as_str());
        if !seen.insert(key) {
            report.findings.push(Finding {
                requirement: "UI-007",
                detail: format!(
                    "role {role:?} reuses icon {:?} with label {:?}; the non-colour channel \
                     cannot distinguish it from the role that already claimed both",
                    channels.icon, channels.label
                ),
            });
        }
    }
}

/// Roles whose confusion would mislead about risk must stay apart under each
/// simulated colour-vision deficiency.
fn check_color_vision(set: &TokenSet, report: &mut Report) {
    // From `policy`, never from `set`. See `check_declared_policy_agrees`.
    let floor = policy::COLOR_SEPARATION_FLOOR;
    for (theme_name, theme) in &set.themes {
        for pair in &set.color_vision_separation.must_remain_distinct {
            let (Some(one), Some(other)) = (theme.colors.get(&pair[0]), theme.colors.get(&pair[1]))
            else {
                report.checks += 1;
                report.findings.push(Finding {
                    requirement: "UI-007",
                    detail: format!(
                        "theme {theme_name:?} does not define {:?} or {:?}",
                        pair[0], pair[1]
                    ),
                });
                continue;
            };
            let (Ok(one), Ok(other)) = (Srgb::parse(one), Srgb::parse(other)) else {
                continue;
            };
            for deficiency in Deficiency::ALL {
                report.checks += 1;
                let difference = delta_e_76(simulate(one, deficiency), simulate(other, deficiency));
                let described = format!(
                    "{theme_name}: {} against {} under {}",
                    pair[0],
                    pair[1],
                    deficiency.name()
                );
                if report
                    .closest_separation
                    .as_ref()
                    .is_none_or(|(seen, _)| difference < *seen)
                {
                    report.closest_separation = Some((difference, described.clone()));
                }
                if difference < floor {
                    report.findings.push(Finding {
                        requirement: "UI-007",
                        detail: format!(
                            "{described} is delta-E {difference:.1}, below the {floor} floor"
                        ),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
