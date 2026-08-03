//! Descriptor-bound Linux loop-device acceptance for WP-020 increment 2e.
//!
//! The one callable entry point, [`run_authorized`], consumes the non-cloneable
//! SAFE-007 [`Authorization`] produced by `partman-fixtures`. It accepts exactly
//! the two fixtures registered for the issue-94 acceptance, passes their held
//! file objects (never their paths) to a private Linux controller, and exposes
//! only normalized success facts. On other operating systems the same API
//! consumes and validates the authorization before returning an explicit typed
//! refusal.
//!
//! No product adapter lives here. The module exists only to prove the narrow,
//! logical-content-read-only Tier-2 loop-control chain authorized by WP-020.
//! Each initial held-file digest must match the fixture bytes compiled into this
//! crate before either loop mapping is configured. The later digest and status
//! checks are samples, not locks, and cannot defeat an ABA change entirely
//! between samples. External run evidence must exclude every other actor able
//! to modify either fixture or administer/rebind loop devices. Ordinary
//! kernel/udev read/open discovery is allowed; bounded detach retries and exact
//! retained-rdev sysfs inspection handle its cleanup effects. The harness
//! refuses the first `LOOP_CONFIGURE` isolation conflict rather than
//! retrying when isolated loop state was not established. The disposable VM
//! bounds consequences but does not itself prove those exclusions. The
//! result must not be reused as a continuous-binding guarantee for a future
//! destructive path.

use core::fmt;
use std::ffi::OsStr;
use std::fs::File;

use partman_fixtures::interlock::Authorization;

#[cfg(any(target_os = "linux", test))]
mod protocol;

#[cfg(target_os = "linux")]
mod linux;

// SAFE-009 permits the reviewed FFI exception only in this exact module. The
// workspace lint denies unsafe everywhere else in this crate.
#[cfg(target_os = "linux")]
mod sys;

const BASIC_NAME: &str = "gpt-basic-512.img";
const CONFLICTING_NAME: &str = "gpt-conflicting-tables-512.img";

/// Normalized proof facts from a completed loop-control acceptance.
///
/// Every boolean getter is true for a constructed report. They remain explicit
/// so the task runner can print the proof obligations it actually established
/// without exposing kernel device numbers, inode numbers, or paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunReport {
    configured_legs: u8,
    clean_observation_bytes: usize,
    detachments_confirmed: u8,
}

impl RunReport {
    /// Number of loop attachments configured by the acceptance.
    #[must_use]
    pub const fn configured_legs(&self) -> u8 {
        self.configured_legs
    }

    /// Bytes returned by the clean in-process positional-read probe.
    #[must_use]
    pub const fn clean_observation_bytes(&self) -> usize {
        self.clean_observation_bytes
    }

    /// Whether flags, block size, backing identity, and node identity matched at
    /// both required sampling points around the clean probe.
    #[must_use]
    pub const fn required_configuration_verified(&self) -> bool {
        true
    }

    /// Whether final verification detected the adversarial backing rebind.
    #[must_use]
    pub const fn adversarial_rebind_detected(&self) -> bool {
        true
    }

    /// Whether the observation pending at rebind was withheld and discarded.
    #[must_use]
    pub const fn adversarial_observation_discarded(&self) -> bool {
        true
    }

    /// Number of configured attachments explicitly detached and confirmed
    /// absent before success was returned.
    #[must_use]
    pub const fn detachments_confirmed(&self) -> u8 {
        self.detachments_confirmed
    }

    /// Whether both released loop nodes had no materialized partition child in
    /// their exact retained-rdev sysfs directory.
    #[must_use]
    pub const fn partition_teardown_confirmed(&self) -> bool {
        true
    }

    /// Whether both initial held-file hashes matched the compiled catalogue.
    #[must_use]
    pub const fn initial_fixture_hashes_matched_catalogue(&self) -> bool {
        true
    }

    /// Whether both authorized fixture hashes matched their pre-run values.
    #[must_use]
    pub const fn fixture_hashes_unchanged(&self) -> bool {
        true
    }
}

