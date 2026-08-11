//! Descriptor-bound Linux loop-device acceptance (WP-020 increments 2e, 2f,
//! and 2h).
//!
//! Three callable entry points. The first two consume the non-cloneable
//! SAFE-007 [`Authorization`] produced by `partman-fixtures`; the third
//! consumes the registry [`Admission`] that itself consumed one. None accepts
//! a caller-selected file, descriptor, loop number, path, or device name:
//!
//! - [`run_authorized`] is increment 2e's two-leg acceptance. Its probe is
//!   in-process and it launches nothing.
//! - [`run_probed_session`] is increment 2f's hold-open session. It configures
//!   from one held verified backing descriptor, then launches the predeclared
//!   external probers itself — compiled absolute paths, structured argv, no
//!   shell, no `PATH` search, cleared environment, bounded output, and a
//!   timeout — re-verifying node identity and the full `LOOP_GET_STATUS64`
//!   binding immediately before and after every launch. Captured prober
//!   output may quote the device node name, so it is quarantined inside the
//!   session and released only after confirmed detach and partition teardown,
//!   when the name no longer designates the verified backing. No public type
//!   carries a name, path, or device number as a field.
//! - [`run_destructive_suite`] is increment 2h's one registered destructive
//!   suite. It is the only entry point that writes, and what it may write is
//!   not its own choice: the admitted contract's byte range and replacement
//!   bytes are the write, and a length disagreement refuses. Its attachment
//!   is deliberately **read-write**, which is what makes `LOOP_CHANGE_FD`
//!   *inapplicable* rather than merely detected — the pre-write discipline
//!   2e's record requires a destructive path to establish for itself instead
//!   of inheriting. Its own adversarial leg attempts that rebind mid-suite
//!   and requires the kernel to refuse; an acceptance voids the run.
//!
//! No product adapter lives here. The module exists only to prove the narrow
//! Tier-2 loop-control chains authorized by WP-020 — logical-content-read-only
//! for 2e and 2f, and for 2h a single contracted range of harness-owned bytes.
//! Each initial held-file digest must match the fixture bytes compiled into this
//! crate before a loop mapping is configured. The digest and status
//! checks are samples, not locks, and cannot defeat an ABA change entirely
//! between samples. External run evidence must exclude every other actor able
//! to modify a fixture or administer/rebind loop devices; the session's open
//! window makes that exclusion *longer*, not weaker in kind — across it the
//! bracketing detects a rebind that happened, it does not prevent one, which
//! is the recorded reason increment 2f is weaker than 2e. Ordinary
//! kernel/udev read/open discovery is allowed; bounded detach retries and exact
//! retained-rdev sysfs inspection handle its cleanup effects. Both entry
//! points refuse the first `LOOP_CONFIGURE` isolation conflict rather than
//! retrying when isolated loop state was not established. The disposable VM
//! bounds consequences but does not itself prove those exclusions. Neither
//! result may be reused as a continuous-binding guarantee for a future
//! destructive path.

use core::fmt;
use std::ffi::OsStr;
use std::fs::File;

use partman_fixtures::interlock::Authorization;
use partman_fixtures::registry::Admission;

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

/// Normalized proof facts from a completed destructive-suite run.
///
/// Every boolean getter is true for a constructed report, exactly as
/// [`RunReport`]'s are: the report exists only on the path where each fact was
/// established, so the getters name the obligations rather than carry
/// unverified state. No device number, inode, or path is exposed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DestructiveReport {
    contracted_bytes_written: usize,
    detachments_confirmed: u8,
    fixtures_executed: u8,
    ranges_written: usize,
}

impl DestructiveReport {
    /// Bytes written through the held loop-device descriptors, summed across
    /// every declared range of every fixture. Equal to the admitted
    /// contract's total replacement length; a disagreement refused.
    #[must_use]
    pub const fn contracted_bytes_written(&self) -> usize {
        self.contracted_bytes_written
    }

    /// Declared fixtures whose complete chains ran. Equal to the suite's
    /// declared fixture count; a run that could not complete every chain
    /// refused instead of reporting.
    #[must_use]
    pub const fn fixtures_executed(&self) -> u8 {
        self.fixtures_executed
    }

    /// Declared ranges written, summed across fixtures. Equal to the
    /// contract's declared range count; each write's length was checked
    /// against its own range.
    #[must_use]
    pub const fn ranges_written(&self) -> usize {
        self.ranges_written
    }

    /// Whether the attachment was configured read-write, so the kernel's own
    /// loop driver makes `LOOP_CHANGE_FD` inapplicable.
    #[must_use]
    pub const fn attachment_was_read_write(&self) -> bool {
        true
    }

    /// Whether the mid-suite `LOOP_CHANGE_FD` attempt was refused by the
    /// kernel, before any byte was written.
    #[must_use]
    pub const fn forbidden_rebind_refused_by_kernel(&self) -> bool {
        true
    }

    /// Whether the full status binding matched the held backing descriptor
    /// both after attach and after the synced write.
    #[must_use]
    pub const fn required_configuration_verified(&self) -> bool {
        true
    }

    /// Number of configured attachments explicitly detached and confirmed
    /// absent before any post-condition was read.
    #[must_use]
    pub const fn detachments_confirmed(&self) -> u8 {
        self.detachments_confirmed
    }

    /// Whether the released loop node had no materialized partition child in
    /// its exact retained-rdev sysfs directory.
    #[must_use]
    pub const fn partition_teardown_confirmed(&self) -> bool {
        true
    }

    /// Whether every declared range *changed* to exactly its contracted
    /// replacement bytes.
    ///
    /// A difference across the run, not equality after it: each range is read
    /// through the held backing descriptor before the writes and required to
    /// differ from its replacement, and read again after confirmed detach and
    /// required to equal it. Equality alone would also hold for a range that
    /// already carried those bytes and was never written.
    #[must_use]
    pub const fn changed_exactly_as_contracted(&self) -> bool {
        true
    }

    /// Whether the digest over every backing byte outside the declared
    /// ranges was unchanged across the run, for every fixture.
    #[must_use]
    pub const fn unchanged_outside_contract(&self) -> bool {
        true
    }
}

/// One predeclared external tool the increment 2f session may launch.
///
/// The set is closed and compiled in; there is no way to name another
/// executable or another argument shape through the public API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeTool {
    /// `udevadm settle` with a fixed bounded timeout; event completion.
    UdevadmSettle,
    /// `udevadm info --query=all` against one session node.
    UdevadmInfo,
    /// `blkid -p -o udev` against one session node.
    BlkidProbe,
    /// `wipefs -n` against one session node; no-act by construction.
    WipefsNoAct,
}

