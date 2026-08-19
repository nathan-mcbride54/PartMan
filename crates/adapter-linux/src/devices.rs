//! Whole-device enumeration and the identity material each device reports
//! (INV-001, INV-002, ADR-0018).
//!
//! Every field is an attributed observation and nothing is elected. Two
//! interfaces reporting something serial-shaped produce two properties under
//! two native names, because they are demonstrably not one fact: the record
//! shows a device reporting `S3Z9NB0K` through the attribute layer and
//! `ata-Samsung_S3Z9NB0K` through the database, and merging them would
//! manufacture a `conflicting` confidence out of two interfaces answering
//! different questions.
//!
//! **What this module does not do.** It builds no `NodeId`, no
//! `protection::Facts`, and no snapshot: those are keyed by ADR-0019 derived
//! addresses, which is increment 3's imported obligation. It reads no
//! interface outside the two this contract closed at increment 1. It reads
//! no partition-table key — a table identifier is topology material
//! (`NamingFields::PartitionTable` and its role discriminant), increment 3's
//! again. And it computes no removability, no boot role, and no identity
//! strength.
//!
//! **What increment 4a adds here is one classification and one key.** Every
//! admitted device carries a [`DeviceKind`] read from the DR3 markers, so a
//! loop, dm, or md node is reported as host-assembled and is no longer named
//! a physical device; and its `major:minor` rides along so the state tables
//! (`crate::state`) can resolve a mount or a swap to it. Neither is a name.

use std::path::Path;

use partman_domain::model::protection::TransportClass;
use partman_domain::model::provenance::PropertyObservations;

use crate::contract::{
    AttributeRead, ContractSource, InterfaceAnswered, Listing, RecordRead, list_bounded,
    read_attribute, read_record,
};
use crate::observation::{Interface, observe, observe_unavailable};

/// The sysfs attributes this contract reads, as (native property, path
/// relative to the device's class directory).
///
/// The roster matches WP-035's shipped one deliberately — a second roster
/// over the same interface would be two answers to one question — but the
/// match is not a warrant, and `schemas/adapter-linux/fields.md` records
/// which of these the measured record actually establishes and which it does
/// not.
pub const SYSFS_FIELDS: &[(&str, &str)] = &[
    ("size", "size"),
    ("ro", "ro"),
    ("removable", "removable"),
    ("logical_block_size", "queue/logical_block_size"),
    ("physical_block_size", "queue/physical_block_size"),
    ("device/vendor", "device/vendor"),
    ("device/model", "device/model"),
    ("device/wwid", "device/wwid"),
    ("device/serial", "device/serial"),
];

/// The database keys this contract reads.
///
/// These six are the keys the record names the database as carrying. No
/// partition-table key is read: a table identifier is increment 3's topology
/// material, and the table *state* is helper-authored under ADR-0014 in any
/// case.
pub const UDEV_KEYS: &[&str] = &[
    "ID_SERIAL",
    "ID_SERIAL_SHORT",
    "ID_WWN",
    "ID_WWN_WITH_EXTENSION",
    "ID_BUS",
    "ID_PATH",
];

/// The cached signature view (increment 4b, third slice): the database's
/// event-time `blkid` result for a device, **reported and consulted by
/// nothing**.
///
/// DR6 measured `ID_FS_TYPE`/`ID_FS_USAGE` naming the member technology on
/// every provisioned member and positively empty on a plain disk; DR14
/// measured `ID_FS_VERSION` carrying the family. And L4/L10 measured the
/// failure mode: over a live-ext4-over-stale-mdraid host the single-answer
/// cache reports exactly the stale `linux_raid_member` and no ext4 at all.
/// So these three are read as `Heuristic`/`inferred` observations — the
/// client's early warning that a disk may carry a signature — and enter no
/// name, no kind, no standing: the member-signature offset round decided
/// that a `BackingSignature`'s fields are the helper's byte layer's, and
/// that an unheld device stays eligible whatever the cache says (`held.rs`).
pub const UDEV_SIGNATURE_KEYS: &[&str] = &["ID_FS_TYPE", "ID_FS_USAGE", "ID_FS_VERSION"];

