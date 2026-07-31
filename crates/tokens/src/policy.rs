//! The accessibility policy, held **outside** the file it judges.
//!
//! This module exists because of a defect found by the 2026-07-29 project
//! audit. Every floor and every required role used to be read from
//! `schemas/design-tokens.json` — the same file the harness audits. Lowering
//! the file's own `text` threshold from 4.5 to 3.0 and dimming a colour to
//! match let normal-size text pass the entire Tier-1 gate at **3.33:1**, and
//! deleting a semantic role from every table at once dropped the audit from 234
//! checks to 228 while still reporting success.
//!
//! Both are the failure `AGENTS.md` already names for the canonical vectors:
//! an implementation checked against a table it also owns proves only
//! self-consistency. The harness was written to enforce that rule on colours
//! and broke it on the policy governing them.
//!
//! So the three contrast constants immediately below are **not** configuration.
//! Two are external standards that this project does not get to choose, and the
//! third is a project decision recorded as one. The remaining declarations pin
//! PartMan's own required token vocabulary independently. The token file may
//! restate all of them for a front end to read, but [`crate::audit`] requires
//! the restatement to *agree* rather than treating it as authority.

/// WCAG 2.2 Success Criterion 1.4.3 Contrast (Minimum), level AA, normal text.
///
/// <https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum>
///
/// Not negotiable and not a project setting: it is the published threshold. A
/// palette that cannot meet it changes its colours.
pub const WCAG_AA_NORMAL_TEXT: f64 = 4.5;

/// WCAG 2.2 Success Criterion 1.4.11 Non-text Contrast, level AA.
///
/// <https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast>
///
/// Applies to user-interface components and meaningful graphical objects. A
/// role carrying this threshold MUST NOT be used to render normal-size text,
/// and that restriction is the entire reason its floor is lower.
pub const WCAG_AA_NON_TEXT: f64 = 3.0;

/// CIE76 floor for role pairs whose confusion would mislead about risk.
///
/// **A project decision, not a standard.** WCAG sets no such threshold. It was
/// chosen to sit comfortably above the delta-E 10.1 collapse WP-030 increment 1
/// found between `severity.reversible` and `severity.destructive` under
/// deuteranopia, and comfortably below the 21.9 the corrected palette achieves.
///
/// Changing it is a policy change and needs an ADR with evidence, not a palette
/// edit. It lives here rather than in the token file so that lowering it is a
/// reviewed code change rather than a data edit.
pub const COLOR_SEPARATION_FLOOR: f64 = 12.0;

/// The specification version this token vocabulary was derived from.
pub const REQUIRED_SPEC_VERSION: &str = "4.0.0";

/// The token-set vocabulary version this harness understands.
///
/// Held here for the same reason as the floors. The 2026-07-29 follow-up audit
/// found `tokenSetVersion` was only required to be non-empty — `"not-a-version"`
/// passed — while WP-030 and the audit response both described parsing as
/// "versioned". A field nothing compares against is documentation, not
/// validation.
///
/// Exact agreement, deliberately, with no forward-compatibility range: a token
/// set that says it is a different vocabulary must be re-derived against this
/// policy rather than assumed compatible, because the roster below is what
/// "compatible" would have to mean.
///
/// Version 2.0.0 is a genuine token-contract break: display text moved from
/// embedded English labels to stable label IDs, and the contract gained the
/// theme-signal, typography, text-flow, layout, cursor, and text-input values
/// needed by the native-shell feasibility work. Version 1 readers cannot
/// interpret those declarations safely, so this is not a compatible addition.
pub const REQUIRED_TOKEN_SET_VERSION: &str = "2.0.0";

/// Independent meanings of the integer measurement suffixes in the token set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeasurementUnitsRule {
    /// Unit named by ordinary `*Px` fields.
    pub px: &'static str,
    /// Unit named by `letterSpacingMilliPx`.
    pub letter_spacing_milli_px: &'static str,
    /// Unit named by `lineHeightPermille`.
    pub line_height_permille: &'static str,
}

/// Exact renderer-neutral measurement semantics required by UI-008.
pub const REQUIRED_MEASUREMENT_UNITS: MeasurementUnitsRule = MeasurementUnitsRule {
    px: "logical-pixel",
    letter_spacing_milli_px: "thousandths-of-logical-pixel",
    line_height_permille: "thousandths-of-font-size",
};