/// Whether a refused run established a reusable loop environment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CleanupState {
    /// No isolation conflict occurred and no run-owned attachment existed, or
    /// every run-owned attachment was confirmed absent.
    NotRequiredOrConfirmed,
    /// Required isolation, detach, or partition teardown could not be proved;
    /// the disposable VM must not be reused.
    Uncertain,
}

impl fmt::Display for CleanupState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotRequiredOrConfirmed => "not-required-or-confirmed",
            Self::Uncertain => "uncertain",
        })
    }
}

/// What a refused run established about fixture immutability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixtureState {
    /// The run ended before both final descriptor hashes could prove equality.
    NotEstablished,
    /// A final descriptor hash differed from its pre-run value.
    Changed,
}

impl fmt::Display for FixtureState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotEstablished => "not-established",
            Self::Changed => "changed",
        })
    }
}

/// A fail-closed reason the acceptance produced no usable evidence.
///
/// Display text is deliberately normalized: it contains no device number,
/// inode, arbitrary path, or raw kernel identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The authorization did not contain exactly the two registered fixtures.
    WrongAuthorizedTargets,
    /// Loop control is available only on Linux.
    UnsupportedPlatform,
    /// The fixed loop-control node was not the expected kernel misc device.
    LoopControlIdentityMismatch,
    /// A named, bounded kernel operation failed.
    KernelOperation {
        /// Stable operation category; never a pathname.
        operation: &'static str,
        /// OS error number when one was available.
        errno: Option<i32>,
    },
    /// Atomic loop configuration reported busy, so isolated loop state was
    /// not established.
    LoopIsolationConflict,
    /// Initial held bytes differed from the compiled fixture catalogue.
    InitialFixtureHashMismatch {
        /// Registered fixture role, not a path.
        fixture: FixtureRole,
    },
    /// Read-back did not name the exact held backing object.
    BackingIdentityMismatch,
    /// The held loop descriptor no longer named the same loop node.
    LoopNodeIdentityMismatch,
    /// The kernel did not retain exactly the required loop flags.
    LoopFlagsMismatch,
    /// Offset or size-limit read-back differed from the zeroed request.
    LoopGeometryMismatch,
    /// The configured logical block size was not 512 bytes.
    BlockSizeMismatch,
    /// Status named a different loop number from the held loop node.
    LoopNumberMismatch,
    /// The bounded in-process positional read did not complete.
    ProbeFailed {
        /// Bounded OS error number, or none for an unexpected short read.
        errno: Option<i32>,
    },
    /// The adversarial `LOOP_CHANGE_FD` operation itself failed.
    AdversarialRebindFailed {
        /// Bounded OS error number, when the kernel supplied one.
        errno: Option<i32>,
    },
    /// Final verification failed to detect a successful backing rebind.
    AdversarialRebindNotDetected,
    /// Explicit detach failed.
    DetachFailed {
        /// Bounded OS error number returned by `LOOP_CLR_FD`.
        errno: Option<i32>,
    },
    /// Status could not establish whether explicit detach completed.
    DetachConfirmationFailed {
        /// Bounded OS error number returned by `LOOP_GET_STATUS64`.
        errno: Option<i32>,
    },
    /// Status still reported an attachment after detach.
    DetachNotConfirmed,
    /// Post-release sysfs inspection could not prove that partitions vanished.
    PartitionTeardownNotConfirmed {
        /// Bounded OS error number for unreadable or ambiguous sysfs state.
        errno: Option<i32>,
    },
    /// A supposedly read-only leg changed one of the fixture hashes.
    FixtureHashChanged {
        /// Registered fixture role, not a path.
        fixture: FixtureRole,
    },
    /// An internal protocol transition attempted to publish evidence early.
    ProtocolOrder,
}

impl Refusal {
    /// Cleanup state safe for operator-facing output.
    #[must_use]
    pub const fn cleanup_state(&self) -> CleanupState {
        match self {
            Self::DetachFailed { .. }
            | Self::DetachConfirmationFailed { .. }
            | Self::DetachNotConfirmed
            | Self::PartitionTeardownNotConfirmed { .. }
            | Self::LoopIsolationConflict => CleanupState::Uncertain,
            _ => CleanupState::NotRequiredOrConfirmed,
        }
    }