/// The attribute whose positively determined **absence** admits a node as a
/// whole device.
///
/// A partition carries it; a whole device does not. The discriminator is
/// `NotPresent` and nothing else — a successful-read test fails open, and a
/// read error would then promote a partition into the device list, where its
/// sector count would be reported as a device capacity.
///
/// The platform claim underneath this rule — that a whole device positively
/// lacks the attribute — has **no qualifying observability row**. It is
/// recorded as an obligation on WP-035 rather than treated as measured, and
/// the rule is written fail-closed so that the unmeasured direction is the
/// safe one: an unreadable attribute admits nothing.
pub const PARTITION_ATTRIBUTE: &str = "partition";

/// The attribute naming the device's major:minor pair, which locates its
/// record in the database — and, since increment 4a, keys the device to the
/// state tables' `major:minor` field.
pub const DEVICE_NUMBER_ATTRIBUTE: &str = "dev";

/// The sysfs directories that positively mark a block node as host-assembled
/// (increment 4a), in the order they are consulted.
///
/// **DR3** (the 2026-08-18 detection-rows sitting) establishes that `dm/`
/// exists under a device-mapper node, `md/` under an mdraid array, `loop/`
/// under a loop device, and none of the three under a plain whole disk, all
/// readable to the client. A marker is a directory, so its presence is
/// established by listing it — the same seam every other read here uses.
pub const KIND_MARKERS: &[(&str, HostAssembledKind)] = &[
    ("dm", HostAssembledKind::DeviceMapper),
    ("md", HostAssembledKind::Mdraid),
    ("loop", HostAssembledKind::Loop),
];

/// The host-assembled kinds this contract can positively mark.
///
/// Closed at three because DR3 measured three. A kind this build cannot
/// mark is not "plain"; it is whatever the plain-disk admission below makes
/// of it, which is why the plain arm is admission on the positively
/// determined **absence** of every marker and nothing weaker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostAssembledKind {
    /// A device-mapper node — a logical volume, an opened LUKS container, or
    /// any other dm target. Which one is DR3's `dm/uuid` prefix and is
    /// increment 4b's to read.
    DeviceMapper,
    /// An mdraid array.
    Mdraid,
    /// A loop device, backed by a file this contract reads by name only
    /// (DR7; issue #94's standing).
    Loop,
}

impl HostAssembledKind {
    /// The marker's compile-time label, for reports.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::DeviceMapper => "device-mapper",
            Self::Mdraid => "mdraid",
            Self::Loop => "loop",
        }
    }
}

/// What kind of block node an admitted whole device is (increment 4a).
///
/// The rule is the `partition` discipline again, applied to three markers:
/// a marker positively present makes the node host-assembled; every marker
/// positively **absent** makes it plain; a marker whose presence could not
/// be determined — a listing that failed for any reason other than
/// not-found — makes the node indeterminate, because admitting it as plain
/// would name a loop or dm device a physical device on the strength of a
/// read that did not answer. Increment 3a admitted every such node as an
/// operand-eligible `PhysicalDevice`; this is what withdraws that.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceKind {
    /// Every marker positively absent: a whole disk this adapter may name.
    Plain,
    /// A marker positively present: reported, never named as a physical
    /// device, not a plan operand until a naming designation names its kind.
    HostAssembled(HostAssembledKind),
    /// A marker's presence could not be determined. Not plain, not an
    /// operand — the fail-closed direction.
    Indeterminate {
        /// Which marker did not answer, and how.
        reason: String,
    },
}

/// Classify one admitted whole device by its kind markers.
#[must_use]
pub fn device_kind(source: &dyn ContractSource, directory: &Path) -> DeviceKind {
    let mut undetermined: Option<String> = None;
    for (marker, kind) in KIND_MARKERS {
        match source.list_dir(&directory.join(marker)) {
            Ok(_) => return DeviceKind::HostAssembled(*kind),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                if undetermined.is_none() {
                    undetermined = Some(format!("the `{marker}` marker did not answer: {error}"));
                }
            }
        }
    }
    match undetermined {
        Some(reason) => DeviceKind::Indeterminate { reason },
        None => DeviceKind::Plain,
    }
}

/// The block class directory, relative to the sysfs root.
pub const BLOCK_CLASS: &str = "class/block";