/// Threshold names a pairing may declare, and the floor each one means.
///
/// A pairing naming anything else is refused rather than skipped: a skipped
/// pairing is a promise that silently stopped being checked.
#[must_use]
pub fn threshold_for(kind: &str) -> Option<f64> {
    match kind {
        "text" => Some(WCAG_AA_NORMAL_TEXT),
        "ui" => Some(WCAG_AA_NON_TEXT),
        _ => None,
    }
}

/// One canonical foreground/background use and its independent contrast class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContrastPairRule {
    /// Foreground colour role.
    pub foreground: &'static str,
    /// Background colour role.
    pub background: &'static str,
    /// Required `text` or `ui` threshold class.
    pub kind: &'static str,
}

/// Exact current contrast-pairing roster required by UI-008.
///
/// The class is pinned with each oriented pair so normal text cannot lower its
/// own floor by relabelling itself as a UI component in the audited file.
pub const REQUIRED_CONTRAST_PAIRINGS: [ContrastPairRule; 35] = [
    ContrastPairRule {
        foreground: "text.primary",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "text.primary",
        background: "surface.raised",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "text.primary",
        background: "surface.overlay",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "text.primary",
        background: "surface.sunken",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "text.secondary",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "text.secondary",
        background: "surface.raised",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "text.muted",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "text.muted",
        background: "surface.raised",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "border.default",
        background: "surface.base",
        kind: "ui",
    },
    ContrastPairRule {
        foreground: "border.strong",
        background: "surface.base",
        kind: "ui",
    },
    ContrastPairRule {
        foreground: "focus.ring",
        background: "surface.base",
        kind: "ui",
    },
    ContrastPairRule {
        foreground: "focus.ring",
        background: "surface.raised",
        kind: "ui",
    },
    ContrastPairRule {
        foreground: "surface.sunken",
        background: "focus.ring",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "entity.device",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "entity.partition",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "entity.container",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "entity.volume",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "entity.encryption",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "entity.filesystem",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "entity.mount",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "entity.freeSpace",
        background: "surface.base",
        kind: "ui",
    },
    ContrastPairRule {
        foreground: "severity.informational",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "severity.reversible",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "severity.disruptive",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "severity.dataMoving",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "severity.destructive",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "severity.destructive",
        background: "surface.raised",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "progress.planning",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "progress.awaitingAuthorization",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "progress.executing",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "progress.verifying",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "progress.rebootPending",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "progress.recovering",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "progress.failed",
        background: "surface.base",
        kind: "text",
    },
    ContrastPairRule {
        foreground: "progress.complete",
        background: "surface.base",
        kind: "text",
    },
];

/// Themes the product must define, per UI-001.
///
/// UI-001 requires a dark charcoal default, system theme support, and an
/// accessible high-contrast theme. All three must exist and must define the
/// same roles, so a component cannot render in the default theme and fall back
/// to nothing in the theme whose entire purpose is legibility.
pub const REQUIRED_THEMES: [&str; 3] = ["dark", "high-contrast", "light"];

/// The theme whose roster the others are compared against, and the UI-001
/// default.
pub const DEFAULT_THEME: &str = "dark";

/// A theme and the stable catalogue ID for its user-facing name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThemeLabelRule {
    /// Theme identifier used by the token contract.
    pub theme: &'static str,
    /// Stable label identifier resolved by the application's string catalogue.
    pub label_id: &'static str,
}

/// Exact theme-to-label bindings required by UI-001 and UI-013.
pub const REQUIRED_THEME_LABELS: [ThemeLabelRule; 3] = [
    ThemeLabelRule {
        theme: "dark",
        label_id: "theme.dark",
    },
    ThemeLabelRule {
        theme: "high-contrast",
        label_id: "theme.highContrast",
    },
    ThemeLabelRule {
        theme: "light",
        label_id: "theme.light",
    },
];

