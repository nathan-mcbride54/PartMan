//! Reading `schemas/design-tokens.json`.
//!
//! The reader is strict in the same way the `pce/1` decoder is strict: it
//! refuses rather than repairs. A token file that names a colour role which
//! does not exist, or a pairing whose threshold is unknown, is an error and not
//! a row to skip — a skipped row is a requirement that silently stops being
//! checked, which is the failure mode this whole crate exists to prevent.

use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use serde::de::{self, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::color::{ColorError, Srgb};

/// Deserialize a string-keyed map without accepting JSON's ambiguous duplicate
/// member names.
///
/// `serde_json` normally resolves a duplicate map key last-wins when the target
/// is a `BTreeMap`. That would let two front ends interpret the canonical token
/// source differently, so every map-shaped token namespace uses this reader.
fn deserialize_unique_map<'de, D, V>(deserializer: D) -> Result<BTreeMap<String, V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct UniqueMapVisitor<V>(PhantomData<fn() -> V>);

    impl<'de, V> Visitor<'de> for UniqueMapVisitor<V>
    where
        V: Deserialize<'de>,
    {
        type Value = BTreeMap<String, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map with unique string keys")
        }

        fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut values = BTreeMap::new();
            while let Some(key) = access.next_key::<String>()? {
                match values.entry(key) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(access.next_value::<V>()?);
                    }
                    std::collections::btree_map::Entry::Occupied(entry) => {
                        let duplicate = entry.key().clone();
                        let _: IgnoredAny = access.next_value()?;
                        return Err(de::Error::custom(format_args!(
                            "duplicate map key {duplicate:?}"
                        )));
                    }
                }
            }
            Ok(values)
        }
    }

    deserializer.deserialize_map(UniqueMapVisitor(PhantomData))
}

/// The parsed token set.
///
/// `deny_unknown_fields` throughout: the module documentation calls this reader
/// strict, and until the 2026-07-29 audit it was not — an unrecognised key was
/// silently dropped, so a misspelled `nonColorChannels` would have disabled
/// UI-007 while the file still looked complete. Prose lives in explicit `note`
/// fields so that denying the rest costs nothing.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenSet {
    /// Version of the token vocabulary itself.
    #[serde(rename = "tokenSetVersion")]
    pub token_set_version: String,
    /// The `AGENT_BUILD_SPEC.md` version this set was written against.
    ///
    /// Checked against [`crate::policy::REQUIRED_SPEC_VERSION`] by the audit:
    /// the roles here are derived from UI-003, PLAN-004 and UI-011, so a set
    /// claiming a different specification version has to be re-derived rather
    /// than assumed compatible.
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    /// Free prose for a human reader. Carried so it can be denied elsewhere.
    #[serde(default)]
    pub note: String,
    /// Canonical meanings of the integer measurement suffixes used below.
    #[serde(rename = "measurementUnits")]
    pub measurement_units: MeasurementUnits,
    /// Every theme, keyed by name. `dark` is the UI-001 default.
    #[serde(deserialize_with = "deserialize_unique_map")]
    pub themes: BTreeMap<String, Theme>,
    /// Renderer-neutral mappings from platform theme signals to canonical themes.
    #[serde(rename = "themeSignals")]
    pub theme_signals: ThemeSignals,
    /// Renderer-neutral type families, styles, flows, and text-input tokens.
    pub typography: Typography,
    /// Renderer-neutral spacing, radius, stroke, focus, and target-size tokens.
    pub layout: Layout,
    /// Renderer-neutral caret and pointer-role tokens.
    pub cursors: Cursors,
    /// The WCAG rules the harness enforces.
    #[serde(rename = "contrastRules")]
    pub contrast_rules: ContrastRules,
    /// UI-007's redundant channels.
    #[serde(rename = "nonColorChannels")]
    pub non_color_channels: NonColorChannels,
    /// The colour-vision smell test.
    #[serde(rename = "colorVisionSeparation")]
    pub color_vision_separation: ColorVisionSeparation,
}

/// One theme's colour roles.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    /// Externalized localization identifier for the theme name.
    #[serde(rename = "labelId")]
    pub label_id: String,
    /// Role name to `#RRGGBB`.
    #[serde(deserialize_with = "deserialize_unique_map")]
    pub colors: BTreeMap<String, String>,
}