impl ProbeTool {
    /// Stable label used in refusals and reports; never a path.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::UdevadmSettle => "udevadm-settle",
            Self::UdevadmInfo => "udevadm-info",
            Self::BlkidProbe => "blkid-probe",
            Self::WipefsNoAct => "wipefs-noact",
        }
    }
}

impl fmt::Display for ProbeTool {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

/// Which session node one probe ran against.
///
/// The partition value is the positional index in the kernel's partition
/// child name, not a device number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeSubject {
    /// The whole attached loop disk.
    Disk,
    /// One materialized partition, by kernel partition index.
    Partition(u32),
}

impl fmt::Display for ProbeSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disk => formatter.write_str("disk"),
            Self::Partition(index) => write!(formatter, "partition-{index}"),
        }
    }
}

/// One captured external probe from a completed session.
///
/// The byte fields are the prober's raw bounded output and may quote the
/// transient device node name; the session releases them only after confirmed
/// detach and partition teardown, when that name no longer designates the
/// verified backing. No field carries a name, path, or device number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProbeRecord {
    tool: ProbeTool,
    subject: ProbeSubject,
    exit_code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl ProbeRecord {
    /// Which predeclared tool ran.
    #[must_use]
    pub const fn tool(&self) -> ProbeTool {
        self.tool
    }

    /// Which session node it ran against.
    #[must_use]
    pub const fn subject(&self) -> ProbeSubject {
        self.subject
    }

    /// The tool's exit code, or `None` when it exited on a signal.
    #[must_use]
    pub const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Raw bounded standard output, quarantined until after teardown.
    #[must_use]
    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    /// Raw bounded standard error, quarantined until after teardown.
    #[must_use]
    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }
}

/// The named sysfs projection facts for the attached disk, read in process
/// from the session's retained-rdev root. Integers only; no identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionDiskFacts {
    size_sectors: u64,
    read_only: bool,
    logical_block_size: u32,
}

impl SessionDiskFacts {
    /// The disk's `size` attribute, in 512-byte sysfs sectors.
    #[must_use]
    pub const fn size_sectors(&self) -> u64 {
        self.size_sectors
    }

    /// The disk's `ro` attribute; the session refuses when this is false.
    #[must_use]
    pub const fn read_only(&self) -> bool {
        self.read_only
    }

    /// The disk's `queue/logical_block_size` attribute.
    #[must_use]
    pub const fn logical_block_size(&self) -> u32 {
        self.logical_block_size
    }
}

/// The named sysfs projection facts for one materialized partition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionPartitionFacts {
    index: u32,
    start_sectors: u64,
    size_sectors: u64,
    read_only: bool,
}

impl SessionPartitionFacts {
    /// The partition's `partition` attribute — its kernel index.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// The partition's `start` attribute, in 512-byte sysfs sectors.
    #[must_use]
    pub const fn start_sectors(&self) -> u64 {
        self.start_sectors
    }

    /// The partition's `size` attribute, in 512-byte sysfs sectors.
    #[must_use]
    pub const fn size_sectors(&self) -> u64 {
        self.size_sectors
    }

    /// The partition's `ro` attribute; the session refuses when this is false.
    #[must_use]
    pub const fn read_only(&self) -> bool {
        self.read_only
    }
}

/// Quarantine-released facts and captures from one completed 2f session.
///
/// Constructed at exactly one site, after the sequence closed, detach and
/// partition teardown were confirmed, and the post-run fixture hash matched.
/// Every boolean getter is true for a constructed report; they stay explicit
/// so the instrument can print the obligations it actually established.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionReport {
    fixture: FixtureRole,
    partitions_observed: u8,
    disk_facts: SessionDiskFacts,
    partition_facts: Vec<SessionPartitionFacts>,
    records: Vec<ProbeRecord>,
}

impl SessionReport {
    /// Which registered fixture backed the session.
    #[must_use]
    pub const fn fixture(&self) -> FixtureRole {
        self.fixture
    }

    /// How many materialized partitions the session enumerated and probed.
    #[must_use]
    pub const fn partitions_observed(&self) -> u8 {
        self.partitions_observed
    }

    /// Every captured probe, in launch order.
    #[must_use]
    pub fn records(&self) -> &[ProbeRecord] {
        &self.records
    }

    /// The disk's named sysfs projection facts, read in process.
    #[must_use]
    pub const fn disk_facts(&self) -> SessionDiskFacts {
        self.disk_facts
    }

    /// Every enumerated partition's named sysfs projection facts, by index.
    #[must_use]
    pub fn partition_facts(&self) -> &[SessionPartitionFacts] {
        &self.partition_facts
    }

    /// Whether node identity and the full status binding were re-verified
    /// immediately before and after every external launch.
    #[must_use]
    pub const fn bindings_verified_around_every_launch(&self) -> bool {
        true
    }

    /// Whether the attached device's complete logical contents, read through
    /// the held loop descriptor, equaled the compiled catalogue digest both
    /// before the first and after the last external launch.
    #[must_use]
    pub const fn loop_content_hashes_matched_catalogue(&self) -> bool {
        true
    }

    /// Whether no session device number appeared in the mount table and every
    /// session node reported read-only, checked in process during the window.
    #[must_use]
    pub const fn nodes_unmounted_and_read_only(&self) -> bool {
        true
    }

    /// Whether explicit detach was confirmed by `ENXIO` status read-back.
    #[must_use]
    pub const fn detachment_confirmed(&self) -> bool {
        true
    }

    /// Whether the exact retained-rdev sysfs root had no partition child
    /// after descriptor release.
    #[must_use]
    pub const fn partition_teardown_confirmed(&self) -> bool {
        true
    }

    /// Whether the initial held-file hash matched the compiled catalogue.
    #[must_use]
    pub const fn initial_fixture_hash_matched_catalogue(&self) -> bool {
        true
    }

    /// Whether the post-run held-file hash equaled the pre-run value.
    #[must_use]
    pub const fn fixture_hash_unchanged(&self) -> bool {
        true
    }

