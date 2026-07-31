//! The WP-030 renderer-neutral visual-contract and accessibility harness.
//!
//! The harness derives its policy from [`crate::policy`], independently of the
//! token file it judges. It checks the exact theme and 31-colour rosters, stable
//! localization identifiers, operating-system theme-signal mappings,
//! typography and text-flow values, selected-text contrast, layout and target
//! dimensions, cursor roles, WCAG contrast, redundant non-colour channels, and
//! colour-vision separation.
//!
//! Each family has a hostile mutation test proving that it can fail. Passing
//! remains evidence about a static declaration rather than a rendered user
//! interface; [`Report::caveats`] keeps that boundary explicit.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Debug, Write as _};

use crate::color::{Deficiency, Srgb, contrast_ratio, delta_e_76, simulate};
use crate::policy;
use crate::tokens::{
    CursorKind, FontFamilyStrategy, HorizontalAlignment, Pairing, TextOverflow, TextWrap, Theme,
    ThemeId, TokenSet, VerticalAlignment,
};

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
    /// an accessibility or localization guarantee it cannot give.
    #[must_use]
    pub fn caveats() -> &'static [&'static str] {
        &[
            "Contrast is computed for the exact independently pinned canonical pairing \
             roster. A surface or pairing the front end invents outside that roster is \
             not covered until the application boundary rejects it.",
            "Colour-vision separation uses a model (Machado 2009) and the crudest \
             delta-E formula (CIE76). Passing is not evidence that two colours are \
             distinguishable in practice; UI-007's redundant channels remain required.",
            "Theme-signal checks prove only the declared mapping. They do not prove that \
             a front end detects operating-system colour-scheme or high-contrast signals, \
             subscribes to changes, or applies the mapped theme.",
            "Typography, text-flow, layout, focus-offset, minimum-target, cursor and caret \
             checks prove token values and references only. They do not measure rendering, \
             hit targets, focus treatment, clipping, wrapping, or platform-font resolution.",
            "Selected-text contrast checks only the declared foreground/background pair. \
             They do not prove selection is rendered, remains perceivable without colour, \
             or exposes selection state to assistive technology.",
            "The redundant-channel roster covers the 21 declared entity, risk-severity, \
             and progress-state roles. It contains no health-state vocabulary; health and \
             non-colour selection remain separate shell-evidence obligations under UI-007.",
            "Stable label IDs do not prove that a catalogue contains or resolves them, that \
             translated text fits, or that every other user-facing string is externalized \
             as UI-013 requires.",
            "Nothing here renders anything. UI-008 separately requires WCAG 2.2 AA, \
             keyboard-only operation, screen-reader semantics, 200% zoom, and reduced \
             motion; none of those application behaviors is established by this harness.",
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

/// Audit a token set against UI-001, UI-003, UI-007, UI-008, UI-011, UI-013,
/// and PLAN-004.
#[must_use]
pub fn audit(set: &TokenSet) -> Report {
    let mut report = Report::default();
    check_declared_policy_agrees(set, &mut report);
    check_measurement_units(set, &mut report);
    check_theme_roster(set, &mut report);
    check_color_rosters(set, &mut report);
    check_label_ids(set, &mut report);
    check_theme_signals(set, &mut report);
    check_typography(set, &mut report);
    check_text_input(set, &mut report);
    check_layout(set, &mut report);
    check_cursors(set, &mut report);
    check_contrast_pairing_roster(set, &mut report);
    check_semantic_contrast_coverage(set, &mut report);
    check_contrast(set, &mut report);
    check_non_color_channels(set, &mut report);
    check_required_distinct_pairs(set, &mut report);
    check_color_vision(set, &mut report);
    report.findings.sort();
    report
}

fn check_measurement_units(set: &TokenSet, report: &mut Report) {
    let actual = &set.measurement_units;
    let required = policy::REQUIRED_MEASUREMENT_UNITS;
    check_exact(
        report,
        "UI-008",
        "measurementUnits.px",
        actual.px.as_str(),
        required.px,
    );
    check_exact(
        report,
        "UI-008",
        "measurementUnits.letterSpacingMilliPx",
        actual.letter_spacing_milli_px.as_str(),
        required.letter_spacing_milli_px,
    );
    check_exact(
        report,
        "UI-008",
        "measurementUnits.lineHeightPermille",
        actual.line_height_permille.as_str(),
        required.line_height_permille,
    );
}

fn finding(report: &mut Report, requirement: &'static str, detail: String) {
    report.findings.push(Finding {
        requirement,
        detail,
    });
}

fn check_exact<T: Debug + PartialEq + ?Sized>(
    report: &mut Report,
    requirement: &'static str,
    description: &str,
    actual: &T,
    required: &T,
) {
    report.checks += 1;
    if actual != required {
        finding(
            report,
            requirement,
            format!("{description} is {actual:?}; policy requires {required:?}"),
        );
    }
}

fn check_named_roster<T>(
    actual: &BTreeMap<String, T>,
    required: &[&str],
    family: &str,
    requirement: &'static str,
    report: &mut Report,
) {
    for name in required {
        report.checks += 1;
        if !actual.contains_key(*name) {
            finding(
                report,
                requirement,
                format!("{family} is missing required token {name:?}"),
            );
        }
    }
    for name in actual.keys() {
        report.checks += 1;
        if !required.contains(&name.as_str()) {
            finding(
                report,
                requirement,
                format!("{family} contains unsupported token {name:?}"),
            );
        }
    }
}

/// The token file may restate policy for a front end to read. It may not decide
/// that policy.
fn check_declared_policy_agrees(set: &TokenSet, report: &mut Report) {
    check_exact(
        report,
        "UI-008",
        "tokenSetVersion",
        set.token_set_version.as_str(),
        policy::REQUIRED_TOKEN_SET_VERSION,
    );
    check_exact(
        report,
        "UI-008",
        "specVersion",
        set.spec_version.as_str(),
        policy::REQUIRED_SPEC_VERSION,
    );
    check_declared_contrast_policy(set, report);
    check_exact(
        report,
        "UI-007",
        "declared colour-separation floor",
        &set.color_vision_separation.minimum_delta_e,
        &policy::COLOR_SEPARATION_FLOOR,
    );
}

fn check_declared_contrast_policy(set: &TokenSet, report: &mut Report) {
    for (kind, declared) in &set.contrast_rules.thresholds {
        report.checks += 1;
        match policy::threshold_for(kind) {
            None => finding(
                report,
                "UI-008",
                format!(
                    "token set declares threshold {kind:?}, which is not a WCAG category this \
                     harness recognises"
                ),
            ),
            Some(required) if (declared - required).abs() > f64::EPSILON => finding(
                report,
                "UI-008",
                format!(
                    "token set declares the {kind:?} floor as {declared}, but WCAG 2.2 AA \
                     requires {required}; the file may restate policy but may not lower it"
                ),
            ),
            Some(_) => {}
        }
    }
    for kind in ["text", "ui"] {
        report.checks += 1;
        if !set.contrast_rules.thresholds.contains_key(kind) {
            finding(
                report,
                "UI-008",
                format!("token set does not restate the {kind:?} WCAG threshold"),
            );
        }
    }
}

/// UI-001's theme vocabulary is exact in both directions.
fn check_theme_roster(set: &TokenSet, report: &mut Report) {
    for required in policy::REQUIRED_THEMES {
        report.checks += 1;
        if !set.themes.contains_key(required) {
            finding(
                report,
                "UI-001",
                format!("no {required:?} theme is defined"),
            );
        }
    }
    for name in set.themes.keys() {
        report.checks += 1;
        if !policy::REQUIRED_THEMES.contains(&name.as_str()) {
            finding(
                report,
                "UI-001",
                format!("theme {name:?} is not in the exact UI-001 theme roster"),
            );
        }
    }
}

/// Every required theme independently carries the exact 31-colour roster.
fn check_color_rosters(set: &TokenSet, report: &mut Report) {
    let required: Vec<&str> = policy::required_color_roles().collect();
    for theme_name in policy::REQUIRED_THEMES {
        if let Some(theme) = set.themes.get(theme_name) {
            check_theme_color_roster(theme_name, theme, &required, report);
        }
    }
}

fn check_theme_color_roster(
    theme_name: &str,
    theme: &Theme,
    required: &[&str],
    report: &mut Report,
) {
    for role in required {
        report.checks += 1;
        if !theme.colors.contains_key(*role) {
            let requirement = policy::vocabulary_requirement(role).unwrap_or("UI-008");
            finding(
                report,
                requirement,
                format!("theme {theme_name:?} is missing required colour role {role:?}"),
            );
        }
    }
    for role in theme.colors.keys() {
        report.checks += 1;
        if !required.contains(&role.as_str()) {
            let requirement = policy::vocabulary_requirement(role).unwrap_or("UI-008");
            finding(
                report,
                requirement,
                format!("theme {theme_name:?} contains unsupported colour role {role:?}"),
            );
        }
    }

    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|role| !theme.colors.contains_key(*role))
        .collect();
    let extra: Vec<&str> = theme
        .colors
        .keys()
        .map(String::as_str)
        .filter(|role| !required.contains(role))
        .collect();
    report.checks += 1;
    if !missing.is_empty() || !extra.is_empty() {
        finding(
            report,
            "UI-001",
            format!(
                "theme {theme_name:?} does not implement the exact shared colour roster: \
                 missing {missing:?}, unexpected {extra:?}"
            ),
        );
    }
}

fn check_label_ids(set: &TokenSet, report: &mut Report) {
    check_theme_label_ids(set, report);
    check_semantic_label_ids(set, report);
    check_label_roster_and_uniqueness(set, report);
}

fn check_theme_label_ids(set: &TokenSet, report: &mut Report) {
    for rule in policy::REQUIRED_THEME_LABELS {
        if let Some(theme) = set.themes.get(rule.theme) {
            check_exact(
                report,
                "UI-013",
                &format!("labelId for theme {:?}", rule.theme),
                theme.label_id.as_str(),
                rule.label_id,
            );
        }
    }
    check_exact(
        report,
        "UI-013",
        "system-selection labelId",
        set.theme_signals.system_selection_label_id.as_str(),
        policy::REQUIRED_THEME_SIGNALS.system_selection_label_id,
    );
}

fn check_semantic_label_ids(set: &TokenSet, report: &mut Report) {
    for rule in policy::REQUIRED_SEMANTIC_LABELS {
        report.checks += 1;
        match set.non_color_channels.roles.get(rule.role) {
            None => finding(
                report,
                "UI-013",
                format!(
                    "semantic role {:?} has no channel entry carrying required labelId {:?}",
                    rule.role, rule.label_id
                ),
            ),
            Some(channels) if channels.label_id != rule.label_id => finding(
                report,
                "UI-013",
                format!(
                    "semantic role {:?} declares labelId {:?}; policy requires {:?}",
                    rule.role, channels.label_id, rule.label_id
                ),
            ),
            Some(_) => {}
        }
    }
}

fn check_label_roster_and_uniqueness(set: &TokenSet, report: &mut Report) {
    let mut owners: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for (theme_name, theme) in &set.themes {
        owners
            .entry(theme.label_id.as_str())
            .or_default()
            .push(format!("theme {theme_name:?}"));
    }
    owners
        .entry(set.theme_signals.system_selection_label_id.as_str())
        .or_default()
        .push("system theme selection".to_owned());
    for (role, channels) in &set.non_color_channels.roles {
        owners
            .entry(channels.label_id.as_str())
            .or_default()
            .push(format!("semantic role {role:?}"));
    }

    let required: BTreeSet<&str> = policy::REQUIRED_LABEL_IDS.into_iter().collect();
    for label_id in &required {
        report.checks += 1;
        if !owners.contains_key(label_id) {
            finding(
                report,
                "UI-013",
                format!("required labelId {label_id:?} is not declared"),
            );
        }
    }
    for (label_id, contexts) in owners {
        report.checks += 1;
        if label_id.trim().is_empty() {
            finding(
                report,
                "UI-013",
                format!("blank labelId is used by {contexts:?}"),
            );
        } else if !required.contains(label_id) {
            finding(
                report,
                "UI-013",
                format!("unsupported labelId {label_id:?} is used by {contexts:?}"),
            );
        }
        report.checks += 1;
        if contexts.len() != 1 {
            finding(
                report,
                "UI-013",
                format!("labelId {label_id:?} is reused by {contexts:?}"),
            );
        }
    }
}

fn theme_id_name(theme: ThemeId) -> &'static str {
    match theme {
        ThemeId::Dark => "dark",
        ThemeId::HighContrast => "high-contrast",
        ThemeId::Light => "light",
    }
}

