//! Safe Linux controller over the confined ioctl module.

use std::fs::File;
use std::io;
use std::path::Path;
use std::time::Duration;

use partman_fixtures::catalogue;
use rustix::fs::{FileType, Mode, OFlags, fstat, major, minor, open};
use rustix::io::{Errno, pread};
use sha2::{Digest as _, Sha256};

use crate::protocol::{
    ConfigureError, ConfigureRequest, Controller, REQUIRED_BLOCK_SIZE, REQUIRED_FLAGS, execute,
};
use crate::{AuthorizedFiles, BASIC_NAME, CONFLICTING_NAME, FixtureRole, Refusal, RunReport, sys};

const LOOP_CONTROL: &str = "/dev/loop-control";
const LOOP_MAJOR: u32 = 7;
const MISC_MAJOR: u32 = 10;
const LOOP_CONTROL_MINOR: u32 = 237;
const PROBE_BYTES: usize = 4096;
const DETACH_ATTEMPTS: usize = 16;
const DETACH_RETRY_DELAY: Duration = Duration::from_millis(100);
const SYS_DEV_BLOCK: &str = "/sys/dev/block";

pub(super) fn run(files: AuthorizedFiles) -> Result<RunReport, Refusal> {
    let mut controller = LinuxController::new(files)?;
    execute(&mut controller)
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
        let control = open_loop_node(LOOP_CONTROL, "loop-control-open")?;
        let control_stat = fstat(&control).map_err(|error| kernel("loop-control-fstat", error))?;
        if FileType::from_raw_mode(control_stat.st_mode) != FileType::CharacterDevice
            || major(control_stat.st_rdev) != MISC_MAJOR
            || minor(control_stat.st_rdev) != LOOP_CONTROL_MINOR
        {
            return Err(Refusal::LoopControlIdentityMismatch);
        }
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

        let raw_number = sys::control_get_free(&self.control)
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

        match sys::configure(&device, &self.basic, request) {
            Ok(()) => {
                // No fallible work is allowed between successful atomic
                // LOOP_CONFIGURE and returning the owned Attachment. From this
                // point the protocol layer is responsible for confirmed cleanup.
                Ok(Attachment {
                    device,
                    number,
                    node_identity,
                })
            }
            Err(Errno::BUSY) => Err(ConfigureError::Busy),
            Err(error) => Err(ConfigureError::Refused(kernel("loop-configure", error))),
        }
    }

    fn verify(
        &mut self,
        attachment: &Self::Attachment,
        expected: FixtureRole,
    ) -> Result<(), Refusal> {
        let status =
            sys::status(&attachment.device).map_err(|error| kernel("loop-get-status64", error))?;
        let backing = backing_identity(self.backing(expected))?;
        if status.backing_device != backing.encoded_device || status.backing_inode != backing.inode
        {
            return Err(Refusal::BackingIdentityMismatch);
        }
        if status.flags != REQUIRED_FLAGS {
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
}
