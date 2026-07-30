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
//! the authorization holds.
//!
//! On Windows the target handle additionally carries a share mode that refuses
//! concurrent opens. State its reach exactly, because an overstated version of
//! this sentence is what let the F-03 hard-link hole ship: **it refuses writes
//! to the unnamed `$DATA` stream through any name, and refuses rename and
//! delete of the directory entry the handle was opened under.** Writes to
//! *named* alternate data streams and changes to file attributes still succeed,
//! and the share mode says nothing at all about this interlock's own write —
//! which is the one that reaches every hard link. Other names are refused by
//! counting them (see [`verify_object`]), never by the share mode.
//!
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
/// reparse-point attribute check in [`verify_object`] — recorded as such rather
/// than claimed as tested everywhere.
#[cfg(windows)]
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;

/// `FILE_FLAG_BACKUP_SEMANTICS`: required to obtain a handle to a *directory*.
///
/// Needed only by [`RootDirectory::open`], and deliberately never passed to
/// [`RootDirectory::open_child`] — see the comment there, which records the
/// measurement. Despite the name it needs no privilege for a directory the
/// caller owns: the measuring host held no `SeBackupPrivilege` and the open
/// succeeded on NTFS, on `ReFS`, over SMB, and on a `subst` drive.
#[cfg(windows)]
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;

/// The Windows facts about an open object that stable `std` will not give.
///
/// **This is the only place `winapi_util` is named in this crate, and a test
/// asserts that.** The confinement is deliberate rather than tidy:
/// `winapi_util::HandleRef` predates Rust's I/O safety, carries no lifetime,
/// and is constructible from safe code — so a later line could obtain an
/// authoritative-looking answer for a *closed* handle, which was demonstrated
/// during review inside a crate with `unsafe_code = "deny"` and no `unsafe`
/// token in it. Taking `&File` and returning plain integers means the borrow
/// checker governs the handle's lifetime and no raw handle ever escapes.
///
/// This is **not** the same guarantee the Unix half gets. `rustix` takes
/// `BorrowedFd<'_>` and is I/O-safe by construction; this wrapper is not, and
/// the dependency is justified on its own audited merits rather than by
/// analogy with `rustix`.
#[cfg(windows)]
fn handle_facts(file: &File) -> io::Result<HandleFacts> {
    let information = winapi_util::file::information(file)?;
    Ok(HandleFacts {
        links: information.number_of_links(),
        identity: (information.file_index(), information.volume_serial_number()),
    })
}

/// What one [`handle_facts`] call answers.
///
/// `identity` is Windows' nearest analogue of an inode — the file index paired
/// with the volume serial. Nothing in the interlock reads it; the *tests* do,
/// because the standing rule here is that a containment regression asserts
/// which object was authorized rather than whether a call refused, and a decoy
/// holding a fixture's exact bytes is indistinguishable by content. It is
/// returned from this one function rather than fetched separately so that the
/// confinement described above stays a single call site.
///
/// Caveat, recorded rather than assumed away: `BY_HANDLE_FILE_INFORMATION`'s
/// 64-bit index is documented as not guaranteed unique on `ReFS`, and this
/// repository's own working copy sits on a `ReFS` Dev Drive. No collision was
/// produced in review — two byte-identical files returned different indices —
/// so the identity assertions are sound on NTFS and *unproven* rather than
/// broken on `ReFS`. The 128-bit `FILE_ID_INFO` that would settle it is not
/// exposed by the safe wrapper.
#[cfg(windows)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HandleFacts {
    /// Hard links naming this object. Excludes any 8.3 short-name alias.
    links: u64,
    /// `(file index, volume serial number)`.
    identity: (u64, u64),
}

/// The identity of an open object, for tests that must name *which* object was
/// authorized. Routed through [`handle_facts`] so there is still one call site.
#[cfg(all(windows, test))]
pub(crate) fn object_identity(file: &File) -> io::Result<(u64, u64)> {
    handle_facts(file).map(|facts| facts.identity)
}

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

