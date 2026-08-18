//! The state layer (increment 4a): the mount table and the swap table, read
//! from the kernel's own procfs tables and reported as attributed
//! observations keyed to the devices this adapter admits.
//!
//! **What a mount is here, and is not.** MODEL-005's body-stability rule and
//! ADR-0005 Rule 2 put the mount set in the envelope: it changes without any
//! storage change, so it is never a topology node and never body content
//! (gitea#1004; `docs/reviews/ISSUE-1004_MOUNT_VARIANT_ROUND_2026-08-18.md`).
//! ADR-0018 keeps mount state and active swap in Regime B — reasons and
//! runtime gates, never the verdict. So this module builds no node, no edge,
//! and no `NamingFields`; it produces MODEL-004 observations whose value is
//! the table line the kernel reported, verbatim, plus the parse of that line
//! into its documented fields, and a keying from each entry's `major:minor`
//! to the admitted device that carries the same number. The Section 5 `Mount`
//! type is WP-010's and arrives with its first consumer; nothing here
//! anticipates its shape.
//!
//! **Every representational claim rests on a row.** The 2026-08-18
//! detection-rows sitting established, for an ordinary client on a real
//! host: **DR1** — `/proc/self/mountinfo` is readable, one line per mount,
//! in the documented shape (mount id, parent id, `major:minor`, root, mount
//! point, options, optional fields, the `-` separator, file-system type,
//! source, super options), keyed by `major:minor` for a whole-disk, loop,
//! and LVM mount and for the guest's root — and **not** for a Btrfs mount,
//! whose `major:minor` is an anonymous device and whose member appears only
//! in the source field; **DR2** — `/proc/swaps` is readable, a header line
//! and one row per active swap (path, type, size, used, priority); **DR9** —
//! the contract's device roster and record are byte-equal across a mount
//! cycle, which is what makes reporting a mount beside a device honest under
//! ADR-0005's own evidence obligation.
//!
//! **No transformation.** Paths in both tables are carried as the kernel
//! wrote them, octal escapes and all: unescaping is a transformation with an
//! edge this sitting did not measure, and a consumer that needs the display
//! form applies it as a display concern.
//!
//! **Refusal, not guessing.** A line whose field count or separator departs
//! from the recorded shape refuses the whole table rather than the line: a
//! partial mount set could present a mounted device as unmounted, which is
//! the fail-open SAFE-005 exists to prevent.

use std::path::Path;

use partman_domain::canonical::Value;
use partman_domain::model::provenance::{Observation, Outcome};

use crate::contract::{ContractSource, RecordRead, read_table};
use crate::devices::Device;
use crate::observation::{Interface, observe_unavailable};

/// The mount table, relative to the procfs root.
pub const MOUNT_TABLE: &str = "self/mountinfo";
/// The swap table, relative to the procfs root.
pub const SWAP_TABLE: &str = "swaps";
/// The swap table's header line, which DR2 recorded and which a table must
/// open with to be the swap table.
pub const SWAP_HEADER_PREFIX: &str = "Filename";

/// One line of the mount table, parsed as DR1 recorded the shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MountEntry {
    /// The MODEL-004 observation: the line the kernel reported, verbatim,
    /// attributed to the procfs interface.
    pub observation: Observation,
    /// The kernel's mount id.
    pub mount_id: u64,
    /// The parent mount's id.
    pub parent_id: u64,
    /// The mounted device's major number. `0` marks an anonymous device — a
    /// pseudo file system, or a Btrfs mount (DR1).
    pub major: u32,
    /// The mounted device's minor number.
    pub minor: u32,
    /// The root of the mount within the file system, verbatim.
    pub root: String,
    /// The mount point, verbatim.
    pub mount_point: String,
    /// The per-mount options, verbatim.
    pub options: String,
    /// The optional fields, verbatim, in order — DR1 recorded exactly one
    /// (`shared:N`) on every provisioned mount.
    pub optional_fields: Vec<String>,
    /// The file-system type.
    pub fs_type: String,
    /// The mount source, verbatim — a device path, or the pseudo file
    /// system's name.
    pub source: String,
    /// The per-superblock options, verbatim.
    pub super_options: String,
}