    /// Whether captured output stayed sealed until detach and teardown were
    /// confirmed. Structural: publication is the only route out of the gate.
    #[must_use]
    pub const fn captured_output_quarantined_until_teardown(&self) -> bool {
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
    /// The session authorization did not contain exactly one registered fixture.
    WrongSessionTarget,
    /// A predeclared prober was absent from its compiled absolute path.
    ProbeToolMissing {
        /// Stable tool label; never a path.
        tool: &'static str,
    },
    /// A predeclared prober failed to launch.
    ProbeLaunchFailed {
        /// Stable tool label; never a path.
        tool: &'static str,
        /// Bounded OS error number when one was available.
        errno: Option<i32>,
    },
    /// A predeclared prober exceeded the session launch time limit.
    ProbeTimedOut {
        /// Stable tool label; never a path.
        tool: &'static str,
    },
    /// A prober stream exceeded the bounded capture limit; truncated output
    /// would be incomplete evidence, so the session refuses instead.
    ProbeOutputOverLimit {
        /// Stable tool label; never a path.
        tool: &'static str,
    },
    /// A prober exited outside its allowed exit set.
    ProbeUnexpectedExit {
        /// Stable tool label; never a path.
        tool: &'static str,
        /// The exit code, or `None` when it exited on a signal.
        code: Option<i32>,
    },
    /// A public device node re-stat did not match the held identity.
    NodePathIdentityMismatch,
    /// The descriptor-derived sysfs partition enumeration failed or was
    /// malformed.
    PartitionEnumerationFailed {
        /// Bounded OS error number for unreadable sysfs state.
        errno: Option<i32>,
    },
    /// More partitions materialized than the session bound permits.
    PartitionCountExceeded,
    /// The attached device's complete logical contents, read through the held
    /// loop descriptor, did not equal the compiled catalogue digest.
    LoopDeviceHashMismatch,
    /// A session device number appeared in the mount table.
    SessionNodeMounted,
    /// A session node did not report read-only in its sysfs `ro` attribute.
    SessionNodeWritable,
    /// An internal protocol transition attempted to publish evidence early.
    ProtocolOrder,
    /// The admitted suite is not one the compiled registry holds.
    SuiteNotRegistered,
    /// The admitted suite's shape is not one this executor runs: at least
    /// one fixture contract, each with at least one declared range whose
    /// replacement is non-empty and matches its length, ranges
    /// non-overlapping, and exactly one verified target per declared
    /// fixture.
    WrongSuiteShape,
    /// The held backing object's bytes are not the compiled fixture the
    /// contract names, so nothing may be attached or written.
    InitialBackingHashMismatch,
    /// The declared range already holds the contracted replacement bytes, so
    /// a run could not establish that it *changed* them.
    RangeAlreadyContracted,
    /// The kernel accepted `LOOP_CHANGE_FD` on what this protocol required to
    /// be a read-write attachment. The attachment is not what the run
    /// believed, and nothing is published.
    RebindUnexpectedlyApplicable,
    /// The contracted write returned a different byte count than the
    /// contract's replacement.
    ContractedWriteWrongLength,
    /// After confirmed detach, the declared range did not equal the
    /// contract's replacement bytes exactly.
    DeclaredRangeMismatch,
    /// After confirmed detach, bytes outside the declared range changed: the
    /// digest bracket did not hold.
    OutsideBracketChanged,
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
            Self::WrongAuthorizedTargets | Self::WrongSessionTarget => {
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

impl Refusal {
    /// Render the increment 2f session variants; `None` for every other
    /// variant, which the acceptance formatter below owns.
    fn fmt_session(&self, formatter: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
        Some(match self {
            Self::WrongSessionTarget => formatter.write_str(
                "session authorization must contain exactly one of gpt-basic-512.img or \
                 gpt-conflicting-tables-512.img",
            ),
            Self::ProbeToolMissing { tool } => write!(
                formatter,
                "predeclared prober {tool} is absent from its compiled location"
            ),
            Self::ProbeLaunchFailed { tool, errno } => match errno {
                Some(errno) => write!(
                    formatter,
                    "predeclared prober {tool} failed to launch (errno {errno})"
                ),
                None => write!(formatter, "predeclared prober {tool} failed to launch"),
            },
            Self::ProbeTimedOut { tool } => write!(
                formatter,
                "predeclared prober {tool} exceeded the session launch time limit"
            ),
            Self::ProbeOutputOverLimit { tool } => write!(
                formatter,
                "predeclared prober {tool} exceeded the bounded output capture limit"
            ),
            Self::ProbeUnexpectedExit { tool, code } => match code {
                Some(code) => write!(
                    formatter,
                    "predeclared prober {tool} exited outside its allowed set (code {code})"
                ),
                None => write!(
                    formatter,
                    "predeclared prober {tool} exited on a signal outside its allowed set"
                ),
            },
            Self::NodePathIdentityMismatch => {
                formatter.write_str("a device-node re-stat did not match the held session identity")
            }
            Self::PartitionEnumerationFailed { errno } => write_errno(
                formatter,
                "descriptor-derived sysfs partition enumeration failed or was malformed",
                *errno,
            ),
            Self::PartitionCountExceeded => {
                formatter.write_str("more partitions materialized than the bounded session permits")
            }
            Self::LoopDeviceHashMismatch => formatter.write_str(
                "the attached device's logical contents did not equal the compiled catalogue \
                 digest",
            ),
            Self::SessionNodeMounted => {
                formatter.write_str("a session device number appeared in the mount table")
            }
            Self::SessionNodeWritable => {
                formatter.write_str("a session node did not report read-only")
            }
            _ => return None,
        })
    }

    /// Increment 2h's destructive refusals, split out for the same reason
    /// [`Self::fmt_session`] is: one `Display` arm per variant would put the
    /// whole vocabulary in a single function that no reviewer can hold, and
    /// the workspace lint says so.
    /// The detach and teardown refusals, shared by every protocol here.
    ///
    /// Split out for the same reason as the two renderers beside it: one
    /// `Display` arm per variant would put a vocabulary this size in a single
    /// function no reviewer can hold, and the workspace lint says so.
    fn fmt_cleanup(&self, formatter: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
        Some(match self {
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
            _ => return None,
        })
    }

    fn fmt_destructive(&self, formatter: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
        Some(match self {
            Self::SuiteNotRegistered => formatter.write_str(
                "the admitted suite is not one the compiled registry holds; a destructive \
                 suite runs only from the registry",
            ),
            Self::InitialBackingHashMismatch => formatter.write_str(
                "the held backing object is not the compiled fixture the contract names; \
                 nothing was attached or written",
            ),
            Self::RangeAlreadyContracted => formatter.write_str(
                "the declared range already holds the contracted replacement bytes, so this \
                 run could not establish that it changed them",
            ),
            Self::WrongSuiteShape => formatter.write_str(
                "the admitted suite is not a shape this executor runs: at least one fixture \
                 contract, each with at least one non-overlapping declared range whose \
                 replacement is non-empty and matches its length, and exactly one verified \
                 target per declared fixture",
            ),
            Self::RebindUnexpectedlyApplicable => formatter.write_str(
                "the kernel accepted LOOP_CHANGE_FD on what must be a read-write attachment; \
                 the run is void and nothing was published",
            ),
            Self::ContractedWriteWrongLength => formatter.write_str(
                "the contracted write did not write exactly the contract's replacement length",
            ),
            Self::DeclaredRangeMismatch => formatter.write_str(
                "after confirmed detach the declared range did not equal the contracted \
                 replacement bytes",
            ),
            Self::OutsideBracketChanged => formatter.write_str(
                "after confirmed detach the digest bracket outside the declared range changed",
            ),
            _ => return None,
        })
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(result) = self.fmt_session(formatter) {
            return result;
        }
        if let Some(result) = self.fmt_destructive(formatter) {
            return result;
        }
        if let Some(result) = self.fmt_cleanup(formatter) {
            return result;
        }
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
            Self::FixtureHashChanged { fixture } => {
                write!(
                    formatter,
                    "the {fixture} fixture hash changed during the read-only run"
                )
            }
            Self::ProtocolOrder => {
                formatter.write_str("evidence publication was attempted before final verification")
            }
            // Rendered above by `fmt_session` and `fmt_destructive`. These
            // arms are unreachable in practice but must stay total without
            // panicking, and they are listed rather than wildcarded so a new
            // variant still has to be placed deliberately.
            Self::WrongSessionTarget
            | Self::LoopDeviceHashMismatch
            | Self::SessionNodeMounted
            | Self::SessionNodeWritable
            | Self::ProbeToolMissing { .. }
            | Self::ProbeLaunchFailed { .. }
            | Self::ProbeTimedOut { .. }
            | Self::ProbeOutputOverLimit { .. }
            | Self::ProbeUnexpectedExit { .. }
            | Self::NodePathIdentityMismatch
            | Self::PartitionEnumerationFailed { .. }
            | Self::PartitionCountExceeded
            | Self::DetachFailed { .. }
            | Self::DetachConfirmationFailed { .. }
            | Self::DetachNotConfirmed
            | Self::PartitionTeardownNotConfirmed { .. }
            | Self::SuiteNotRegistered
            | Self::WrongSuiteShape
            | Self::InitialBackingHashMismatch
            | Self::RangeAlreadyContracted
            | Self::RebindUnexpectedlyApplicable
            | Self::ContractedWriteWrongLength
            | Self::DeclaredRangeMismatch
            | Self::OutsideBracketChanged => Ok(()),
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

/// Run one increment 2f hold-open session over a single authorized fixture.
///
/// The authorization must select exactly one of the two registered fixtures.
/// The session configures a read-only loop mapping from that held descriptor,
/// launches the predeclared external probers itself with node identity and the
/// full status binding re-verified immediately before and after every launch,
/// detaches under the confirmed-detach and partition-teardown discipline, and
/// releases captured output only after that teardown is confirmed. There is no
/// API that accepts a caller-selected file, raw descriptor, loop number, or
/// path, and no return value lets a caller reach the loop device while bound.
///
/// # Errors
///
/// Returns [`Refusal`] on every unsupported platform, authorization mismatch,
/// missing prober, launch failure, identity mismatch, cleanup failure, or
/// fixture-byte change. A refusal carries no captured prober output.
///
/// ```compile_fail
/// use std::fs::File;
/// use partman_ffi_linux_loop::run_probed_session;
///
/// fn bypass_interlock(file: File) {
///     let _ = run_probed_session(file);
/// }
/// ```
pub fn run_probed_session(authorization: Authorization) -> Result<SessionReport, Refusal> {
    let selected = consume_single_target(authorization)?;

    #[cfg(target_os = "linux")]
    {
        let (fixture, backing) = selected;
        linux::run_session(fixture, backing)
    }

    #[cfg(not(target_os = "linux"))]
    {
        drop(selected);
        Err(Refusal::UnsupportedPlatform)
    }
}

/// One declared byte range of an admitted fixture contract.
///
/// The offsets, lengths, and replacement bytes here are the registry's
/// compiled data — never an argument, and never re-derived from the fixture
/// on disk.
pub(crate) struct AdmittedRange {
    pub(crate) offset: u64,
    pub(crate) length: u64,
    pub(crate) replacement: &'static [u8],
}

/// One admitted fixture: its held verified object and its declared ranges,
/// in the contract's declared order.
pub(crate) struct AdmittedFixture {
    /// The contract's catalogue basename, used to resolve the compiled
    /// digest the held object must match before anything is attached.
    pub(crate) fixture: &'static str,
    pub(crate) ranges: Vec<AdmittedRange>,
    pub(crate) backing: File,
}

/// The admitted contract, reduced to exactly what the executor writes:
/// one entry per declared fixture, each holding the verified object the
/// interlock opened for it.
///
/// Constructed only by [`consume_admission`].
pub(crate) struct AdmittedContract {
    pub(crate) fixtures: Vec<AdmittedFixture>,
}

/// Run one registered destructive suite over its admitted targets.
///
/// Increment 2i generalized this executor from the 2h shape (exactly one
/// fixture, exactly one range) to the registry's full contract shape: any
/// number of declared fixtures, each with any number of non-overlapping
/// declared ranges. Before **any** fixture is attached, every fixture's held
/// bytes must match the compiled catalogue — a suite whose second fixture is
/// wrong refuses before its first is touched — and each fixture then runs a
/// complete attach-to-post-condition chain in declared order.
///
/// Per fixture, the attachment is read-write, which is what makes
/// `LOOP_CHANGE_FD` inapplicable rather than merely detectable; the chain
/// attempts that rebind mid-run, before any write, and requires the kernel to
/// refuse it. The writes are exactly the admitted contract's replacement
/// bytes at their declared offsets, through the held loop-device descriptor,
/// in declared order. Post-conditions are read only after confirmed detach
/// and partition teardown, through the still-held backing descriptor: every
/// declared range must equal its contracted bytes and the digest over
/// everything outside the declared ranges must be unchanged.
///
/// Regenerating the mutated backing file to the compiled catalogue is the
/// caller's teardown obligation, recorded on the suite and discharged by the
/// task runner after this returns.
///
/// # Errors
///
/// Returns [`Refusal`] on an unsupported platform, a suite shape this
/// executor does not run, a kernel-accepted rebind, an identity mismatch, a
/// short or over-long write, a cleanup failure, or either post-condition
/// failing. Nothing is published on any of those paths.
///
/// The call-site boundary is structural: an admission is the only accepted
/// argument, and an `Authorization` — which has not passed the registry's
/// contract checks — is not one.
///
/// ```compile_fail
/// use partman_fixtures::interlock::Authorization;
/// use partman_ffi_linux_loop::run_destructive_suite;
///
/// fn bypass_registry(authorization: Authorization) {
///     let _ = run_destructive_suite(authorization);
/// }
/// ```
pub fn run_destructive_suite(admission: Admission) -> Result<DestructiveReport, Refusal> {
    let contract = consume_admission(admission)?;

    #[cfg(target_os = "linux")]
    {
        linux::run_destructive(contract)
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Destructured rather than dropped whole, matching `run_authorized`:
        // every field is named on this path too, so the fields stay live for
        // the compiler on platforms where no controller reads them.
        let AdmittedContract { fixtures } = contract;
        for AdmittedFixture {
            fixture,
            ranges,
            backing,
        } in fixtures
        {
            for AdmittedRange {
                offset,
                length,
                replacement,
            } in ranges
            {
                let _ = (offset, length, replacement);
            }
            drop((fixture, backing));
        }
        Err(Refusal::UnsupportedPlatform)
    }
}

/// Reduce an admission to its contracts and held objects, one per fixture.
///
/// The registry already refused a malformed contract and a target set that is
/// not exactly one verified handle per declared fixture. This executor
/// nonetheless states its own preconditions — at least one fixture, each with
/// at least one declared range, every replacement non-empty and matching its
/// range's length, ranges non-overlapping, and exactly one held object per
/// declared fixture matched by basename — because "the caller checked" is
/// the shape of reasoning this package declines. Until increment 2i this
/// executor ran exactly the 2h shape (one fixture, one range) and its comment
/// said a suite that outgrows the shape gets a new executor; 2i is that
/// reviewed replacement, and the preconditions above are the general shape's,
/// stronger at every point where the arity check used to stand in for them.
fn consume_admission(admission: Admission) -> Result<AdmittedContract, Refusal> {
    let suite = admission.suite();

    // The suite must be one the compiled registry holds. `Admission::admit`
    // deliberately does not require this — it exists to check a contract
    // against the catalogue, and the registry's own tests exercise it with a
    // test-only suite — but `Suite` is a public struct with public fields, so
    // without this the executor would write on behalf of any `&'static Suite`
    // a caller could name and the registry would gate nothing. An adversarial
    // review found exactly that. Compared by address rather than by value: a
    // suite equal to a registered one but not *in* the registry is still not
    // registered.
    if !partman_fixtures::registry::registered()
        .iter()
        .any(|candidate| core::ptr::eq(candidate, suite))
    {
        return Err(Refusal::SuiteNotRegistered);
    }

    reduce_admission(admission)
}

/// The shape reduction half of [`consume_admission`], after the registry
/// membership check.
///
/// Split out so the general shape is measurable at Tier 1: the registry holds
/// exactly the 2h suite, so a multi-fixture or multi-range shape can never
/// reach this code through `consume_admission` until a suite of that shape is
/// registered — and a reduction whose first exercise is a registered suite's
/// VM acceptance is exactly the untested-until-privileged shape issue #250
/// was filed about.
fn reduce_admission(admission: Admission) -> Result<AdmittedContract, Refusal> {
    let suite = admission.suite();

    if suite.fixtures.is_empty() {
        return Err(Refusal::WrongSuiteShape);
    }

    let mut targets = admission.into_targets();
    if targets.len() != suite.fixtures.len() {
        return Err(Refusal::WrongSuiteShape);
    }

    let mut fixtures = Vec::with_capacity(suite.fixtures.len());
    for contract in suite.fixtures {
        // Exactly one held object per declared fixture, matched by basename
        // and *removed* as it is matched: a second handle carrying the same
        // name cannot satisfy a different declared fixture, and anything left
        // over afterwards refuses below.
        let position = targets
            .iter()
            .position(|target| target.path().file_name() == Some(OsStr::new(contract.fixture)))
            .ok_or(Refusal::WrongSuiteShape)?;
        let target = targets.remove(position);

        if contract.may_change.is_empty() {
            return Err(Refusal::WrongSuiteShape);
        }
        let mut ranges = Vec::with_capacity(contract.may_change.len());
        for range in contract.may_change {
            if range.replacement.is_empty()
                || u64::try_from(range.replacement.len()).is_ok_and(|len| len != range.length)
            {
                return Err(Refusal::WrongSuiteShape);
            }
            ranges.push(AdmittedRange {
                offset: range.offset,
                length: range.length,
                replacement: range.replacement,
            });
        }
        // Overlapping — or end-overflowing — declared ranges make "everything
        // outside the contract" ambiguous, so the bracket's meaning depends
        // on refusing them here even though the registry already did.
        let mut pairs: Vec<(u64, u64)> = ranges
            .iter()
            .map(|range| (range.offset, range.length))
            .collect();
        pairs.sort_unstable();
        for pair in pairs.windows(2) {
            if pair[0]
                .0
                .checked_add(pair[0].1)
                .is_none_or(|end| end > pair[1].0)
            {
                return Err(Refusal::WrongSuiteShape);
            }
        }
        if let Some(last) = pairs.last()
            && last.0.checked_add(last.1).is_none()
        {
            return Err(Refusal::WrongSuiteShape);
        }

        fixtures.push(AdmittedFixture {
            fixture: contract.fixture,
            ranges,
            backing: target.into_file(),
        });
    }
    if !targets.is_empty() {
        return Err(Refusal::WrongSuiteShape);
    }

    Ok(AdmittedContract { fixtures })
}

fn consume_single_target(authorization: Authorization) -> Result<(FixtureRole, File), Refusal> {
    let mut targets = authorization.into_targets();
    if targets.len() != 1 {
        return Err(Refusal::WrongSessionTarget);
    }
    let target = targets.remove(0);
    let fixture = match target.path().file_name() {
        Some(name) if name == OsStr::new(BASIC_NAME) => FixtureRole::Basic,
        Some(name) if name == OsStr::new(CONFLICTING_NAME) => FixtureRole::Conflicting,
        _ => return Err(Refusal::WrongSessionTarget),
    };
    Ok((fixture, target.into_file()))
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
    use partman_fixtures::registry::{
        FixtureContract, IntendedChange, Suite, TargetClass, registered,
    };
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static UNREGISTERED_REPLACEMENT: [u8; 8] = [0; 8];
    static UNREGISTERED_RANGES: [IntendedChange; 1] = [IntendedChange {
        offset: 512,
        length: 8,
        replacement: &UNREGISTERED_REPLACEMENT,
        reason: "a well-formed contract that the registry nonetheless does not hold",
    }];
    static UNREGISTERED_FIXTURES: [FixtureContract; 1] = [FixtureContract {
        fixture: "gpt-basic-512.img",
        may_change: &UNREGISTERED_RANGES,
    }];
    /// A suite that is well-formed in every way except the one that matters.
    static UNREGISTERED_SUITE: Suite = Suite {
        name: "test-only-unregistered-suite",
        target_class: TargetClass::GeneratedFixtureFile,
        fixtures: &UNREGISTERED_FIXTURES,
        teardown: &[],
    };

    /// A general-shape suite for the reduction test: two fixtures, one of
    /// them carrying two declared ranges. Never registered; the reduction is
    /// tested directly, past the membership check `consume_admission` runs.
    static GENERAL_SECOND_REPLACEMENT: [u8; 4] = [0xAA; 4];
    static GENERAL_BASIC_RANGES: [IntendedChange; 2] = [
        IntendedChange {
            offset: 512,
            length: 8,
            replacement: &UNREGISTERED_REPLACEMENT,
            reason: "the primary GPT header signature",
        },
        IntendedChange {
            offset: 1024,
            length: 4,
            replacement: &GENERAL_SECOND_REPLACEMENT,
            reason: "a second declared range, so the reduction carries more than one",
        },
    ];
    static GENERAL_BLANK_RANGES: [IntendedChange; 1] = [IntendedChange {
        offset: 2048,
        length: 8,
        replacement: &UNREGISTERED_REPLACEMENT,
        reason: "one range of the second declared fixture",
    }];
    static GENERAL_FIXTURES: [FixtureContract; 2] = [
        FixtureContract {
            fixture: "gpt-basic-512.img",
            may_change: &GENERAL_BASIC_RANGES,
        },
        FixtureContract {
            fixture: "blank-512.img",
            may_change: &GENERAL_BLANK_RANGES,
        },
    ];
    static GENERAL_SUITE: Suite = Suite {
        name: "test-only-general-suite",
        target_class: TargetClass::GeneratedFixtureFile,
        fixtures: &GENERAL_FIXTURES,
        teardown: &[],
    };

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

    // Requirements: SAFE-007, SAFE-005
    //   The destructive executor runs only a suite the compiled registry holds; a well-formed but unregistered suite is refused even though the interlock authorized its target
    // Work-Package: WP-020
    // Evidence: the_executor_refuses_a_suite_the_registry_does_not_hold
    #[test]
    fn the_executor_refuses_a_suite_the_registry_does_not_hold() {
        // `Suite` is a public type with public fields and `Admission::admit`
        // accepts any `&'static Suite`, so without the executor's membership
        // check the registry would gate nothing. This constructs exactly the
        // suite that check exists to refuse: same fixture, same contract, same
        // shape as the registered one — and not in the registry.
        let unregistered: *const Suite = &raw const UNREGISTERED_SUITE;
        assert!(
            !registered()
                .iter()
                .any(|suite| core::ptr::eq(suite, unregistered)),
            "the fixture of this test is that the suite is NOT registered"
        );

        let sandbox = Sandbox::new();
        let authorization = sandbox.authorization(&["gpt-basic-512.img"]);
        let admission =
            partman_fixtures::registry::Admission::admit(&UNREGISTERED_SUITE, authorization)
                .expect("the contract itself is well formed, so admission succeeds");

        assert_eq!(
            run_destructive_suite(admission)
                .expect_err("an unregistered suite must never reach a write"),
            Refusal::SuiteNotRegistered
        );
    }

    // Requirements: SAFE-007, SAFE-005
    //   The shipped two-range suite reduces through the real executor path — registry membership included — to one fixture carrying both declared ranges in declared order, so its shape is executed at Tier 1 rather than first met by a privileged kernel
    // Work-Package: WP-020
    // Evidence: the_shipped_two_range_suite_reduces_through_the_executor_path
    #[test]
    fn the_shipped_two_range_suite_reduces_through_the_executor_path() {
        let suite = &registered()[1];
        assert_eq!(suite.name, "gpt-basic-512-both-signatures-erase");

        let sandbox = Sandbox::new();
        let authorization = sandbox.authorization(&["gpt-basic-512.img"]);
        let admission = partman_fixtures::registry::Admission::admit(suite, authorization)
            .expect("the shipped suite admits over its declared fixture");
        let contract = consume_admission(admission)
            .expect("a registered suite passes the membership check and reduces");

        assert_eq!(contract.fixtures.len(), 1);
        let fixture = &contract.fixtures[0];
        assert_eq!(fixture.fixture, "gpt-basic-512.img");
        assert_eq!(fixture.ranges.len(), 2, "both declared ranges carried");
        assert_eq!(
            (fixture.ranges[0].offset, fixture.ranges[0].length),
            (512, 8)
        );
        assert_eq!(
            (fixture.ranges[1].offset, fixture.ranges[1].length),
            (4 * 1024 * 1024 - 512, 8),
            "the backup GPT header's signature at the last LBA"
        );
        assert_eq!(fixture.ranges[0].replacement, [0_u8; 8]);
        assert_eq!(fixture.ranges[1].replacement, [0_u8; 8]);
    }

    // Requirements: SAFE-007, SAFE-005
    //   The general reduction binds each declared fixture to the verified object of exactly that name, in declared order, carrying every declared range — proven by reading the held objects' bytes, not by matching names to names
    // Work-Package: WP-020
    // Evidence: the_general_reduction_binds_each_contract_to_its_own_held_object
    #[test]
    fn the_general_reduction_binds_each_contract_to_its_own_held_object() {
        use std::io::{Read as _, Seek as _, SeekFrom};

        let sandbox = Sandbox::new();
        // Authorized in the OPPOSITE order to the suite's declaration, so a
        // reduction that zipped targets to contracts positionally would hand
        // each contract the wrong object — which the byte reads below would
        // catch, because the two fixtures differ at offset 512.
        let authorization = sandbox.authorization(&["blank-512.img", "gpt-basic-512.img"]);
        let admission = partman_fixtures::registry::Admission::admit(&GENERAL_SUITE, authorization)
            .expect("the general contract is well formed, so admission succeeds");

        let contract = reduce_admission(admission).expect("the general shape reduces");
        assert_eq!(contract.fixtures.len(), 2, "one entry per declared fixture");

        let mut fixtures = contract.fixtures;
        let blank = fixtures.pop().expect("second declared fixture");
        let basic = fixtures.pop().expect("first declared fixture");

        assert_eq!(basic.fixture, "gpt-basic-512.img");
        assert_eq!(basic.ranges.len(), 2, "both declared ranges carried");
        assert_eq!(
            (basic.ranges[0].offset, basic.ranges[0].length),
            (512, 8),
            "ranges carried in declared order with the compiled offsets"
        );
        assert_eq!((basic.ranges[1].offset, basic.ranges[1].length), (1024, 4));
        assert_eq!(blank.fixture, "blank-512.img");
        assert_eq!(blank.ranges.len(), 1);

        // The binding proof: read through each held object. The GPT fixture
        // carries its signature at offset 512; the blank fixture carries
        // zeros there. A name-to-name match with the wrong object attached
        // would pass every assertion above and fail here.
        let mut signature = [0_u8; 8];
        let mut basic_backing = basic.backing;
        basic_backing
            .seek(SeekFrom::Start(512))
            .expect("seek the held object");
        basic_backing
            .read_exact(&mut signature)
            .expect("read the held object");
        assert_eq!(
            &signature, b"EFI PART",
            "the first contract holds the GPT fixture"
        );

        let mut blank_bytes = [0_u8; 8];
        let mut blank_backing = blank.backing;
        blank_backing
            .seek(SeekFrom::Start(512))
            .expect("seek the held object");
        blank_backing
            .read_exact(&mut blank_bytes)
            .expect("read the held object");
        assert_eq!(
            blank_bytes, [0; 8],
            "the second contract holds the blank fixture"
        );
    }

    // Requirements: SAFE-007, SAFE-009
    //   The crate exposes exactly the three authorized entry points plus four named
    //   borrowing getters, all in lib.rs, each consuming Authorization or the
    //   registry Admission directly, or reading an already-published report.
    // Work-Package: WP-020
    // Evidence: public_callable_surface_is_exactly_the_authorized_entry_points
    #[test]
    fn public_callable_surface_is_exactly_the_authorized_entry_points() {
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
            [
                (
                    PathBuf::from("lib.rs"),
                    "pub fn stdout(&self) -> &[u8] {".to_owned(),
                ),
                (
                    PathBuf::from("lib.rs"),
                    "pub fn stderr(&self) -> &[u8] {".to_owned(),
                ),
                (
                    PathBuf::from("lib.rs"),
                    "pub fn records(&self) -> &[ProbeRecord] {".to_owned(),
                ),
                (
                    PathBuf::from("lib.rs"),
                    "pub fn partition_facts(&self) -> &[SessionPartitionFacts] {".to_owned(),
                ),
                (
                    PathBuf::from("lib.rs"),
                    "pub fn run_authorized(authorization: Authorization) -> Result<RunReport, Refusal> {"
                        .to_owned(),
                ),
                (
                    PathBuf::from("lib.rs"),
                    "pub fn run_probed_session(authorization: Authorization) -> Result<SessionReport, Refusal> {"
                        .to_owned(),
                ),
                // Widened by increment 2h, as its recorded boundary requires:
                // exactly one new entry point, and it takes the registry
                // `Admission` — not an `Authorization`, and not a file — so
                // the compiled contract is unavoidable on the write path.
                (
                    PathBuf::from("lib.rs"),
                    "pub fn run_destructive_suite(admission: Admission) -> Result<DestructiveReport, Refusal> {"
                        .to_owned(),
                ),
            ]
        );
    }

    // Requirements: SAFE-006, SAFE-007
    //   No public signature returns a descriptor, borrowed or owned fd, file,
    //   device name, path, or device number. The scan reads every `pub fn` and
    //   `pub const fn` line in the crate; its reach is exactly those lines, so a
    //   public field would need its own escape — and every public struct here
    //   declares only private fields, which the companion assertion checks.
    // Evidence: no_public_signature_returns_a_descriptor_name_path_or_device_number
    #[test]
    fn no_public_signature_returns_a_descriptor_name_path_or_device_number() {
        let forbidden_returns = [
            "RawFd",
            "OwnedFd",
            "BorrowedFd",
            "File",
            "PathBuf",
            "Path",
            "Dev",
            "OsStr",
            "OsString",
            "CString",
        ];
        let mut signatures_seen = 0;
        for (path, source) in rust_sources() {
            for line in source.lines().map(str::trim) {
                let is_public_fn = line.starts_with("pub fn ") || line.starts_with("pub const fn ");
                if !is_public_fn {
                    continue;
                }
                signatures_seen += 1;
                let return_type = line.split("->").nth(1).unwrap_or("");
                for forbidden in forbidden_returns {
                    assert!(
                        !return_type.contains(forbidden),
                        "public signature in {path:?} returns {forbidden}: {line}"
                    );
                }
            }
            // A `pub` field would bypass the getter scan; require none.
            for line in source.lines().map(str::trim) {
                if line.starts_with("pub ")
                    && !line.starts_with("pub fn ")
                    && !line.starts_with("pub const fn ")
                    && !line.starts_with("pub struct ")
                    && !line.starts_with("pub enum ")
                {
                    let looks_like_field = line.ends_with(',')
                        && line.contains(':')
                        && !line.contains('(')
                        && !line.contains("fn ");
                    assert!(
                        !looks_like_field,
                        "public field in {path:?} bypasses the signature scan: {line}"
                    );
                }
            }
        }
        assert!(
            signatures_seen >= 20,
            "the scan stopped seeing public signatures, so it proves nothing"
        );
    }

    // Requirements: SAFE-004
    //   The prober launch reaches no shell and no PATH lookup: every launch site
    //   is the one bounded launcher, its program comes from the compiled
    //   absolute roster, the environment is cleared with one fixed locale pin,
    //   and no shell binary or `-c` argument appears anywhere in the crate.
    //   Self-referential literals are built by concatenation so this test's own
    //   source cannot satisfy or violate the scan.
    // Evidence: the_prober_launch_reaches_no_shell_and_no_path_lookup
    #[test]
    fn the_prober_launch_reaches_no_shell_and_no_path_lookup() {
        let launch_call = ["Command", "::new("].concat();
        let launch_with_roster_parameter = ["Command", "::new(path)"].concat();
        let env_clear_call = [".env", "_clear()"].concat();
        let env_call = [".env", "("].concat();
        let locale_pin = ["\"LC_ALL\"", ", ", "\"C\""].concat();
        let shell_literal = ["\"", "s", "h", "\""].concat();
        let shell_path_literal = ["\"/bin/", "s", "h", "\""].concat();
        let shell_flag_literal = ["\"-", "c", "\""].concat();
        let roster_literals = [
            ["\"/usr/bin/", "udevadm", "\""].concat(),
            ["\"/usr/sbin/", "blkid", "\""].concat(),
            ["\"/usr/sbin/", "wipefs", "\""].concat(),
        ];
        let mut launch_sites = 0;
        let mut env_clear_sites = 0;
        let mut roster_paths = 0;
        for (path, source) in rust_sources() {
            for line in source.lines().map(str::trim) {
                if line.contains(&launch_call) {
                    launch_sites += 1;
                    assert_eq!(
                        path,
                        PathBuf::from("linux.rs"),
                        "a launch site escaped the session module"
                    );
                    assert!(
                        line.contains(&launch_with_roster_parameter),
                        "the launcher must receive the roster path parameter: {line}"
                    );
                }
                if line.contains(&env_clear_call) {
                    env_clear_sites += 1;
                }
                if line.contains(&env_call) && !line.contains(&env_clear_call) {
                    assert!(
                        line.contains(&locale_pin),
                        "only the fixed locale pin may enter the child environment: {line}"
                    );
                }
                assert!(
                    !line.contains(&shell_literal) && !line.contains(&shell_path_literal),
                    "a shell binary appears in {path:?}: {line}"
                );
                assert!(
                    !line.contains(&shell_flag_literal),
                    "a shell -c argument appears in {path:?}: {line}"
                );
                for roster in &roster_literals {
                    if line.contains(roster) {
                        roster_paths += 1;
                        assert_eq!(
                            path,
                            PathBuf::from("linux.rs"),
                            "a roster path escaped the session module"
                        );
                        assert!(
                            line.starts_with("const "),
                            "roster paths may only be compiled constants: {line}"
                        );
                    }
                }
            }
        }
        assert_eq!(
            launch_sites, 1,
            "exactly one bounded launcher may spawn a process"
        );
        assert_eq!(
            env_clear_sites, 1,
            "the one launcher must clear the child environment"
        );
        assert_eq!(
            roster_paths, 3,
            "the roster is exactly the three compiled absolute paths"
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

    // Requirements: SAFE-001, SAFE-007
    //   A session consumes exactly one registered fixture, in either role, and
    //   refuses every other authorized set before loop control is touched.
    // Evidence: session_call_accepts_exactly_one_registered_fixture_role
    #[test]
    fn session_call_accepts_exactly_one_registered_fixture_role() {
        for (name, role) in [
            (BASIC_NAME, FixtureRole::Basic),
            (CONFLICTING_NAME, FixtureRole::Conflicting),
        ] {
            let sandbox = Sandbox::new();
            let authorization = sandbox.authorization(&[name]);
            let (fixture, file) =
                consume_single_target(authorization).expect("single registered fixture");
            assert_eq!(fixture, role);
            drop(file);
        }

        let sandbox = Sandbox::new();
        let two = sandbox.authorization(&[BASIC_NAME, CONFLICTING_NAME]);
        assert!(matches!(
            consume_single_target(two),
            Err(Refusal::WrongSessionTarget)
        ));

        let sandbox = Sandbox::new();
        let unregistered = sandbox.authorization(&["blank-512.img"]);
        assert!(matches!(
            consume_single_target(unregistered),
            Err(Refusal::WrongSessionTarget)
        ));

        #[cfg(not(target_os = "linux"))]
        {
            let sandbox = Sandbox::new();
            let authorization = sandbox.authorization(&[BASIC_NAME]);
            assert_eq!(
                run_probed_session(authorization),
                Err(Refusal::UnsupportedPlatform)
            );
        }
    }

    // Requirements: SAFE-005, SAFE-006
    //   Session refusals render normalized: no path, node name, or kernel identity,
    //   and every probe refusal carries only the stable tool label.
    // Evidence: session_refusal_display_stays_normalized_and_cleanup_accurate
    #[test]
    fn session_refusal_display_stays_normalized_and_cleanup_accurate() {
        let samples = [
            Refusal::WrongSessionTarget,
            Refusal::ProbeToolMissing {
                tool: "udevadm-settle",
            },
            Refusal::ProbeLaunchFailed {
                tool: "blkid-probe",
                errno: Some(2),
            },
            Refusal::ProbeTimedOut {
                tool: "wipefs-noact",
            },
            Refusal::ProbeOutputOverLimit {
                tool: "udevadm-info",
            },
            Refusal::ProbeUnexpectedExit {
                tool: "blkid-probe",
                code: Some(4),
            },
            Refusal::ProbeUnexpectedExit {
                tool: "blkid-probe",
                code: None,
            },
            Refusal::NodePathIdentityMismatch,
            Refusal::PartitionEnumerationFailed { errno: Some(13) },
            Refusal::PartitionCountExceeded,
        ];
        for sample in samples {
            let rendered = sample.to_string();
            assert!(!rendered.contains("/dev/"), "node path leaked: {rendered}");
            assert!(!rendered.contains("/usr/"), "tool path leaked: {rendered}");
            assert!(!rendered.contains("/sys/"), "sysfs path leaked: {rendered}");
            assert!(!rendered.contains("inode"), "identity leaked: {rendered}");
            // Mid-window failures leave cleanup to the unconditional detach
            // that follows them; the detach's own refusal wins when it fails,
            // so these report the confirmed state, not uncertainty.
            assert_eq!(sample.cleanup_state(), CleanupState::NotRequiredOrConfirmed);
            assert_eq!(sample.fixture_state(), FixtureState::NotEstablished);
        }
    }
}
