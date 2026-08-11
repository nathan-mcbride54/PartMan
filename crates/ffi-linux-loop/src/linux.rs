//! Safe Linux controllers over the confined ioctl module.
//!
//! Two controllers share one attachment vocabulary and one cleanup
//! discipline: increment 2e's two-leg acceptance and increment 2f's
//! hold-open session. The session additionally owns the launch of its
//! predeclared external probers; no descriptor, node name, path, or device
//! number leaves this module through a return value.

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

use partman_fixtures::catalogue;
use rustix::fs::{FileType, Mode, OFlags, fstat, lstat, major, makedev, minor, open};
use rustix::io::{Errno, pread, pwrite};
use sha2::{Digest as _, Sha256};

use crate::protocol::{
    CapturedProbe, ConfigureError, ConfigureRequest, Controller, DESTRUCTIVE_FLAGS,
    DestructiveController, FLAG_READ_ONLY, REQUIRED_BLOCK_SIZE, REQUIRED_FLAGS, RebindProbe,
    SessionController, execute, execute_destructive_suite, execute_session,
};
use crate::{
    AdmittedContract, AdmittedFixture, AdmittedRange, AuthorizedFiles, BASIC_NAME,
    CONFLICTING_NAME, DestructiveReport, FixtureRole, ProbeSubject, ProbeTool, Refusal, RunReport,
    SessionDiskFacts, SessionPartitionFacts, SessionReport, sys,
};

const LOOP_CONTROL: &str = "/dev/loop-control";
const LOOP_MAJOR: u32 = 7;
const MISC_MAJOR: u32 = 10;
const LOOP_CONTROL_MINOR: u32 = 237;
const PROBE_BYTES: usize = 4096;
const DETACH_ATTEMPTS: usize = 16;
const DETACH_RETRY_DELAY: Duration = Duration::from_millis(100);
const SYS_DEV_BLOCK: &str = "/sys/dev/block";

/// Compiled absolute prober locations. This roster is the allow-list: launch
/// resolves nothing from `PATH` and accepts no caller-supplied path.
const UDEVADM_PATH: &str = "/usr/bin/udevadm";
const BLKID_PATH: &str = "/usr/sbin/blkid";
const WIPEFS_PATH: &str = "/usr/sbin/wipefs";

/// How long one prober may run before it is killed. `udevadm settle` carries
/// its own shorter internal timeout, so this bound dominates every launch.
const PROBE_TIME_LIMIT: Duration = Duration::from_secs(15);

/// Per-stream capture bound. Exceeding it refuses the session rather than
/// truncating: truncated prober output would be incomplete evidence.
const PROBE_OUTPUT_LIMIT_PER_STREAM: usize = 16 * 1024;

pub(super) fn run(files: AuthorizedFiles) -> Result<RunReport, Refusal> {
    let mut controller = LinuxController::new(files)?;
    execute(&mut controller)
}

pub(super) fn run_session(fixture: FixtureRole, backing: File) -> Result<SessionReport, Refusal> {
    let mut controller = LinuxSessionController::new(fixture, backing)?;
    execute_session(&mut controller, fixture)
}

pub(super) fn run_destructive(contract: AdmittedContract) -> Result<DestructiveReport, Refusal> {
    // The expected bytes are read out before each fixture's contract moves
    // into its controller, so the protocol's post-conditions compare against
    // the registry's compiled data rather than against anything a controller
    // could have recomputed on the way through.
    let mut fixtures = Vec::with_capacity(contract.fixtures.len());
    for fixture in contract.fixtures {
        let replacements: Vec<&[u8]> = fixture
            .ranges
            .iter()
            .map(|range| range.replacement)
            .collect();
        let controller = LinuxDestructiveController::new(fixture)?;
        fixtures.push((controller, replacements));
    }
    execute_destructive_suite(&mut fixtures)
}

/// Open `/dev/loop-control` and require the exact kernel misc-device identity.
fn open_verified_loop_control() -> Result<File, Refusal> {
    let control = open_loop_node(LOOP_CONTROL, "loop-control-open")?;
    let control_stat = fstat(&control).map_err(|error| kernel("loop-control-fstat", error))?;
    if FileType::from_raw_mode(control_stat.st_mode) != FileType::CharacterDevice
        || major(control_stat.st_rdev) != MISC_MAJOR
        || minor(control_stat.st_rdev) != LOOP_CONTROL_MINOR
    {
        return Err(Refusal::LoopControlIdentityMismatch);
    }
    Ok(control)
}

struct LinuxController {
    basic: File,
    conflicting: File,
    control: File,
    expected_basic: [u8; 32],
    expected_conflicting: [u8; 32],
}

impl LinuxController {
    fn new(files: AuthorizedFiles) -> Result<Self, Refusal> {
        let expected_basic = compiled_expected_digest(BASIC_NAME);
        let expected_conflicting = compiled_expected_digest(CONFLICTING_NAME);
        let control = open_verified_loop_control()?;
        Ok(Self {
            basic: files.basic,
            conflicting: files.conflicting,
            control,
            expected_basic,
            expected_conflicting,
        })
    }