impl MountEntry {
    /// The entry's `major:minor` in the form the `dev` attribute reports it.
    #[must_use]
    pub fn device_number(&self) -> String {
        format!("{}:{}", self.major, self.minor)
    }
}

/// One row of the swap table, parsed as DR2 recorded the shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwapEntry {
    /// The MODEL-004 observation: the row the kernel reported, verbatim.
    pub observation: Observation,
    /// The swap's path, verbatim.
    pub path: String,
    /// The kernel's type word — `partition` or `file`.
    pub kind: String,
    /// Size in KiB, as reported.
    pub size_kib: u64,
    /// Used KiB, as reported.
    pub used_kib: u64,
    /// The priority, as reported.
    pub priority: i64,
}

/// One state table's outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Table<T> {
    /// The interface answered and every line parsed.
    Listed {
        /// The entries, in the kernel's order.
        entries: Vec<T>,
    },
    /// The interface did not answer, or answered in a shape this build does
    /// not recognize. Carries the observation recording why — `unavailable`
    /// where the interface did not answer, `failed` where it answered
    /// something this contract refuses to parse — and never an empty list.
    Refused {
        /// The recorded reason.
        observation: Observation,
    },
}

/// Read and parse the mount table.
#[must_use]
pub fn read_mounts(source: &dyn ContractSource, procfs_root: &Path) -> Table<MountEntry> {
    match table_text(source, &procfs_root.join(MOUNT_TABLE)) {
        Ok(text) => parse_mounts(&text),
        Err(observation) => Table::Refused { observation },
    }
}

/// Read and parse the swap table.
#[must_use]
pub fn read_swaps(source: &dyn ContractSource, procfs_root: &Path) -> Table<SwapEntry> {
    match table_text(source, &procfs_root.join(SWAP_TABLE)) {
        Ok(text) => parse_swaps(&text),
        Err(observation) => Table::Refused { observation },
    }
}

/// The interface half: one bounded read, three answers kept apart.
fn table_text(source: &dyn ContractSource, path: &Path) -> Result<String, Observation> {
    match read_table(source, path) {
        RecordRead::Present { text, .. } => Ok(text),
        RecordRead::NoRecord => Err(observe_unavailable(
            Interface::Procfs,
            "the table is not present on this host",
        )),
        RecordRead::OverLimit { seen } => Err(failed(format!(
            "the table is {seen} bytes, over the {} byte limit, and was not truncated",
            crate::contract::TABLE_LIMIT
        ))),
        RecordRead::NotText => Err(failed("the table is not UTF-8".to_owned())),
        RecordRead::Failed { error } => {
            Err(failed(format!("the table could not be read: {error}")))
        }
    }
}

/// Parse the mount table's text (DR1's shape).
///
/// Every line must carry at least ten fields, a `-` separator, and exactly
/// three fields after it; the first three fields must parse as two ids and a
/// `major:minor`. One departing line refuses the table.
#[must_use]
pub fn parse_mounts(text: &str) -> Table<MountEntry> {
    let mut entries = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        let fields: Vec<&str> = line.split(' ').collect();
        let Some(separator) = fields.iter().position(|field| *field == "-") else {
            return malformed(number, "no `-` separator");
        };
        if separator < 6 {
            return malformed(number, "fewer than six fields before the separator");
        }
        if fields.len() != separator + 4 {
            return malformed(number, "not exactly three fields after the separator");
        }
        let (Ok(mount_id), Ok(parent_id)) = (fields[0].parse::<u64>(), fields[1].parse::<u64>())
        else {
            return malformed(number, "the mount or parent id is not a decimal number");
        };
        let Some((major, minor)) = fields[2].split_once(':').and_then(|(major, minor)| {
            Some((major.parse::<u32>().ok()?, minor.parse::<u32>().ok()?))
        }) else {
            return malformed(number, "the `major:minor` field is not two decimal numbers");
        };
        entries.push(MountEntry {
            observation: observed(line),
            mount_id,
            parent_id,
            major,
            minor,
            root: fields[3].to_owned(),
            mount_point: fields[4].to_owned(),
            options: fields[5].to_owned(),
            optional_fields: fields[6..separator]
                .iter()
                .map(|f| (*f).to_owned())
                .collect(),
            fs_type: fields[separator + 1].to_owned(),
            source: fields[separator + 2].to_owned(),
            super_options: fields[separator + 3].to_owned(),
        });
    }
    Table::Listed { entries }
}