/// ADR-0018's transport answer for every device this contract sees.
///
/// `Unrecognized`, unconditionally — and deliberately not a function of the
/// device, because no input could change it. ADR-0018's own evidence
/// obligation, "fabric-versus-local transport discrimination rows per
/// platform for each listed local transport", is outstanding on every
/// platform, and a table mapping interface strings to classes could
/// therefore come only from vendor documentation, which is the one thing
/// this package's evidence rule forbids.
///
/// What is missing is the **protocol**, not the values. This comment used to
/// say that no value of any classifying key was recorded for any Linux host;
/// the 2026-08-13 readback made that false — `ID_BUS=usb` and two `ID_PATH`
/// values are recorded for real USB mass storage — without changing the
/// answer, because a recorded value still names no class until a
/// discrimination protocol says which classes are local. Two of the six
/// positive-local classes now have a real-hardware Linux measurement (USB
/// mass storage, and SD/MMC since the S5 sitting); neither classifies
/// anything.
///
/// This is the discharge of the assignment's obligation, not a shortfall:
/// `Unrecognized` is "the only answer for a class this build cannot
/// positively name", it resolves to `Indeterminate` at the protection
/// closure — never `Permitted` — and ADR-0018 prices exactly this
/// availability cost under "Negative, accepted knowingly".
#[must_use]
pub const fn transport_class() -> TransportClass {
    TransportClass::Unrecognized
}

/// One device's reported material.
pub struct Device {
    /// The session-local selector. Unstable across runs by construction:
    /// ADR-0019's derived addresses are increment 3's, and inventing a
    /// stable handle here would be naming without the naming rules.
    pub selector: String,
    /// Each property's observation set, keyed `interface:native-property`.
    ///
    /// The interface qualifier is part of the key because nothing here
    /// elects one identifier. A set is a singleton today and becomes plural
    /// only if two interfaces ever report the *same* native property.
    pub properties: Vec<(String, PropertyObservations)>,
    /// ADR-0018's transport answer. Always [`transport_class`].
    pub transport: TransportClass,
    /// The block-class entry this device was enumerated under — a
    /// session-local **locator** for re-reading its directory (increment
    /// 4b reads an array's `md/` attributes through it), never a name:
    /// kernel entry names renumber across boots and carry no identity.
    pub entry: String,
    /// The kind markers' verdict (increment 4a).
    pub kind: DeviceKind,
    /// The device's `major:minor` as the `dev` attribute reported it, or
    /// `None` where the attribute did not answer. The key the state tables
    /// resolve against; never a name and never body content.
    pub device_number: Option<String>,
}

/// The outcome of one enumeration.
pub enum Enumeration {
    /// The interface answered.
    Listed {
        /// The admitted whole devices, in listing order.
        devices: Vec<Device>,
    },
    /// The class listing exceeded the entry bound and was not truncated.
    OverLimit {
        /// How many entries were seen.
        seen: usize,
    },
    /// The platform did not expose the block class. Never an empty device
    /// list: "there is no such interface here" is not "this host has no
    /// devices".
    Unavailable {
        /// Why the interface could not answer.
        reason: String,
    },
    /// The listing itself failed.
    Failed {
        /// The error, as the operating system reported it.
        error: String,
    },
}

/// The session-local selector for one device.
#[must_use]
pub fn selector(index: usize) -> String {
    format!("device:{index}")
}

/// Enumerate whole devices through the contract, reporting each one's
/// material as attributed observations.
#[must_use]
pub fn enumerate(source: &dyn ContractSource, sysfs_root: &Path, udev_root: &Path) -> Enumeration {
    let class = sysfs_root.join(BLOCK_CLASS);
    let (entries, answered) = match list_bounded(source, &class) {
        Listing::Listed { entries, answered } => (entries, answered),
        Listing::OverLimit { seen } => return Enumeration::OverLimit { seen },
        Listing::Unavailable { reason } => return Enumeration::Unavailable { reason },
        Listing::Failed { error } => return Enumeration::Failed { error },
    };

    let mut devices = Vec::new();
    for name in entries {
        let directory = class.join(&name);
        // Admission on positively determined absence, and nothing else.
        if !matches!(
            read_attribute(source, &directory.join(PARTITION_ATTRIBUTE), &answered),
            AttributeRead::NotPresent
        ) {
            continue;
        }
        devices.push(read_device(
            source,
            &directory,
            udev_root,
            &answered,
            selector(devices.len()),
            name,
        ));
    }
    Enumeration::Listed { devices }
}