/// UI-001's system mapping is exact, and high contrast remains a separate
/// signal rather than a fourth colour-scheme value.
fn check_theme_signals(set: &TokenSet, report: &mut Report) {
    let actual = &set.theme_signals;
    let required = policy::REQUIRED_THEME_SIGNALS;
    check_exact(
        report,
        "UI-001",
        "themeSignals.defaultTheme",
        theme_id_name(actual.default_theme),
        required.default_theme,
    );
    check_exact(
        report,
        "UI-001",
        "themeSignals.systemSelectionLabelId",
        actual.system_selection_label_id.as_str(),
        required.system_selection_label_id,
    );
    check_exact(
        report,
        "UI-001",
        "themeSignals.systemColorScheme.unknown",
        theme_id_name(actual.system_color_scheme.unknown),
        required.unknown_color_scheme_theme,
    );
    check_exact(
        report,
        "UI-001",
        "themeSignals.systemColorScheme.dark",
        theme_id_name(actual.system_color_scheme.dark),
        required.dark_color_scheme_theme,
    );
    check_exact(
        report,
        "UI-001",
        "themeSignals.systemColorScheme.light",
        theme_id_name(actual.system_color_scheme.light),
        required.light_color_scheme_theme,
    );
    check_exact(
        report,
        "UI-001",
        "themeSignals.highContrastTheme (a separate platform signal)",
        theme_id_name(actual.high_contrast_theme),
        required.high_contrast_theme,
    );
}

