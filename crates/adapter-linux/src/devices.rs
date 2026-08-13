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
/// record in the database.
pub const DEVICE_NUMBER_ATTRIBUTE: &str = "dev";

/// The block class directory, relative to the sysfs root.
pub const BLOCK_CLASS: &str = "class/block";

/// ADR-0018's transport answer for every device this contract sees.
///
/// `Unrecognized`, unconditionally — and deliberately not a function of the
/// device, because no input could change it. ADR-0018's own evidence
/// obligation, "fabric-versus-local transport discrimination rows per
/// platform for each listed local transport", is outstanding on every
/// platform; no value of any classifying key is recorded anywhere in this
/// repository for any Linux host; and five of the six positive-local classes
/// have no Linux measurement of any kind. A table mapping interface strings
/// to classes could therefore come only from vendor documentation, which is
/// the one thing this package's evidence rule forbids.
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
) -> Device {
    let mut properties = Vec::new();
    for (property, relative) in SYSFS_FIELDS {
        let read = read_attribute(source, &directory.join(relative), answered);
        properties.push((
            key(Interface::Sysfs, property),
            single(observe(Interface::Sysfs, &read)),
        ));
    }
    read_database_half(source, directory, udev_root, answered, &mut properties);
    Device {
        selector,
        properties,
        transport: transport_class(),
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
    directory: &Path,
    udev_root: &Path,
    answered: &InterfaceAnswered,
    properties: &mut Vec<(String, PropertyObservations)>,
) {
    let AttributeRead::Text(number) =
        read_attribute(source, &directory.join(DEVICE_NUMBER_ATTRIBUTE), answered)
    else {
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
    // record class and is not read.
    for wanted in UDEV_KEYS {
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
    for wanted in UDEV_KEYS {
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