    /// Fixture state safe for operator-facing output.
    #[must_use]
    pub const fn fixture_state(&self) -> FixtureState {
        match self {
            Self::FixtureHashChanged { .. } => FixtureState::Changed,
            _ => FixtureState::NotEstablished,
        }
    }

    /// A normalized safe next step containing no path or kernel identity.
    #[must_use]
    pub const fn remediation(&self) -> &'static str {
        match self {
            Self::DetachFailed { .. }
            | Self::DetachConfirmationFailed { .. }
            | Self::DetachNotConfirmed
            | Self::PartitionTeardownNotConfirmed { .. }
            | Self::LoopIsolationConflict
            | Self::InitialFixtureHashMismatch { .. }
            | Self::FixtureHashChanged { .. } => {
                "discard or revert the disposable VM; do not reuse it"
            }
            Self::UnsupportedPlatform => "run the exact acceptance in a native-Linux disposable VM",
            Self::WrongAuthorizedTargets => {
                "regenerate the registered fixtures and rerun with a fresh disposable authorization"
            }
            _ => {
                "withhold this run as evidence, investigate in the disposable VM, revert it, and rerun with fresh authorization"
            }
        }
    }
}

fn write_errno(
    formatter: &mut fmt::Formatter<'_>,
    message: &str,
    errno: Option<i32>,
) -> fmt::Result {
    match errno {
        Some(errno) => write!(formatter, "{message} (errno {errno})"),
        None => formatter.write_str(message),
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongAuthorizedTargets => formatter.write_str(
                "authorization must contain exactly gpt-basic-512.img and \
                 gpt-conflicting-tables-512.img",
            ),
            Self::UnsupportedPlatform => {
                formatter.write_str("Linux loop control is unsupported on this platform")
            }
            Self::LoopControlIdentityMismatch => formatter
                .write_str("the fixed loop-control node was not the expected kernel device"),
            Self::KernelOperation { operation, errno } => match errno {
                Some(errno) => write!(
                    formatter,
                    "kernel operation {operation} failed (errno {errno})"
                ),
                None => write!(formatter, "kernel operation {operation} failed"),
            },
            Self::LoopIsolationConflict => formatter
                .write_str("selected loop was busy; isolated loop state was not established"),
            Self::InitialFixtureHashMismatch { fixture } => write!(
                formatter,
                "the initial {fixture} fixture hash did not match the compiled catalogue"
            ),
            Self::BackingIdentityMismatch => {
                formatter.write_str("loop status did not match the held backing descriptor")
            }
            Self::LoopNodeIdentityMismatch => {
                formatter.write_str("the held loop-node identity changed during the acceptance")
            }
            Self::LoopFlagsMismatch => {
                formatter.write_str("loop status did not retain the exact required flags")
            }
            Self::LoopGeometryMismatch => {
                formatter.write_str("loop status did not retain the zero offset and size limit")
            }
            Self::BlockSizeMismatch => {
                formatter.write_str("loop logical block size was not the required 512 bytes")
            }
            Self::LoopNumberMismatch => {
                formatter.write_str("loop status did not match the held loop-node number")
            }
            Self::ProbeFailed { errno } => write_errno(
                formatter,
                "the bounded in-process positional-read probe failed",
                *errno,
            ),
            Self::AdversarialRebindFailed { errno } => write_errno(
                formatter,
                "the adversarial read-only loop rebind could not be exercised",
                *errno,
            ),
            Self::AdversarialRebindNotDetected => formatter
                .write_str("final verification did not detect the adversarial backing rebind"),
            Self::DetachFailed { errno } => {
                write_errno(formatter, "explicit loop detach failed", *errno)
            }
            Self::DetachConfirmationFailed { errno } => {
                write_errno(formatter, "loop detach status confirmation failed", *errno)
            }
            Self::DetachNotConfirmed => {
                formatter.write_str("loop detach could not be confirmed by status read-back")
            }
            Self::PartitionTeardownNotConfirmed { errno } => write_errno(
                formatter,
                "loop partition teardown could not be confirmed after descriptor release",
                *errno,
            ),
            Self::FixtureHashChanged { fixture } => {
                write!(
                    formatter,
                    "the {fixture} fixture hash changed during the read-only run"
                )
            }
            Self::ProtocolOrder => {
                formatter.write_str("evidence publication was attempted before final verification")
            }
        }
    }
}