/// Is this canonical root served by a filesystem whose share modes Windows
/// enforces?
///
/// A pure function of the path prefix, so it is unit-testable without needing a
/// network filesystem to exist on the machine running the tests.
///
/// **Why this exists.** The whole Windows containment argument is that the
/// filesystem refuses to rename or delete a directory with a live handle. A
/// redirector is free not to implement that, and one measurably does not: with
/// the root handle held on `\\wsl.localhost\Debian\...`, a swap staged from the
/// Linux side succeeded and the subsequent child open returned the decoy's
/// object identity. NTFS, `ReFS` and the Windows SMB server all refused the same
/// attack.
///
/// So a UNC root is refused. That **over-refuses** SMB to a Windows server,
/// which was measured to hold, and that is the deliberate direction: SAFE-005
/// requires failing closed, and the cost of refusing a working configuration is
/// an error message, while the cost of trusting a broken one is a destructive
/// write outside the fixture root.
///
/// It does **not** catch a third-party filesystem mounted at a *drive letter*
/// (`WinFsp`, Dokan, sshfs-win, or a mapped drive that canonicalizes to one).
/// Separating those needs a volume-class query that no safe wrapper here
/// exposes; it is recorded as residual risk in `docs/work-packages/WP-020.md`
/// rather than silently assumed away.
#[cfg(windows)]
fn root_namespace_is_local(path: &Path) -> bool {
    use std::path::{Component, Prefix};

    match path.components().next() {
        Some(Component::Prefix(prefix)) => {
            !matches!(prefix.kind(), Prefix::UNC(..) | Prefix::VerbatimUNC(..))
        }
        // No prefix at all means the path is not canonical, which
        // `canonicalize` should have made impossible. Refuse rather than
        // reason about how it happened.
        _ => false,
    }
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
/// The two platforms establish containment by different mechanisms, and the
/// difference is load-bearing rather than an implementation detail.
///
/// **Unix resolves.** `openat` from a held descriptor starts path resolution at
/// an object nothing can rebind, so containment holds on every Unix filesystem
/// because it is a property of how the name is resolved.
///
/// **Windows refuses.** There is no safe handle-relative open in stable `std`,
/// so the child is still opened by pathname — and what makes that sound is that
/// no component of the pathname can be exchanged while the handle lives. That
/// is *the filesystem driver's* behaviour, not the resolver's, and it therefore
/// holds only as far as the driver does. Measured: NTFS, `ReFS` and the Windows
/// SMB server refuse the swap; the WSL 9p redirector permits it, and a swap
/// staged from the Linux side redirected the open to a decoy with the root
/// handle held. Roots on a filesystem Windows does not serve are therefore
/// refused outright by [`RootDirectory::open`] rather than silently trusted.
#[derive(Debug)]
pub struct RootDirectory {
    /// The directory handle.
    ///
    /// On Unix this is what `openat` resolves from. On Windows it is held for
    /// its share mode: opened without `FILE_SHARE_DELETE`, it makes the
    /// filesystem refuse rename and delete of the root for as long as it lives,
    /// which closes the one window the target handles do not cover — between
    /// this open and the first [`Self::open_child`].
    ///
    /// Never read on Windows, and that is the point: its effect is the share
    /// mode the kernel enforces while it is alive, not anything this code does
    /// with it. `the_root_handle_alone_refuses_renaming_the_root` is what
    /// proves it is doing something, since a lint cannot.
    #[cfg_attr(
        windows,
        expect(
            dead_code,
            reason = "held for its share mode; reading it is not the purpose"
        )
    )]
    handle: File,
    /// The canonical path the directory was opened at, for reporting and for
    /// the Windows child open.
    path: PathBuf,
}

impl RootDirectory {
    /// Open the fixture root and hold it.
    fn open(root: &Path) -> Result<Self, Refusal> {
        let path = root
            .canonicalize()
            .map_err(|error| Refusal::ManifestUnreadable(format!("fixture root: {error}")))?;
        Self::hold(path)
    }

    /// Hold an already-canonical root.
    ///
    /// Split out from [`Self::open`] so a test can prove the namespace
    /// precondition is *reached*, not merely that the classifier it calls
    /// returns the right answer. The first version of this increment tested
    /// only the classifier, and deleting the call site left the whole suite
    /// green — the same shape of defect as a traceability row naming evidence
    /// that no longer exists. A test can hand this a literal UNC path without
    /// needing one to exist on the machine running it.
    fn hold(path: PathBuf) -> Result<Self, Refusal> {
        // Before the handle is taken, because on a filesystem Windows does not
        // serve, taking it proves nothing.
        #[cfg(windows)]
        if !root_namespace_is_local(&path) {
            return Err(Refusal::RootNotLocallyServed { path });
        }

        #[cfg(unix)]
        let handle = File::open(&path)
            .map_err(|error| Refusal::ManifestUnreadable(format!("fixture root: {error}")))?;

        #[cfg(windows)]
        let handle = {
            use std::os::windows::fs::OpenOptionsExt as _;
            std::fs::OpenOptions::new()
                .read(true)
                // No `FILE_SHARE_DELETE`. This is the entire mechanism: with it
                // the root can be renamed aside mid-authorization, and without
                // it the filesystem refuses. Proved behaviourally rather than by
                // reading the constant back — see
                // `the_root_handle_alone_refuses_renaming_the_root`.
                .share_mode(FILE_SHARE_READ)
                // `BACKUP_SEMANTICS` is what makes a directory openable at all.
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&path)
                .map_err(|error| Refusal::RootUnavailable {
                    path: path.clone(),
                    reason: error.to_string(),
                })?
        };

