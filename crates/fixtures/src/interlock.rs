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
//!
//! **Authorization holds the object it verified, not the name it found it
//! under.** Until 2026-07-29 [`Authorization`] carried a `Vec<PathBuf>` — a
//! list of names — and the 2026-07-29 project audit called that the most
//! important precondition before any Tier-2 write, because a name can be
//! rebound between verification and use. Every check that decides
//! disposability now runs against an **open file handle**: `fstat` through the
//! handle says regular file, the length and every byte are read through the
//! handle, and the same handle — the verified object itself — is what the
//! destructive consumer receives. Renaming or swapping the path afterwards
//! changes which object the *name* refers to; it cannot change which object
//! the authorization holds. On Windows the handle is opened with a share mode
//! that additionally refuses concurrent writes, deletion, and renames — via
//! any name, including a hard link — for as long as the authorization lives.
//! The path checks (fixture root, exact expected location, symlink refusal)
//! are kept as hygiene, but nothing safety-relevant rests on a path once the
//! handle is open.

use core::fmt;
use std::fs::File;
use std::io::{self, Read as _, Seek as _, SeekFrom};
use std::path::{Path, PathBuf};

use sha2::{Digest as _, Sha256};

use crate::manifest::{Manifest, hex};

/// The profile word a caller must pass to request destructive execution.
pub const DESTRUCTIVE_PROFILE: &str = "destructive";

/// `FILE_SHARE_READ`: readers permitted, writers and deleters refused.
///
/// Written out because Rust's standard library exposes `share_mode` without
/// exposing the constants, and pulling in `windows-sys` for two integers would
/// be a larger change than it saves. Both values are fixed Win32 API contract
/// and have not moved since NT.
#[cfg(windows)]
const FILE_SHARE_READ: u32 = 0x0000_0001;

/// `FILE_FLAG_OPEN_REPARSE_POINT`: open a reparse point rather than follow it.
///
/// The Windows analogue of `O_NOFOLLOW`. Verified behaviourally on Unix by
/// [`tests::a_symlink_swapped_in_before_open_is_refused`]; on Windows creating
/// a symlink needs `SeCreateSymbolicLinkPrivilege`, which a CI runner cannot be
/// relied on to hold, so that platform's refusal rests on this flag plus the
/// standard library reporting a reparse point as a symlink — recorded as such
/// rather than claimed as tested.
#[cfg(windows)]
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;

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

/// The fixture directory, held open as an object.
///
/// Increment 2b bound every check to the target's handle and still opened that
/// handle by absolute pathname, which the 2026-07-29 second follow-up audit
/// showed is not enough: `O_NOFOLLOW` constrains only the **final** path
/// component, so renaming the fixture root aside and leaving a symlink at its
/// name redirects the open to an out-of-root file whose length, digest, type
/// and link count all match. Containment cannot be established by any check on
/// the object, because a user's ordinary file may hold a fixture's exact bytes —
/// `object_verification_alone_cannot_prove_root_membership` records that
/// directly.
///
/// So the directory itself becomes an object, and targets are opened *relative
/// to it* by catalogue basename. Path resolution then starts from a handle
/// nothing can rebind, rather than from a name that can be.
#[derive(Debug)]
pub struct RootDirectory {
    /// The directory handle. Unix only: this is what `openat` resolves from.
    #[cfg(unix)]
    handle: File,
    /// The path the directory was opened at, for reporting and for the
    /// Windows fallback below.
    path: PathBuf,
}

impl RootDirectory {
    /// Open the fixture root and hold it.
    fn open(root: &Path) -> Result<Self, Refusal> {
        let path = root
            .canonicalize()
            .map_err(|error| Refusal::ManifestUnreadable(format!("fixture root: {error}")))?;
        #[cfg(unix)]
        {
            let handle = File::open(&path)
                .map_err(|error| Refusal::ManifestUnreadable(format!("fixture root: {error}")))?;
            Ok(Self { handle, path })
        }
        #[cfg(not(unix))]
        {
            Ok(Self { path })
        }
    }