/// Independent policy for mapping operating-system theme signals.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThemeSignalsRule {
    /// Theme used before, or in the absence of, a usable system signal.
    pub default_theme: &'static str,
    /// Catalogue label identifying the system-selection option.
    pub system_selection_label_id: &'static str,
    /// Theme selected when the system colour scheme is unknown.
    pub unknown_color_scheme_theme: &'static str,
    /// Theme selected when the system reports a dark colour scheme.
    pub dark_color_scheme_theme: &'static str,
    /// Theme selected when the system reports a light colour scheme.
    pub light_color_scheme_theme: &'static str,
    /// Separate theme selected when the system requests high contrast.
    pub high_contrast_theme: &'static str,
}

/// Exact UI-001 theme-signal mapping.
pub const REQUIRED_THEME_SIGNALS: ThemeSignalsRule = ThemeSignalsRule {
    default_theme: "dark",
    system_selection_label_id: "theme.system",
    unknown_color_scheme_theme: "dark",
    dark_color_scheme_theme: "dark",
    light_color_scheme_theme: "light",
    high_contrast_theme: "high-contrast",
};

/// Foundational visual roles that do not themselves carry semantic meaning.
///
/// Pinning these alongside the semantic roster prevents a coordinated deletion
/// from the palette and all of its pairings from shrinking the audit silently.
pub const REQUIRED_FOUNDATIONAL_COLOR_ROLES: [&str; 10] = [
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
];

/// Storage entity roles, from UI-003.
///
/// UI-003 requires physical devices, partitions, containers, volumes,
/// encryption, file systems, mounts, and free space to be distinguished
/// visually *and* textually. Each needs a role; deleting one means the product
/// cannot represent that concept, which is a specification violation and not a
/// smaller product.
pub const REQUIRED_ENTITY_ROLES: [&str; 8] = [
    "entity.device",
    "entity.partition",
    "entity.container",
    "entity.volume",
    "entity.encryption",
    "entity.filesystem",
    "entity.mount",
    "entity.freeSpace",
];

/// Severity roles, from PLAN-004's ordinal scale.
///
/// Exactly the five classes the specification defines: 0 Informational,
/// 1 Reversible, 2 Disruptive, 3 Data-moving, 4 Destructive. Not four, and not
/// six.
pub const REQUIRED_SEVERITY_ROLES: [&str; 5] = [
    "severity.informational",
    "severity.reversible",
    "severity.disruptive",
    "severity.dataMoving",
    "severity.destructive",
];

/// Progress roles, from UI-011.
///
/// UI-011 requires progress UI to distinguish planning, waiting for
/// authorization, executing, verifying, reboot pending, recovering, failed, and
/// complete.
pub const REQUIRED_PROGRESS_ROLES: [&str; 8] = [
    "progress.planning",
    "progress.awaitingAuthorization",
    "progress.executing",
    "progress.verifying",
    "progress.rebootPending",
    "progress.recovering",
    "progress.failed",
    "progress.complete",
];

/// A semantic colour role and the stable catalogue ID for its text label.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticLabelRule {
    /// Semantic colour-role identifier.
    pub role: &'static str,
    /// Stable label identifier resolved by the application's string catalogue.
    pub label_id: &'static str,
}