        Ok(Self { handle, path })
    }

    /// Open a direct child by name, refusing to follow a link at that name.
    ///
    /// `name` is a catalogue basename and must contain no separator, so there
    /// are no intermediate components for anything to redirect.
    fn open_child(&self, name: &str, target: &Path) -> Result<File, Refusal> {
        // `:` joins the list on Windows, where `name:stream` addresses an
        // alternate data stream of `name` rather than a file called
        // `name:stream`. Catalogue basenames make it unreachable today, and
        // without it the refusal would come from the manifest lookup failing —
        // which is failing closed by accident rather than by design.
        if name.contains('/') || name.contains('\\') || name.contains(':') {
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
            // Windows still opens by pathname, because stable `std` exposes no
            // handle-relative open. What makes that sound here is that
            // `self.path` is canonical — every reparse point and `subst`
            // mapping was collapsed out of it before the root handle was taken
            // — and that the held root handle stops any component of it being
            // exchanged while this runs. See the `RootDirectory` doc comment
            // for the exact reach of that, and for the filesystem classes where
            // it does not hold and the root is refused instead.
            let mut options = std::fs::OpenOptions::new();
            options.read(true).write(true);
            {
                use std::os::windows::fs::OpenOptionsExt as _;
                options.share_mode(FILE_SHARE_READ);
                // `FILE_FLAG_OPEN_REPARSE_POINT` and **nothing else**. Adding
                // `FILE_FLAG_BACKUP_SEMANTICS` here — the flag the root open
                // two functions up now needs — would open a junction planted at
                // this name instead of refusing it, and it would report
                // `is_file()`, so the regular-file check would not catch it
                // either. Measured, all three variants, junction at the child
                // name: no flags -> refused os 5; `OPEN_REPARSE_POINT` ->
                // refused os 5; `OPEN_REPARSE_POINT | BACKUP_SEMANTICS` ->
                // **opened**. `an_entry_replaced_by_a_junction_is_refused`
                // fails if the flag is ever copied down here.
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
    /// Nothing depends on that, on either platform, and for the same reason:
    /// containment is a property of the returned descriptor. Once the child is
    /// open, the descriptor refers to that object whatever later happens to the
    /// directory, so renaming or replacing the root afterwards cannot reach
    /// through it.
    ///
    /// **The window it does cover differs by platform, and on Windows it is
    /// load-bearing.** On Unix the descriptor is resolved by `openat` from this
    /// handle, so the handle is the resolution root and needs no help. On
    /// Windows the child is opened by pathname, and this handle's share mode is
    /// what stops the root being renamed aside between [`authorize`] taking it
    /// and the first child open — the one window the target handles cannot
    /// cover, because no target is open yet. Measured: with this handle the
    /// swap is refused; without it the same swap succeeds and the child open
    /// lands on the decoy.
    ///
    /// It is also worth holding for the narrower reason that no accessor is
    /// offered: handing out the root path is how a consumer would end up
    /// reopening by name, which is the habit this increment exists to remove.
    #[cfg_attr(
        unix,
        expect(
            dead_code,
            reason = "held for its Drop lifetime; reading it is not the purpose"
        )
    )]
    #[cfg_attr(
        windows,
        expect(
            dead_code,
            reason = "held for its share mode; reading it is not the purpose"
        )
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
    /// A target is reachable under more than one hard link.
    ///
    /// Deliberately not "more than one name". On an 8.3-enabled NTFS volume a
    /// file also has a short-name directory entry that this count does not
    /// include — measured, `BLANK-~1.IMG` beside `blank-512.img` with the count
    /// still reading 1. That alias cannot leave its parent directory, so it is
    /// not a route out of the fixture root; but a refusal that claimed "exactly
    /// one name" would be stating something false on most Windows volumes.
    TargetHasOtherNames {
        /// The path as supplied.
        path: PathBuf,
        /// How many hard links refer to this file.
        links: u64,
    },
    /// The fixture root is not on a filesystem whose share modes Windows
    /// enforces, so containment cannot be established there.
    #[cfg(windows)]
    RootNotLocallyServed {
        /// The canonical root path.
        path: PathBuf,
    },
    /// The fixture root could not be held open — most often because another
    /// process holds it with a sharing mode that excludes this one.
    ///
    /// Separate from [`Self::ManifestUnreadable`] because that variant's
    /// message names a manifest, and no manifest is involved in holding a
    /// directory. There is deliberately **no retry**: retrying a safety
    /// precondition until it passes turns a gate into advice.
    #[cfg(windows)]
    RootUnavailable {
        /// The canonical root path.
        path: PathBuf,
        /// The underlying reason.
        reason: String,
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
                "{} is reachable under {links} hard links; a destructive suite must address a \
                 file with exactly one, because a write through this handle reaches every one \
                 of them",
                path.display()
            ),
            #[cfg(windows)]
            Self::RootNotLocallyServed { path } => write!(
                formatter,
                "{} is not on a locally served volume; Windows containment relies on the \
                 filesystem refusing to rename a directory that is held open, which a network \
                 redirector need not do. Generate fixtures on a local NTFS or ReFS volume",
                path.display()
            ),
            #[cfg(windows)]
            Self::RootUnavailable { path, reason } => write!(
                formatter,
                "cannot hold the fixture root {}: {reason}. Another process holds it with an \
                 incompatible sharing mode; identify the holder rather than retrying",
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

    // Deduplicate before opening anything. The share mode on a verified target
    // refuses a second write handle to the same object — including the one this
    // loop would take on the next iteration — so a request naming a target
    // twice used to refuse *itself*, reporting "used by another process" and
    // pointing an operator at a race that was not happening. Fails closed
    // either way; this makes it fail for the true reason.
    let mut seen = std::collections::BTreeSet::new();
    let mut verified = Vec::with_capacity(request.targets.len());
    for target in &request.targets {
        if !seen.insert(target.as_path()) {
            continue;
        }
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
    // This is also what refuses a **reparse point** on Windows, and that is
    // worth stating because a review of this increment asserted otherwise —
    // that a swapped-in file symlink was caught only by the length check and by
    // the raceable by-path `symlink_metadata` above. Measured, through the
    // handle, for a file symlink and for a directory junction, with and without
    // backup semantics: `is_file()` is `false` and `is_symlink()` is `true` in
    // every case, so the refusal happens here and is neither raceable nor
    // accidental. An explicit `FILE_ATTRIBUTE_REPARSE_POINT` test was written
    // during this increment and then removed: no handle the interlock can
    // produce reaches it, and an unreachable guard reads as protection while
    // proving nothing.
    if !metadata.is_file() {
        return Err(Refusal::TargetNotRegularFile {
            path: target.to_path_buf(),
        });
    }

    let links = object_facts(&metadata, file, target)?;

    // A hard link is a regular file, and canonicalizing one still yields a path
    // under the root, so neither name check sees it. Requiring the content to
    // equal a generated fixture already means a link can only ever point at
    // something that *is* a fixture by content — but a user's own file may hold
    // a fixture's bytes, and a write through this handle reaches every name the
    // object has, including one outside the fixture root. So the count is
    // refused on both platforms.
    //
    // **This used to say the Windows share mode closed the same hole. It does
    // not, and that sentence is why F-03 shipped.** The share mode refuses
    // *other* openers; the destructive write here goes through the handle that
    // was already authorized, and no share mode constrains it. Reproduced
    // during review: a user file hard-linked in at the fixture's name, length
    // and digest both passing, written through, destroyed.
    if links > 1 {
        return Err(Refusal::TargetHasOtherNames {
            path: target.to_path_buf(),
            links,
        });
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

/// How many hard links name this object, read through the open handle rather
/// than through a path.
///
/// Split by platform because the two answer it differently, not because they
/// mean different things.
#[cfg(unix)]
#[expect(
    clippy::unnecessary_wraps,
    reason = "the Windows half of this pair genuinely fails, and one signature across both is \
              what keeps the caller free of a platform branch"
)]
fn object_facts(
    metadata: &std::fs::Metadata,
    _file: &File,
    _target: &Path,
) -> Result<u64, Refusal> {
    use std::os::unix::fs::MetadataExt as _;

    Ok(metadata.nlink())
}

/// The Windows half of [`object_facts`].
///
/// Stable `std` will not answer this — `number_of_links` is unstable behind
/// `windows_by_handle` — which is why this crate carries a safe wrapper
/// dependency. See [`handle_facts`] for why the call is confined to one place.
#[cfg(windows)]
fn object_facts(_metadata: &std::fs::Metadata, file: &File, target: &Path) -> Result<u64, Refusal> {
    handle_facts(file)
        .map(|facts| facts.links)
        .map_err(|error| unresolvable(target, &error))
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
