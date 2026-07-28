//! The SAFE-007 disposable-target interlock.
//!
//! SAFE-007 requires that the test runner refuse destructive suites unless a
//! disposable-test token, a verified image or VM target, and an explicit
//! destructive-test profile are **all** present, and states plainly that a
//! single environment variable is not sufficient proof.
//!
//! Two design rules follow, and both are load-bearing.
//!
//! **Disposability is computed, never declared.** No caller may assert that a
//! target is safe to destroy. The interlock re-reads the target and re-hashes
//! it, and accepts it only if those bytes are one of the images this
//! repository's generator produced. A block device cannot pass that test, and
//! neither can a user's disk, because neither will ever hash to a generated
//! fixture.
//!
//! **Every failure is a refusal.** An unreadable manifest, an unresolvable
//! path, a missing target — every error returns [`Refusal`], never a pass.
//! SAFE-005 requires failing closed, and an interlock that fails open under an
//! I/O error protects nothing on exactly the damaged systems where it matters.

use core::fmt;
use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest as _, Sha256};

use crate::manifest::{Manifest, hex};

/// The profile word a caller must pass to request destructive execution.
pub const DESTRUCTIVE_PROFILE: &str = "destructive";

/// Environment variable carrying the disposable-test token.
pub const TOKEN_VARIABLE: &str = "PARTMAN_DISPOSABLE_TOKEN";

/// A request to run a destructive suite.
#[derive(Clone, Debug)]
pub struct Request {
    /// The profile named on the command line. Deliberately not read from the
    /// environment: SAFE-007 rules out proving intent with one variable, and an
    /// argument cannot be inherited by accident from a parent shell.
    pub profile: Option<String>,
    /// The token supplied out of band, normally through [`TOKEN_VARIABLE`].
    pub token: Option<String>,
    /// Every target the suite intends to write to.
    pub targets: Vec<PathBuf>,
}

/// Proof that a destructive suite may run against [`Authorization::targets`].
///
/// Constructible only by [`authorize`]. A function that requires one of these
/// cannot be called without the interlock having passed, so "did anyone check?"
/// is answered by the type rather than by review.
#[derive(Clone, Debug)]
pub struct Authorization {
    targets: Vec<PathBuf>,
}

impl Authorization {
    /// The verified targets, canonicalized.
    #[must_use]
    pub fn targets(&self) -> &[PathBuf] {
        &self.targets
    }
}

/// Why a destructive run was refused.
#[derive(Debug)]
pub enum Refusal {
    /// No `--profile destructive` was given, or it named something else.
    ProfileMissing,
    /// No token was supplied.
    TokenMissing,
    /// The token did not match the generated fixture set.
    TokenMismatch,
    /// No targets were named. Refusing this is deliberate: "every target was
    /// verified" is vacuously true of an empty list, and a destructive suite
    /// that runs against nothing has no business claiming it passed the
    /// interlock.
    NoTargets,
    /// The fixture manifest could not be read or parsed.
    ManifestUnreadable(String),
    /// A target's path could not be resolved.
    TargetUnresolvable {
        /// The path as supplied.
        path: PathBuf,
        /// The underlying reason.
        reason: String,
    },
    /// A target resolved outside the generated-fixture root.
    TargetOutsideRoot {
        /// The path as supplied.
        path: PathBuf,
    },
    /// A target is not a regular file — a device, directory, or symlink.
    TargetNotRegularFile {
        /// The path as supplied.
        path: PathBuf,
    },
    /// A target is not, by name and bytes, an image this build generates.
    TargetNotGenerated {
        /// The path as supplied.
        path: PathBuf,
    },
    /// A target is reachable under more than one name.
    TargetHasOtherNames {
        /// The path as supplied.
        path: PathBuf,
        /// How many names refer to this file.
        links: u64,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProfileMissing => write!(
                formatter,
                "no destructive profile: pass `--profile {DESTRUCTIVE_PROFILE}`"
            ),
            Self::TokenMissing => write!(
                formatter,
                "no disposable-test token: set {TOKEN_VARIABLE} from the generated MANIFEST"
            ),
            Self::TokenMismatch => write!(
                formatter,
                "{TOKEN_VARIABLE} does not match the generated fixture set; regenerate with \
                 `cargo xtask fixtures` and use the token it records"
            ),
            Self::NoTargets => formatter
                .write_str("no targets named; a destructive suite must state what it writes to"),
            Self::ManifestUnreadable(reason) => {
                write!(formatter, "fixture manifest unusable: {reason}")
            }
            Self::TargetUnresolvable { path, reason } => {
                write!(formatter, "cannot resolve {}: {reason}", path.display())
            }
            Self::TargetOutsideRoot { path } => write!(
                formatter,
                "{} is outside the generated-fixture directory",
                path.display()
            ),
            Self::TargetNotRegularFile { path } => write!(
                formatter,
                "{} is not a regular file; destructive suites never address a device",
                path.display()
            ),
            Self::TargetNotGenerated { path } => write!(
                formatter,
                "{} is not, by name and bytes, an image this build generates",
                path.display()
            ),
            Self::TargetHasOtherNames { path, links } => write!(
                formatter,
                "{} is reachable under {links} names; a destructive suite must address a file \
                 with exactly one",
                path.display()
            ),
        }
    }
}

impl std::error::Error for Refusal {}