/// Renderer-neutral meanings of the integer measurement suffixes.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementUnits {
    /// Unit named by ordinary `*Px` fields: one logical pixel.
    pub px: String,
    /// Unit named by `letterSpacingMilliPx`.
    #[serde(rename = "letterSpacingMilliPx")]
    pub letter_spacing_milli_px: String,
    /// Unit named by `lineHeightPermille`.
    #[serde(rename = "lineHeightPermille")]
    pub line_height_permille: String,
}

/// Canonical theme identifiers accepted by the renderer-neutral contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeId {
    /// The dark charcoal theme.
    Dark,
    /// The dedicated high-contrast theme.
    HighContrast,
    /// The light theme.
    Light,
}

/// Theme choices derived from platform color-scheme signals.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeSignals {
    /// Theme used when no platform preference is selected.
    #[serde(rename = "defaultTheme")]
    pub default_theme: ThemeId,
    /// Externalized label identifier for the system-theme choice.
    #[serde(rename = "systemSelectionLabelId")]
    pub system_selection_label_id: String,
    /// Mapping from every supported system color-scheme signal.
    #[serde(rename = "systemColorScheme")]
    pub system_color_scheme: SystemColorScheme,
    /// Theme selected by the separate high-contrast signal.
    #[serde(rename = "highContrastTheme")]
    pub high_contrast_theme: ThemeId,
}

/// Canonical interpretation of platform color-scheme values.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemColorScheme {
    /// Theme selected when the platform color scheme is unknown.
    pub unknown: ThemeId,
    /// Theme selected when the platform requests a light scheme.
    pub light: ThemeId,
    /// Theme selected when the platform requests a dark scheme.
    pub dark: ThemeId,
}

/// Renderer-neutral typography declarations.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Typography {
    /// Named font-family strategies.
    #[serde(deserialize_with = "deserialize_unique_map")]
    pub families: BTreeMap<String, FontFamily>,
    /// Named text styles.
    #[serde(deserialize_with = "deserialize_unique_map")]
    pub styles: BTreeMap<String, TextStyle>,
    /// Named text-flow policies.
    #[serde(deserialize_with = "deserialize_unique_map")]
    pub flows: BTreeMap<String, TextFlow>,
    /// Tokens applied to editable text controls.
    #[serde(rename = "textInput")]
    pub text_input: TextInputTokens,
}

/// One renderer-neutral font family.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FontFamily {
    /// How the renderer resolves the family.
    pub strategy: FontFamilyStrategy,
}

/// Closed font-family resolution strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "kebab-case")]
pub enum FontFamilyStrategy {
    /// Use the platform's default user-interface font.
    PlatformDefault,
}

/// One named text style, expressed with integer units.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextStyle {
    /// Name of the font family this style uses.
    pub family: String,
    /// Font size in whole logical pixels.
    #[serde(rename = "sizePx")]
    pub size_px: u16,
    /// Numeric font weight.
    pub weight: u16,
    /// Whether the style requests italic text.
    pub italic: bool,
    /// Letter spacing in thousandths of one logical pixel.
    #[serde(rename = "letterSpacingMilliPx")]
    pub letter_spacing_milli_px: i16,
    /// Line height in thousandths of the font size.
    #[serde(rename = "lineHeightPermille")]
    pub line_height_permille: u16,
}

/// One named text-flow policy.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextFlow {
    /// Line-wrapping policy.
    pub wrap: TextWrap,
    /// Overflow policy.
    pub overflow: TextOverflow,
    /// Horizontal alignment policy.
    #[serde(rename = "horizontalAlignment")]
    pub horizontal_alignment: HorizontalAlignment,
    /// Vertical alignment policy.
    #[serde(rename = "verticalAlignment")]
    pub vertical_alignment: VerticalAlignment,
}

/// Closed line-wrapping policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextWrap {
    /// Keep content on one line.
    NoWrap,
    /// Wrap at word boundaries.
    WordWrap,
}

/// Closed text-overflow policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextOverflow {
    /// Elide content that exceeds its available width.
    Elide,
    /// Clip content that exceeds its available bounds.
    Clip,
}