fn font_family_strategy_name(strategy: FontFamilyStrategy) -> &'static str {
    match strategy {
        FontFamilyStrategy::PlatformDefault => "platform-default",
    }
}

fn check_typography(set: &TokenSet, report: &mut Report) {
    check_font_families(set, report);
    check_typography_styles(set, report);
    check_text_flows(set, report);
    check_typography_references(set, report);
}

fn check_font_families(set: &TokenSet, report: &mut Report) {
    let required: Vec<&str> = policy::REQUIRED_FONT_FAMILIES
        .iter()
        .map(|rule| rule.id)
        .collect();
    check_named_roster(
        &set.typography.families,
        &required,
        "typography.families",
        "UI-008",
        report,
    );
    for rule in policy::REQUIRED_FONT_FAMILIES {
        if let Some(family) = set.typography.families.get(rule.id) {
            check_exact(
                report,
                "UI-008",
                &format!("typography family {:?} strategy", rule.id),
                font_family_strategy_name(family.strategy),
                rule.strategy,
            );
        }
    }
}

fn check_typography_styles(set: &TokenSet, report: &mut Report) {
    let required: Vec<&str> = policy::REQUIRED_TYPOGRAPHY_STYLES
        .iter()
        .map(|rule| rule.id)
        .collect();
    check_named_roster(
        &set.typography.styles,
        &required,
        "typography.styles",
        "UI-008",
        report,
    );
    for rule in policy::REQUIRED_TYPOGRAPHY_STYLES {
        let Some(style) = set.typography.styles.get(rule.id) else {
            continue;
        };
        let description = |field: &str| format!("typography style {:?} {field}", rule.id);
        check_exact(
            report,
            "UI-008",
            &description("family"),
            style.family.as_str(),
            rule.family,
        );
        check_exact(
            report,
            "UI-008",
            &description("sizePx"),
            &style.size_px,
            &rule.size_px,
        );
        check_exact(
            report,
            "UI-008",
            &description("weight"),
            &style.weight,
            &rule.weight,
        );
        check_exact(
            report,
            "UI-008",
            &description("italic"),
            &style.italic,
            &rule.italic,
        );
        check_exact(
            report,
            "UI-008",
            &description("letterSpacingMilliPx"),
            &style.letter_spacing_milli_px,
            &rule.letter_spacing_milli_px,
        );
        check_exact(
            report,
            "UI-008",
            &description("lineHeightPermille"),
            &style.line_height_permille,
            &rule.line_height_permille,
        );
    }
}