impl std::error::Error for Refusal {}

/// One of the two fixed, repository-generated backing roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixtureRole {
    /// The ordinary GPT fixture used for both initial attachments.
    Basic,
    /// The same-size, conflicting GPT fixture used only for adversarial rebind.
    Conflicting,
}

impl fmt::Display for FixtureRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Basic => "basic GPT",
            Self::Conflicting => "conflicting GPT",
        })
    }
}

struct AuthorizedFiles {
    basic: File,
    conflicting: File,
}

/// Run the one registered descriptor-bound loop-control acceptance.
///
/// The authorization is consumed even when the platform is unsupported or the
/// target set is wrong. There is no API that accepts a caller-selected file,
/// raw descriptor, loop number, or path.
///
/// # Errors
///
/// Returns [`Refusal`] on every unsupported platform, authorization mismatch,
/// kernel failure, identity mismatch, cleanup failure, or fixture-byte change.
///
/// The call-site boundary is structural: a caller-selected [`File`] is not an
/// accepted argument, and the private controller cannot be named externally.
///
/// ```compile_fail
/// use std::fs::File;
/// use partman_ffi_linux_loop::run_authorized;
///
/// fn bypass_interlock(file: File) {
///     let _ = run_authorized(file);
/// }
/// ```
pub fn run_authorized(authorization: Authorization) -> Result<RunReport, Refusal> {
    let files = consume_exact_targets(authorization)?;

    #[cfg(target_os = "linux")]
    {
        linux::run(files)
    }

    #[cfg(not(target_os = "linux"))]
    {
        let AuthorizedFiles { basic, conflicting } = files;
        drop((basic, conflicting));
        Err(Refusal::UnsupportedPlatform)
    }
}