/// Closed horizontal text-alignment policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HorizontalAlignment {
    /// Align text to the leading left edge.
    Left,
}

/// Closed vertical text-alignment policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerticalAlignment {
    /// Center text within its bounds.
    Center,
    /// Align text to the top edge.
    Top,
}

/// Typography tokens for editable text controls.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextInputTokens {
    /// Name of the text style used by the control.
    pub style: String,
    /// Name of the flow policy used by the control.
    pub flow: String,
    /// Foreground/background pair used for selected text.
    #[serde(rename = "selectionPair")]
    pub selection_pair: SelectionPair,
}

/// One foreground/background color-role pair.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionPair {
    /// Foreground color role.
    pub foreground: String,
    /// Background color role.
    pub background: String,
}

/// Renderer-neutral layout tokens.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    /// Named spacing values in whole logical pixels.
    #[serde(rename = "spacingPx", deserialize_with = "deserialize_unique_map")]
    pub spacing_px: BTreeMap<String, u16>,
    /// Named corner-radius values in whole logical pixels.
    #[serde(rename = "radiusPx", deserialize_with = "deserialize_unique_map")]
    pub radius_px: BTreeMap<String, u16>,
    /// Named stroke widths in whole logical pixels.
    #[serde(rename = "strokePx", deserialize_with = "deserialize_unique_map")]
    pub stroke_px: BTreeMap<String, u16>,
    /// Spacing-token name used for default layout padding.
    #[serde(rename = "defaultLayoutPadding")]
    pub default_layout_padding: String,
    /// Spacing-token name used for default layout spacing.
    #[serde(rename = "defaultLayoutSpacing")]
    pub default_layout_spacing: String,
    /// Gap between a focused control and its focus ring, in logical pixels.
    #[serde(rename = "focusRingOffsetPx")]
    pub focus_ring_offset_px: u16,
    /// Minimum interactive target dimension in logical pixels.
    #[serde(rename = "minimumTargetSizePx")]
    pub minimum_target_size_px: u16,
}

/// Renderer-neutral caret and pointer-role tokens.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cursors {
    /// Editable-text caret width in whole logical pixels.
    #[serde(rename = "textCaretWidthPx")]
    pub text_caret_width_px: u16,
    /// Semantic pointer roles and their closed cursor values.
    #[serde(deserialize_with = "deserialize_unique_map")]
    pub roles: BTreeMap<String, CursorKind>,
}

/// Closed cursor values supported by the token contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CursorKind {
    /// Platform default pointer.
    Default,
    /// Pointer for an actionable control.
    Pointer,
    /// Pointer indicating an unavailable action.
    NotAllowed,
    /// Text-selection pointer.
    Text,
}

/// WCAG thresholds and the pairings they apply to.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContrastRules {
    /// Free prose for a human reader.
    #[serde(default)]
    pub note: String,
    /// Threshold name (`text`, `ui`) to minimum ratio.
    ///
    /// **Restated for a front end to read, not authoritative.** The audit takes
    /// its floors from [`crate::policy`] and requires these to agree; the file
    /// may not lower the standard it is judged by.
    #[serde(deserialize_with = "deserialize_unique_map")]
    pub thresholds: BTreeMap<String, f64>,
    /// Every foreground/background pair the product promises to render.
    pub pairings: Vec<Pairing>,
}

/// One foreground/background promise.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pairing {
    /// Role drawn on top.
    pub foreground: String,
    /// Role drawn underneath.
    pub background: String,
    /// Which threshold applies.
    pub kind: String,
}

/// UI-007's table of redundant, non-colour channels.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NonColorChannels {
    /// Free prose for a human reader.
    #[serde(default)]
    pub note: String,
    /// Role name to the channels that must accompany it.
    #[serde(deserialize_with = "deserialize_unique_map")]
    pub roles: BTreeMap<String, Channels>,
}

/// The channels that carry a role's meaning when colour cannot.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Channels {
    /// Icon identifier.
    pub icon: String,
    /// Externalized localization identifier for the visible text label.
    #[serde(rename = "labelId")]
    pub label_id: String,
    /// Shape or outline treatment.
    pub shape: String,
}