    fn backing(&self, fixture: FixtureRole) -> &File {
        match fixture {
            FixtureRole::Basic => &self.basic,
            FixtureRole::Conflicting => &self.conflicting,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BackingIdentity {
    encoded_device: u64,
    inode: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LoopNodeIdentity {
    filesystem_device: u64,
    inode: u64,
    represented_device: u64,
}

struct Attachment {
    device: File,
    number: u32,
    node_identity: LoopNodeIdentity,
    /// The kernel-derived node name, retained for the session's node re-stat
    /// and prober launch only. It never leaves this module: no public type
    /// carries it and no return value exposes it.
    path: String,
}

/// Configure a kernel-selected loop device from the exact held backing
/// descriptor. Shared verbatim by both controllers; the only degrees of
/// freedom are the descriptors already held.
fn configure_attachment(
    control: &File,
    backing: &File,
    request: ConfigureRequest,
) -> Result<Attachment, ConfigureError> {
    let raw_number = sys::control_get_free(control)
        .map_err(|error| ConfigureError::Refused(kernel("loop-control-get-free", error)))?;
    let number = u32::try_from(raw_number).map_err(|_| {
        ConfigureError::Refused(Refusal::KernelOperation {
            operation: "loop-control-get-free",
            errno: None,
        })
    })?;
    let path = format!("/dev/loop{number}");
    let device = open_loop_node(&path, "loop-device-open").map_err(ConfigureError::Refused)?;
    let node_identity = loop_node_identity(&device).map_err(ConfigureError::Refused)?;
    if FileType::from_raw_mode(
        fstat(&device)
            .map_err(|error| ConfigureError::Refused(kernel("loop-device-fstat", error)))?
            .st_mode,
    ) != FileType::BlockDevice
        || !is_loop_device(node_identity.represented_device)
    {
        return Err(ConfigureError::Refused(Refusal::LoopNodeIdentityMismatch));
    }

    match sys::configure(&device, backing, request) {
        Ok(()) => {
            // No fallible work is allowed between successful atomic
            // LOOP_CONFIGURE and returning the owned Attachment. From this
            // point the protocol layer is responsible for confirmed cleanup.
            Ok(Attachment {
                device,
                number,
                node_identity,
                path,
            })
        }
        Err(Errno::BUSY) => Err(ConfigureError::Busy),
        Err(error) => Err(ConfigureError::Refused(kernel("loop-configure", error))),
    }
}

/// Verify the kernel's record of the attachment against the exact held
/// backing descriptor and the retained node identity. Shared verbatim by
/// both controllers.
fn verify_attachment(backing: &File, attachment: &Attachment) -> Result<(), Refusal> {
    verify_attachment_with_flags(backing, attachment, REQUIRED_FLAGS)
}

/// The body of [`verify_attachment`], with the expected flag set named by the
/// caller rather than assumed.
///
/// Increment 2h's attachment is read-write by design, so its flag set differs
/// from 2e's and 2f's by exactly `LO_FLAGS_READ_ONLY`. The expectation is a
/// parameter rather than a relaxed comparison: each protocol still requires
/// its flags to match **exactly**, and a mapping that came back with the
/// other protocol's flags refuses here.
fn verify_attachment_with_flags(
    backing: &File,
    attachment: &Attachment,
    expected_flags: u32,
) -> Result<(), Refusal> {
    let status =
        sys::status(&attachment.device).map_err(|error| kernel("loop-get-status64", error))?;
    let backing = backing_identity(backing)?;
    if status.backing_device != backing.encoded_device || status.backing_inode != backing.inode {
        return Err(Refusal::BackingIdentityMismatch);
    }
    if status.flags != expected_flags {
        return Err(Refusal::LoopFlagsMismatch);
    }
    if status.offset != 0 || status.size_limit != 0 {
        return Err(Refusal::LoopGeometryMismatch);
    }
    if status.number != attachment.number {
        return Err(Refusal::LoopNumberMismatch);
    }
    let block_size =
        sys::block_size(&attachment.device).map_err(|error| kernel("block-size-get", error))?;
    if block_size != i32::try_from(REQUIRED_BLOCK_SIZE).expect("512 fits i32") {
        return Err(Refusal::BlockSizeMismatch);
    }
    if loop_node_identity(&attachment.device)? != attachment.node_identity {
        return Err(Refusal::LoopNodeIdentityMismatch);
    }
    Ok(())
}

/// Detach the exact held attachment, confirm by `ENXIO`, release the
/// descriptor, and confirm partition teardown at the retained-rdev sysfs
/// root. Shared verbatim by both controllers.
fn detach_attachment(attachment: Attachment) -> Result<(), Refusal> {
    let represented_device = attachment.node_identity.represented_device;
    clear_and_confirm_with_retry(
        || sys::clear_fd(&attachment.device),
        || sys::status(&attachment.device).map(drop),
        || std::thread::sleep(DETACH_RETRY_DELAY),
    )?;
    release_then_confirm(attachment.device, || {
        confirm_partition_teardown(represented_device)
    })
}

/// The destructive controller: one held backing object, one fixture's
/// compiled contract. Introduced by increment 2h for the one-range shape;
/// carried per fixture by 2i's general executor.
///
/// It holds no fixture roles and no second object. The offsets, lengths, and
/// replacement bytes are the admitted contract's, carried here so that no
/// method takes them as bytes — a protocol bug can misorder or skip the
/// indexed calls, which the pure protocol's tests cover, but it cannot
/// redirect a write to a range the contract does not declare.
struct LinuxDestructiveController {
    contract: AdmittedFixture,
    control: File,
}

impl LinuxDestructiveController {
    fn new(contract: AdmittedFixture) -> Result<Self, Refusal> {
        let control = open_verified_loop_control()?;
        Ok(Self { contract, control })
    }

    /// The indexed declared range, refusing an index the contract does not
    /// declare rather than panicking on it.
    fn range(&self, index: usize) -> Result<&AdmittedRange, Refusal> {
        self.contract
            .ranges
            .get(index)
            .ok_or(Refusal::WrongSuiteShape)
    }
}

impl DestructiveController for LinuxDestructiveController {
    type Attachment = Attachment;

    fn initial_backing_matches_catalogue(&mut self) -> Result<bool, Refusal> {
        // The digest comes from this crate's compiled catalogue, resolved by
        // the contract's fixture name — not from the manifest on disk, and
        // not from anything the caller passed.
        let expected = compiled_expected_digest(self.contract.fixture);
        Ok(digest_file(&self.contract.backing)? == expected)
    }

    fn bracket_digest(&mut self) -> Result<[u8; 32], Refusal> {
        // Sorted here because the bracket walks the file forward; the
        // executor already refused overlap, and the sort cannot reorder
        // which bytes are outside the union.
        let mut ranges: Vec<(u64, u64)> = self
            .contract
            .ranges
            .iter()
            .map(|range| (range.offset, range.length))
            .collect();
        ranges.sort_unstable();
        digest_outside_ranges(&self.contract.backing, &ranges)
    }

    fn configure(&mut self, request: ConfigureRequest) -> Result<Self::Attachment, ConfigureError> {
        configure_attachment(&self.control, &self.contract.backing, request)
    }

    fn verify(&mut self, attachment: &Self::Attachment) -> Result<(), Refusal> {
        verify_attachment_with_flags(&self.contract.backing, attachment, DESTRUCTIVE_FLAGS)
    }

    fn probe_forbidden_rebind(
        &mut self,
        attachment: &Self::Attachment,
    ) -> Result<RebindProbe, Refusal> {
        // The backing offered is the one already attached. That is deliberate:
        // the kernel checks read-write-ness before it looks at the new
        // descriptor, so this needs no second fixture, and the worst case if
        // the driver ever accepted it is a rebind to the same object — which
        // this protocol still treats as void rather than harmless.
        classify_rebind_probe(
            sys::change_fd(&attachment.device, &self.contract.backing),
            || {
                sys::status(&attachment.device)
                    .map(|status| status.flags)
                    .map_err(|error| kernel("rebind-probe-restat", error))
            },
        )
    }

    fn write_contracted(
        &mut self,
        attachment: &Self::Attachment,
        index: usize,
    ) -> Result<usize, Refusal> {
        // Through the held loop-device descriptor, at the indexed range's
        // offset. The mapping is offset 0 with no size limit, so this
        // addresses the same byte of the backing object the contract names.
        write_contracted_range(&attachment.device, self.range(index)?)
    }

    fn sync(&mut self, attachment: &Self::Attachment) -> Result<(), Refusal> {
        rustix::fs::fdatasync(&attachment.device)
            .map_err(|error| kernel("contracted-write-sync", error))
    }

    fn detach(&mut self, attachment: Self::Attachment) -> Result<(), Refusal> {
        detach_attachment(attachment)
    }

    fn read_declared_range(&mut self, index: usize) -> Result<Vec<u8>, Refusal> {
        let range = self.range(index)?;
        read_exact_range(&self.contract.backing, range.offset, range.length)
    }
}

/// Write the contract's replacement bytes at the contract's offset through
/// the given device descriptor, returning the count the kernel reports.
///
/// Split from `write_contracted` so the destination is measurable at Tier 1
/// (issue #250): this is the only line in the repository that writes to
/// storage under a Tier-2 gate, and the protocol tests drive a fake whose
/// write never involves the offset, so a transposed field or an offset
/// arithmetic change would otherwise pass every unprivileged test and be
/// caught only by a VM sitting's post-conditions — which are checked by the
/// same protocol whose write is in question. Against a regular scratch file
/// the unit test pins which contract field supplies the offset, which
/// supplies the bytes, and that nothing outside them is touched. The loop
/// mapping itself (offset 0, no size limit, so device offsets are backing
/// offsets) remains the acceptance's measurement.
fn write_contracted_range(device: &File, range: &AdmittedRange) -> Result<usize, Refusal> {
    pwrite(device, range.replacement, range.offset)
        .map_err(|error| kernel("contracted-write", error))
}

/// Classify the kernel's answer to the forbidden mid-suite rebind attempt.
///
/// `LOOP_CHANGE_FD` returning `EINVAL` is not, by itself, the refusal the
/// suite requires (issue #248): `EINVAL` is the most overloaded errno on the
/// ioctl path, and reading it as "the attachment is read-write, so the rebind
/// is inapplicable" would name a kernel state nothing observed. A
/// misclassified `EINVAL` fails *open* — the run proceeds into the write —
/// while every other refusal in this protocol fails closed.
///
/// So on `EINVAL` the attachment's status flags are re-read, and only an
/// attachment observed to be read-write may name the answer
/// [`RebindProbe::KernelRefused`]: the returned value then names a state that
/// was observed, not a reason that was assumed. `EINVAL` on an attachment
/// whose flags carry `LO_FLAGS_READ_ONLY` — unreachable through this protocol,
/// whose `verify` requires the destructive flag set immediately before the
/// probe — is an unexplained refusal and refuses the run rather than passing
/// it. The flag check is deliberately only `LO_FLAGS_READ_ONLY`'s absence:
/// read-write-ness is the exact property the classification rests on, and the
/// full flag-set binding is `verify`'s job on either side of the probe.
fn classify_rebind_probe(
    attempt: rustix::io::Result<()>,
    observed_flags: impl FnOnce() -> Result<u32, Refusal>,
) -> Result<RebindProbe, Refusal> {
    match attempt {
        Ok(()) => Ok(RebindProbe::KernelAccepted),
        Err(Errno::INVAL) => {
            if observed_flags()? & FLAG_READ_ONLY != 0 {
                return Err(Refusal::AdversarialRebindFailed {
                    errno: Some(Errno::INVAL.raw_os_error()),
                });
            }
            Ok(RebindProbe::KernelRefused)
        }
        Err(error) => Err(Refusal::AdversarialRebindFailed {
            errno: Some(error.raw_os_error()),
        }),
    }
}

/// Hash every byte of the held object **except** the declared ranges.
///
/// This is the digest bracket: it is what makes "nothing outside the contract
/// changed" a measured fact rather than an assumption about what writes of
/// N bytes can reach. Read through the held descriptor, never a path.
///
/// `ranges` must be sorted by offset and non-overlapping. Both are refused
/// here rather than trusted, even though the executor already refused them:
/// the bracket's meaning — "exactly the bytes outside the union" — depends on
/// them, and a bracket that silently mis-partitioned the file would report
/// success over bytes it never hashed.
fn digest_outside_ranges(file: &File, ranges: &[(u64, u64)]) -> Result<[u8; 32], Refusal> {
    let mut bounds = Vec::with_capacity(ranges.len());
    let mut previous_end = 0_u64;
    for (index, (offset, length)) in ranges.iter().enumerate() {
        let end = offset
            .checked_add(*length)
            .ok_or(Refusal::WrongSuiteShape)?;
        if index > 0 && *offset < previous_end {
            return Err(Refusal::WrongSuiteShape);
        }
        previous_end = end;
        bounds.push((*offset, end));
    }
    let mut hasher = Sha256::new();
    let mut bytes = [0_u8; 8 * 1024];
    let mut position = 0_u64;
    loop {
        let read =
            pread(file, &mut bytes, position).map_err(|error| kernel("bracket-hash", error))?;
        if read == 0 {
            break;
        }
        let chunk_end = position
            .checked_add(u64::try_from(read).expect("read length fits u64"))
            .ok_or(Refusal::KernelOperation {
                operation: "bracket-hash",
                errno: None,
            })?;
        // Walk the chunk left to right: for each declared range, hash the
        // gap between the cursor and the range's start, then jump the cursor
        // past the range; whatever remains after the last range is hashed
        // too. A range wholly before or after the chunk moves nothing.
        let mut cursor = position;
        for (start, end) in &bounds {
            if *end <= cursor {
                continue;
            }
            if *start >= chunk_end {
                break;
            }
            let stop = (*start).max(cursor).min(chunk_end);
            if cursor < stop {
                let from = usize::try_from(cursor - position).expect("chunk offset fits usize");
                let to = usize::try_from(stop - position).expect("chunk offset fits usize");
                hasher.update(&bytes[from..to]);
            }
            cursor = cursor.max((*end).min(chunk_end));
        }
        if cursor < chunk_end {
            let from = usize::try_from(cursor - position).expect("chunk offset fits usize");
            let to = usize::try_from(chunk_end - position).expect("chunk offset fits usize");
            hasher.update(&bytes[from..to]);
        }
        position = chunk_end;
    }
    Ok(hasher.finalize().into())
}

/// Read exactly the declared range through the held backing descriptor.
///
/// A short read is a refusal rather than a shorter answer: an object that
/// cannot supply the contracted range is not the object the contract was
/// admitted against.
fn read_exact_range(file: &File, offset: u64, length: u64) -> Result<Vec<u8>, Refusal> {
    let wanted = usize::try_from(length).map_err(|_| Refusal::WrongSuiteShape)?;
    let mut buffer = vec![0_u8; wanted];
    let mut done = 0_usize;
    while done < wanted {
        let at = offset
            .checked_add(u64::try_from(done).expect("progress fits u64"))
            .ok_or(Refusal::WrongSuiteShape)?;
        let read = pread(file, &mut buffer[done..], at)
            .map_err(|error| kernel("declared-range-read", error))?;
        if read == 0 {
            return Err(Refusal::DeclaredRangeMismatch);
        }
        done += read;
    }
    Ok(buffer)
}

impl Controller for LinuxController {
    type Attachment = Attachment;

    fn expected_digest(&self, fixture: FixtureRole) -> [u8; 32] {
        match fixture {
            FixtureRole::Basic => self.expected_basic,
            FixtureRole::Conflicting => self.expected_conflicting,
        }
    }

    fn digest(&mut self, fixture: FixtureRole) -> Result<[u8; 32], Refusal> {
        digest_file(self.backing(fixture))
    }

    fn configure(
        &mut self,
        fixture: FixtureRole,
        request: ConfigureRequest,
    ) -> Result<Self::Attachment, ConfigureError> {
        if fixture != FixtureRole::Basic {
            return Err(ConfigureError::Refused(Refusal::WrongAuthorizedTargets));
        }
        configure_attachment(&self.control, &self.basic, request)
    }

    fn verify(
        &mut self,
        attachment: &Self::Attachment,
        expected: FixtureRole,
    ) -> Result<(), Refusal> {
        verify_attachment(self.backing(expected), attachment)
    }

    fn probe(&mut self, attachment: &Self::Attachment) -> Result<usize, Refusal> {
        let mut bytes = [0_u8; PROBE_BYTES];
        let mut filled = 0;
        while filled < bytes.len() {
            let offset = u64::try_from(filled).expect("probe buffer length fits u64");
            let read =
                pread(&attachment.device, &mut bytes[filled..], offset).map_err(|error| {
                    Refusal::ProbeFailed {
                        errno: Some(error.raw_os_error()),
                    }
                })?;
            if read == 0 {
                return Err(Refusal::ProbeFailed { errno: None });
            }
            filled += read;
        }
        Ok(filled)
    }

    fn rebind(
        &mut self,
        attachment: &Self::Attachment,
        replacement: FixtureRole,
    ) -> Result<(), Refusal> {
        if replacement != FixtureRole::Conflicting {
            return Err(Refusal::AdversarialRebindFailed { errno: None });
        }
        sys::change_fd(&attachment.device, &self.conflicting).map_err(|error| {
            Refusal::AdversarialRebindFailed {
                errno: Some(error.raw_os_error()),
            }
        })
    }

    fn detach(&mut self, attachment: Self::Attachment) -> Result<(), Refusal> {
        detach_attachment(attachment)
    }
}

/// One materialized partition the session enumerated from the
/// descriptor-derived sysfs root. Internal only; the node path and device
/// number never leave this module.
#[derive(Debug, PartialEq, Eq)]
struct SessionPartition {
    index: u32,
    node_path: String,
    device: rustix::fs::Dev,
}

struct LinuxSessionController {
    fixture: FixtureRole,
    backing: File,
    control: File,
    expected: [u8; 32],
    partitions: Vec<SessionPartition>,
}

impl LinuxSessionController {
    fn new(fixture: FixtureRole, backing: File) -> Result<Self, Refusal> {
        // Refuse a missing prober before any privilege is exercised: the
        // roster is the allow-list, and a session that cannot run its
        // probes must not attach anything.
        for tool in [
            ProbeTool::UdevadmSettle,
            ProbeTool::UdevadmInfo,
            ProbeTool::BlkidProbe,
            ProbeTool::WipefsNoAct,
        ] {
            let path = tool_path(tool);
            if !std::fs::metadata(path).is_ok_and(|metadata| metadata.is_file()) {
                return Err(Refusal::ProbeToolMissing { tool: tool.label() });
            }
        }
        let expected = compiled_expected_digest(match fixture {
            FixtureRole::Basic => BASIC_NAME,
            FixtureRole::Conflicting => CONFLICTING_NAME,
        });
        let control = open_verified_loop_control()?;
        Ok(Self {
            fixture,
            backing,
            control,
            expected,
            partitions: Vec::new(),
        })
    }

    fn partition(&self, index: u32) -> Result<&SessionPartition, Refusal> {
        self.partitions
            .iter()
            .find(|partition| partition.index == index)
            .ok_or(Refusal::ProtocolOrder)
    }
}

impl SessionController for LinuxSessionController {
    type Attachment = Attachment;

    fn expected_digest(&self, _fixture: FixtureRole) -> [u8; 32] {
        self.expected
    }

    fn digest(&mut self, _fixture: FixtureRole) -> Result<[u8; 32], Refusal> {
        digest_file(&self.backing)
    }

    fn configure(
        &mut self,
        fixture: FixtureRole,
        request: ConfigureRequest,
    ) -> Result<Self::Attachment, ConfigureError> {
        if fixture != self.fixture {
            return Err(ConfigureError::Refused(Refusal::WrongSessionTarget));
        }
        configure_attachment(&self.control, &self.backing, request)
    }

    fn verify(
        &mut self,
        attachment: &Self::Attachment,
        _expected: FixtureRole,
    ) -> Result<(), Refusal> {
        verify_attachment(&self.backing, attachment)
    }

    fn verify_node(
        &mut self,
        attachment: &Self::Attachment,
        subject: ProbeSubject,
    ) -> Result<(), Refusal> {
        let (node_path, expected_device) = match subject {
            ProbeSubject::Disk => (
                attachment.path.as_str(),
                attachment.node_identity.represented_device,
            ),
            ProbeSubject::Partition(index) => {
                let partition = self.partition(index)?;
                (partition.node_path.as_str(), partition.device)
            }
        };
        // lstat, not stat: a symlink planted at the node name must be seen as
        // a symlink and refused, never followed to wherever it points.
        let stat = lstat(node_path).map_err(|error| kernel("node-restat", error))?;
        if FileType::from_raw_mode(stat.st_mode) != FileType::BlockDevice
            || stat.st_rdev != expected_device
        {
            return Err(Refusal::NodePathIdentityMismatch);
        }
        Ok(())
    }

    fn enumerate_partitions(&mut self, attachment: &Self::Attachment) -> Result<Vec<u32>, Refusal> {
        self.partitions = enumerate_session_partitions(
            Path::new(SYS_DEV_BLOCK),
            attachment.number,
            attachment.node_identity.represented_device,
        )?;
        Ok(self
            .partitions
            .iter()
            .map(|partition| partition.index)
            .collect())
    }

    fn device_digest(&mut self, attachment: &Self::Attachment) -> Result<[u8; 32], Refusal> {
        // The same positional-read hasher the backing files use, pointed at
        // the held loop descriptor: the device's logical contents, never a
        // path reopen.
        digest_file(&attachment.device)
    }

    fn capture_facts(
        &mut self,
        attachment: &Self::Attachment,
    ) -> Result<(SessionDiskFacts, Vec<SessionPartitionFacts>), Refusal> {
        let base = Path::new(SYS_DEV_BLOCK);
        let device = attachment.node_identity.represented_device;
        let disk_root = base.join(format!("{}:{}", major(device), minor(device)));
        let disk_read_only = read_sysfs_u64(&disk_root.join("ro"))? == 1;
        if !disk_read_only {
            return Err(Refusal::SessionNodeWritable);
        }
        let disk_facts = SessionDiskFacts {
            size_sectors: read_sysfs_u64(&disk_root.join("size"))?,
            read_only: disk_read_only,
            logical_block_size: u32::try_from(read_sysfs_u64(
                &disk_root.join("queue").join("logical_block_size"),
            )?)
            .map_err(|_| Refusal::KernelOperation {
                operation: "sysfs-facts",
                errno: None,
            })?,
        };

        let mut partition_facts = Vec::with_capacity(self.partitions.len());
        let mut session_devices = vec![device];
        for partition in &self.partitions {
            let child = disk_root.join(format!("loop{}p{}", attachment.number, partition.index));
            let read_only = read_sysfs_u64(&child.join("ro"))? == 1;
            if !read_only {
                return Err(Refusal::SessionNodeWritable);
            }
            partition_facts.push(SessionPartitionFacts {
                index: partition.index,
                start_sectors: read_sysfs_u64(&child.join("start"))?,
                size_sectors: read_sysfs_u64(&child.join("size"))?,
                read_only,
            });
            session_devices.push(partition.device);
        }

        let mountinfo = std::fs::read_to_string("/proc/self/mountinfo").map_err(|error| {
            Refusal::KernelOperation {
                operation: "mountinfo-read",
                errno: error.raw_os_error(),
            }
        })?;
        if any_device_mounted(&mountinfo, &session_devices) {
            return Err(Refusal::SessionNodeMounted);
        }

        Ok((disk_facts, partition_facts))
    }

    fn launch(
        &mut self,
        attachment: &Self::Attachment,
        subject: ProbeSubject,
        tool: ProbeTool,
    ) -> Result<CapturedProbe, Refusal> {
        let node_path = match subject {
            ProbeSubject::Disk => attachment.path.clone(),
            ProbeSubject::Partition(index) => self.partition(index)?.node_path.clone(),
        };
        let arguments = tool_arguments(tool, &node_path);
        let capture = launch_probe_bounded(tool, tool_path(tool), &arguments)?;
        if !allowed_exit(tool, capture.exit_code) {
            return Err(Refusal::ProbeUnexpectedExit {
                tool: tool.label(),
                code: capture.exit_code,
            });
        }
        Ok(capture)
    }

    fn detach(&mut self, attachment: Self::Attachment) -> Result<(), Refusal> {
        detach_attachment(attachment)
    }
}

/// The compiled absolute location for one predeclared prober.
fn tool_path(tool: ProbeTool) -> &'static str {
    match tool {
        ProbeTool::UdevadmSettle | ProbeTool::UdevadmInfo => UDEVADM_PATH,
        ProbeTool::BlkidProbe => BLKID_PATH,
        ProbeTool::WipefsNoAct => WIPEFS_PATH,
    }
}

/// The fixed argument vector for one predeclared prober. The node path is the
/// only variable element, and it is always the session's own retained node,
/// never caller input.
fn tool_arguments(tool: ProbeTool, node_path: &str) -> Vec<String> {
    match tool {
        ProbeTool::UdevadmSettle => vec!["settle".to_owned(), "--timeout=10".to_owned()],
        ProbeTool::UdevadmInfo => vec![
            "info".to_owned(),
            "--query=all".to_owned(),
            "--name".to_owned(),
            node_path.to_owned(),
        ],
        ProbeTool::BlkidProbe => vec![
            "-p".to_owned(),
            "-o".to_owned(),
            "udev".to_owned(),
            node_path.to_owned(),
        ],
        ProbeTool::WipefsNoAct => vec!["-n".to_owned(), node_path.to_owned()],
    }
}

/// Whether one prober exit code is inside its allowed set. `blkid -p` exits 2
/// when it detects nothing, which is a correct answer for an empty partition,
/// not a failure; every other tool must exit 0. A signal exit is never allowed.
fn allowed_exit(tool: ProbeTool, code: Option<i32>) -> bool {
    match (tool, code) {
        (ProbeTool::BlkidProbe, Some(0 | 2)) => true,
        (ProbeTool::BlkidProbe, _) => false,
        (_, Some(0)) => true,
        (_, _) => false,
    }
}

/// Launch one predeclared prober with the crate-owned controls: absolute
/// path, structured argv, cleared environment plus a fixed `LC_ALL=C`, null
/// stdin, both pipes drained on threads under the per-stream bound, and a
/// kill at the deadline. Mirrors the SAFE-004-derived launcher WP-035's
/// dependency doctor already applies.
fn launch_probe_bounded(
    tool: ProbeTool,
    path: &str,
    arguments: &[String],
) -> Result<CapturedProbe, Refusal> {
    let label = tool.label();
    let mut command = std::process::Command::new(path);
    command
        .args(arguments)
        .env_clear()
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| Refusal::ProbeLaunchFailed {
            tool: label,
            errno: error.raw_os_error(),
        })?;

    // Drain both pipes on threads so a chatty child can flush and exit; the
    // drains keep reading past the cap and report the overflow instead of
    // stalling the child on a full pipe until the deadline mislabels it
    // timed-out. Results come back over channels rather than joins: a
    // descendant that inherited the pipe keeps it open after the child
    // exits, and this session must not hang on someone else's daemon.
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();
    let (stdout_sender, stdout_receiver) = std::sync::mpsc::channel();
    let (stderr_sender, stderr_receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = stdout_sender.send(drain_probe_stream(stdout_pipe));
    });
    std::thread::spawn(move || {
        let _ = stderr_sender.send(drain_probe_stream(stderr_pipe));
    });

    let deadline = Instant::now() + PROBE_TIME_LIMIT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(Refusal::ProbeTimedOut { tool: label });
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => {
                let _ = child.kill();
                return Err(Refusal::ProbeLaunchFailed {
                    tool: label,
                    errno: error.raw_os_error(),
                });
            }
        }
    };

    let remaining = deadline.saturating_duration_since(Instant::now());
    let Ok((stdout, stdout_overflowed)) = stdout_receiver.recv_timeout(remaining) else {
        return Err(Refusal::ProbeTimedOut { tool: label });
    };
    let remaining = deadline.saturating_duration_since(Instant::now());
    let Ok((stderr, stderr_overflowed)) = stderr_receiver.recv_timeout(remaining) else {
        return Err(Refusal::ProbeTimedOut { tool: label });
    };
    if stdout_overflowed || stderr_overflowed {
        return Err(Refusal::ProbeOutputOverLimit { tool: label });
    }
    Ok(CapturedProbe {
        exit_code: status.code(),
        stdout,
        stderr,
    })
}