/// Parse the swap table's text (DR2's shape): the header, then rows of
/// exactly five whitespace-separated fields.
#[must_use]
pub fn parse_swaps(text: &str) -> Table<SwapEntry> {
    let mut lines = text.lines();
    match lines.next() {
        Some(header) if header.starts_with(SWAP_HEADER_PREFIX) => {}
        Some(_) => return malformed(1, "the first line is not the swap table's header"),
        None => return malformed(1, "the table is empty — not even a header"),
    }
    let mut entries = Vec::new();
    for (index, line) in lines.enumerate() {
        let number = index + 2;
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() != 5 {
            return malformed(number, "not exactly five fields");
        }
        let (Ok(size_kib), Ok(used_kib), Ok(priority)) = (
            fields[2].parse::<u64>(),
            fields[3].parse::<u64>(),
            fields[4].parse::<i64>(),
        ) else {
            return malformed(number, "size, used, or priority is not a decimal number");
        };
        entries.push(SwapEntry {
            observation: observed(line),
            path: fields[0].to_owned(),
            kind: fields[1].to_owned(),
            size_kib,
            used_kib,
            priority,
        });
    }
    Table::Listed { entries }
}

/// The mount entries resolved against the admitted devices.
///
/// Keying is by `major:minor` and nothing else — never by the source path,
/// which is a name (a mapper path, a loop node path) and which for a Btrfs
/// mount names one member of a file system the entry does not key to at all
/// (DR1). A device that reported no `dev` attribute keys nothing.
#[derive(Debug, PartialEq, Eq)]
pub struct KeyedMounts<'a> {
    /// Per admitted device that carries at least one mount: its selector and
    /// its entries, in the kernel's order. Devices without a mount are
    /// absent from this list — an empty entry list would say "asked, none",
    /// which is exactly what it means, so they are simply not listed.
    pub by_device: Vec<(&'a str, Vec<&'a MountEntry>)>,
    /// Entries whose `major:minor` is no admitted device's: pseudo file
    /// systems and Btrfs (major 0), partitions and every other node this
    /// adapter does not admit as a whole device today.
    pub unkeyed: Vec<&'a MountEntry>,
}

/// Resolve mount entries to the admitted devices by `major:minor`.
#[must_use]
pub fn key_mounts<'a>(entries: &'a [MountEntry], devices: &'a [Device]) -> KeyedMounts<'a> {
    let mut by_device: Vec<(&'a str, Vec<&'a MountEntry>)> = Vec::new();
    let mut unkeyed = Vec::new();
    for entry in entries {
        let number = entry.device_number();
        let owner = devices
            .iter()
            .find(|device| device.device_number.as_deref() == Some(number.as_str()));
        match owner {
            Some(device) => match by_device.iter_mut().find(|(s, _)| *s == device.selector) {
                Some((_, list)) => list.push(entry),
                None => by_device.push((device.selector.as_str(), vec![entry])),
            },
            None => unkeyed.push(entry),
        }
    }
    KeyedMounts { by_device, unkeyed }
}

fn observed(line: &str) -> Observation {
    Observation {
        adapter: Interface::Procfs.adapter(),
        adapter_version: crate::VERSION.to_owned(),
        method: Interface::Procfs.method(),
        outcome: Outcome::Observed {
            value: Value::Text(line.to_owned()),
        },
    }
}

fn failed(error: String) -> Observation {
    Observation {
        adapter: Interface::Procfs.adapter(),
        adapter_version: crate::VERSION.to_owned(),
        method: Interface::Procfs.method(),
        outcome: Outcome::Failed { error },
    }
}

fn malformed<T>(line: usize, reason: &str) -> Table<T> {
    Table::Refused {
        observation: failed(format!(
            "line {line} of the table departs from the recorded shape ({reason}); the table is refused, not read partially"
        )),
    }
}