/// Read one admitted device's whole roster.
fn read_device(
    source: &dyn ContractSource,
    directory: &Path,
    udev_root: &Path,
    answered: &InterfaceAnswered,
    selector: String,
    entry: String,
) -> Device {
    let mut properties = Vec::new();
    for (property, relative) in SYSFS_FIELDS {
        let read = read_attribute(source, &directory.join(relative), answered);
        properties.push((
            key(Interface::Sysfs, property),
            single(observe(Interface::Sysfs, &read)),
        ));
    }
    let device_number =
        match read_attribute(source, &directory.join(DEVICE_NUMBER_ATTRIBUTE), answered) {
            AttributeRead::Text(number) => Some(number),
            _ => None,
        };
    read_database_half(source, udev_root, device_number.as_deref(), &mut properties);
    Device {
        selector,
        properties,
        transport: transport_class(),
        entry,
        kind: device_kind(source, directory),
        device_number,
    }
}

/// Append the database half of one device's roster.
///
/// The record is located from the device-number attribute, so a device whose
/// number cannot be read has no locatable record: every key is then
/// `unavailable`, never absent, because absence would claim the database
/// answered and said nothing.
fn read_database_half(
    source: &dyn ContractSource,
    udev_root: &Path,
    device_number: Option<&str>,
    properties: &mut Vec<(String, PropertyObservations)>,
) {
    let Some(number) = device_number else {
        return unavailable_half("the device number attribute did not answer", properties);
    };
    let record = read_record(source, &udev_root.join(format!("b{number}")));
    let text = match record {
        RecordRead::Present { text, .. } => text,
        RecordRead::NoRecord => {
            return unavailable_half("the database holds no record for this device", properties);
        }
        RecordRead::OverLimit { seen } => {
            return unavailable_half(
                &format!("the record is {seen} bytes, over the limit, and was not truncated"),
                properties,
            );
        }
        RecordRead::NotText => {
            return unavailable_half("the record is not UTF-8", properties);
        }
        RecordRead::Failed { error } => {
            return unavailable_half(
                &format!("the record could not be read: {error}"),
                properties,
            );
        }
    };

    // Property lines are `E:KEY=value`. Every other line belongs to another
    // record class and is not read. The identity keys and the cached
    // signature view are read from the one record the same way; only their
    // consumers differ, and the signature view has none.
    for wanted in UDEV_KEYS.iter().chain(UDEV_SIGNATURE_KEYS) {
        let found = text.lines().find_map(|line| {
            let rest = line.strip_prefix("E:")?;
            let (name, value) = rest.split_once('=')?;
            (name == *wanted).then(|| value.to_owned())
        });
        let read = match found {
            Some(value) if value.is_empty() => AttributeRead::Empty,
            Some(value) => AttributeRead::Text(value),
            // The record answered and does not carry this key: an absence.
            None => AttributeRead::NotPresent,
        };
        properties.push((
            key(Interface::UdevDatabase, wanted),
            single(observe(Interface::UdevDatabase, &read)),
        ));
    }
}

/// Record every database key as unavailable for one device.
fn unavailable_half(reason: &str, properties: &mut Vec<(String, PropertyObservations)>) {
    for wanted in UDEV_KEYS.iter().chain(UDEV_SIGNATURE_KEYS) {
        properties.push((
            key(Interface::UdevDatabase, wanted),
            single(observe_unavailable(Interface::UdevDatabase, reason)),
        ));
    }
}

/// The property key: the interface that answered, then its own native name.
fn key(interface: Interface, property: &str) -> String {
    format!("{}:{property}", interface.label())
}

/// One observation is a set of one. It becomes plural only if two interfaces
/// ever report the same native property.
fn single(observation: partman_domain::model::provenance::Observation) -> PropertyObservations {
    PropertyObservations {
        observations: vec![observation],
    }
}