fn consume_exact_targets(authorization: Authorization) -> Result<AuthorizedFiles, Refusal> {
    let targets = authorization.into_targets();
    if targets.len() != 2 {
        return Err(Refusal::WrongAuthorizedTargets);
    }

    let mut basic = None;
    let mut conflicting = None;
    for target in targets {
        match target.path().file_name() {
            Some(name) if name == OsStr::new(BASIC_NAME) && basic.is_none() => {
                basic = Some(target.into_file());
            }
            Some(name) if name == OsStr::new(CONFLICTING_NAME) && conflicting.is_none() => {
                conflicting = Some(target.into_file());
            }
            _ => return Err(Refusal::WrongAuthorizedTargets),
        }
    }

    match (basic, conflicting) {
        (Some(basic), Some(conflicting)) => Ok(AuthorizedFiles { basic, conflicting }),
        _ => Err(Refusal::WrongAuthorizedTargets),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use partman_fixtures::catalogue::generate;
    use partman_fixtures::interlock::{DESTRUCTIVE_PROFILE, Request, authorize};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Sandbox(PathBuf);

    impl Sandbox {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "partman-loop-authorized-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = std::fs::remove_dir_all(&root);
            Self(root)
        }

        fn authorization(&self, names: &[&str]) -> Authorization {
            let manifest = generate(&self.0).expect("generate disposable repository fixtures");
            authorize(
                &self.0,
                &Request {
                    profile: Some(DESTRUCTIVE_PROFILE.to_owned()),
                    token: Some(manifest.token().to_owned()),
                    targets: names.iter().map(|name| self.0.join(name)).collect(),
                },
            )
            .expect("all SAFE-007 factors agree")
        }
    }

    impl Drop for Sandbox {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn rust_sources() -> Vec<(PathBuf, String)> {
        fn collect(directory: &Path, sources: &mut Vec<PathBuf>) {
            for entry in std::fs::read_dir(directory).expect("read crate source directory") {
                let entry = entry.expect("read crate source entry");
                let file_type = entry.file_type().expect("read source entry type");
                assert!(
                    !file_type.is_symlink(),
                    "source entries may not be symlinks"
                );
                if file_type.is_dir() {
                    collect(&entry.path(), sources);
                } else if file_type.is_file() && entry.path().extension() == Some(OsStr::new("rs"))
                {
                    sources.push(entry.path());
                }
            }
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut paths = Vec::new();
        collect(&root, &mut paths);
        paths.sort();
        paths
            .into_iter()
            .map(|path| {
                let relative = path
                    .strip_prefix(&root)
                    .expect("source remains under crate root")
                    .to_path_buf();
                let source = std::fs::read_to_string(&path).expect("read Rust source");
                (relative, source)
            })
            .collect()
    }

    #[test]
    fn refusal_display_never_formats_raw_identity_fields() {
        let samples = [
            Refusal::UnsupportedPlatform,
            Refusal::KernelOperation {
                operation: "loop-configure",
                errno: Some(16),
            },
            Refusal::BackingIdentityMismatch,
            Refusal::LoopNodeIdentityMismatch,
            Refusal::FixtureHashChanged {
                fixture: FixtureRole::Basic,
            },
        ];

        for sample in samples {
            let rendered = sample.to_string();
            assert!(!rendered.contains("/dev/"));
            assert!(!rendered.contains("inode"));
            assert!(!rendered.contains("0x"));
        }
    }

    // Requirements: SAFE-005, SAFE-006, SAFE-007
    //   Cleanup uncertainty and fixture state have normalized operator remediation.
    // Evidence: refusal_state_and_remediation_fail_closed_without_raw_identity
    #[test]
    fn refusal_state_and_remediation_fail_closed_without_raw_identity() {
        for refusal in [
            Refusal::DetachFailed { errno: Some(16) },
            Refusal::DetachConfirmationFailed { errno: Some(5) },
            Refusal::DetachNotConfirmed,
            Refusal::PartitionTeardownNotConfirmed { errno: None },
        ] {
            assert_eq!(refusal.cleanup_state(), CleanupState::Uncertain);
            assert_eq!(refusal.fixture_state(), FixtureState::NotEstablished);
            assert_eq!(
                refusal.remediation(),
                "discard or revert the disposable VM; do not reuse it"
            );
            let rendered = refusal.to_string();
            assert!(!rendered.contains("/dev/"));
            assert!(!rendered.contains("inode"));
            assert!(!rendered.contains("0x"));
        }

        let changed = Refusal::FixtureHashChanged {
            fixture: FixtureRole::Basic,
        };
        assert_eq!(
            changed.cleanup_state(),
            CleanupState::NotRequiredOrConfirmed
        );
        assert_eq!(changed.fixture_state(), FixtureState::Changed);
        assert_eq!(
            changed.remediation(),
            "discard or revert the disposable VM; do not reuse it"
        );

        let wrong_start = Refusal::InitialFixtureHashMismatch {
            fixture: FixtureRole::Conflicting,
        };
        assert_eq!(wrong_start.fixture_state(), FixtureState::NotEstablished);
        assert_eq!(
            wrong_start.cleanup_state(),
            CleanupState::NotRequiredOrConfirmed
        );
        assert_eq!(
            wrong_start.remediation(),
            "discard or revert the disposable VM; do not reuse it"
        );

        let isolation_conflict = Refusal::LoopIsolationConflict;
        assert_eq!(isolation_conflict.cleanup_state(), CleanupState::Uncertain);
        assert_eq!(
            isolation_conflict.fixture_state(),
            FixtureState::NotEstablished
        );
        assert_eq!(
            isolation_conflict.remediation(),
            "discard or revert the disposable VM; do not reuse it"
        );
    }

    // Requirements: SAFE-009
    //   Unsafe syntax is confined to the one reviewed sys.rs module.
    // Evidence: unsafe_syntax_is_confined_to_the_reviewed_sys_module
    #[test]
    fn unsafe_syntax_is_confined_to_the_reviewed_sys_module() {
        let unsafe_block = ["unsafe", " {"].concat();
        let unsafe_impl = ["unsafe", " impl"].concat();
        let unsafe_function = ["unsafe", " fn"].concat();
        let allow_unsafe = ["allow(", "unsafe_code", ")"].concat();
        let mut exception_files = Vec::new();
        for (path, source) in rust_sources() {
            let has_boundary_syntax = source.contains(&unsafe_block)
                || source.contains(&unsafe_impl)
                || source.contains(&unsafe_function);
            let has_lint_exception = source.contains(&allow_unsafe);
            if has_boundary_syntax || has_lint_exception {
                exception_files.push(path.clone());
            }
            if path == Path::new("sys.rs") {
                assert!(
                    has_boundary_syntax,
                    "sys.rs must exercise the reviewed boundary"
                );
                assert!(
                    has_lint_exception,
                    "sys.rs must declare the narrow exception"
                );
            } else {
                assert!(!has_boundary_syntax, "unsafe syntax escaped into {path:?}");
                assert!(
                    !has_lint_exception,
                    "unsafe lint exception escaped into {path:?}"
                );
            }
        }
        assert_eq!(exception_files, [PathBuf::from("sys.rs")]);
    }

    // Requirements: SAFE-007, SAFE-009
    //   The crate exposes one non-const callable, which consumes Authorization directly.
    // Evidence: public_callable_surface_is_exactly_the_authorized_entry_point
    #[test]
    fn public_callable_surface_is_exactly_the_authorized_entry_point() {
        let mut public_functions = Vec::new();
        let public_async = ["pub ", "async fn "].concat();
        let public_unsafe = ["pub ", "unsafe", " fn "].concat();
        let public_extern = ["pub ", "extern "].concat();
        for (path, source) in rust_sources() {
            for line in source.lines().map(str::trim) {
                assert!(!line.starts_with("pub mod "), "public module in {path:?}");
                assert!(
                    !line.starts_with("pub use "),
                    "public re-export in {path:?}"
                );
                assert!(
                    !line.starts_with(&public_async),
                    "async public API in {path:?}"
                );
                assert!(
                    !line.starts_with(&public_unsafe),
                    "unsafe public API in {path:?}"
                );
                assert!(
                    !line.starts_with(&public_extern),
                    "extern public API in {path:?}"
                );
                if line.starts_with("pub fn ") {
                    public_functions.push((path.clone(), line.to_owned()));
                }
            }
        }
        assert_eq!(
            public_functions,
            [(
                PathBuf::from("lib.rs"),
                "pub fn run_authorized(authorization: Authorization) -> Result<RunReport, Refusal> {"
                    .to_owned(),
            )]
        );
    }

    // Requirements: SAFE-001, SAFE-007
    //   The harness consumes exactly the two registered generated fixtures, in either order.
    // Evidence: authorized_call_accepts_only_the_registered_two_fixture_roles
    #[test]
    fn authorized_call_accepts_only_the_registered_two_fixture_roles() {
        let sandbox = Sandbox::new();
        let authorization = sandbox.authorization(&[CONFLICTING_NAME, BASIC_NAME]);

        #[cfg(target_os = "linux")]
        {
            let files = consume_exact_targets(authorization).expect("exact target set");
            drop(files);
        }

        #[cfg(not(target_os = "linux"))]
        {
            assert_eq!(
                run_authorized(authorization),
                Err(Refusal::UnsupportedPlatform)
            );
        }
    }

    #[test]
    fn any_other_authorized_fixture_set_is_refused_before_loop_control() {
        let sandbox = Sandbox::new();
        let authorization = sandbox.authorization(&[BASIC_NAME, "blank-512.img"]);
        assert!(matches!(
            consume_exact_targets(authorization),
            Err(Refusal::WrongAuthorizedTargets)
        ));
    }
}