fn text_wrap_name(value: TextWrap) -> &'static str {
    match value {
        TextWrap::NoWrap => "no-wrap",
        TextWrap::WordWrap => "word-wrap",
    }
}

fn text_overflow_name(value: TextOverflow) -> &'static str {
    match value {
        TextOverflow::Elide => "elide",
        TextOverflow::Clip => "clip",
    }
}

fn horizontal_alignment_name(value: HorizontalAlignment) -> &'static str {
    match value {
        HorizontalAlignment::Left => "left",
    }
}

fn vertical_alignment_name(value: VerticalAlignment) -> &'static str {
    match value {
        VerticalAlignment::Center => "center",
        VerticalAlignment::Top => "top",
    }
}

fn check_text_flows(set: &TokenSet, report: &mut Report) {
    let required: Vec<&str> = policy::REQUIRED_TEXT_FLOWS
        .iter()
        .map(|rule| rule.id)
        .collect();
    check_named_roster(
        &set.typography.flows,
        &required,
        "typography.flows",
        "UI-008",
        report,
    );
    for rule in policy::REQUIRED_TEXT_FLOWS {
        let Some(flow) = set.typography.flows.get(rule.id) else {
            continue;
        };
        let description = |field: &str| format!("text flow {:?} {field}", rule.id);
        check_exact(
            report,
            "UI-008",
            &description("wrap"),
            text_wrap_name(flow.wrap),
            rule.wrap,
        );
        check_exact(
            report,
            "UI-008",
            &description("overflow"),
            text_overflow_name(flow.overflow),
            rule.overflow,
        );
        check_exact(
            report,
            "UI-008",
            &description("horizontalAlignment"),
            horizontal_alignment_name(flow.horizontal_alignment),
            rule.horizontal_alignment,
        );
        check_exact(
            report,
            "UI-008",
            &description("verticalAlignment"),
            vertical_alignment_name(flow.vertical_alignment),
            rule.vertical_alignment,
        );
    }
}