    /// Open a direct child by name, refusing to follow a link at that name.
    ///
    /// `name` is a catalogue basename and must contain no separator, so there
    /// are no intermediate components for anything to redirect.
    fn open_child(&self, name: &str, target: &Path) -> Result<File, Refusal> {
        if name.contains('/') || name.contains('\\') {
            return Err(Refusal::TargetOutsideRoot {
                path: target.to_path_buf(),
            });
        }

        #[cfg(unix)]
        {
            // Resolution starts at the held directory, so an intermediate
            // component cannot be swapped: there are no intermediate
            // components. `NOFOLLOW` covers the last one. No `unsafe` appears
            // here or anywhere in this crate — `rustix` is a safe wrapper, so
            // SAFE-009's prohibition needs no exception.
            use rustix::fs::{Mode, OFlags};
            let opened = rustix::fs::openat(
                &self.handle,
                name,
                OFlags::RDWR | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|error| Refusal::TargetUnresolvable {
                path: target.to_path_buf(),
                reason: error.to_string(),
            })?;
            Ok(File::from(opened))
        }

        #[cfg(not(unix))]
        {
            // **Windows is not yet closed, and this is the residual.** There is
            // no stable, safe handle-relative open in the standard library, and
            // the `NtCreateFile` route needs FFI, which SAFE-009 permits only
            // in an adapter/FFI/helper crate — not here. So this platform still
            // opens by pathname and remains exposed to a swapped root
            // directory. Recorded in `docs/work-packages/WP-020.md`; Tier 2
            // must stay unavailable on Windows until it is closed.
            let mut options = std::fs::OpenOptions::new();
            options.read(true).write(true);
            {
                use std::os::windows::fs::OpenOptionsExt as _;
                options.share_mode(FILE_SHARE_READ);
                options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
            }
            options
                .open(self.path.join(name))
                .map_err(|error| Refusal::TargetUnresolvable {
                    path: target.to_path_buf(),
                    reason: error.to_string(),
                })
        }
    }

    /// The canonical path the directory was opened at.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// One verified target: the open file object, and the name it was found under.
///
/// The [`File`] is the verification: every disposability check ran against
/// this handle, and writes through it reach the object that was checked even
/// if the path has since been renamed, deleted, or pointed at something else.
/// The path is carried for reporting only.
#[derive(Debug)]
pub struct VerifiedTarget {
    path: PathBuf,
    file: File,
}

impl VerifiedTarget {
    /// The canonical path the object was verified under, for reporting.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The verified object itself, positioned at offset zero.
    ///
    /// Consuming, like the authorization that carried it: the handle is the
    /// proof, and handing out copies of a proof is how a proof stops meaning
    /// anything. The cursor is rewound before the handle leaves
    /// [`authorize`], so a consumer may treat this as a freshly opened file.
    #[must_use]
    pub fn into_file(self) -> File {
        self.file
    }
}

/// Proof that a destructive suite may run against the targets it carries.
///
/// Constructible only by [`authorize`]. A function that requires one of these
/// cannot be called without the interlock having passed, so "did anyone check?"
/// is answered by the type rather than by review.
///
/// Deliberately **not** `Clone`, and consumed by [`Authorization::into_targets`]:
/// one authorization is one destructive run. A copyable proof could be stashed
/// and replayed against a directory whose contents have long since changed,
/// and [`File`] not being `Clone` makes this a property of the type system
/// rather than of discipline:
///
/// ```compile_fail
/// # use partman_fixtures::interlock::Authorization;
/// fn replay(authorization: &Authorization) -> Authorization {
///     authorization.clone()
/// }
/// ```
#[derive(Debug)]
pub struct Authorization {
    /// The fixture root, held from `authorize` through acquisition and
    /// verification.
    ///
    /// **What it does:** every target is opened *relative to this handle* by
    /// catalogue basename, so there is no pathname for anything to redirect
    /// between the check and the open. That is where containment is
    /// established.
    ///
    /// **What it does not do:** keep containment true afterwards. An earlier
    /// version of this comment said the handle "is held for at least as long as
    /// the verified targets", and the 2026-07-30 audit was right that this is
    /// false — [`Authorization::into_targets`] moves `targets` out and drops
    /// this field before the caller uses the handles it returned.
    ///
    /// Nothing depends on the claim, which is why the code needed no change
    /// when the claim did. Containment is a property of the returned descriptor:
    /// once `openat` has resolved the name, the descriptor refers to that object
    /// whatever later happens to the directory. Renaming or replacing the root
    /// afterwards cannot reach through an already open file.
    ///
    /// It is still worth holding, for the narrower reason that no accessor is
    /// offered: handing out the root path is how a consumer would end up
    /// reopening by name, which is the habit this increment exists to remove.
    /// Whether a held directory handle also *prevents* replacement is
    /// platform-specific and unsettled — on Unix it does not, and the Windows
    /// half is open in issue #51 — so nothing here rests on it.
    #[expect(
        dead_code,
        reason = "held for its Drop lifetime; reading it is not the purpose"
    )]
    root: RootDirectory,
    targets: Vec<VerifiedTarget>,
}

impl Authorization {
    /// The verified targets, for inspection and reporting.
    #[must_use]
    pub fn targets(&self) -> &[VerifiedTarget] {
        &self.targets
    }