/// Read up to the per-stream bound from a pipe, then keep draining and
/// discarding so the writer can finish. Returns the bounded bytes and whether
/// the limit was exceeded.
fn drain_probe_stream(pipe: Option<impl Read>) -> (Vec<u8>, bool) {
    let Some(mut pipe) = pipe else {
        return (Vec::new(), false);
    };
    let mut bounded = Vec::new();
    let mut overflowed = false;
    let mut chunk = [0_u8; 4096];
    loop {
        match pipe.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                if overflowed {
                    continue;
                }
                let remaining = PROBE_OUTPUT_LIMIT_PER_STREAM.saturating_sub(bounded.len());
                if read > remaining {
                    bounded.extend_from_slice(&chunk[..remaining]);
                    overflowed = true;
                } else {
                    bounded.extend_from_slice(&chunk[..read]);
                }
            }
        }
    }
    (bounded, overflowed)
}

/// Enumerate materialized partitions from the exact retained-rdev sysfs root.
///
/// Every child carrying a `partition` attribute must have the exact kernel
/// name `loop{number}p{index}` for the index its attribute records, and its
/// `dev` attribute supplies the device number the node re-stat requires.
/// Anything malformed refuses rather than being skipped: an unexpected child
/// under the session's own disk root means the attached object is not
/// understood, and probing it anyway would produce unattributable evidence.
fn enumerate_session_partitions(
    base: &Path,
    number: u32,
    device: rustix::fs::Dev,
) -> Result<Vec<SessionPartition>, Refusal> {
    let disk = base.join(format!("{}:{}", major(device), minor(device)));
    let entries =
        std::fs::read_dir(&disk).map_err(|error| Refusal::PartitionEnumerationFailed {
            errno: error.raw_os_error(),
        })?;
    let mut partitions = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| Refusal::PartitionEnumerationFailed {
            errno: error.raw_os_error(),
        })?;
        let child = entry.path();
        let has_partition_attribute = partition_attribute_present(&child).map_err(|error| {
            Refusal::PartitionEnumerationFailed {
                errno: error.raw_os_error(),
            }
        })?;
        if !has_partition_attribute {
            continue;
        }
        let index = read_trimmed(&child.join("partition"))?
            .parse::<u32>()
            .map_err(|_| Refusal::PartitionEnumerationFailed { errno: None })?;
        if index == 0 {
            return Err(Refusal::PartitionEnumerationFailed { errno: None });
        }
        let expected_name = format!("loop{number}p{index}");
        if entry.file_name() != std::ffi::OsStr::new(&expected_name) {
            return Err(Refusal::PartitionEnumerationFailed { errno: None });
        }
        let device_text = read_trimmed(&child.join("dev"))?;
        let (major_text, minor_text) = device_text
            .split_once(':')
            .ok_or(Refusal::PartitionEnumerationFailed { errno: None })?;
        let child_major = major_text
            .parse::<u32>()
            .map_err(|_| Refusal::PartitionEnumerationFailed { errno: None })?;
        let child_minor = minor_text
            .parse::<u32>()
            .map_err(|_| Refusal::PartitionEnumerationFailed { errno: None })?;
        partitions.push(SessionPartition {
            index,
            node_path: format!("/dev/{expected_name}"),
            device: makedev(child_major, child_minor),
        });
    }
    partitions.sort_by_key(|partition| partition.index);
    Ok(partitions)
}