fn check_typography_references(set: &TokenSet, report: &mut Report) {
    for (style_name, style) in &set.typography.styles {
        report.checks += 1;
        if !set.typography.families.contains_key(style.family.as_str()) {
            finding(
                report,
                "UI-008",
                format!(
                    "typography style {style_name:?} references unknown family {:?}",
                    style.family
                ),
            );
        }
    }
}

fn check_text_input(set: &TokenSet, report: &mut Report) {
    let actual = &set.typography.text_input;
    let required = policy::REQUIRED_TEXT_INPUT;
    check_exact(
        report,
        "UI-008",
        "typography.textInput.style",
        actual.style.as_str(),
        required.style,
    );
    check_exact(
        report,
        "UI-008",
        "typography.textInput.flow",
        actual.flow.as_str(),
        required.flow,
    );
    check_exact(
        report,
        "UI-008",
        "typography.textInput.selectionPair.foreground",
        actual.selection_pair.foreground.as_str(),
        required.selection_foreground_role,
    );
    check_exact(
        report,
        "UI-008",
        "typography.textInput.selectionPair.background",
        actual.selection_pair.background.as_str(),
        required.selection_background_role,
    );
    check_text_input_references(set, report);
    check_selection_contrast_pairing(set, report);
}

fn check_text_input_references(set: &TokenSet, report: &mut Report) {
    let input = &set.typography.text_input;
    report.checks += 1;
    if !set.typography.styles.contains_key(input.style.as_str()) {
        finding(
            report,
            "UI-008",
            format!("text input references unknown style {:?}", input.style),
        );
    }
    report.checks += 1;
    if !set.typography.flows.contains_key(input.flow.as_str()) {
        finding(
            report,
            "UI-008",
            format!("text input references unknown flow {:?}", input.flow),
        );
    }
    for (theme_name, theme) in &set.themes {
        for role in [
            input.selection_pair.foreground.as_str(),
            input.selection_pair.background.as_str(),
        ] {
            report.checks += 1;
            if !theme.colors.contains_key(role) {
                finding(
                    report,
                    "UI-008",
                    format!(
                        "theme {theme_name:?} does not resolve text-input selection role {role:?}"
                    ),
                );
            }
        }
    }
}