    /// Consume the proof, yielding the verified objects for one destructive
    /// run. There is intentionally no way to get the handles while keeping
    /// the authorization.
    #[must_use]
    pub fn into_targets(self) -> Vec<VerifiedTarget> {
        self.targets
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

    // Hold the directory as an object before verifying anything inside it, and
    // keep it alive inside the returned `Authorization`. Targets are opened
    // relative to this handle, so the directory a verified file lives in cannot
    // be exchanged between the check and the write.
    let root = RootDirectory::open(root)?;

    let mut verified = Vec::with_capacity(request.targets.len());
    for target in &request.targets {
        verified.push(verify_target(&root, manifest, target)?);
    }

    Ok(Authorization {
        root,
        targets: verified,
    })
}

/// Verify one target, returning the open, verified file object.
fn verify_target(
    root: &RootDirectory,
    manifest: &Manifest,
    target: &Path,
) -> Result<VerifiedTarget, Refusal> {
    // Everything from here to the open is **hygiene**: it gives a caller's
    // mistake an honest, specific refusal. All of it is by-name and therefore
    // raceable, and after this increment none of it is safety-critical — the
    // open below starts from a held directory object rather than from any of
    // these strings.
    let link_metadata =
        std::fs::symlink_metadata(target).map_err(|error| unresolvable(target, &error))?;
    if !link_metadata.is_file() {
        return Err(Refusal::TargetNotRegularFile {
            path: target.to_path_buf(),
        });
    }

    let resolved = target
        .canonicalize()
        .map_err(|error| unresolvable(target, &error))?;
    if !resolved.starts_with(root.path()) {
        return Err(Refusal::TargetOutsideRoot {
            path: target.to_path_buf(),
        });
    }

    // Anchored to the compiled catalogue: this must be a fixture *by name*.
    // Membership by digest alone was too weak — it let any file pass so long as
    // some entry, anywhere in the manifest, shared its digest.
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

    // The caller must have asked for exactly where this fixture is generated,
    // not merely somewhere beneath the root with the right file name.
    // `starts_with` plus `file_name` admits `<root>/sub/blank-512.img` — an
    // ordinary copy in a subdirectory with a matching name, length and digest.
    if resolved != root.path().join(name) {
        return Err(Refusal::TargetOutsideRoot {
            path: target.to_path_buf(),
        });
    }

    // A test seam, and the only way the race below can be scheduled rather than
    // sampled. Compiled out of release builds entirely.
    #[cfg(test)]
    tests::run_before_open_hook(&resolved);

    // The decisive step. `name` is a catalogue basename opened relative to the
    // held root directory, so there are no intermediate components for anything
    // to redirect and no absolute pathname to re-resolve. Everything after this
    // reads through the returned handle.
    let mut file = root.open_child(name, target)?;

    verify_object(&mut file, entry.length, &entry.digest, target)?;

    // `verify_object` read to the end to hash the contents, so the cursor is at
    // EOF. Rewind before the handle leaves this function: a destructive consumer
    // handed a "fresh" file will reasonably assume offset zero, and documenting
    // the contract instead would make the unsafe default the easy one.
    file.seek(SeekFrom::Start(0))
        .map_err(|error| unresolvable(target, &error))?;

    Ok(VerifiedTarget {
        path: resolved,
        file,
    })
}

/// Verify the opened object itself: regular file, single name, exact length,
/// exact bytes — every read through the handle, none through a path.
///
/// This function exists as a seam on purpose. The first version of the handle
/// binding read metadata through the handle, but a deletion sweep showed that
/// downgrading it to a by-path `stat` kept every test green: nothing
/// distinguished "checked what I hold" from "checked what the name points at",
/// because the two only differ during a race. Extracting the object checks
/// into a function that takes no usable path lets a test prove handle-purity
/// deterministically — it deletes or renames the path away and then calls
/// this, which can only succeed if every check goes through the handle.
/// `target` is for error reporting only.
fn verify_object(
    file: &mut File,
    expected_length: u64,
    expected_digest: &str,
    target: &Path,
) -> Result<(), Refusal> {
    // `fstat` on the handle, not `stat` on the path: this answers "what did I
    // actually open", which no rebinding of the name can retroactively change.
    let metadata = file
        .metadata()
        .map_err(|error| unresolvable(target, &error))?;
    if !metadata.is_file() {
        return Err(Refusal::TargetNotRegularFile {
            path: target.to_path_buf(),
        });
    }

    // A hard link is a regular file, and canonicalizing one still yields a path
    // under the root, so neither name check sees it. Requiring the content to
    // equal a generated fixture already means a link can only ever point at
    // something that *is* a fixture — but a second name for the file is still a
    // second thing a destructive suite could reach, so refuse it where the
    // platform will say. On Windows the share mode on this handle closes the
    // same hole for the duration of the authorization instead: a hard link is
    // another name for the same file object, and opening that object for
    // writing while this handle lives is refused.
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

    if metadata.len() != expected_length {
        return Err(Refusal::TargetNotGenerated {
            path: target.to_path_buf(),
        });
    }

    let mut bytes = Vec::with_capacity(usize::try_from(expected_length).unwrap_or(0));
    file.read_to_end(&mut bytes)
        .map_err(|error| unresolvable(target, &error))?;
    let digest = hex(&Sha256::digest(&bytes));
    if !constant_time_eq(digest.as_bytes(), expected_digest.as_bytes()) {
        return Err(Refusal::TargetNotGenerated {
            path: target.to_path_buf(),
        });
    }
    Ok(())
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