/// Exact semantic-role-to-label bindings required by UI-003, UI-007, UI-011,
/// UI-013, and PLAN-004.
pub const REQUIRED_SEMANTIC_LABELS: [SemanticLabelRule; 21] = [
    SemanticLabelRule {
        role: "entity.device",
        label_id: "meaning.entity.device",
    },
    SemanticLabelRule {
        role: "entity.partition",
        label_id: "meaning.entity.partition",
    },
    SemanticLabelRule {
        role: "entity.container",
        label_id: "meaning.entity.container",
    },
    SemanticLabelRule {
        role: "entity.volume",
        label_id: "meaning.entity.volume",
    },
    SemanticLabelRule {
        role: "entity.encryption",
        label_id: "meaning.entity.encryption",
    },
    SemanticLabelRule {
        role: "entity.filesystem",
        label_id: "meaning.entity.filesystem",
    },
    SemanticLabelRule {
        role: "entity.mount",
        label_id: "meaning.entity.mount",
    },
    SemanticLabelRule {
        role: "entity.freeSpace",
        label_id: "meaning.entity.freeSpace",
    },
    SemanticLabelRule {
        role: "severity.informational",
        label_id: "meaning.severity.informational",
    },
    SemanticLabelRule {
        role: "severity.reversible",
        label_id: "meaning.severity.reversible",
    },
    SemanticLabelRule {
        role: "severity.disruptive",
        label_id: "meaning.severity.disruptive",
    },
    SemanticLabelRule {
        role: "severity.dataMoving",
        label_id: "meaning.severity.dataMoving",
    },
    SemanticLabelRule {
        role: "severity.destructive",
        label_id: "meaning.severity.destructive",
    },
    SemanticLabelRule {
        role: "progress.planning",
        label_id: "meaning.progress.planning",
    },
    SemanticLabelRule {
        role: "progress.awaitingAuthorization",
        label_id: "meaning.progress.awaitingAuthorization",
    },
    SemanticLabelRule {
        role: "progress.executing",
        label_id: "meaning.progress.executing",
    },
    SemanticLabelRule {
        role: "progress.verifying",
        label_id: "meaning.progress.verifying",
    },
    SemanticLabelRule {
        role: "progress.rebootPending",
        label_id: "meaning.progress.rebootPending",
    },
    SemanticLabelRule {
        role: "progress.recovering",
        label_id: "meaning.progress.recovering",
    },
    SemanticLabelRule {
        role: "progress.failed",
        label_id: "meaning.progress.failed",
    },
    SemanticLabelRule {
        role: "progress.complete",
        label_id: "meaning.progress.complete",
    },
];

/// Complete roster of stable label IDs required by the token contract.
///
/// The three concrete theme names, the system-selection name, and the 21
/// semantic names total exactly 25 identifiers.
pub const REQUIRED_LABEL_IDS: [&str; 25] = [
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
];

/// A named font-family token and its platform resolution strategy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FontFamilyRule {
    /// Stable font-family token identifier.
    pub id: &'static str,
    /// Closed strategy name interpreted by each front end.
    pub strategy: &'static str,
}

/// The platform-native UI font family required by the shell contract.
pub const REQUIRED_FONT_FAMILIES: [FontFamilyRule; 1] = [FontFamilyRule {
    id: "platform-ui",
    strategy: "platform-default",
}];

/// One exact typography-style declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TypographyStyleRule {
    /// Stable style identifier.
    pub id: &'static str,
    /// Font-family token identifier.
    pub family: &'static str,
    /// Font size in whole logical pixels.
    pub size_px: u16,
    /// Numeric CSS-compatible font weight.
    pub weight: u16,
    /// Whether the style is italic.
    pub italic: bool,
    /// Letter spacing in thousandths of a logical pixel.
    pub letter_spacing_milli_px: i16,
    /// Line height in thousandths of the font size.
    pub line_height_permille: u16,
}

/// The seven typography styles required by the UI-008 token contract.
pub const REQUIRED_TYPOGRAPHY_STYLES: [TypographyStyleRule; 7] = [
    TypographyStyleRule {
        id: "body",
        family: "platform-ui",
        size_px: 16,
        weight: 400,
        italic: false,
        letter_spacing_milli_px: 0,
        line_height_permille: 1_500,
    },
    TypographyStyleRule {
        id: "body-small",
        family: "platform-ui",
        size_px: 14,
        weight: 400,
        italic: false,
        letter_spacing_milli_px: 0,
        line_height_permille: 1_500,
    },
    TypographyStyleRule {
        id: "caption",
        family: "platform-ui",
        size_px: 12,
        weight: 400,
        italic: false,
        letter_spacing_milli_px: 0,
        line_height_permille: 1_500,
    },
    TypographyStyleRule {
        id: "heading",
        family: "platform-ui",
        size_px: 16,
        weight: 700,
        italic: false,
        letter_spacing_milli_px: 0,
        line_height_permille: 1_250,
    },
    TypographyStyleRule {
        id: "title",
        family: "platform-ui",
        size_px: 18,
        weight: 700,
        italic: false,
        letter_spacing_milli_px: -360,
        line_height_permille: 1_200,
    },
    TypographyStyleRule {
        id: "eyebrow",
        family: "platform-ui",
        size_px: 11,
        weight: 700,
        italic: false,
        letter_spacing_milli_px: 1_000,
        line_height_permille: 1_200,
    },
    TypographyStyleRule {
        id: "exact-value",
        family: "platform-ui",
        size_px: 12,
        weight: 400,
        italic: false,
        letter_spacing_milli_px: 0,
        line_height_permille: 1_500,
    },
];