fn check_selection_contrast_pairing(set: &TokenSet, report: &mut Report) {
    let required = policy::REQUIRED_SELECTION_CONTRAST;
    let related: Vec<_> = set
        .contrast_rules
        .pairings
        .iter()
        .filter(|pairing| {
            (pairing.foreground == required.foreground_role
                && pairing.background == required.background_role)
                || (pairing.foreground == required.background_role
                    && pairing.background == required.foreground_role)
        })
        .collect();
    report.checks += 1;
    if related.len() != 1 {
        finding(
            report,
            "UI-008",
            format!(
                "selected text requires exactly one pairing between {:?} and {:?}; found {}",
                required.foreground_role,
                required.background_role,
                related.len()
            ),
        );
    }
    if let [pairing] = related.as_slice() {
        check_exact(
            report,
            "UI-008",
            "selected-text pairing foreground",
            pairing.foreground.as_str(),
            required.foreground_role,
        );
        check_exact(
            report,
            "UI-008",
            "selected-text pairing background",
            pairing.background.as_str(),
            required.background_role,
        );
        check_exact(
            report,
            "UI-008",
            "selected-text pairing kind",
            pairing.kind.as_str(),
            required.threshold,
        );
    }
}

fn check_pixel_scale(
    actual: &BTreeMap<String, u16>,
    required: &[policy::PixelTokenRule],
    family: &str,
    report: &mut Report,
) {
    let names: Vec<&str> = required.iter().map(|rule| rule.id).collect();
    check_named_roster(actual, &names, family, "UI-008", report);
    for rule in required {
        if let Some(value) = actual.get(rule.id) {
            check_exact(
                report,
                "UI-008",
                &format!("{family}.{:?}", rule.id),
                value,
                &rule.value_px,
            );
        }
    }
}

fn check_layout(set: &TokenSet, report: &mut Report) {
    let layout = &set.layout;
    check_pixel_scale(
        &layout.spacing_px,
        &policy::REQUIRED_SPACING_PX,
        "layout.spacingPx",
        report,
    );
    check_pixel_scale(
        &layout.radius_px,
        &policy::REQUIRED_RADIUS_PX,
        "layout.radiusPx",
        report,
    );
    check_pixel_scale(
        &layout.stroke_px,
        &policy::REQUIRED_STROKE_PX,
        "layout.strokePx",
        report,
    );
    let required = policy::REQUIRED_LAYOUT;
    check_exact(
        report,
        "UI-008",
        "layout.defaultLayoutPadding",
        layout.default_layout_padding.as_str(),
        required.default_layout_padding,
    );
    check_exact(
        report,
        "UI-008",
        "layout.defaultLayoutSpacing",
        layout.default_layout_spacing.as_str(),
        required.default_layout_spacing,
    );
    check_exact(
        report,
        "UI-008",
        "layout.focusRingOffsetPx",
        &layout.focus_ring_offset_px,
        &required.focus_ring_offset_px,
    );
    check_exact(
        report,
        "UI-008",
        "layout.minimumTargetSizePx",
        &layout.minimum_target_size_px,
        &required.minimum_target_size_px,
    );
    check_layout_references(set, report);
}

fn check_layout_references(set: &TokenSet, report: &mut Report) {
    for (description, reference) in [
        (
            "layout.defaultLayoutPadding",
            set.layout.default_layout_padding.as_str(),
        ),
        (
            "layout.defaultLayoutSpacing",
            set.layout.default_layout_spacing.as_str(),
        ),
    ] {
        report.checks += 1;
        if !set.layout.spacing_px.contains_key(reference) {
            finding(
                report,
                "UI-008",
                format!("{description} references unknown spacing token {reference:?}"),
            );
        }
    }
}

fn cursor_kind_name(cursor: CursorKind) -> &'static str {
    match cursor {
        CursorKind::Default => "default",
        CursorKind::Pointer => "pointer",
        CursorKind::NotAllowed => "not-allowed",
        CursorKind::Text => "text",
    }
}

fn check_cursors(set: &TokenSet, report: &mut Report) {
    let required: Vec<&str> = policy::REQUIRED_CURSOR_ROLES
        .iter()
        .map(|rule| rule.role)
        .collect();
    check_named_roster(
        &set.cursors.roles,
        &required,
        "cursors.roles",
        "UI-008",
        report,
    );
    for rule in policy::REQUIRED_CURSOR_ROLES {
        if let Some(cursor) = set.cursors.roles.get(rule.role) {
            check_exact(
                report,
                "UI-008",
                &format!("cursor role {:?}", rule.role),
                cursor_kind_name(*cursor),
                rule.cursor,
            );
        }
    }
    check_exact(
        report,
        "UI-008",
        "cursors.textCaretWidthPx",
        &set.cursors.text_caret_width_px,
        &policy::REQUIRED_TEXT_CARET_WIDTH_PX,
    );
}