/// Read one small sysfs attribute and trim its trailing newline.
fn read_trimmed(path: &Path) -> Result<String, Refusal> {
    std::fs::read_to_string(path)
        .map(|content| content.trim().to_owned())
        .map_err(|error| Refusal::PartitionEnumerationFailed {
            errno: error.raw_os_error(),
        })
}

/// Read one numeric sysfs attribute for the facts capture.
fn read_sysfs_u64(path: &Path) -> Result<u64, Refusal> {
    let content = std::fs::read_to_string(path).map_err(|error| Refusal::KernelOperation {
        operation: "sysfs-facts",
        errno: error.raw_os_error(),
    })?;
    content
        .trim()
        .parse::<u64>()
        .map_err(|_| Refusal::KernelOperation {
            operation: "sysfs-facts",
            errno: None,
        })
}

/// Whether any session device number appears as a mount source in the given
/// `mountinfo` content. Field three of each mountinfo line is `major:minor`;
/// comparing there catches a mount regardless of which name mounted it.
fn any_device_mounted(mountinfo: &str, devices: &[rustix::fs::Dev]) -> bool {
    let rendered: Vec<String> = devices
        .iter()
        .map(|device| format!("{}:{}", major(*device), minor(*device)))
        .collect();
    mountinfo.lines().any(|line| {
        line.split_whitespace()
            .nth(2)
            .is_some_and(|field| rendered.iter().any(|device| device == field))
    })
}

fn compiled_expected_digest(name: &str) -> [u8; 32] {
    let fixture = catalogue::catalogue()
        .into_iter()
        .find(|fixture| fixture.name == name)
        .expect("fixed loop-acceptance fixture remains in the compiled catalogue");
    let image = (fixture.build)();
    Sha256::digest(image.bytes()).into()
}

fn clear_and_confirm_with_retry(
    mut clear: impl FnMut() -> rustix::io::Result<()>,
    mut status: impl FnMut() -> rustix::io::Result<()>,
    mut pause: impl FnMut(),
) -> Result<(), Refusal> {
    for attempt in 0..DETACH_ATTEMPTS {
        clear().map_err(|error| Refusal::DetachFailed {
            errno: Some(error.raw_os_error()),
        })?;
        match status() {
            Err(Errno::NXIO) => return Ok(()),
            Ok(()) if attempt + 1 < DETACH_ATTEMPTS => pause(),
            Ok(()) => return Err(Refusal::DetachNotConfirmed),
            Err(error) => {
                return Err(Refusal::DetachConfirmationFailed {
                    errno: Some(error.raw_os_error()),
                });
            }
        }
    }
    Err(Refusal::DetachNotConfirmed)
}

fn release_then_confirm<T>(
    held: T,
    confirm: impl FnOnce() -> Result<(), Refusal>,
) -> Result<(), Refusal> {
    drop(held);
    confirm()
}

fn confirm_partition_teardown(device: rustix::fs::Dev) -> Result<(), Refusal> {
    let base = Path::new(SYS_DEV_BLOCK);
    retry_partition_teardown(
        || partition_materialization_present(base, device),
        || std::thread::sleep(DETACH_RETRY_DELAY),
    )
}

fn retry_partition_teardown(
    mut inspect: impl FnMut() -> io::Result<bool>,
    mut pause: impl FnMut(),
) -> Result<(), Refusal> {
    for attempt in 0..DETACH_ATTEMPTS {
        match inspect() {
            Ok(false) => return Ok(()),
            Ok(true) if attempt + 1 < DETACH_ATTEMPTS => pause(),
            Ok(true) => {
                return Err(Refusal::PartitionTeardownNotConfirmed { errno: None });
            }
            Err(error) => {
                return Err(Refusal::PartitionTeardownNotConfirmed {
                    errno: error.raw_os_error(),
                });
            }
        }
    }
    Err(Refusal::PartitionTeardownNotConfirmed { errno: None })
}