/// Roles that must not collapse onto one another under simulated colour-vision
/// deficiency.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorVisionSeparation {
    /// Free prose for a human reader.
    #[serde(default)]
    pub note: String,
    /// CIE76 floor.
    ///
    /// **Restated, not authoritative.** See
    /// [`crate::policy::COLOR_SEPARATION_FLOOR`].
    #[serde(rename = "minimumDeltaE")]
    pub minimum_delta_e: f64,
    /// Pairs whose confusion would mislead about risk or outcome.
    #[serde(rename = "mustRemainDistinct")]
    pub must_remain_distinct: Vec<[String; 2]>,
}

/// Why a token set could not be loaded.
#[derive(Debug)]
pub enum TokenError {
    /// The file could not be read.
    Read {
        /// The file that could not be read.
        path: PathBuf,
        /// The underlying I/O failure.
        source: std::io::Error,
    },
    /// The file was not well-formed JSON, or did not match the shape above.
    Parse {
        /// The file that could not be parsed.
        path: PathBuf,
        /// The underlying deserialization failure.
        source: serde_json::Error,
    },
    /// A colour string was malformed.
    Color {
        /// Theme the bad colour was found in.
        theme: String,
        /// Role the bad colour was declared for.
        role: String,
        /// Why the colour could not be read.
        source: ColorError,
    },
}

impl fmt::Display for TokenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(formatter, "cannot read {}: {source}", path.display())
            }
            Self::Parse { path, source } => {
                write!(formatter, "cannot parse {}: {source}", path.display())
            }
            Self::Color {
                theme,
                role,
                source,
            } => write!(formatter, "theme {theme:?} role {role:?}: {source}"),
        }
    }
}

impl std::error::Error for TokenError {}

impl TokenSet {
    /// Read and validate a token file.
    ///
    /// # Errors
    ///
    /// [`TokenError::Read`] if the file cannot be opened, [`TokenError::Parse`]
    /// if it is not well-formed JSON of the expected shape, and
    /// [`TokenError::Color`] if any colour in any theme is malformed. Every
    /// colour is parsed here rather than at first use, so a bad value in a role
    /// nothing currently pairs is still refused.
    pub fn load(path: &Path) -> Result<Self, TokenError> {
        let text = std::fs::read_to_string(path).map_err(|source| TokenError::Read {
            path: path.to_owned(),
            source,
        })?;
        let set: Self = serde_json::from_str(&text).map_err(|source| TokenError::Parse {
            path: path.to_owned(),
            source,
        })?;
        // Parse every colour now rather than at first use. A malformed colour
        // in a theme nothing currently pairs would otherwise sit undetected
        // until the day something paired it.
        for (theme_name, theme) in &set.themes {
            for (role, value) in &theme.colors {
                Srgb::parse(value).map_err(|source| TokenError::Color {
                    theme: theme_name.clone(),
                    role: role.clone(),
                    source,
                })?;
            }
        }
        Ok(set)
    }

    /// The token file this repository ships.
    ///
    /// # Errors
    ///
    /// As [`TokenSet::load`].
    pub fn load_repository_tokens() -> Result<Self, TokenError> {
        Self::load(&repository_token_path())
    }

    /// Resolve a role to a colour within `theme`.
    #[must_use]
    pub fn color(&self, theme: &str, role: &str) -> Option<Srgb> {
        self.themes
            .get(theme)?
            .colors
            .get(role)
            .and_then(|value| Srgb::parse(value).ok())
    }
}

/// Path to `schemas/design-tokens.json`, resolved from this crate's location so
/// it does not depend on the working directory a test or task was started in.
#[must_use]
pub fn repository_token_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas/design-tokens.json")
        .clean()
}

/// Minimal path normalisation, so error messages name a readable path rather
/// than one threaded with `../`.
trait Clean {
    fn clean(self) -> PathBuf;
}

impl Clean for PathBuf {
    fn clean(self) -> PathBuf {
        let mut parts: Vec<std::ffi::OsString> = Vec::new();
        for part in self.components() {
            match part {
                std::path::Component::ParentDir if !parts.is_empty() => {
                    parts.pop();
                }
                other => parts.push(other.as_os_str().to_owned()),
            }
        }
        parts.into_iter().collect()
    }
}

#[cfg(test)]
mod tests;