/// Every canonical use is present exactly once, in its required orientation
/// and threshold class. The token file may carry the roster for generators; it
/// may not choose which uses qualify as normal text.
fn check_contrast_pairing_roster(set: &TokenSet, report: &mut Report) {
    for required in policy::REQUIRED_CONTRAST_PAIRINGS {
        let count = set
            .contrast_rules
            .pairings
            .iter()
            .filter(|pairing| {
                pairing.foreground == required.foreground
                    && pairing.background == required.background
                    && pairing.kind == required.kind
            })
            .count();
        report.checks += 1;
        if count != 1 {
            finding(
                report,
                "UI-008",
                format!(
                    "canonical contrast pairing ({:?}, {:?}, {:?}) must appear exactly once; \
                     found {count}",
                    required.foreground, required.background, required.kind
                ),
            );
        }
    }

    for pairing in &set.contrast_rules.pairings {
        report.checks += 1;
        if !policy::REQUIRED_CONTRAST_PAIRINGS.iter().any(|required| {
            pairing.foreground == required.foreground
                && pairing.background == required.background
                && pairing.kind == required.kind
        }) {
            finding(
                report,
                "UI-008",
                format!(
                    "unsupported contrast pairing ({:?}, {:?}, {:?}); the oriented pair and \
                     threshold class are independent policy",
                    pairing.foreground, pairing.background, pairing.kind
                ),
            );
        }
    }
}

fn check_semantic_contrast_coverage(set: &TokenSet, report: &mut Report) {
    for role in policy::required_meaning_bearing_roles() {
        report.checks += 1;
        let paired = set
            .contrast_rules
            .pairings
            .iter()
            .any(|pairing| pairing.foreground == role || pairing.background == role);
        if !paired {
            finding(
                report,
                "UI-008",
                format!(
                    "role {role:?} appears in no contrast pairing, so nothing checks whether \
                     it is legible"
                ),
            );
        }
    }
}

/// UI-008: every declared pairing meets its WCAG 2.2 AA threshold, in every
/// declared theme. Computed, never recorded.
fn check_contrast(set: &TokenSet, report: &mut Report) {
    for (theme_name, theme) in &set.themes {
        for pairing in &set.contrast_rules.pairings {
            report.checks += 1;
            let Some(threshold) = policy::threshold_for(&pairing.kind) else {
                finding(
                    report,
                    "UI-008",
                    format!(
                        "pairing {}/{} names unknown threshold {:?}",
                        pairing.foreground, pairing.background, pairing.kind
                    ),
                );
                continue;
            };
            let (Some(foreground), Some(background)) = (
                theme.colors.get(&pairing.foreground),
                theme.colors.get(&pairing.background),
            ) else {
                finding(
                    report,
                    "UI-008",
                    format!(
                        "theme {theme_name:?} does not define {:?} or {:?}",
                        pairing.foreground, pairing.background
                    ),
                );
                continue;
            };
            let (Ok(foreground), Ok(background)) =
                (Srgb::parse(foreground), Srgb::parse(background))
            else {
                finding(
                    report,
                    "UI-008",
                    format!(
                        "theme {theme_name:?} pairing {}/{} has an unparseable colour",
                        pairing.foreground, pairing.background
                    ),
                );
                continue;
            };
            record_contrast(
                report, theme_name, pairing, threshold, foreground, background,
            );
        }
    }
}

fn record_contrast(
    report: &mut Report,
    theme_name: &str,
    pairing: &Pairing,
    threshold: f64,
    foreground: Srgb,
    background: Srgb,
) {
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
        finding(
            report,
            "UI-008",
            format!(
                "{described} is {ratio:.2}:1, below the {threshold}:1 required for {:?}",
                pairing.kind
            ),
        );
    }
}

