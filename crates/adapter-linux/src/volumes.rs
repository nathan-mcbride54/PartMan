//! Increment 4b, second slice: device-mapper nodes classified by their
//! `dm/uuid` prefix, LVM logical volumes named from `dm/name` under their
//! designator-absent volume-group aggregates, dm-crypt mappings reported
//! and not named, and loop devices' backing paths reported for the node
//! 3b will let them have.
//!
//! **Every rule here is ADR-0053's, on the DR rows.** A dm node's kind is
//! read from `dm/uuid`'s prefix as a *classification* input — `LVM-` marks
//! a logical volume, `CRYPT-` an opened container, anything else an
//! unrecognized target — never as a name (DR3 measured the two prefixes;
//! ADR-0053 fixes the reading). The (Linux, Volume, LVM2 logical volume)
//! name is `dm/name`, bytes verbatim, trailing newline included, through
//! the bytes-preserving path (DR12: stable across re-assembly and a
//! reboot); its role is undesignated and stays absent. The (Linux,
//! Aggregate, LVM2) designator is **undesignated** — no client-readable
//! interface reports the volume-group id (L7) — so every volume group is a
//! designator-absent `Aggregate { Lvm2, None }`, indeterminate and not an
//! operand (ADR-0019; slice 3q), and each logical volume names under it as
//! its producer. The dm-crypt mapping name is undesignated (DR12: the
//! opener's argument), so a container yields no `Volume`. The (Linux,
//! `BackingExtent`, loop) path is `loop/backing_file`, verbatim (DR13), and
//! its host is the file-system node the Linux client draft cannot build
//! until 3b — so the path is reported here and no node is built.
//!
//! **How many volume groups.** The client cannot read a VG id, but
//! `dm/uuid` on an LVM mapping is `LVM-<32 vg bytes><32 lv bytes>`, and the
//! 32-byte VG class is used to partition logical volumes into groups —
//! a classification, never a name: every group's naming fields are
//! identical (designator absent), so two or more absorb into one collision
//! group whose count is the number of classes seen and whose shared address
//! is every logical volume's producer. That is the decided representation
//! of "an ordinary client sees LVs but no VG identity"; the helper repairs
//! it at HLP-002. Reading the class from the uuid is a transformation
//! applied to nothing that enters a name, which is what ADR-0019's
//! no-transformation rule protects.

use std::path::Path;

use partman_domain::model::naming::{
    AggregateTechnology, NamingError, NamingFields, NodeEntry, NodeId, absorb, derive_id,
};

use crate::contract::{
    ContractSource, InterfaceAnswered, Listing, NamingRead, list_bounded, read_naming_source,
};
use crate::devices::{BLOCK_CLASS, Device, DeviceKind, HostAssembledKind};

/// The dm target's classification source, relative to the class directory.
pub const DM_UUID_ATTRIBUTE: &str = "dm/uuid";
/// ADR-0053's designated LVM logical-volume name source.
pub const DM_NAME_ATTRIBUTE: &str = "dm/name";
/// ADR-0053's designated loop backing-path source.
pub const LOOP_BACKING_FILE_ATTRIBUTE: &str = "loop/backing_file";
/// The `dm/uuid` prefix of an LVM mapping (DR3).
pub const LVM_PREFIX: &[u8] = b"LVM-";
/// The `dm/uuid` prefix of a dm-crypt mapping (DR3).
pub const CRYPT_PREFIX: &[u8] = b"CRYPT-";
/// The width of the volume-group class inside an LVM `dm/uuid`.
pub const VG_CLASS_WIDTH: usize = 32;

/// How a designated source answered — ADR-0034's three outcomes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceRead {
    /// The source answered; the bytes are the naming input, verbatim.
    Present(Vec<u8>),
    /// Positively absent or empty — a measured absence.
    Absent,
    /// The read failed or exceeded the bound — not absence.
    Unreadable {
        /// Why.
        reason: String,
    },
}

fn source_read(
    source: &dyn ContractSource,
    path: &Path,
    answered: &InterfaceAnswered,
) -> SourceRead {
    match read_naming_source(source, path, answered) {
        NamingRead::Bytes(bytes) => SourceRead::Present(bytes),
        NamingRead::Empty | NamingRead::NotPresent => SourceRead::Absent,
        NamingRead::OverLimit { seen } => SourceRead::Unreadable {
            reason: format!("{seen} bytes, over the limit"),
        },
        NamingRead::Failed { error } => SourceRead::Unreadable { reason: error },
    }
}

/// What a device-mapper node's `dm/uuid` prefix classifies it as.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MappingKind {
    /// An LVM logical volume; carries its volume-group class (the 32 bytes
    /// after `LVM-`) — a classification input, never a name.
    LvmLogicalVolume {
        /// The volume-group class bytes.
        group_class: Vec<u8>,
    },
    /// An opened dm-crypt container. Reported; names nothing (ADR-0053).
    CryptMapping,
    /// A target this build does not classify; reported, names nothing.
    Unrecognized {
        /// The `dm/uuid` bytes as read.
        raw: Vec<u8>,
    },
    /// `dm/uuid` did not answer; nothing can be classified, so nothing is.
    Undetermined {
        /// Why.
        reason: String,
    },
}