/// Decide whether a destructive suite may run.
///
/// `root` is the generated-fixture directory. `manifest` is the manifest the
/// generator wrote alongside those images.
///
/// # Errors
///
/// Returns [`Refusal`] whenever any of SAFE-007's three factors is absent or
/// any check cannot be completed.
pub fn authorize(root: &Path, request: &Request) -> Result<Authorization, Refusal> {
    // Expectations come from the compiled catalogue, never from a file inside
    // the directory being verified. Accepting a caller-supplied manifest was the
    // defect that let a hand-written one authorize an arbitrary target.
    let manifest = &crate::catalogue::expected();

    // Factor 1: an explicit profile, from the command line.
    if request.profile.as_deref() != Some(DESTRUCTIVE_PROFILE) {
        return Err(Refusal::ProfileMissing);
    }

    // Factor 2: a token that matches this fixture set.
    let token = request.token.as_deref().ok_or(Refusal::TokenMissing)?;
    if !constant_time_eq(token.as_bytes(), manifest.token().as_bytes()) {
        return Err(Refusal::TokenMismatch);
    }

    // Factor 3: every target verified, and there must be at least one.
    if request.targets.is_empty() {
        return Err(Refusal::NoTargets);
    }

    let root = root
        .canonicalize()
        .map_err(|error| Refusal::ManifestUnreadable(format!("fixture root: {error}")))?;

    let mut verified = Vec::with_capacity(request.targets.len());
    for target in &request.targets {
        verified.push(verify_target(&root, manifest, target)?);
    }

    Ok(Authorization { targets: verified })
}

/// Verify one target, returning its canonical path.
fn verify_target(root: &Path, manifest: &Manifest, target: &Path) -> Result<PathBuf, Refusal> {
    // `symlink_metadata` does not follow links, so a symlink aimed at a device
    // is rejected here rather than silently resolved into one.
    let link_metadata =
        std::fs::symlink_metadata(target).map_err(|error| unresolvable(target, &error))?;
    if !link_metadata.is_file() {
        return Err(Refusal::TargetNotRegularFile {
            path: target.to_path_buf(),
        });
    }

    // Canonicalize only after establishing it is a regular file, then confirm it
    // is inside the fixture root. Doing this on the resolved path is what makes
    // `..` traversal and a relative path from an unexpected working directory
    // both harmless.
    let resolved = target
        .canonicalize()
        .map_err(|error| unresolvable(target, &error))?;
    if !resolved.starts_with(root) {
        return Err(Refusal::TargetOutsideRoot {
            path: target.to_path_buf(),
        });
    }

    let metadata = std::fs::metadata(&resolved).map_err(|error| unresolvable(target, &error))?;
    if !metadata.is_file() {
        return Err(Refusal::TargetNotRegularFile {
            path: target.to_path_buf(),
        });
    }

    // A hard link is a regular file, and canonicalizing one still yields a path
    // under the root, so neither check above sees it. Requiring the content to
    // equal a generated fixture already means a link can only ever point at
    // something that *is* a fixture — but a second name for the file is still a
    // second thing a destructive suite could reach, so refuse it where the
    // platform will say.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        if metadata.nlink() > 1 {
            return Err(Refusal::TargetHasOtherNames {
                path: target.to_path_buf(),
                links: metadata.nlink(),
            });
        }
    }

    // The decisive check, and it is now anchored to the compiled catalogue: this
    // file must be a fixture *by name*, and its length and bytes must be exactly
    // that fixture's. Membership by digest alone was too weak — it let any file
    // pass so long as some entry, anywhere in the manifest, shared its digest.
    let name = resolved
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Refusal::TargetNotGenerated {
            path: target.to_path_buf(),
        })?;
    let entry = manifest
        .entry(name)
        .ok_or_else(|| Refusal::TargetNotGenerated {
            path: target.to_path_buf(),
        })?;

    // The path must be *exactly* the one this fixture is generated at, not
    // merely somewhere beneath the root with the right file name. `starts_with`
    // plus `file_name` admits `<root>/sub/blank-512.img` — an ordinary copy in a
    // subdirectory, with a matching name, length, and digest. That combination
    // was verified to authorize before this equality replaced it.
    if resolved != root.join(name) {
        return Err(Refusal::TargetOutsideRoot {
            path: target.to_path_buf(),
        });
    }

    if metadata.len() != entry.length {
        return Err(Refusal::TargetNotGenerated {
            path: target.to_path_buf(),
        });
    }

    let bytes = std::fs::read(&resolved).map_err(|error| unresolvable(target, &error))?;
    let digest = hex(&Sha256::digest(&bytes));
    if !constant_time_eq(digest.as_bytes(), entry.digest.as_bytes()) {
        return Err(Refusal::TargetNotGenerated {
            path: target.to_path_buf(),
        });
    }

    Ok(resolved)
}

fn unresolvable(path: &Path, error: &io::Error) -> Refusal {
    Refusal::TargetUnresolvable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    }
}

/// Compare two byte strings without an early exit.
///
/// The token is not a secret in the cryptographic sense, but a length-and-prefix
/// comparison invites the habit of writing one where it does matter.
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0_u8;
    for (a, b) in left.iter().zip(right) {
        difference |= a ^ b;
    }
    difference == 0
}

#[cfg(test)]
mod tests;