/// One exact text-flow declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextFlowRule {
    /// Stable flow identifier.
    pub id: &'static str,
    /// Closed wrapping-mode spelling.
    pub wrap: &'static str,
    /// Closed overflow-mode spelling.
    pub overflow: &'static str,
    /// Closed horizontal-alignment spelling.
    pub horizontal_alignment: &'static str,
    /// Closed vertical-alignment spelling.
    pub vertical_alignment: &'static str,
}

/// The single-line and multi-line flows required by the UI-008 contract.
pub const REQUIRED_TEXT_FLOWS: [TextFlowRule; 2] = [
    TextFlowRule {
        id: "single-line",
        wrap: "no-wrap",
        overflow: "elide",
        horizontal_alignment: "left",
        vertical_alignment: "center",
    },
    TextFlowRule {
        id: "multi-line",
        wrap: "word-wrap",
        overflow: "clip",
        horizontal_alignment: "left",
        vertical_alignment: "top",
    },
];

/// Independent text-input style, flow, and selection-colour policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextInputRule {
    /// Typography style used by text input.
    pub style: &'static str,
    /// Text-flow policy used by text input.
    pub flow: &'static str,
    /// Foreground role for selected text.
    pub selection_foreground_role: &'static str,
    /// Background role for selected text.
    pub selection_background_role: &'static str,
}

/// Exact text-input policy required by the UI-008 contract.
pub const REQUIRED_TEXT_INPUT: TextInputRule = TextInputRule {
    style: "body",
    flow: "single-line",
    selection_foreground_role: "surface.sunken",
    selection_background_role: "focus.ring",
};

/// A named whole-pixel visual dimension.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PixelTokenRule {
    /// Stable token identifier.
    pub id: &'static str,
    /// Token value in whole logical pixels.
    pub value_px: u16,
}

/// Exact spacing-token roster required by the UI-008 contract.
pub const REQUIRED_SPACING_PX: [PixelTokenRule; 7] = [
    PixelTokenRule {
        id: "none",
        value_px: 0,
    },
    PixelTokenRule {
        id: "xs",
        value_px: 4,
    },
    PixelTokenRule {
        id: "sm",
        value_px: 8,
    },
    PixelTokenRule {
        id: "md",
        value_px: 12,
    },
    PixelTokenRule {
        id: "lg",
        value_px: 16,
    },
    PixelTokenRule {
        id: "xl",
        value_px: 24,
    },
    PixelTokenRule {
        id: "xxl",
        value_px: 32,
    },
];

/// Exact corner-radius roster required by the UI-008 contract.
pub const REQUIRED_RADIUS_PX: [PixelTokenRule; 5] = [
    PixelTokenRule {
        id: "none",
        value_px: 0,
    },
    PixelTokenRule {
        id: "sm",
        value_px: 4,
    },
    PixelTokenRule {
        id: "md",
        value_px: 8,
    },
    PixelTokenRule {
        id: "lg",
        value_px: 14,
    },
    PixelTokenRule {
        id: "pill",
        value_px: 999,
    },
];

/// Exact stroke-width roster required by the UI-008 contract.
pub const REQUIRED_STROKE_PX: [PixelTokenRule; 3] = [
    PixelTokenRule {
        id: "hairline",
        value_px: 1,
    },
    PixelTokenRule {
        id: "strong",
        value_px: 2,
    },
    PixelTokenRule {
        id: "focus",
        value_px: 3,
    },
];

/// Independent layout defaults and accessible interaction dimensions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutRule {
    /// Spacing token used for default container padding.
    pub default_layout_padding: &'static str,
    /// Spacing token used between default layout children.
    pub default_layout_spacing: &'static str,
    /// Visible focus-ring offset in whole logical pixels.
    pub focus_ring_offset_px: u16,
    /// Minimum interaction-target dimension in whole logical pixels.
    pub minimum_target_size_px: u16,
}