/// UI-007: every required semantic role carries icon, label ID and shape, and
/// every channel entry belongs to that exact semantic roster.
fn check_non_color_channels(set: &TokenSet, report: &mut Report) {
    for role in policy::required_meaning_bearing_roles() {
        report.checks += 1;
        match set.non_color_channels.roles.get(role) {
            None => finding(
                report,
                "UI-007",
                format!("role {role:?} carries meaning but declares no non-colour channel"),
            ),
            Some(channels) => {
                for (name, value) in [
                    ("icon", channels.icon.as_str()),
                    ("labelId", channels.label_id.as_str()),
                    ("shape", channels.shape.as_str()),
                ] {
                    report.checks += 1;
                    if value.trim().is_empty() {
                        finding(
                            report,
                            "UI-007",
                            format!("role {role:?} has an empty {name}"),
                        );
                    }
                }
            }
        }
    }

    let required: BTreeSet<&str> = policy::required_meaning_bearing_roles().collect();
    for role in set.non_color_channels.roles.keys() {
        report.checks += 1;
        if !required.contains(role.as_str()) {
            finding(
                report,
                "UI-007",
                format!("non-colour channels declared for unknown role {role:?}"),
            );
        }
    }
    check_channel_uniqueness(set, report);
}

fn check_channel_uniqueness(set: &TokenSet, report: &mut Report) {
    let mut seen: BTreeSet<(&str, &str)> = BTreeSet::new();
    for (role, channels) in &set.non_color_channels.roles {
        report.checks += 1;
        let key = (channels.icon.as_str(), channels.label_id.as_str());
        if !seen.insert(key) {
            finding(
                report,
                "UI-007",
                format!(
                    "role {role:?} reuses icon {:?} with labelId {:?}; the non-colour channel \
                     cannot distinguish it from the role that already claimed both",
                    channels.icon, channels.label_id
                ),
            );
        }
    }
}

fn check_required_distinct_pairs(set: &TokenSet, report: &mut Report) {
    fn unordered<'a>(one: &'a str, other: &'a str) -> (&'a str, &'a str) {
        if one <= other {
            (one, other)
        } else {
            (other, one)
        }
    }

    let required: BTreeSet<(&str, &str)> = policy::REQUIRED_DISTINCT_PAIRS
        .into_iter()
        .map(|(one, other)| unordered(one, other))
        .collect();
    let mut actual: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for pair in &set.color_vision_separation.must_remain_distinct {
        *actual
            .entry(unordered(pair[0].as_str(), pair[1].as_str()))
            .or_default() += 1;
    }

    for pair in &required {
        report.checks += 1;
        let count = actual.get(pair).copied().unwrap_or_default();
        if count != 1 {
            finding(
                report,
                "UI-007",
                format!(
                    "the unordered pair ({:?}, {:?}) must appear exactly once in the \
                     colour-vision roster; found {count}",
                    pair.0, pair.1
                ),
            );
        }
    }
    for (pair, count) in actual {
        report.checks += 1;
        if !required.contains(&pair) {
            finding(
                report,
                "UI-007",
                format!(
                    "unsupported colour-vision pair ({:?}, {:?}); the roster is exact \
                     independent policy",
                    pair.0, pair.1
                ),
            );
        }
        if count > 1 {
            finding(
                report,
                "UI-007",
                format!(
                    "colour-vision pair ({:?}, {:?}) is declared {count} times; unordered \
                     pairs must be unique",
                    pair.0, pair.1
                ),
            );
        }
    }
}

/// Roles whose confusion would mislead about risk must stay apart under each
/// simulated colour-vision deficiency.
fn check_color_vision(set: &TokenSet, report: &mut Report) {
    let floor = policy::COLOR_SEPARATION_FLOOR;
    for (theme_name, theme) in &set.themes {
        for pair in &set.color_vision_separation.must_remain_distinct {
            let (Some(one), Some(other)) = (theme.colors.get(&pair[0]), theme.colors.get(&pair[1]))
            else {
                report.checks += 1;
                finding(
                    report,
                    "UI-007",
                    format!(
                        "theme {theme_name:?} does not define {:?} or {:?}",
                        pair[0], pair[1]
                    ),
                );
                continue;
            };
            let (Ok(one), Ok(other)) = (Srgb::parse(one), Srgb::parse(other)) else {
                continue;
            };
            for deficiency in Deficiency::ALL {
                report.checks += 1;
                let difference = delta_e_76(simulate(one, deficiency), simulate(other, deficiency));
                record_separation(report, theme_name, pair, deficiency, difference, floor);
            }
        }
    }
}

fn record_separation(
    report: &mut Report,
    theme_name: &str,
    pair: &[String; 2],
    deficiency: Deficiency,
    difference: f64,
    floor: f64,
) {
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
        finding(
            report,
            "UI-007",
            format!("{described} is delta-E {difference:.1}, below the {floor} floor"),
        );
    }
}

#[cfg(test)]
mod tests;
