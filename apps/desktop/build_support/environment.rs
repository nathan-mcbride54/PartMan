//! Dependency-free environment policy shared by the Slint build and its auditors.
//!
//! Slint's compiler and future runtime both inspect `SLINT_*` variables. PartMan
//! refuses every such variable before constructing either boundary so ambient
//! developer or deployment state cannot silently change compiled or rendered
//! behavior. The one non-prefix `DEP_MCU_*` input read by the pinned compiler is
//! rejected as well. The PartMan nonce is rejected too: CI uses it to prove the
//! guard ran, never as configuration.

use std::ffi::{OsStr, OsString};
use std::fmt;

/// The CI-only name that proves the ambient-variable guard ran.
pub const PARTMAN_SLINT_GUARD_NONCE: &str = "PARTMAN_SLINT_GUARD_NONCE";

/// Non-prefix input read by `CompilerConfiguration::new` in Slint 1.17.1.
///
/// Its presence selects embedded textures and panics when the compiler is built
/// without the software-renderer feature, so it is governed exactly like the
/// `SLINT_*` namespace.
pub const DEP_MCU_EMBED_TEXTURES: &str = "DEP_MCU_BOARD_SUPPORT_MCU_EMBED_TEXTURES";

/// Environment names whose presence Cargo must treat as a build invalidation.
///
/// The guard rejects the complete `SLINT_*` namespace, including names added by
/// later Slint versions. This fixed roster records every name known to the
/// pinned compiler/runtime and lets Cargo re-run the build when a previously
/// absent known name appears. The nonce is listed separately by
/// [`PARTMAN_SLINT_GUARD_NONCE`].
#[allow(
    dead_code,
    reason = "the outer xtask shares the prefix guard but does not emit Cargo invalidation directives"
)]
pub const KNOWN_SLINT_ENVIRONMENT_NAMES: &[&str] = &[
    "SLINT_ASSET_SECTION",
    "SLINT_BACKEND",
    "SLINT_BUNDLE_TRANSLATIONS",
    "SLINT_COMPILER_DENY_WARNINGS",
    "SLINT_CPP_NAMESPACE",
    "SLINT_DEBUG_PERFORMANCE",
    "SLINT_DEFAULT_FONT",
    "SLINT_DESTROY_WINDOW_ON_HIDE",
    "SLINT_EMBED_RESOURCES",
    "SLINT_EMBED_TEXTURES",
    "SLINT_EMIT_DEBUG_INFO",
    "SLINT_ENABLE_EXPERIMENTAL_FEATURES",
    "SLINT_FONT_PATH",
    "SLINT_FONT_SIZES",
    "SLINT_FULLSCREEN",
    "SLINT_INCLUDE_GENERATED",
    "SLINT_INLINING",
    "SLINT_LINE_BY_LINE",
    "SLINT_LIVE_PREVIEW",
    "SLINT_MACRO_CACHE",
    "SLINT_SCALE_FACTOR",
    "SLINT_SLOW_ANIMATIONS",
    "SLINT_SOFTWARE_RENDERER_PARLEY_DISABLED",
    "SLINT_STYLE",
    "SLINT_WGPU_CPU",
];

/// Name-comparison rules used by an operating system's environment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameSemantics {
    /// Windows environment names are ASCII-case-insensitive for this policy.
    Windows,
    /// Unix environment names are byte strings and are compared byte-exactly.
    Unix,
}

/// A forbidden environment name found without reading or retaining its value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForbiddenEnvironmentName {
    name: OsString,
}

impl ForbiddenEnvironmentName {
    /// Returns the rejected name. Its associated value is deliberately absent.
    #[must_use]
    #[allow(
        dead_code,
        reason = "used by path-including audit/test consumers, not build.rs"
    )]
    pub fn name(&self) -> &OsStr {
        &self.name
    }
}

impl fmt::Display for ForbiddenEnvironmentName {
    #[allow(
        clippy::unnecessary_debug_formatting,
        reason = "Debug escapes non-Unicode and control characters in an untrusted environment name"
    )]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "ambient environment name {:?} is forbidden by the Slint boundary",
            self.name
        )
    }
}

impl std::error::Error for ForbiddenEnvironmentName {}

/// Classify a byte-oriented name under explicit platform semantics.
///
/// This is the portable proof surface for Unix names that are not Unicode.
/// Windows callers should pass the UTF-8 bytes of a Unicode environment name;
/// only the ASCII policy prefix and nonce participate in comparison.
#[must_use]
pub fn is_forbidden_name_bytes(name: &[u8], semantics: NameSemantics) -> bool {
    const PREFIX: &[u8] = b"SLINT_";
    let prefix_matches = match semantics {
        NameSemantics::Windows => name
            .get(..PREFIX.len())
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(PREFIX)),
        NameSemantics::Unix => name.starts_with(PREFIX),
    };
    prefix_matches
        || names_equal(name, PARTMAN_SLINT_GUARD_NONCE.as_bytes(), semantics)
        || names_equal(name, DEP_MCU_EMBED_TEXTURES.as_bytes(), semantics)
}

/// Reject forbidden names from an iterator of environment name/value pairs.
///
/// Values are accepted only to match [`std::env::vars_os`]; they are never
/// inspected, cloned, retained, or included in diagnostics.
pub fn guard_environment_entries<I, K, V>(
    entries: I,
    semantics: NameSemantics,
) -> Result<(), ForbiddenEnvironmentName>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
{
    for (name, _value) in entries {
        if is_forbidden_os_name(name.as_ref(), semantics) {
            return Err(ForbiddenEnvironmentName {
                name: name.as_ref().to_os_string(),
            });
        }
    }
    Ok(())
}

/// Reject forbidden variables in the current process environment.
pub fn guard_current_environment() -> Result<(), ForbiddenEnvironmentName> {
    guard_environment_entries(std::env::vars_os(), current_name_semantics())
}

fn names_equal(left: &[u8], right: &[u8], semantics: NameSemantics) -> bool {
    match semantics {
        NameSemantics::Windows => left.eq_ignore_ascii_case(right),
        NameSemantics::Unix => left == right,
    }
}

fn current_name_semantics() -> NameSemantics {
    if cfg!(windows) {
        NameSemantics::Windows
    } else {
        NameSemantics::Unix
    }
}

#[cfg(unix)]
fn is_forbidden_os_name(name: &OsStr, semantics: NameSemantics) -> bool {
    use std::os::unix::ffi::OsStrExt;

    is_forbidden_name_bytes(name.as_bytes(), semantics)
}

#[cfg(not(unix))]
fn is_forbidden_os_name(name: &OsStr, semantics: NameSemantics) -> bool {
    is_forbidden_name_bytes(name.to_string_lossy().as_bytes(), semantics)
}