/// Exact layout policy required by the UI-008 contract.
pub const REQUIRED_LAYOUT: LayoutRule = LayoutRule {
    default_layout_padding: "md",
    default_layout_spacing: "sm",
    focus_ring_offset_px: 3,
    minimum_target_size_px: 44,
};

/// A semantic pointer role and its closed cursor spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CursorRoleRule {
    /// Stable semantic cursor role.
    pub role: &'static str,
    /// Closed cursor value interpreted by each front end.
    pub cursor: &'static str,
}

/// Exact cursor-role mappings required by the UI-008 contract.
pub const REQUIRED_CURSOR_ROLES: [CursorRoleRule; 4] = [
    CursorRoleRule {
        role: "default",
        cursor: "default",
    },
    CursorRoleRule {
        role: "action",
        cursor: "pointer",
    },
    CursorRoleRule {
        role: "disabled",
        cursor: "not-allowed",
    },
    CursorRoleRule {
        role: "text",
        cursor: "text",
    },
];

/// Text-caret width required by the UI-008 contract, in whole logical pixels.
pub const REQUIRED_TEXT_CARET_WIDTH_PX: u16 = 2;

/// Required independent contrast check for selected text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectionContrastRule {
    /// Foreground colour role used for selected text.
    pub foreground_role: &'static str,
    /// Background colour role used for selected text.
    pub background_role: &'static str,
    /// Independent contrast-threshold name applied to the pair.
    pub threshold: &'static str,
}

/// The selected-text pair must meet normal-text contrast in every theme.
pub const REQUIRED_SELECTION_CONTRAST: SelectionContrastRule = SelectionContrastRule {
    foreground_role: "surface.sunken",
    background_role: "focus.ring",
    threshold: "text",
};

/// Role pairs whose confusion would mislead a user about risk or outcome.
///
/// Held here rather than in the token file for the same reason as the floors:
/// the audit demonstrated that a pair could be deleted from the file and the
/// harness would simply check one fewer thing and still report success. The
/// most important entry is the first — PLAN-004 severity 1, "fully undoable via
/// an emitted reversal plan", against severity 4, "data is intentionally
/// destroyed".
pub const REQUIRED_DISTINCT_PAIRS: [(&str, &str); 7] = [
    ("severity.reversible", "severity.destructive"),
    ("severity.reversible", "severity.dataMoving"),
    ("severity.disruptive", "severity.destructive"),
    ("severity.informational", "severity.destructive"),
    ("progress.complete", "progress.failed"),
    ("progress.executing", "progress.failed"),
    ("progress.complete", "progress.recovering"),
];

/// Every role the product's vocabulary requires, in one iterator.
///
/// No `#[must_use]`: `impl Iterator` already carries it.
pub fn required_meaning_bearing_roles() -> impl Iterator<Item = &'static str> {
    REQUIRED_ENTITY_ROLES
        .into_iter()
        .chain(REQUIRED_SEVERITY_ROLES)
        .chain(REQUIRED_PROGRESS_ROLES)
}

/// Every foundational and semantic colour role required by the visual policy.
///
/// The independent ten-role foundational roster and 21-role semantic roster
/// total exactly 31. No `#[must_use]`: `impl Iterator` already carries it.
pub fn required_color_roles() -> impl Iterator<Item = &'static str> {
    REQUIRED_FOUNDATIONAL_COLOR_ROLES
        .into_iter()
        .chain(required_meaning_bearing_roles())
}

/// Whether a role name carries meaning UI-007 protects.
///
/// Surfaces, text and borders do not: requiring an icon for `surface.base`
/// would be noise that trains a reader to ignore the rule.
#[must_use]
pub fn carries_meaning(role: &str) -> bool {
    vocabulary_requirement(role).is_some()
}

/// The requirement that owns a meaning-bearing role family.
///
/// This keeps audit findings honest: a missing progress state is a UI-011
/// vocabulary failure, not UI-003 merely because the same roster loop found it.
#[must_use]
pub fn vocabulary_requirement(role: &str) -> Option<&'static str> {
    if role.starts_with("entity.") {
        Some("UI-003")
    } else if role.starts_with("severity.") {
        Some("PLAN-004")
    } else if role.starts_with("progress.") {
        Some("UI-011")
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