fn partition_materialization_present(base: &Path, device: rustix::fs::Dev) -> io::Result<bool> {
    // ENXIO from LOOP_GET_STATUS64 does not universally prove that partition
    // child kobjects have disappeared: kernel history includes a detach/open
    // race fixed by upstream 18048c1af783. Inspect the retained main-node rdev
    // behaviorally instead of version-gating a floor kernel or trusting ENXIO.
    let disk = base.join(format!("{}:{}", major(device), minor(device)));
    let entries = std::fs::read_dir(&disk)?;
    if partition_attribute_present(&disk)? {
        return Ok(true);
    }
    for entry in entries {
        let entry = entry?;
        if partition_attribute_present(&entry.path())? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn partition_attribute_present(entry: &Path) -> io::Result<bool> {
    match std::fs::metadata(entry.join("partition")) {
        Ok(_) => Ok(true),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
            ) =>
        {
            Ok(false)
        }
        Err(error) => Err(error),
    }
}

fn open_loop_node(path: &str, operation: &'static str) -> Result<File, Refusal> {
    open(
        path,
        OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map(File::from)
    .map_err(|error| kernel(operation, error))
}

fn backing_identity(file: &File) -> Result<BackingIdentity, Refusal> {
    let stat = fstat(file).map_err(|error| kernel("backing-fstat", error))?;
    Ok(BackingIdentity {
        encoded_device: huge_encode_dev(stat.st_dev),
        inode: stat.st_ino,
    })
}

fn loop_node_identity(file: &File) -> Result<LoopNodeIdentity, Refusal> {
    let stat = fstat(file).map_err(|error| kernel("loop-device-fstat", error))?;
    Ok(LoopNodeIdentity {
        filesystem_device: stat.st_dev,
        inode: stat.st_ino,
        represented_device: stat.st_rdev,
    })
}

fn is_loop_device(device: rustix::fs::Dev) -> bool {
    // The loop index is not generally the device minor: kernels configured
    // with `max_part` encode it as `index << part_shift`. Pre-configuration we
    // require the loop block major and retain the complete rdev. The first
    // status verification then binds `lo_number` to the requested index while
    // the protocol owns an Attachment and can detach on disagreement.
    major(device) == LOOP_MAJOR
}

fn huge_encode_dev(device: rustix::fs::Dev) -> u64 {
    huge_encode_components(major(device), minor(device))
}

fn huge_encode_components(major: u32, minor: u32) -> u64 {
    u64::from(minor & 0xff) | (u64::from(major) << 8) | (u64::from(minor & !0xff) << 12)
}

fn digest_file(file: &File) -> Result<[u8; 32], Refusal> {
    let mut hasher = Sha256::new();
    let mut bytes = [0_u8; 8 * 1024];
    let mut offset = 0_u64;
    loop {
        let read =
            pread(file, &mut bytes, offset).map_err(|error| kernel("fixture-hash", error))?;
        if read == 0 {
            break;
        }
        hasher.update(&bytes[..read]);
        offset = offset
            .checked_add(u64::try_from(read).expect("read length fits u64"))
            .ok_or(Refusal::KernelOperation {
                operation: "fixture-hash",
                errno: None,
            })?;
    }
    Ok(hasher.finalize().into())
}

fn kernel(operation: &'static str, error: Errno) -> Refusal {
    Refusal::KernelOperation {
        operation,
        errno: Some(error.raw_os_error()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::collections::VecDeque;
    use std::io::Write as _;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    // Requirements: SAFE-005, SAFE-007, SAFE-009
    //   Kernel loop status device numbers are compared using huge_encode_dev.
    // Evidence: huge_encode_dev_matches_kernel_split_minor_layout
    #[cfg(target_os = "linux")]
    #[test]
    fn huge_encode_dev_matches_kernel_split_minor_layout() {
        assert_eq!(huge_encode_components(8, 1), 0x801);
        assert_eq!(huge_encode_components(0xabc, 0x12_345), 0x123a_bc45);
        assert_eq!(huge_encode_components(0, 0), 0);
        assert_eq!(huge_encode_components(0xfff, 0xf_ffff), 0xffff_ffff);
    }

    // Requirements: SAFE-005, SAFE-007
    //   Loop-node binding does not confuse a kernel loop index with a shifted device minor.
    // Evidence: loop_index_is_not_assumed_to_equal_the_device_minor
    #[cfg(target_os = "linux")]
    #[test]
    fn loop_index_is_not_assumed_to_equal_the_device_minor() {
        let index = 12;
        let partition_shift = 4;
        let represented = rustix::fs::makedev(LOOP_MAJOR, index << partition_shift);
        assert_ne!(minor(represented), index);
        assert!(is_loop_device(represented));
    }

    // Requirements: SAFE-005, SAFE-007
    //   Detach is confirmed only by ENXIO and retries a still-bound status within a fixed bound.
    // Evidence: detach_retries_clear_after_a_still_bound_status_then_confirms_enxio
    #[cfg(target_os = "linux")]
    #[test]
    fn detach_retries_clear_after_a_still_bound_status_then_confirms_enxio() {
        let mut clear_calls = 0;
        let mut status_calls = 0;
        let mut pause_calls = 0;
        let mut statuses = VecDeque::from([Ok(()), Err(Errno::NXIO)]);
        let result = clear_and_confirm_with_retry(
            || {
                clear_calls += 1;
                Ok(())
            },
            || {
                status_calls += 1;
                statuses.pop_front().expect("status script is complete")
            },
            || pause_calls += 1,
        );
        assert_eq!(result, Ok(()));
        assert_eq!(clear_calls, 2);
        assert_eq!(status_calls, 2);
        assert_eq!(pause_calls, 1);
        assert!(statuses.is_empty());
    }

    // Requirements: SAFE-005, SAFE-007
    //   A persistently bound loop cannot turn a bounded cleanup attempt into success.
    // Evidence: detach_retry_bound_exhaustion_refuses_after_every_clear_and_status_sample
    #[cfg(target_os = "linux")]
    #[test]
    fn detach_retry_bound_exhaustion_refuses_after_every_clear_and_status_sample() {
        let mut clear_calls = 0;
        let mut status_calls = 0;
        let mut pause_calls = 0;
        let result = clear_and_confirm_with_retry(
            || {
                clear_calls += 1;
                Ok(())
            },
            || {
                status_calls += 1;
                Ok(())
            },
            || pause_calls += 1,
        );
        assert_eq!(result, Err(Refusal::DetachNotConfirmed));
        assert_eq!(clear_calls, DETACH_ATTEMPTS);
        assert_eq!(status_calls, DETACH_ATTEMPTS);
        assert_eq!(pause_calls, DETACH_ATTEMPTS - 1);
    }

    // Requirements: SAFE-005, SAFE-007
    //   Clear and non-ENXIO status errors refuse immediately rather than being retried away.
    // Evidence: detach_clear_and_status_errors_refuse_without_further_retries
    #[cfg(target_os = "linux")]
    #[test]
    fn detach_clear_and_status_errors_refuse_without_further_retries() {
        let mut status_calls = 0;
        let mut pause_calls = 0;
        let clear_error = clear_and_confirm_with_retry(
            || Err(Errno::PERM),
            || {
                status_calls += 1;
                Ok(())
            },
            || pause_calls += 1,
        );
        assert_eq!(
            clear_error,
            Err(Refusal::DetachFailed {
                errno: Some(Errno::PERM.raw_os_error()),
            })
        );
        assert_eq!(status_calls, 0);
        assert_eq!(pause_calls, 0);

        let mut clear_calls = 0;
        let mut status_calls = 0;
        let mut pause_calls = 0;
        let status_error = clear_and_confirm_with_retry(
            || {
                clear_calls += 1;
                Ok(())
            },
            || {
                status_calls += 1;
                Err(Errno::IO)
            },
            || pause_calls += 1,
        );
        assert_eq!(
            status_error,
            Err(Refusal::DetachConfirmationFailed {
                errno: Some(Errno::IO.raw_os_error()),
            })
        );
        assert_eq!(clear_calls, 1);
        assert_eq!(status_calls, 1);
        assert_eq!(pause_calls, 0);
    }

    // Requirements: SAFE-005, SAFE-007
    //   CLR_FD ENXIO always signals interference and cannot satisfy status confirmation.
    // Evidence: clear_enxio_refuses_on_the_first_and_every_later_attempt
    #[cfg(target_os = "linux")]
    #[test]
    fn clear_enxio_refuses_on_the_first_and_every_later_attempt() {
        let mut status_calls = 0;
        let first = clear_and_confirm_with_retry(
            || Err(Errno::NXIO),
            || {
                status_calls += 1;
                Err(Errno::NXIO)
            },
            || panic!("clear failure must not pause"),
        );
        assert_eq!(
            first,
            Err(Refusal::DetachFailed {
                errno: Some(Errno::NXIO.raw_os_error()),
            })
        );
        assert_eq!(status_calls, 0);

        let mut clears = VecDeque::from([Ok(()), Err(Errno::NXIO)]);
        let mut statuses = VecDeque::from([Ok(())]);
        let mut pauses = 0;
        let later = clear_and_confirm_with_retry(
            || clears.pop_front().expect("clear script is complete"),
            || statuses.pop_front().expect("status script is complete"),
            || pauses += 1,
        );
        assert_eq!(
            later,
            Err(Refusal::DetachFailed {
                errno: Some(Errno::NXIO.raw_os_error()),
            })
        );
        assert!(clears.is_empty());
        assert!(statuses.is_empty(), "no second status sample was taken");
        assert_eq!(pauses, 1);
    }

    struct PartitionSandbox {
        root: PathBuf,
        base: PathBuf,
        device: rustix::fs::Dev,
    }

    impl PartitionSandbox {
        fn new(create_disk: bool) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "partman-loop-sysfs-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let base = root.join("sys").join("dev").join("block");
            std::fs::create_dir_all(&base).expect("create fake sysfs base");
            let device = rustix::fs::makedev(LOOP_MAJOR, 23);
            if create_disk {
                std::fs::create_dir(base.join("7:23")).expect("create exact fake disk root");
            }
            Self { root, base, device }
        }

        fn disk(&self) -> PathBuf {
            self.base.join("7:23")
        }
    }

    impl Drop for PartitionSandbox {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    // Requirements: SAFE-005, SAFE-007
    //   Only a readable exact sysfs disk root with no partition attributes proves teardown.
    // Evidence: sysfs_partition_scan_accepts_only_a_readable_empty_exact_disk_root
    #[cfg(target_os = "linux")]
    #[test]
    fn sysfs_partition_scan_accepts_only_a_readable_empty_exact_disk_root() {
        let empty = PartitionSandbox::new(true);
        assert!(
            !partition_materialization_present(&empty.base, empty.device)
                .expect("read empty exact disk root")
        );

        for ordinary in ["queue", "holders", "loop"] {
            std::fs::create_dir(empty.disk().join(ordinary)).expect("create ordinary sysfs entry");
        }
        std::fs::write(empty.disk().join("dev"), b"7:23\n")
            .expect("create ordinary sysfs attribute");
        assert!(
            !partition_materialization_present(&empty.base, empty.device)
                .expect("ordinary sysfs entries are not partitions")
        );

        let partition_node = PartitionSandbox::new(true);
        std::fs::write(partition_node.disk().join("partition"), b"1\n")
            .expect("mark retained rdev as a partition node");
        assert!(
            partition_materialization_present(&partition_node.base, partition_node.device)
                .expect("read exact partition-node root")
        );

        let missing = PartitionSandbox::new(false);
        let missing_error = retry_partition_teardown(
            || partition_materialization_present(&missing.base, missing.device),
            || panic!("an ambiguous scan must not retry"),
        );
        assert_eq!(
            missing_error,
            Err(Refusal::PartitionTeardownNotConfirmed {
                errno: Some(Errno::NOENT.raw_os_error()),
            })
        );
    }

    // Requirements: SAFE-005, SAFE-007
    //   A positively present partition child is retried only to the fixed bound, then refuses.
    // Evidence: surviving_partition_child_exhausts_the_bounded_teardown_check
    #[cfg(target_os = "linux")]
    #[test]
    fn surviving_partition_child_exhausts_the_bounded_teardown_check() {
        let sandbox = PartitionSandbox::new(true);
        let child = sandbox.disk().join("loop23p1");
        std::fs::create_dir(&child).expect("create fake partition child");
        std::fs::write(child.join("partition"), b"1\n").expect("mark fake partition child");
        let mut inspections = 0;
        let mut pauses = 0;
        let result = retry_partition_teardown(
            || {
                inspections += 1;
                partition_materialization_present(&sandbox.base, sandbox.device)
            },
            || pauses += 1,
        );
        assert_eq!(
            result,
            Err(Refusal::PartitionTeardownNotConfirmed { errno: None })
        );
        assert_eq!(inspections, DETACH_ATTEMPTS);
        assert_eq!(pauses, DETACH_ATTEMPTS - 1);
    }

    // Requirements: SAFE-005, SAFE-007
    //   Unreadable or ambiguous sysfs state refuses immediately and is cleanup-uncertain.
    // Evidence: ambiguous_partition_teardown_state_refuses_without_retry
    #[cfg(target_os = "linux")]
    #[test]
    fn ambiguous_partition_teardown_state_refuses_without_retry() {
        let mut pauses = 0;
        let injected = retry_partition_teardown(
            || Err(io::Error::from_raw_os_error(Errno::IO.raw_os_error())),
            || pauses += 1,
        );
        assert_eq!(
            injected,
            Err(Refusal::PartitionTeardownNotConfirmed {
                errno: Some(Errno::IO.raw_os_error()),
            })
        );
        assert_eq!(pauses, 0);

        let sandbox = PartitionSandbox::new(true);
        let child = sandbox.disk().join("loop23p1");
        std::fs::create_dir(&child).expect("create fake child");
        std::os::unix::fs::symlink("partition", child.join("partition"))
            .expect("create ambiguous attribute loop");
        assert!(partition_materialization_present(&sandbox.base, sandbox.device).is_err());

        let not_a_directory = sandbox.root.join("not-a-directory");
        std::fs::write(&not_a_directory, b"not sysfs").expect("create non-directory base");
        assert!(partition_materialization_present(&not_a_directory, sandbox.device).is_err());
    }

    // Requirements: SAFE-005, SAFE-007
    //   The held loop descriptor is dropped before post-release sysfs inspection begins.
    // Evidence: release_seam_drops_the_descriptor_before_partition_confirmation
    #[cfg(target_os = "linux")]
    #[test]
    fn release_seam_drops_the_descriptor_before_partition_confirmation() {
        struct DropWitness<'a>(&'a Cell<bool>);
        impl Drop for DropWitness<'_> {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        let dropped = Cell::new(false);
        let result = release_then_confirm(DropWitness(&dropped), || {
            assert!(dropped.get(), "confirmation ran before descriptor release");
            Ok(())
        });
        assert_eq!(result, Ok(()));
    }

    // Requirements: SAFE-001, SAFE-005, SAFE-007
    //   Initial descriptor hashes are bound to the two exact compiled catalogue fixtures.
    // Evidence: compiled_expected_hashes_are_pinned_to_the_registered_fixture_roles
    #[cfg(target_os = "linux")]
    #[test]
    fn compiled_expected_hashes_are_pinned_to_the_registered_fixture_roles() {
        assert_eq!(
            partman_fixtures::manifest::hex(&compiled_expected_digest(BASIC_NAME)),
            "6d398dd2e69834dec2432cac8192a5533b71cfab169f73895391a1cc83322ec9"
        );
        assert_eq!(
            partman_fixtures::manifest::hex(&compiled_expected_digest(CONFLICTING_NAME)),
            "065d6461eba8ea66d10742277b5b4deda12a0e5e71e45dc56dd6c60ef0da05cc"
        );
    }

    // Requirements: SAFE-004
    //   Each predeclared prober has a fixed argument shape whose only variable
    //   element is the session's own retained node path, and settle takes none.
    // Evidence: prober_argument_vectors_are_fixed_with_only_the_session_node_variable
    #[cfg(target_os = "linux")]
    #[test]
    fn prober_argument_vectors_are_fixed_with_only_the_session_node_variable() {
        assert_eq!(
            tool_arguments(crate::ProbeTool::UdevadmSettle, "/dev/loop9"),
            ["settle", "--timeout=10"]
        );
        assert_eq!(
            tool_arguments(crate::ProbeTool::UdevadmInfo, "/dev/loop9"),
            ["info", "--query=all", "--name", "/dev/loop9"]
        );
        assert_eq!(
            tool_arguments(crate::ProbeTool::BlkidProbe, "/dev/loop9p1"),
            ["-p", "-o", "udev", "/dev/loop9p1"]
        );
        assert_eq!(
            tool_arguments(crate::ProbeTool::WipefsNoAct, "/dev/loop9"),
            ["-n", "/dev/loop9"]
        );
        assert_eq!(
            tool_path(crate::ProbeTool::UdevadmSettle),
            tool_path(crate::ProbeTool::UdevadmInfo),
            "both udevadm forms launch the same compiled binary"
        );
    }

    // Requirements: SAFE-004, SAFE-005
    //   blkid may exit 0 or 2 (nothing detected is an answer); every other tool
    //   must exit 0, and a signal exit is never allowed.
    // Evidence: prober_exit_sets_accept_blkid_nothing_found_and_refuse_signals
    #[cfg(target_os = "linux")]
    #[test]
    fn prober_exit_sets_accept_blkid_nothing_found_and_refuse_signals() {
        assert!(allowed_exit(crate::ProbeTool::BlkidProbe, Some(0)));
        assert!(allowed_exit(crate::ProbeTool::BlkidProbe, Some(2)));
        assert!(!allowed_exit(crate::ProbeTool::BlkidProbe, Some(4)));
        assert!(!allowed_exit(crate::ProbeTool::BlkidProbe, None));
        for tool in [
            crate::ProbeTool::UdevadmSettle,
            crate::ProbeTool::UdevadmInfo,
            crate::ProbeTool::WipefsNoAct,
        ] {
            assert!(allowed_exit(tool, Some(0)));
            assert!(!allowed_exit(tool, Some(2)));
            assert!(!allowed_exit(tool, Some(1)));
            assert!(!allowed_exit(tool, None));
        }
    }

    // Requirements: SAFE-005, SAFE-007
    //   Partition enumeration accepts only exactly-named children whose partition
    //   and dev attributes parse, and refuses malformed sysfs state rather than
    //   skipping it.
    // Evidence: partition_enumeration_is_exact_or_refuses
    #[cfg(target_os = "linux")]
    #[test]
    fn partition_enumeration_is_exact_or_refuses() {
        // One well-formed partition child enumerates with its dev number.
        let sandbox = PartitionSandbox::new(true);
        let child = sandbox.disk().join("loop23p1");
        std::fs::create_dir(&child).expect("create partition child");
        std::fs::write(child.join("partition"), b"1\n").expect("write partition index");
        std::fs::write(child.join("dev"), b"259:5\n").expect("write partition devnum");
        let partitions = enumerate_session_partitions(&sandbox.base, 23, sandbox.device)
            .expect("well-formed child enumerates");
        assert_eq!(partitions.len(), 1);
        assert_eq!(partitions[0].index, 1);
        assert_eq!(partitions[0].node_path, "/dev/loop23p1");
        assert_eq!(partitions[0].device, makedev(259, 5));

        // A child whose name disagrees with its partition attribute refuses.
        let sandbox = PartitionSandbox::new(true);
        let child = sandbox.disk().join("loop23p1");
        std::fs::create_dir(&child).expect("create partition child");
        std::fs::write(child.join("partition"), b"2\n").expect("write disagreeing index");
        std::fs::write(child.join("dev"), b"259:5\n").expect("write partition devnum");
        assert_eq!(
            enumerate_session_partitions(&sandbox.base, 23, sandbox.device),
            Err(Refusal::PartitionEnumerationFailed { errno: None })
        );

        // Index zero is not a partition index.
        let sandbox = PartitionSandbox::new(true);
        let child = sandbox.disk().join("loop23p0");
        std::fs::create_dir(&child).expect("create partition child");
        std::fs::write(child.join("partition"), b"0\n").expect("write zero index");
        assert_eq!(
            enumerate_session_partitions(&sandbox.base, 23, sandbox.device),
            Err(Refusal::PartitionEnumerationFailed { errno: None })
        );

        // A malformed dev attribute refuses rather than being guessed at.
        let sandbox = PartitionSandbox::new(true);
        let child = sandbox.disk().join("loop23p1");
        std::fs::create_dir(&child).expect("create partition child");
        std::fs::write(child.join("partition"), b"1\n").expect("write partition index");
        std::fs::write(child.join("dev"), b"not-a-devnum\n").expect("write malformed devnum");
        assert_eq!(
            enumerate_session_partitions(&sandbox.base, 23, sandbox.device),
            Err(Refusal::PartitionEnumerationFailed { errno: None })
        );

        // A missing dev attribute carries the OS error number.
        let sandbox = PartitionSandbox::new(true);
        let child = sandbox.disk().join("loop23p1");
        std::fs::create_dir(&child).expect("create partition child");
        std::fs::write(child.join("partition"), b"1\n").expect("write partition index");
        assert_eq!(
            enumerate_session_partitions(&sandbox.base, 23, sandbox.device),
            Err(Refusal::PartitionEnumerationFailed {
                errno: Some(Errno::NOENT.raw_os_error()),
            })
        );

        // Ordinary sysfs children without a partition attribute are not
        // partitions, and an empty disk enumerates empty.
        let sandbox = PartitionSandbox::new(true);
        for ordinary in ["queue", "holders", "loop"] {
            std::fs::create_dir(sandbox.disk().join(ordinary)).expect("create ordinary entry");
        }
        std::fs::write(sandbox.disk().join("dev"), b"7:23\n").expect("write disk devnum");
        let partitions = enumerate_session_partitions(&sandbox.base, 23, sandbox.device)
            .expect("ordinary children are not partitions");
        assert!(partitions.is_empty());

        // A missing disk root refuses with its OS error number.
        let missing = PartitionSandbox::new(false);
        assert_eq!(
            enumerate_session_partitions(&missing.base, 23, missing.device),
            Err(Refusal::PartitionEnumerationFailed {
                errno: Some(Errno::NOENT.raw_os_error()),
            })
        );
    }

    // Requirements: SAFE-005, SAFE-007
    //   The mount check matches mountinfo's device field exactly, and a mounted
    //   session device is found regardless of the name that mounted it.
    // Evidence: mountinfo_matching_finds_session_devices_by_number_only
    #[cfg(target_os = "linux")]
    #[test]
    fn mountinfo_matching_finds_session_devices_by_number_only() {
        let mountinfo = "\
36 25 7:23 / /mnt/evil rw,relatime shared:1 - ext4 /dev/renamed rw\n\
37 25 259:5 / /mnt/part rw - vfat /dev/whatever rw\n\
38 25 8:1 / / rw - ext4 /dev/sda1 rw\n";
        assert!(any_device_mounted(mountinfo, &[rustix::fs::makedev(7, 23)]));
        assert!(any_device_mounted(
            mountinfo,
            &[rustix::fs::makedev(259, 5)]
        ));
        assert!(!any_device_mounted(
            mountinfo,
            &[rustix::fs::makedev(7, 0), rustix::fs::makedev(259, 1)]
        ));
        // A device number appearing in another column is not a mount.
        assert!(!any_device_mounted(
            "39 25 8:2 / /somewhere rw - ext4 7:23 rw\n",
            &[rustix::fs::makedev(7, 23)]
        ));
    }

    // Requirements: SAFE-005, SAFE-007
    //   Numeric sysfs facts parse exactly or refuse; nothing is guessed.
    // Evidence: sysfs_fact_reads_parse_exactly_or_refuse
    #[cfg(target_os = "linux")]
    #[test]
    fn sysfs_fact_reads_parse_exactly_or_refuse() {
        let sandbox = PartitionSandbox::new(true);
        std::fs::write(sandbox.disk().join("size"), b"40\n").expect("write size");
        assert_eq!(read_sysfs_u64(&sandbox.disk().join("size")), Ok(40));

        std::fs::write(sandbox.disk().join("ro"), b"not-a-number\n").expect("write junk");
        assert_eq!(
            read_sysfs_u64(&sandbox.disk().join("ro")),
            Err(Refusal::KernelOperation {
                operation: "sysfs-facts",
                errno: None,
            })
        );

        assert_eq!(
            read_sysfs_u64(&sandbox.disk().join("absent")),
            Err(Refusal::KernelOperation {
                operation: "sysfs-facts",
                errno: Some(Errno::NOENT.raw_os_error()),
            })
        );
    }

    // Requirements: SAFE-004, SAFE-005
    //   The bounded stream drain returns complete bytes under the limit and
    //   reports overflow instead of truncating silently.
    // Evidence: probe_stream_drain_bounds_and_reports_overflow
    #[cfg(target_os = "linux")]
    #[test]
    fn probe_stream_drain_bounds_and_reports_overflow() {
        let (bytes, overflowed) = drain_probe_stream(Some(&b"bounded output"[..]));
        assert_eq!(bytes, b"bounded output");
        assert!(!overflowed);

        let oversized = vec![7_u8; PROBE_OUTPUT_LIMIT_PER_STREAM + 1];
        let (bytes, overflowed) = drain_probe_stream(Some(&oversized[..]));
        assert_eq!(bytes.len(), PROBE_OUTPUT_LIMIT_PER_STREAM);
        assert!(overflowed);

        let (bytes, overflowed) = drain_probe_stream(None::<&[u8]>);
        assert!(bytes.is_empty());
        assert!(!overflowed);
    }

    // Requirements: SAFE-001, SAFE-005, SAFE-007
    //   Fixture hashing reads the held descriptor and matches a known SHA-256 vector.
    // Evidence: descriptor_hash_matches_the_sha256_abc_vector
    #[cfg(target_os = "linux")]
    #[test]
    fn descriptor_hash_matches_the_sha256_abc_vector() {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "partman-loop-digest-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("create disposable regular file");
        file.write_all(b"abc").expect("write digest vector");
        file.sync_all().expect("make vector visible through pread");
        let digest = digest_file(&file).expect("hash held descriptor");
        assert_eq!(
            digest,
            [
                0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
                0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
                0xf2, 0x00, 0x15, 0xad,
            ]
        );
        drop(file);
        std::fs::remove_file(path).expect("remove disposable digest vector");
    }

    /// A scratch regular file holding `bytes`, opened read-write.
    fn scratch_file(tag: &str, bytes: &[u8]) -> (std::path::PathBuf, File) {
        use std::io::Write as _;
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "partman-2h-{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let mut file = std::fs::File::create(&path).expect("create scratch file");
        file.write_all(bytes).expect("write scratch bytes");
        drop(file);
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .expect("reopen scratch file");
        (path, file)
    }

    /// SHA-256 of `bytes`, for comparing against the bracket's answer.
    fn sha256(bytes: &[u8]) -> [u8; 32] {
        Sha256::digest(bytes).into()
    }

    // Requirements: SAFE-005, SAFE-007
    //   The digest bracket hashes exactly the bytes outside the declared range, for a range at the start, in the middle, spanning an internal chunk boundary, and at the end, and it changes when a byte outside the range changes while ignoring every byte inside it
    // Work-Package: WP-020
    // Evidence: the_digest_bracket_covers_exactly_the_bytes_outside_the_declared_range
    #[cfg(target_os = "linux")]
    #[test]
    fn the_digest_bracket_covers_exactly_the_bytes_outside_the_declared_range() {
        // Larger than the 8 KiB read chunk, so a range can straddle one.
        let original: Vec<u8> = (0..20_000_u32).map(|index| (index % 251) as u8).collect();
        let chunk = 8 * 1024_u64;

        for (tag, offset, length) in [
            ("start", 0_u64, 8_u64),
            ("middle", 512, 8),
            ("chunk-boundary", chunk - 4, 8),
            ("spans-a-whole-chunk", 100, chunk + 32),
            ("at-eof", 20_000 - 8, 8),
        ] {
            let (path, file) = scratch_file(tag, &original);

            // The independent answer: the bytes outside the range, concatenated.
            let mut outside = Vec::new();
            let start = usize::try_from(offset).expect("fits");
            let end = usize::try_from(offset + length).expect("fits");
            outside.extend_from_slice(&original[..start]);
            outside.extend_from_slice(&original[end..]);
            assert_eq!(
                digest_outside_ranges(&file, &[(offset, length)]).expect("bracket must compute"),
                sha256(&outside),
                "{tag}: the bracket must equal the hash of exactly the outside bytes"
            );

            // Changing a byte INSIDE the range must not move the bracket.
            let mut inside_changed = original.clone();
            inside_changed[start] ^= 0xFF;
            let (inside_path, inside_file) = scratch_file(tag, &inside_changed);
            assert_eq!(
                digest_outside_ranges(&inside_file, &[(offset, length)])
                    .expect("bracket must compute"),
                sha256(&outside),
                "{tag}: a change inside the declared range must not move the bracket"
            );

            // Changing a byte OUTSIDE it must move the bracket. Pick an index
            // that is genuinely outside for every case above.
            let victim = if start == 0 { end } else { start - 1 };
            let mut outside_changed = original.clone();
            outside_changed[victim] ^= 0xFF;
            let (outside_path, outside_file) = scratch_file(tag, &outside_changed);
            assert_ne!(
                digest_outside_ranges(&outside_file, &[(offset, length)])
                    .expect("bracket must compute"),
                sha256(&outside),
                "{tag}: a change outside the declared range must move the bracket"
            );

            drop((file, inside_file, outside_file));
            for path in [path, inside_path, outside_path] {
                std::fs::remove_file(path).expect("remove scratch file");
            }
        }
    }

    // Requirements: SAFE-005, SAFE-007
    //   The declared-range read returns exactly the contracted bytes through the held descriptor, and a file too short to supply the range refuses rather than returning a shorter answer
    // Work-Package: WP-020
    // Evidence: the_declared_range_read_returns_exactly_the_range_or_refuses
    #[cfg(target_os = "linux")]
    #[test]
    fn the_declared_range_read_returns_exactly_the_range_or_refuses() {
        let original: Vec<u8> = (0..1024_u32).map(|index| (index % 97) as u8).collect();
        let (path, file) = scratch_file("range", &original);

        assert_eq!(
            read_exact_range(&file, 512, 8).expect("the range must read"),
            original[512..520].to_vec()
        );
        assert_eq!(
            read_exact_range(&file, 0, 4).expect("a range at offset zero must read"),
            original[..4].to_vec()
        );
        assert_eq!(
            read_exact_range(&file, 1016, 8).expect("a range ending at EOF must read"),
            original[1016..].to_vec()
        );

        // Past the end: a short read is a refusal, never a shorter answer.
        assert_eq!(
            read_exact_range(&file, 1020, 8),
            Err(Refusal::DeclaredRangeMismatch),
            "a range extending past EOF must refuse"
        );
        assert_eq!(
            read_exact_range(&file, 4096, 8),
            Err(Refusal::DeclaredRangeMismatch),
            "a range wholly past EOF must refuse"
        );

        drop(file);
        std::fs::remove_file(path).expect("remove scratch file");
    }

    // Requirements: SAFE-005, SAFE-007
    //   A backing object smaller than the declared range still brackets and refuses rather than panicking on the conversion or the subtraction
    // Work-Package: WP-020
    // Evidence: a_backing_object_shorter_than_the_declared_range_does_not_panic
    #[cfg(target_os = "linux")]
    #[test]
    fn a_backing_object_shorter_than_the_declared_range_does_not_panic() {
        let (path, file) = scratch_file("short", &[0xAB; 16]);

        // The whole file is outside a range that starts past its end, so the
        // bracket is simply the file's hash.
        assert_eq!(
            digest_outside_ranges(&file, &[(4096, 8)]).expect("bracket must compute"),
            sha256(&[0xAB; 16])
        );
        // And an offset whose end overflows refuses instead of wrapping.
        assert_eq!(
            digest_outside_ranges(&file, &[(u64::MAX, 8)]),
            Err(Refusal::WrongSuiteShape)
        );

        drop(file);
        std::fs::remove_file(path).expect("remove scratch file");
    }

    // Requirements: SAFE-005, SAFE-007
    //   The contracted write lands exactly the contract's replacement bytes at the contract's offset and touches nothing else, measured through a held descriptor rather than inferred from the protocol's own post-conditions
    // Work-Package: WP-020
    // Evidence: the_contracted_write_lands_exactly_at_the_contracted_offset
    #[cfg(target_os = "linux")]
    #[test]
    fn the_contracted_write_lands_exactly_at_the_contracted_offset() {
        static REPLACEMENT: [u8; 8] = [0xC3; 8];
        let original: Vec<u8> = (0..4096_u32).map(|index| (index % 251) as u8).collect();
        // The stand-in for the loop device is a regular scratch file: the
        // helper writes through whatever descriptor it is handed, and the
        // mapping from device offsets to backing offsets is the acceptance's
        // measurement, not this test's.
        let (device_path, device) = scratch_file("write-device", &original);
        // The range must not already hold the replacement, or a write that
        // did nothing would satisfy the read-back below.
        assert_ne!(original[512..520], REPLACEMENT);

        let range = AdmittedRange {
            offset: 512,
            length: 8,
            replacement: &REPLACEMENT,
        };
        let written =
            write_contracted_range(&device, &range).expect("the contracted write must land");
        assert_eq!(written, REPLACEMENT.len());

        // The whole object, read back independently: the replacement at
        // exactly the contract's offset, and every other byte untouched.
        let mut expected = original.clone();
        expected[512..520].copy_from_slice(&REPLACEMENT);
        assert_eq!(
            std::fs::read(&device_path).expect("read the written scratch object back"),
            expected,
            "the write must land at the contracted offset and nowhere else"
        );

        drop(device);
        std::fs::remove_file(device_path).expect("remove scratch file");
    }

    // Requirements: SAFE-005, SAFE-007
    //   The multi-range digest bracket hashes exactly the bytes outside the union of sorted non-overlapping ranges, and refuses unsorted or overlapping input rather than mis-partitioning the file
    // Work-Package: WP-020
    // Evidence: the_multi_range_bracket_covers_exactly_the_bytes_outside_the_union
    #[cfg(target_os = "linux")]
    #[test]
    fn the_multi_range_bracket_covers_exactly_the_bytes_outside_the_union() {
        // Larger than the 8 KiB read chunk, so ranges can straddle one.
        let original: Vec<u8> = (0..20_000_u32).map(|index| (index % 251) as u8).collect();
        let chunk = 8 * 1024_u64;

        for (tag, ranges) in [
            ("two-disjoint", vec![(512_u64, 8_u64), (1024, 16)]),
            ("touching", vec![(512, 8), (520, 8)]),
            ("straddling-a-chunk", vec![(chunk - 4, 8), (chunk + 100, 8)]),
            ("one-inside-each-chunk", vec![(100, 8), (chunk + 200, 8)]),
            ("first-at-zero-last-at-eof", vec![(0, 8), (20_000 - 8, 8)]),
            ("three", vec![(0, 4), (512, 8), (9000, 1000)]),
        ] {
            let (path, file) = scratch_file(tag, &original);

            // The independent answer: the bytes outside every range,
            // concatenated in file order.
            let mut outside = Vec::new();
            let mut cursor = 0_usize;
            for (offset, length) in &ranges {
                let start = usize::try_from(*offset).expect("fits");
                let end = usize::try_from(offset + length).expect("fits");
                outside.extend_from_slice(&original[cursor..start]);
                cursor = end;
            }
            outside.extend_from_slice(&original[cursor..]);

            assert_eq!(
                digest_outside_ranges(&file, &ranges).expect("bracket must compute"),
                sha256(&outside),
                "{tag}: the bracket must equal the hash of exactly the bytes outside the union"
            );

            // A byte inside ANY range must not move the bracket; a byte
            // outside every range must.
            let inside_victim = usize::try_from(ranges[1].0).expect("fits");
            let mut inside_changed = original.clone();
            inside_changed[inside_victim] ^= 0xFF;
            let (inside_path, inside_file) = scratch_file(tag, &inside_changed);
            assert_eq!(
                digest_outside_ranges(&inside_file, &ranges).expect("bracket must compute"),
                sha256(&outside),
                "{tag}: a change inside a declared range must not move the bracket"
            );

            let outside_victim = usize::try_from(ranges[0].0 + ranges[0].1).expect("fits");
            let outside_victim = if ranges.len() > 1
                && u64::try_from(outside_victim).expect("fits") == ranges[1].0
            {
                // Touching ranges: the byte after range 0 is range 1's first.
                usize::try_from(ranges[1].0 + ranges[1].1).expect("fits")
            } else {
                outside_victim
            };
            let mut outside_changed = original.clone();
            outside_changed[outside_victim] ^= 0xFF;
            let (outside_path, outside_file) = scratch_file(tag, &outside_changed);
            assert_ne!(
                digest_outside_ranges(&outside_file, &ranges).expect("bracket must compute"),
                sha256(&outside),
                "{tag}: a change outside every declared range must move the bracket"
            );

            drop((file, inside_file, outside_file));
            for path in [path, inside_path, outside_path] {
                std::fs::remove_file(path).expect("remove scratch file");
            }
        }

        // Unsorted or overlapping input refuses rather than hashing a
        // mis-partitioned complement.
        let (path, file) = scratch_file("refuse", &original);
        assert_eq!(
            digest_outside_ranges(&file, &[(1024, 8), (512, 8)]),
            Err(Refusal::WrongSuiteShape),
            "unsorted ranges must refuse"
        );
        assert_eq!(
            digest_outside_ranges(&file, &[(512, 16), (520, 8)]),
            Err(Refusal::WrongSuiteShape),
            "overlapping ranges must refuse"
        );
        drop(file);
        std::fs::remove_file(path).expect("remove scratch file");
    }

    // Requirements: SAFE-005, SAFE-007
    //   EINVAL from the forbidden rebind names KernelRefused only after the attachment is re-observed read-write; an EINVAL the observation cannot explain and every other errno refuse rather than passing into the write
    // Work-Package: WP-020
    // Evidence: the_rebind_probe_names_only_an_observed_kernel_state
    #[cfg(target_os = "linux")]
    #[test]
    fn the_rebind_probe_names_only_an_observed_kernel_state() {
        // An accepted rebind is named without consulting flags: the run is
        // void whatever the status says.
        assert_eq!(
            classify_rebind_probe(Ok(()), || panic!("acceptance must not consult flags")),
            Ok(RebindProbe::KernelAccepted)
        );

        // EINVAL plus an attachment re-observed read-write is the refusal the
        // suite requires — and the observation must actually be taken.
        let mut observations = 0;
        assert_eq!(
            classify_rebind_probe(Err(Errno::INVAL), || {
                observations += 1;
                Ok(DESTRUCTIVE_FLAGS)
            }),
            Ok(RebindProbe::KernelRefused)
        );
        assert_eq!(observations, 1, "the flags were observed, not assumed");

        // EINVAL on an attachment observed read-only cannot be the read-write
        // refusal, whether the rest of the flag set matches a known protocol
        // or not: both refuse instead of passing into the write.
        for flags in [REQUIRED_FLAGS, FLAG_READ_ONLY] {
            assert_eq!(
                classify_rebind_probe(Err(Errno::INVAL), || Ok(flags)),
                Err(Refusal::AdversarialRebindFailed {
                    errno: Some(Errno::INVAL.raw_os_error()),
                })
            );
        }

        // A failed observation wins over any classification: nothing may be
        // named on a status that could not be read.
        assert_eq!(
            classify_rebind_probe(Err(Errno::INVAL), || {
                Err(Refusal::KernelOperation {
                    operation: "rebind-probe-restat",
                    errno: Some(Errno::IO.raw_os_error()),
                })
            }),
            Err(Refusal::KernelOperation {
                operation: "rebind-probe-restat",
                errno: Some(Errno::IO.raw_os_error()),
            })
        );

        // Every other errno refuses without consulting flags, carrying the
        // errno it saw.
        assert_eq!(
            classify_rebind_probe(Err(Errno::BADF), || panic!(
                "a non-EINVAL error must not consult flags"
            )),
            Err(Refusal::AdversarialRebindFailed {
                errno: Some(Errno::BADF.raw_os_error()),
            })
        );
    }
}
