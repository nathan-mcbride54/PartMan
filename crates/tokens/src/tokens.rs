//! Reading `schemas/design-tokens.json`.
//!
//! The reader is strict in the same way the `pce/1` decoder is strict: it
//! refuses rather than repairs. A token file that names a colour role which
//! does not exist, or a pairing whose threshold is unknown, is an error and not
//! a row to skip — a skipped row is a requirement that silently stops being
//! checked, which is the failure mode this whole crate exists to prevent.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::color::{ColorError, Srgb};

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
    /// Every theme, keyed by name. `dark` is the UI-001 default.
    pub themes: BTreeMap<String, Theme>,
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
    /// Human-readable name.
    pub label: String,
    /// The requirement this theme satisfies.
    pub requirement: String,
    /// Role name to `#RRGGBB`.
    pub colors: BTreeMap<String, String>,
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
    pub roles: BTreeMap<String, Channels>,
}

/// The channels that carry a role's meaning when colour cannot.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Channels {
    /// Icon identifier.
    pub icon: String,
    /// Visible text label.
    pub label: String,
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