/// One device-mapper node's report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MappingReport {
    /// The device's session-local selector.
    pub selector: String,
    /// The classification.
    pub kind: MappingKind,
    /// The designated name source, as it answered — consulted only for a
    /// logical volume, read for every mapping so the report is complete.
    pub name: SourceRead,
}

/// One logical volume the slice may name: a `Volume` under its group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VolumeReport {
    /// The device's session-local selector.
    pub selector: String,
    /// The volume-group class it belongs to.
    pub group_class: Vec<u8>,
    /// `Volume { producer: <the designator-absent LVM2 address>, name, role: None }`.
    pub fields: NamingFields,
}

/// One loop device's backing path, reported for the `BackingExtent` 3b's
/// host node will let it have.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopReport {
    /// The device's session-local selector.
    pub selector: String,
    /// `loop/backing_file`, verbatim (DR7, DR13) — by-name evidence on
    /// issue #94's terms.
    pub backing_path: SourceRead,
}

/// Everything this module reports over one enumeration.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mappings {
    /// One designator-absent LVM2 aggregate per volume-group class seen.
    pub groups: Vec<NamingFields>,
    /// The logical volumes that had a readable name.
    pub volumes: Vec<VolumeReport>,
    /// Every device-mapper node, classified — including the logical
    /// volumes above, the containers, and the unrecognized.
    pub mappings: Vec<MappingReport>,
    /// Every loop device's backing path.
    pub loops: Vec<LoopReport>,
}

/// The designator-absent LVM2 aggregate every Linux volume group is under
/// ADR-0053, and therefore every logical volume's producer address.
#[must_use]
pub fn lvm_group_fields() -> NamingFields {
    NamingFields::Aggregate {
        technology: AggregateTechnology::Lvm2,
        designator: None,
    }
}

/// Classify and report the device-mapper and loop nodes among the
/// enumerated devices.
#[must_use]
pub fn report_mappings(
    source: &dyn ContractSource,
    sysfs_root: &Path,
    devices: &[Device],
) -> Mappings {
    let class = sysfs_root.join(BLOCK_CLASS);
    let Listing::Listed { answered, .. } = list_bounded(source, &class) else {
        return Mappings::default();
    };
    let producer: Option<NodeId> = derive_id(&lvm_group_fields()).ok();
    let mut out = Mappings::default();
    let mut classes: Vec<Vec<u8>> = Vec::new();
    for device in devices {
        let directory = class.join(&device.entry);
        match device.kind {
            DeviceKind::HostAssembled(HostAssembledKind::DeviceMapper) => {
                let kind = classify(&source_read(
                    source,
                    &directory.join(DM_UUID_ATTRIBUTE),
                    &answered,
                ));
                let name = source_read(source, &directory.join(DM_NAME_ATTRIBUTE), &answered);
                if let (
                    MappingKind::LvmLogicalVolume { group_class },
                    SourceRead::Present(bytes),
                    Some(producer),
                ) = (&kind, &name, producer)
                {
                    if !classes.contains(group_class) {
                        classes.push(group_class.clone());
                    }
                    out.volumes.push(VolumeReport {
                        selector: device.selector.clone(),
                        group_class: group_class.clone(),
                        fields: NamingFields::Volume {
                            producer,
                            name: bytes.clone(),
                            role: None,
                        },
                    });
                }
                out.mappings.push(MappingReport {
                    selector: device.selector.clone(),
                    kind,
                    name,
                });
            }
            DeviceKind::HostAssembled(HostAssembledKind::Loop) => out.loops.push(LoopReport {
                selector: device.selector.clone(),
                backing_path: source_read(
                    source,
                    &directory.join(LOOP_BACKING_FILE_ATTRIBUTE),
                    &answered,
                ),
            }),
            _ => {}
        }
    }
    out.groups = classes.iter().map(|_| lvm_group_fields()).collect();
    out
}

/// Classify one `dm/uuid` reading.
#[must_use]
pub fn classify(uuid: &SourceRead) -> MappingKind {
    match uuid {
        SourceRead::Present(bytes) => {
            if let Some(rest) = bytes.strip_prefix(LVM_PREFIX) {
                if rest.len() >= VG_CLASS_WIDTH {
                    return MappingKind::LvmLogicalVolume {
                        group_class: rest[..VG_CLASS_WIDTH].to_vec(),
                    };
                }
                MappingKind::Unrecognized { raw: bytes.clone() }
            } else if bytes.starts_with(CRYPT_PREFIX) {
                MappingKind::CryptMapping
            } else {
                MappingKind::Unrecognized { raw: bytes.clone() }
            }
        }
        SourceRead::Absent => MappingKind::Undetermined {
            reason: format!("`{DM_UUID_ATTRIBUTE}` is absent"),
        },
        SourceRead::Unreadable { reason } => MappingKind::Undetermined {
            reason: format!("`{DM_UUID_ATTRIBUTE}` did not answer: {reason}"),
        },
    }
}

/// Absorb the groups and volumes into ADR-0019's node set: the groups
/// collapse into one designator-absent aggregate or one collision group
/// (count = classes seen); each volume names under that address by its
/// own `dm/name`.
///
/// # Errors
///
/// [`NamingError`] as the domain's absorption reports it.
pub fn absorb_mappings(mappings: &Mappings) -> Result<Vec<NodeEntry>, NamingError> {
    absorb(
        mappings
            .groups
            .iter()
            .cloned()
            .chain(mappings.volumes.iter().map(|v| v.fields.clone()))
            .collect(),
    )
}
